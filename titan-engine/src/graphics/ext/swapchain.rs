use std::ffi::CStr;
use std::ops::Deref;

use ash::extensions::khr::Swapchain as SwapchainLoader;
use ash::vk;
use winit::window::Window;

use proc_macro::SlotMappable;

use crate::error::{Error, Result};

use super::super::{
    device::{self, Device, PhysicalDevice},
    image::{self, Image},
    instance::Instance,
    slotmap::SlotMappable,
    surface::{self, Surface},
};

slotmap::new_key_type! {
    pub struct Key;
}

#[derive(SlotMappable)]
pub struct Swapchain {
    key: Key,
    handle: vk::SwapchainKHR,
    format: vk::SurfaceFormatKHR,
    extent: vk::Extent2D,
    loader: SwapchainLoader,
    parent_device: device::Key,
    parent_surface: surface::Key,
}

impl Swapchain {
    pub fn new(window: &Window, device_key: device::Key, surface_key: surface::Key) -> Result<Key> {
        let slotmap_device = SlotMappable::slotmap().read().unwrap();
        let device: &Device = slotmap_device.get(device_key).expect("device not found");
        let slotmap_surface = SlotMappable::slotmap().read().unwrap();
        let surface: &Surface = slotmap_surface.get(surface_key).expect("surface not found");

        let physical_device_key = device.parent_physical_device();
        let slotmap_physical_device = SlotMappable::slotmap().read().unwrap();
        let physical_device: &PhysicalDevice = slotmap_physical_device
            .get(physical_device_key)
            .expect("physical device not found");

        if !surface.is_suitable(physical_device)? {
            return Err(Error::Other {
                message: String::from("surface must be supported by the given device"),
                source: None,
            });
        }

        let surface_instance = surface.parent_instance();
        let physical_device_instance = physical_device.parent_instance();
        if surface_instance != physical_device_instance {
            return Err(Error::Other {
                message: String::from("surface and physical device parents must be the same"),
                source: None,
            });
        }

        let slotmap_instance = SlotMappable::slotmap().read().unwrap();
        let instance: &Instance = slotmap_instance
            .get(surface_instance)
            .expect("instance not found");

        let formats = surface.physical_device_formats(physical_device)?;
        let suitable_format = Self::pick_format(&formats).ok_or_else(|| Error::Other {
            message: String::from("no suitable format found"),
            source: None,
        })?;

        let present_modes = surface.physical_device_present_modes(physical_device)?;
        let suitable_present_mode = Self::pick_present_mode(&present_modes);

        let capabilities = surface.physical_device_capabilities(physical_device)?;
        let suitable_extent = Self::pick_extent(window, &capabilities);

        if suitable_extent.height == 0 || suitable_extent.width == 0 {
            return Err(Error::Other {
                message: String::from("imageExtent width and height must both be non-zero"),
                source: None,
            });
        }

        let min_image_count = {
            let mut min_image_count = capabilities.min_image_count + 1;
            if capabilities.max_image_count > 0 && min_image_count > capabilities.max_image_count {
                min_image_count = capabilities.max_image_count
            };
            min_image_count
        };

        let graphics_index = physical_device.graphics_family_index()?;
        let present_index = physical_device.present_family_index(surface)?;
        let queue_family_indices = [graphics_index, present_index];
        let create_info = {
            let create_info = vk::SwapchainCreateInfoKHR::builder()
                .surface(surface.handle())
                .min_image_count(min_image_count)
                .image_format(suitable_format.format)
                .image_color_space(suitable_format.color_space)
                .image_extent(suitable_extent)
                .image_array_layers(1)
                .image_usage(vk::ImageUsageFlags::COLOR_ATTACHMENT)
                .pre_transform(capabilities.current_transform)
                .composite_alpha(vk::CompositeAlphaFlagsKHR::OPAQUE)
                .present_mode(*suitable_present_mode)
                .clipped(true);
            if graphics_index != present_index {
                create_info
                    .image_sharing_mode(vk::SharingMode::CONCURRENT)
                    .queue_family_indices(&queue_family_indices)
            } else {
                create_info.image_sharing_mode(vk::SharingMode::EXCLUSIVE)
            }
        };

        let loader = instance.loader();
        let loader = SwapchainLoader::new(loader.instance(), device.loader().deref());
        let handle = unsafe { loader.create_swapchain(&create_info, None)? };

        let mut slotmap = SlotMappable::slotmap().write().unwrap();
        let key = slotmap.insert_with_key(|key| Self {
            key,
            loader,
            handle,
            format: *suitable_format,
            extent: suitable_extent,
            parent_device: device_key,
            parent_surface: surface_key,
        });
        Ok(key)
    }

    pub fn loader(&self) -> &SwapchainLoader {
        &self.loader
    }

    pub fn handle(&self) -> vk::SwapchainKHR {
        self.handle
    }

    pub fn parent_device(&self) -> device::Key {
        self.parent_device
    }

    pub fn parent_surface(&self) -> surface::Key {
        self.parent_surface
    }

    pub fn format(&self) -> vk::SurfaceFormatKHR {
        self.format
    }

    pub fn extent(&self) -> vk::Extent2D {
        self.extent
    }

    pub fn enumerate_images(&self) -> Result<Vec<image::Key>> {
        let device = self.parent_device();
        let handles = unsafe { self.loader.get_swapchain_images(self.handle)? };
        handles
            .into_iter()
            .map(|handle| unsafe { Image::from_raw(device, handle) })
            .collect()
    }

    fn pick_format(formats: &[vk::SurfaceFormatKHR]) -> Option<&vk::SurfaceFormatKHR> {
        let found_format = formats.iter().find(|format| {
            format.format == vk::Format::B8G8R8A8_SRGB
                || format.color_space == vk::ColorSpaceKHR::SRGB_NONLINEAR
        });
        if found_format.is_none() {
            formats.first()
        } else {
            found_format
        }
    }

    fn pick_present_mode(present_modes: &[vk::PresentModeKHR]) -> &vk::PresentModeKHR {
        let found_mode = present_modes
            .iter()
            .find(|&&mode| mode == vk::PresentModeKHR::MAILBOX);
        found_mode.unwrap_or(&vk::PresentModeKHR::FIFO)
    }

    fn pick_extent(window: &Window, capabilities: &vk::SurfaceCapabilitiesKHR) -> vk::Extent2D {
        if capabilities.current_extent.width != u32::MAX {
            capabilities.current_extent
        } else {
            let window_size = window.inner_size();
            vk::Extent2D {
                width: window_size.width.clamp(
                    capabilities.min_image_extent.width,
                    capabilities.max_image_extent.width,
                ),
                height: window_size.height.clamp(
                    capabilities.min_image_extent.height,
                    capabilities.max_image_extent.height,
                ),
            }
        }
    }

    pub fn name() -> &'static CStr {
        SwapchainLoader::name()
    }
}

impl Drop for Swapchain {
    fn drop(&mut self) {
        unsafe {
            self.loader.destroy_swapchain(self.handle, None);
        }
    }
}
