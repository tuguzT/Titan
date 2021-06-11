use std::cmp::Ordering;
use std::collections::HashSet;
use std::error::Error;
use std::ffi::CStr;
use std::ops::Deref;
use std::os::raw::c_char;
use std::sync::{Arc, Weak};

use ash::prelude::VkResult;
use ash::version::{DeviceV1_0, InstanceV1_0};
use ash::vk;

use super::ext::Swapchain;
use super::utils;
use super::Instance;
use super::Surface;

lazy_static::lazy_static! {
    static ref REQUIRED_EXTENSIONS: Vec<&'static CStr> = vec![Swapchain::name()];
}

pub struct PhysicalDevice {
    properties: vk::PhysicalDeviceProperties,
    features: vk::PhysicalDeviceFeatures,
    memory_properties: vk::PhysicalDeviceMemoryProperties,
    queue_family_properties: Vec<vk::QueueFamilyProperties>,
    layer_properties: Vec<vk::LayerProperties>,
    extension_properties: Vec<vk::ExtensionProperties>,
    handle: vk::PhysicalDevice,
    parent_instance: Weak<Instance>,
}

impl PhysicalDevice {
    pub unsafe fn new(
        instance: &Arc<Instance>,
        handle: vk::PhysicalDevice,
    ) -> Result<Self, Box<dyn Error>> {
        let properties = instance.loader().get_physical_device_properties(handle);
        let features = instance.loader().get_physical_device_features(handle);
        let memory_properties = instance
            .loader()
            .get_physical_device_memory_properties(handle);
        let queue_family_properties = instance
            .loader()
            .get_physical_device_queue_family_properties(handle);
        let layer_properties = enumerate_device_layer_properties(instance.loader(), handle)?;
        let extension_properties = instance
            .loader()
            .enumerate_device_extension_properties(handle)?;

        Ok(Self {
            handle,
            properties,
            features,
            queue_family_properties,
            memory_properties,
            layer_properties,
            extension_properties,
            parent_instance: Arc::downgrade(instance),
        })
    }

    pub fn handle(&self) -> ash::vk::PhysicalDevice {
        self.handle
    }

    pub fn parent_instance(&self) -> Option<Arc<Instance>> {
        self.parent_instance.upgrade()
    }

    pub fn is_suitable(&self) -> bool {
        let mut graphics_queue_family_properties = self
            .queue_family_properties_with(vk::QueueFlags::GRAPHICS)
            .peekable();
        let mut extension_properties_names =
            self.extension_properties
                .iter()
                .map(|extension_property| unsafe {
                    CStr::from_ptr(extension_property.extension_name.as_ptr())
                });
        let has_required_extensions = REQUIRED_EXTENSIONS.iter().any(|required_name| {
            extension_properties_names
                .find(|item| item == required_name)
                .is_some()
        });
        graphics_queue_family_properties.peek().is_some() && has_required_extensions
    }

    pub fn score(&self) -> u32 {
        let mut score = match self.properties.device_type {
            vk::PhysicalDeviceType::DISCRETE_GPU => 1000,
            vk::PhysicalDeviceType::INTEGRATED_GPU => 100,
            _ => 0,
        };
        score += self.properties.limits.max_image_dimension2_d;
        score
    }

    pub fn queue_family_properties(&self) -> &Vec<vk::QueueFamilyProperties> {
        &self.queue_family_properties
    }

    pub fn queue_family_properties_with(
        &self,
        flags: vk::QueueFlags,
    ) -> impl Iterator<Item = (usize, &vk::QueueFamilyProperties)> {
        self.queue_family_properties.iter().enumerate().filter(
            move |(_index, queue_family_properties)| {
                let ref inner_flags = queue_family_properties.queue_flags;
                inner_flags.contains(flags)
            },
        )
    }

    pub fn layer_properties(&self) -> &Vec<vk::LayerProperties> {
        &self.layer_properties
    }

    pub fn extension_properties(&self) -> &Vec<vk::ExtensionProperties> {
        &self.extension_properties
    }

    pub fn graphics_family_index(&self) -> Result<u32, Box<dyn Error>> {
        let graphics_queue_family_properties =
            self.queue_family_properties_with(vk::QueueFlags::GRAPHICS);
        let graphics_family_index = graphics_queue_family_properties
            .peekable()
            .peek()
            .ok_or_else(|| utils::make_error("no queues with graphics support"))?
            .0 as u32;
        Ok(graphics_family_index)
    }

    pub fn present_family_index(&self, surface: &Surface) -> Result<u32, Box<dyn Error>> {
        let present_queue_family_properties =
            surface.physical_device_queue_family_properties_support(&self);
        let present_family_index = present_queue_family_properties
            .peekable()
            .peek()
            .ok_or_else(|| utils::make_error("no queues with surface present support"))?
            .0 as u32;
        Ok(present_family_index)
    }
}

