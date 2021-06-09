use std::borrow::Borrow;
use std::error::Error;
use std::ffi::CStr;

use ash::extensions::khr::Swapchain as AshSwapchain;
use ash::vk;

use crate::graphics::{
    device::{Device, PhysicalDevice},
    instance::Instance,
    surface::Surface,
    utils,
};
use crate::impl_window::Window;

pub struct Swapchain {
    loader: AshSwapchain,
    handle: vk::SwapchainKHR,
}

impl Swapchain {
    pub fn new(
        window: &Window,
        instance: &Instance,
        physical_device: &PhysicalDevice,
        device: &Device,
        surface: &Surface,
    ) -> Result<Self, Box<dyn Error>> {
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
        let present_index = physical_device.present_family_index(&surface)?;
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
        Ok(Self { loader, handle })
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
            let window_size = window.window().inner_size();
            vk::Extent2D {
                width: u32::max(
                    capabilities.min_image_extent.width,
                    u32::min(capabilities.max_image_extent.width, window_size.width),
                ),
                height: u32::max(
                    capabilities.min_image_extent.height,
                    u32::min(capabilities.max_image_extent.height, window_size.height),
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
