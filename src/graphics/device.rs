use std::cmp::Ordering;
use std::error::Error;
use std::os::raw::c_char;

use ash::version::{DeviceV1_0, InstanceV1_0};
use ash::vk;

use crate::graphics::instance::Instance;

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

        let graphics_queue_family_properties =
            physical_device.queue_family_properties_with(vk::QueueFlags::GRAPHICS);
        let priorities = [1.0];
        let queue_create_infos = vec![vk::DeviceQueueCreateInfo {
            queue_family_index: graphics_queue_family_properties
                .get(0)
                .ok_or(Error::new(
                    "no queues with support of graphics",
                    ErrorType::Graphics,
                ))?
                .0 as u32,
            queue_count: 1,
            p_queue_priorities: priorities.as_ptr(),
            ..Default::default()
        }];

        let create_info = vk::DeviceCreateInfo {
            queue_create_info_count: queue_create_infos.len() as u32,
            p_queue_create_infos: queue_create_infos.as_ptr(),
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
