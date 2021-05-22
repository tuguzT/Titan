use std::cmp::Ordering;
use std::error::Error;
use std::ops::Deref;
use std::os::raw::c_char;

use ash::prelude::VkResult;
use ash::version::{DeviceV1_0, InstanceV1_0};
use ash::vk;

use super::Instance;

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
    extension_properties: Vec<vk::ExtensionProperties>,
    loader: ash::Device,
}

impl Device {
    pub fn new(
        instance: &Instance,
        physical_device: &PhysicalDevice,
    ) -> Result<Self, Box<dyn Error>> {
        use crate::error::{Error, ErrorType};

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
        let priorities = vec![1.0];
        let queue_family_index = graphics_queue_family_properties
            .get(0)
            .ok_or(Error::new(
                "no queues with support of graphics",
                ErrorType::Graphics,
            ))?
            .0 as u32;
        let device_queue_create_info = vk::DeviceQueueCreateInfo::builder()
            .queue_family_index(queue_family_index)
            .queue_priorities(priorities.as_slice());
        let queue_create_infos = vec![*device_queue_create_info];
        let features = vk::PhysicalDeviceFeatures::default();

        let create_info = vk::DeviceCreateInfo::builder()
            .queue_create_infos(queue_create_infos.as_slice())
            .enabled_layer_names(p_layer_properties_names.deref())
            .enabled_features(&features);
        let loader = unsafe {
            instance
                .loader()
                .create_device(physical_device.handle, &create_info, None)?
        };

        Ok(Self {
            extension_properties,
            loader,
        })
    }

    pub fn loader(&self) -> &ash::Device {
        &self.loader
    }
}

impl Drop for Device {
    fn drop(&mut self) {
        unsafe { self.loader.destroy_device(None) };
    }
}
