use std::borrow::Borrow;
use std::error::Error;
use std::ffi::CStr;

use ash::extensions::khr::Swapchain as AshSwapchain;
use ash::vk;
use winit::window::Window;

use proc_macro::SlotMappable;

use super::super::{
    device::{self, Device, PhysicalDevice},
    image,
    instance::Instance,
    slotmap::SlotMappable,
    surface,
    surface::Surface,
    utils, Image,
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
    loader: AshSwapchain,
    parent_device: device::Key,
    parent_surface: surface::Key,
}

impl Swapchain {
    pub fn new(
        window: &Window,
        device_key: device::Key,
        surface_key: surface::Key,
    ) -> Result<Key, Box<dyn Error>> {
        let slotmap_device = Device::slotmap().read()?;
        let device = slotmap_device
            .get(device_key)
            .ok_or_else(|| utils::make_error("device not found"))?;
        let slotmap_surface = Surface::slotmap().read()?;
        let surface = slotmap_surface
            .get(surface_key)
            .ok_or_else(|| utils::make_error("surface not found"))?;

        let physical_device_key = device.parent_physical_device();
        let slotmap_physical_device = PhysicalDevice::slotmap().read()?;
        let physical_device = slotmap_physical_device
            .get(physical_device_key)
            .ok_or_else(|| utils::make_error("physical device not found"))?;

        let surface_instance = surface.parent_instance();
        let physical_device_instance = physical_device.parent_instance();
        if surface_instance != physical_device_instance {
            return Err(
                utils::make_error("surface and physical device parents must be the same").into(),
            );
        }

        let slotmap_instance = Instance::slotmap().read()?;
        let instance = slotmap_instance
            .get(surface_instance)
            .ok_or_else(|| utils::make_error("instance not found"))?;

        let formats = surface.physical_device_formats(physical_device)?;
        let suitable_format = Self::pick_format(&formats)
            .ok_or_else(|| utils::make_error("no suitable format found"))?;

        let present_modes = surface.physical_device_present_modes(physical_device)?;
        let suitable_present_mode = Self::pick_present_mode(&present_modes);

        let capabilities = surface.physical_device_capabilities(physical_device)?;
        let suitable_extent = Self::pick_extent(window, &capabilities);

        let mut image_count = capabilities.min_image_count + 1;
        if capabilities.max_image_count > 0 && image_count > capabilities.max_image_count {
            image_count = capabilities.max_image_count
        };
        let mut create_info = vk::SwapchainCreateInfoKHR::builder()
            .surface(surface.handle())
            .min_image_count(image_count)
            .image_format(suitable_format.format)
            .image_color_space(suitable_format.color_space)
            .image_extent(suitable_extent)
            .image_array_layers(1)
            .image_usage(vk::ImageUsageFlags::COLOR_ATTACHMENT)
            .pre_transform(capabilities.current_transform)
            .composite_alpha(vk::CompositeAlphaFlagsKHR::OPAQUE)
            .present_mode(*suitable_present_mode)
            .clipped(true);
        let graphics_index = physical_device.graphics_family_index()?;
        let present_index = physical_device.present_family_index(surface)?;
        let queue_family_indices = [graphics_index, present_index];
        if graphics_index != present_index {
            create_info = create_info
                .image_sharing_mode(vk::SharingMode::CONCURRENT)
                .queue_family_indices(queue_family_indices.borrow());
        } else {
            create_info = create_info.image_sharing_mode(vk::SharingMode::EXCLUSIVE);
        }

        let loader = AshSwapchain::new(instance.loader(), device.loader());
        let handle = unsafe { loader.create_swapchain(&create_info, None)? };

        let mut slotmap = SlotMappable::slotmap().write()?;
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

    pub fn loader(&self) -> &AshSwapchain {
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

    pub fn enumerate_images(&self) -> Result<Vec<image::Key>, Box<dyn Error>> {
        let device = self.parent_device();
        let handles = unsafe { self.loader.get_swapchain_images(self.handle)? };
        handles
            .into_iter()
            .map(|handle| unsafe { Image::from_raw(device, handle) })
            .collect()
    }

    fn pick_format(formats: &Vec<vk::SurfaceFormatKHR>) -> Option<&vk::SurfaceFormatKHR> {
        let found_format = formats.iter().find(|format| {
            format.format == vk::Format::B8G8R8A8_SRGB
                || format.color_space == vk::ColorSpaceKHR::SRGB_NONLINEAR
        });
        if let None = found_format {
            formats.first()
        } else {
            found_format
        }
    }

    fn pick_present_mode(present_modes: &Vec<vk::PresentModeKHR>) -> &vk::PresentModeKHR {
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
        AshSwapchain::name()
    }
}

impl Drop for Swapchain {
    fn drop(&mut self) {
        unsafe {
            self.loader.destroy_swapchain(self.handle, None);
        }
    }
}
