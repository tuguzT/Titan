use ash::version::InstanceV1_0;
use ash::vk;

use crate::graphics::instance::Instance;

pub struct PhysicalDevice {
    handle: vk::PhysicalDevice,
    properties: vk::PhysicalDeviceProperties,
    features: vk::PhysicalDeviceFeatures,
}

impl PhysicalDevice {
    pub fn new(instance: &Instance, handle: vk::PhysicalDevice) -> Self {
        let properties = unsafe {
            instance.loader().get_physical_device_properties(handle)
        };
        let features = unsafe {
            instance.loader().get_physical_device_features(handle)
        };
        Self {
            handle,
            properties,
            features,
        }
    }
}
