use std::error::Error;
use std::ffi::CStr;

use ash::version::InstanceV1_0;
use ash::vk;

use crate::graphics::instance::Instance;
use crate::graphics::utils;
use crate::version::Version;

pub struct PhysicalDevice<'a> {
    handle: vk::PhysicalDevice,
    properties: vk::PhysicalDeviceProperties,
    features: vk::PhysicalDeviceFeatures,
    queue_family_properties: Vec<vk::QueueFamilyProperties>,
    memory_properties: vk::PhysicalDeviceMemoryProperties,
    layer_properties: Vec<vk::LayerProperties>,
    extension_properties: Vec<vk::ExtensionProperties>,
    instance: &'a Instance,
}

impl<'a> PhysicalDevice<'a> {
    pub fn new(instance: &'a Instance, handle: vk::PhysicalDevice) -> Result<Self, Box<dyn Error>> {
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

        let layer_properties = unsafe {
            let mut count: u32 = 0;
            instance.loader().fp_v1_0().enumerate_device_layer_properties(
                handle,
                &mut count,
                std::ptr::null_mut(),
            ).result()?;
            let mut vector = Vec::with_capacity(count as usize);
            instance.loader().fp_v1_0().enumerate_device_layer_properties(
                handle,
                &mut count,
                vector.as_mut_ptr(),
            ).result()?;
            vector.set_len(count as usize);
            vector
        };

        let extension_properties = unsafe {
            instance.loader().enumerate_device_extension_properties(handle)?
        };

        Ok(Self {
            handle,
            properties,
            features,
            queue_family_properties,
            memory_properties,
            layer_properties,
            extension_properties,
            instance,
        })
    }

    pub fn version(&self) -> Version {
        utils::from_vk_version(self.properties.api_version)
    }

    pub fn name(&self) -> &CStr {
        unsafe {
            CStr::from_ptr(self.properties.device_name.as_ptr())
        }
    }

    pub fn instance(&self) -> &'a Instance {
        self.instance
    }
}
