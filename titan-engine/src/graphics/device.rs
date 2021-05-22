use std::cmp::Ordering;
use std::collections::HashSet;
use std::error::Error;
use std::ops::Deref;
use std::os::raw::c_char;

use ash::prelude::VkResult;
use ash::version::{DeviceV1_0, InstanceV1_0};
use ash::vk;

use super::Instance;
use super::Surface;
use super::utils;

pub struct PhysicalDevice {
    properties: vk::PhysicalDeviceProperties,
    features: vk::PhysicalDeviceFeatures,
    queue_family_properties: Vec<vk::QueueFamilyProperties>,
    memory_properties: vk::PhysicalDeviceMemoryProperties,
    handle: vk::PhysicalDevice,
}

impl PhysicalDevice {
    pub fn new(instance: &Instance, handle: vk::PhysicalDevice) -> Self {
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

        Self {
            handle,
            properties,
            features,
            queue_family_properties,
            memory_properties,
        }
    }

    pub fn handle(&self) -> ash::vk::PhysicalDevice {
        self.handle
    }

    pub fn is_suitable(&self) -> bool {
        let graphics_queue_family_properties =
            self.queue_family_properties_with(vk::QueueFlags::GRAPHICS);
        !graphics_queue_family_properties.is_empty()
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
    ) -> Vec<(usize, &vk::QueueFamilyProperties)> {
        let mut vector = Vec::with_capacity(self.queue_family_properties.len());
        for (index, queue_family_properties) in self.queue_family_properties.iter().enumerate() {
            let ref inner_flags = queue_family_properties.queue_flags;
            if inner_flags.contains(flags) {
                vector.push((index, queue_family_properties));
            }
        }
        vector
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
    layer_properties: Vec<vk::LayerProperties>,
    extension_properties: Vec<vk::ExtensionProperties>,
    queues: Vec<Queue>,
    loader: ash::Device,
}

impl Device {
    pub fn new(
        instance: &Instance,
        surface: &Surface,
        physical_device: &PhysicalDevice,
    ) -> Result<Self, Box<dyn Error>> {
        let layer_properties = unsafe {
            enumerate_device_layer_properties(instance.loader(), physical_device.handle)
        }?;
        let p_layer_properties_names: Vec<*const c_char> = layer_properties
            .iter()
            .map(|layer_property| layer_property.layer_name.as_ptr())
            .collect();

        let extension_properties = unsafe {
            instance
                .loader()
                .enumerate_device_extension_properties(physical_device.handle)?
        };

        let graphics_queue_family_properties =
            physical_device.queue_family_properties_with(vk::QueueFlags::GRAPHICS);
        let graphics_family_index = graphics_queue_family_properties
            .get(0)
            .ok_or_else(|| utils::make_error("no queues with graphics support"))?
            .0 as u32;
        let present_queue_family_properties =
            surface.physical_device_queue_family_properties_support(physical_device)?;
        let present_family_index = present_queue_family_properties
            .get(0)
            .ok_or_else(|| utils::make_error("no queues with surface present support"))?
            .0 as u32;

        let mut unique_family_indices = HashSet::new();
        unique_family_indices.insert(graphics_family_index);
        unique_family_indices.insert(present_family_index);

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

        let features = vk::PhysicalDeviceFeatures::builder();
        let create_info = vk::DeviceCreateInfo::builder()
            .queue_create_infos(queue_create_infos.as_slice())
            .enabled_layer_names(p_layer_properties_names.deref())
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

        Ok(Self {
            layer_properties,
            extension_properties,
            queues,
            loader,
        })
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
