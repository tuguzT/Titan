use std::cmp::Ordering;
use std::collections::HashSet;
use std::error::Error;
use std::ffi::CStr;
use std::ops::Deref;
use std::os::raw::c_char;

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
}

impl PhysicalDevice {
    pub fn new(instance: &Instance, handle: vk::PhysicalDevice) -> Result<Self, Box<dyn Error>> {
        let properties = unsafe { instance.loader().get_physical_device_properties(handle) };
        let features = unsafe { instance.loader().get_physical_device_features(handle) };
        let memory_properties = unsafe {
            instance
                .loader()
                .get_physical_device_memory_properties(handle)
        };
        let queue_family_properties = unsafe {
            instance
                .loader()
                .get_physical_device_queue_family_properties(handle)
        };
        let layer_properties =
            unsafe { enumerate_device_layer_properties(instance.loader(), handle)? };
        let extension_properties = unsafe {
            instance
                .loader()
                .enumerate_device_extension_properties(handle)?
        };

        Ok(Self {
            handle,
            properties,
            features,
            queue_family_properties,
            memory_properties,
            layer_properties,
            extension_properties,
        })
    }

    pub fn handle(&self) -> ash::vk::PhysicalDevice {
        self.handle
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
    queues: Vec<Queue>,
    loader: ash::Device,
}

impl Device {
    pub fn new(
        instance: &Instance,
        surface: &Surface,
        physical_device: &PhysicalDevice,
    ) -> Result<Self, Box<dyn Error>> {
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
        let mut queues = Vec::new();
        for create_info in queue_create_infos.iter() {
            let range = 0..create_info.queue_count;
            queues.extend(range.map(|index| unsafe {
                Queue::new(&loader, create_info.queue_family_index, index)
            }));
        }

        Ok(Self { queues, loader })
    }

    pub fn loader(&self) -> &ash::Device {
        &self.loader
    }

    pub fn queues(&self) -> &Vec<Queue> {
        &self.queues
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
}

impl Queue {
    unsafe fn new(device: &ash::Device, family_index: u32, index: u32) -> Self {
        let handle = device.get_device_queue(family_index, index);
        Self {
            family_index,
            handle,
        }
    }
}
