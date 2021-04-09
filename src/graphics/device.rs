use std::cmp::Ordering;
use std::error::Error;
use std::ffi::CStr;

use ash::version::{DeviceV1_0, InstanceV1_0};
use ash::vk;

use crate::graphics::instance::Instance;
use crate::graphics::utils;
use crate::version::Version;

#[derive(Clone)]
pub struct PhysicalDevice {
    properties: vk::PhysicalDeviceProperties,
    features: vk::PhysicalDeviceFeatures,
    queue_family_properties: Vec<vk::QueueFamilyProperties>,
    memory_properties: vk::PhysicalDeviceMemoryProperties,
    layer_properties: Vec<vk::LayerProperties>,
    extension_properties: Vec<vk::ExtensionProperties>,
    handle: vk::PhysicalDevice,
}

impl PhysicalDevice {
    pub fn new(instance: &Instance, handle: vk::PhysicalDevice) -> Result<Self, Box<dyn Error>> {
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

    pub fn is_suitable(&self) -> bool {
        true
    }

    pub fn score(&self) -> u32 {
        let mut score = match self.properties.device_type {
            vk::PhysicalDeviceType::DISCRETE_GPU => 1000,
            vk::PhysicalDeviceType::INTEGRATED_GPU => 100,
            _ => 0
        };
        score += self.properties.limits.max_image_dimension2_d;
        score
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

pub struct Device {
    physical_device: PhysicalDevice,
    loader: ash::Device,
}

impl Device {
    pub fn new(
        instance: &Instance,
        physical_device: PhysicalDevice,
    ) -> Result<Self, Box<dyn Error>> {
        let create_info = vk::DeviceCreateInfo {
            ..Default::default()
        };

        let loader = unsafe {
            instance.loader().create_device(
                physical_device.handle,
                &create_info,
                None,
            )?
        };

        Ok(Self {
            loader,
            physical_device,
        })
    }

    pub fn loader(&self) -> &ash::Device {
        &self.loader
    }

    pub fn physical_device(&self) -> &PhysicalDevice {
        &self.physical_device
    }
}

impl Drop for Device {
    fn drop(&mut self) {
        unsafe {
            self.loader.destroy_device(None)
        };
    }
}
