use std::ffi::CStr;

use ash::version::InstanceV1_0;
use ash::vk;

use crate::graphics::instance::Instance;
use crate::graphics::utils;
use crate::version::Version;

pub struct PhysicalDevice {
    handle: vk::PhysicalDevice,
    properties: vk::PhysicalDeviceProperties,
    features: vk::PhysicalDeviceFeatures,
    queue_family_properties: Vec<vk::QueueFamilyProperties>,
    memory_properties: vk::PhysicalDeviceMemoryProperties,
}

impl PhysicalDevice {
    pub fn new(instance: &Instance, handle: vk::PhysicalDevice) -> Self {
        let properties = unsafe {
            instance.loader().get_physical_device_properties(handle)
        };
        let features = unsafe {
            instance.loader().get_physical_device_features(handle)
        };
        let queue_family_properties = unsafe {
            instance.loader().get_physical_device_queue_family_properties(handle)
        };
        let memory_properties = unsafe {
            instance.loader().get_physical_device_memory_properties(handle)
        };
        Self {
            handle,
            properties,
            features,
            queue_family_properties,
            memory_properties,
        }
    }

    pub fn version(&self) -> Version {
        utils::from_vk_version(self.properties.api_version)
    }

    pub fn name(&self) -> &CStr {
        unsafe {
            CStr::from_ptr(self.properties.device_name.as_ptr())
        }
    }
}
