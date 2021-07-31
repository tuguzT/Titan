use std::cmp::Ordering;
use std::ffi::CStr;
use std::ops::Deref;
use std::sync::{Mutex, MutexGuard};

use ash::prelude::VkResult;
use ash::version::InstanceV1_0;
use ash::vk;
use owning_ref::MutexGuardRef;

use proc_macro::SlotMappable;

use crate::error::{Error, Result};

use super::super::{
    instance::{self, Instance},
    slotmap::{HasParent, SlotMappable},
    surface::Surface,
    utils::{HasHandle, HasLoader},
};

slotmap::new_key_type! {
    pub struct Key;
}

#[derive(SlotMappable)]
pub struct PhysicalDevice {
    #[key]
    key: Key,
    properties: vk::PhysicalDeviceProperties,
    features: vk::PhysicalDeviceFeatures,
    memory_properties: vk::PhysicalDeviceMemoryProperties,
    queue_family_properties: Vec<vk::QueueFamilyProperties>,
    layer_properties: Vec<vk::LayerProperties>,
    extension_properties: Vec<vk::ExtensionProperties>,
    handle: Mutex<vk::PhysicalDevice>,
    parent_instance: instance::Key,
}

impl HasParent<Instance> for PhysicalDevice {
    fn parent_key(&self) -> instance::Key {
        self.parent_instance
    }
}

impl HasHandle for PhysicalDevice {
    type Handle = vk::PhysicalDevice;

    fn handle(&self) -> Box<dyn Deref<Target = Self::Handle> + '_> {
        Box::new(MutexGuardRef::new(self.handle.lock().unwrap()))
    }
}

impl PhysicalDevice {
    pub unsafe fn new(instance_key: instance::Key, handle: vk::PhysicalDevice) -> Result<Key> {
        let slotmap_instance = SlotMappable::slotmap().read().unwrap();
        let instance: &Instance = slotmap_instance
            .get(instance_key)
            .expect("instance not found");

        let instance_loader = instance.loader();
        let properties = instance_loader
            .instance()
            .get_physical_device_properties(handle);
        let features = instance_loader
            .instance()
            .get_physical_device_features(handle);
        let memory_properties = instance_loader
            .instance()
            .get_physical_device_memory_properties(handle);
        let queue_family_properties = instance_loader
            .instance()
            .get_physical_device_queue_family_properties(handle);
        let layer_properties =
            enumerate_device_layer_properties(instance_loader.instance(), handle)?;
        let extension_properties = instance_loader
            .instance()
            .enumerate_device_extension_properties(handle)?;

        let mut slotmap = SlotMappable::slotmap().write().unwrap();
        let key = slotmap.insert_with_key(|key| Self {
            key,
            handle: Mutex::new(handle),
            properties,
            features,
            queue_family_properties,
            memory_properties,
            layer_properties,
            extension_properties,
            parent_instance: instance_key,
        });
        Ok(key)
    }

    pub fn handle(&self) -> MutexGuard<vk::PhysicalDevice> {
        self.handle.lock().unwrap()
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
        let has_required_extensions = super::REQUIRED_EXTENSIONS
            .iter()
            .any(|&required_name| extension_properties_names.any(|item| item == required_name));
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
                let inner_flags = &queue_family_properties.queue_flags;
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

    pub fn graphics_family_index(&self) -> Result<u32> {
        let graphics_queue_family_properties =
            self.queue_family_properties_with(vk::QueueFlags::GRAPHICS);
        let graphics_family_index = graphics_queue_family_properties
            .peekable()
            .peek()
            .ok_or_else(|| Error::Other {
                message: String::from("no queues with graphics support"),
                source: None,
            })?
            .0 as u32;
        Ok(graphics_family_index)
    }

    pub fn present_family_index(&self, surface: &Surface) -> Result<u32> {
        let present_queue_family_properties =
            surface.physical_device_queue_family_properties_support(self)?;
        let present_family_index = present_queue_family_properties
            .into_iter()
            .peekable()
            .peek()
            .ok_or_else(|| Error::Other {
                message: String::from("no queues with surface present support"),
                source: None,
            })?
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