impl PartialEq for PhysicalDevice {
    fn eq(&self, other: &Self) -> bool {
        self.score().eq(&other.score())
    }
}

impl PartialOrd for PhysicalDevice {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        self.score().partial_cmp(&other.score())
    }
}

impl Eq for PhysicalDevice {}

impl Ord for PhysicalDevice {
    fn cmp(&self, other: &Self) -> Ordering {
        self.score().cmp(&other.score())
    }
}

unsafe fn enumerate_device_layer_properties(
    instance: &ash::Instance,
    physical_device: vk::PhysicalDevice,
) -> VkResult<Vec<vk::LayerProperties>> {
    let mut count = 0;
    instance
        .fp_v1_0()
        .enumerate_device_layer_properties(physical_device, &mut count, std::ptr::null_mut())
        .result()?;
    let mut data = Vec::with_capacity(count as usize);
    let err_code = instance.fp_v1_0().enumerate_device_layer_properties(
        physical_device,
        &mut count,
        data.as_mut_ptr(),
    );
    data.set_len(count as usize);
    err_code.result_with_success(data)
}

pub struct Device {
    loader: ash::Device,
    queue_create_infos: Vec<vk::DeviceQueueCreateInfo>,
    parent_physical_device: Weak<PhysicalDevice>,
}

impl Device {
    pub fn new(
        surface: &Surface,
        physical_device: &Arc<PhysicalDevice>,
    ) -> Result<Self, Box<dyn Error>> {
        let surface_instance = surface
            .parent_instance()
            .ok_or_else(|| utils::make_error("surface parent was lost"))?;
        let physical_device_instance = physical_device
            .parent_instance()
            .ok_or_else(|| utils::make_error("physical device parent was lost"))?;
        if surface_instance.handle() != physical_device_instance.handle() {
            return Err(
                utils::make_error("surface and physical device parents must be the same").into(),
            );
        }
        let instance = surface_instance;

        let mut unique_family_indices = HashSet::new();
        unique_family_indices.insert(physical_device.graphics_family_index()?);
        unique_family_indices.insert(physical_device.present_family_index(surface)?);

        let priorities = [1.0];
        let queue_create_infos: Vec<_> = unique_family_indices
            .into_iter()
            .map(|family_index| {
                vk::DeviceQueueCreateInfo::builder()
                    .queue_family_index(family_index)
                    .queue_priorities(&priorities)
                    .build()
            })
            .collect();

        let p_layer_properties_names: Vec<*const c_char> = physical_device
            .layer_properties
            .iter()
            .map(|item| item.layer_name.as_ptr())
            .collect();
        let p_extension_properties_names: Vec<*const c_char> = physical_device
            .extension_properties
            .iter()
            .filter(|item| {
                let name = unsafe { CStr::from_ptr(item.extension_name.as_ptr()) };
                REQUIRED_EXTENSIONS.contains(&name)
            })
            .map(|item| item.extension_name.as_ptr())
            .collect();
        let features = vk::PhysicalDeviceFeatures::builder();
        let create_info = vk::DeviceCreateInfo::builder()
            .queue_create_infos(queue_create_infos.as_slice())
            .enabled_layer_names(p_layer_properties_names.deref())
            .enabled_extension_names(p_extension_properties_names.deref())
            .enabled_features(&features);
        let loader = unsafe {
            instance
                .loader()
                .create_device(physical_device.handle, &create_info, None)?
        };

        Ok(Self {
            queue_create_infos,
            loader,
            parent_physical_device: Arc::downgrade(physical_device),
        })
    }

    pub fn loader(&self) -> &ash::Device {
        &self.loader
    }

    pub fn handle(&self) -> vk::Device {
        self.loader.handle()
    }

    pub fn parent_physical_device(&self) -> Option<Arc<PhysicalDevice>> {
        self.parent_physical_device.upgrade()
    }

    pub fn enumerate_queues(this: &Arc<Self>) -> Vec<Queue> {
        let mut queues = Vec::new();
        for create_info in this.queue_create_infos.iter() {
            let range = 0..create_info.queue_count;
            queues.extend(
                range.map(|index| unsafe {
                    Queue::new(this, create_info.queue_family_index, index)
                }),
            );
        }
        queues
    }
}

impl Drop for Device {
    fn drop(&mut self) {
        unsafe { self.loader.destroy_device(None) };
    }
}

pub struct Queue {
    family_index: u32,
    handle: vk::Queue,
    parent_device: Weak<Device>,
}

impl Queue {
    unsafe fn new(device: &Arc<Device>, family_index: u32, index: u32) -> Self {
        let handle = device.loader().get_device_queue(family_index, index);
        Self {
            family_index,
            handle,
            parent_device: Arc::downgrade(device),
        }
    }

    pub fn handle(&self) -> vk::Queue {
        self.handle
    }

    pub fn parent_device(&self) -> Option<Arc<Device>> {
        self.parent_device.upgrade()
    }
}
