use std::cmp::Ordering;
use std::error::Error;
use std::os::raw::c_char;
use std::rc::Weak;

use ash::version::{DeviceV1_0, InstanceV1_0};
use ash::vk;

use crate::graphics::instance::Instance;

pub struct PhysicalDevice {
    properties: vk::PhysicalDeviceProperties,
    features: vk::PhysicalDeviceFeatures,
    queue_family_properties: Vec<vk::QueueFamilyProperties>,
    memory_properties: vk::PhysicalDeviceMemoryProperties,
    handle: vk::PhysicalDevice,
    instance_weak: Weak<Instance>,
}

impl PhysicalDevice {
    pub fn new(instance: &Instance, handle: vk::PhysicalDevice) -> Result<Self, Box<dyn Error>> {
        let properties = unsafe { instance.loader().get_physical_device_properties(handle) };
        let features = unsafe { instance.loader().get_physical_device_features(handle) };
        let queue_family_properties = unsafe {
            instance
                .loader()
                .get_physical_device_queue_family_properties(handle)
        };
        let memory_properties = unsafe {
            instance
                .loader()
                .get_physical_device_memory_properties(handle)
        };

        let instance_weak = unsafe { Weak::from_raw(instance) };
        Ok(Self {
            handle,
            properties,
            features,
            queue_family_properties,
            memory_properties,
            instance_weak,
        })
    }

    pub fn properties(&self) -> &vk::PhysicalDeviceProperties {
        &self.properties
    }

    pub fn features(&self) -> &vk::PhysicalDeviceFeatures {
        &self.features
    }

    pub fn queue_family_properties(&self) -> &Vec<vk::QueueFamilyProperties> {
        &self.queue_family_properties
    }

    pub fn memory_properties(&self) -> &vk::PhysicalDeviceMemoryProperties {
        &self.memory_properties
    }

    pub fn instance(&self) -> &Instance {
        unsafe { &*self.instance_weak.as_ptr() }
    }

    pub fn is_suitable(&self) -> bool {
        true
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
    extension_properties: Vec<vk::ExtensionProperties>,
    loader: ash::Device,
    physical_device_weak: Weak<PhysicalDevice>,
}

impl Device {
    pub fn new(
        instance: &Instance,
        physical_device: &PhysicalDevice,
    ) -> Result<Self, Box<dyn Error>> {
        let layer_properties = unsafe {
            let mut count = 0;
            instance
                .loader()
                .fp_v1_0()
                .enumerate_device_layer_properties(
                    physical_device.handle,
                    &mut count,
                    std::ptr::null_mut(),
                )
                .result()?;
            let mut vector = Vec::with_capacity(count as usize);
            instance
                .loader()
                .fp_v1_0()
                .enumerate_device_layer_properties(
                    physical_device.handle,
                    &mut count,
                    vector.as_mut_ptr(),
                )
                .result()?;
            vector.set_len(count as usize);
            vector
        };
        let p_layer_properties_names: Vec<*const c_char> = layer_properties
            .into_iter()
            .map(|layer_property| layer_property.layer_name.as_ptr())
            .collect();

        let extension_properties = unsafe {
            instance
                .loader()
                .enumerate_device_extension_properties(physical_device.handle)?
        };

        let features = vk::PhysicalDeviceFeatures::default();
        let create_info = vk::DeviceCreateInfo {
            enabled_layer_count: p_layer_properties_names.len() as u32,
            pp_enabled_layer_names: p_layer_properties_names.as_ptr(),
            p_enabled_features: &features,
            ..Default::default()
        };
        let loader = unsafe {
            instance
                .loader()
                .create_device(physical_device.handle, &create_info, None)?
        };

        let physical_device_weak = unsafe { Weak::from_raw(physical_device) };
        Ok(Self {
            loader,
            physical_device_weak,
            extension_properties,
        })
    }

    pub fn extension_properties(&self) -> &Vec<vk::ExtensionProperties> {
        &self.extension_properties
    }

    pub fn loader(&self) -> &ash::Device {
        &self.loader
    }

    pub fn physical_device(&self) -> &PhysicalDevice {
        unsafe { &*self.physical_device_weak.as_ptr() }
    }
}

impl Drop for Device {
    fn drop(&mut self) {
        unsafe { self.loader.destroy_device(None) };
    }
}
