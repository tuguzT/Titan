use std::error::Error;
use std::sync::Arc;

use ash::vk;

use device::{Device, PhysicalDevice, Queue};
use ext::{DebugUtils, Swapchain};
use image::{Image, ImageView};
use instance::Instance;
use surface::Surface;

use super::config::Config;
use super::impl_window::Window;

mod device;
mod ext;
mod image;
mod instance;
mod surface;
mod utils;

pub struct Renderer {
    swapchain_image_views: Vec<Arc<ImageView>>,
    swapchain_images: Vec<Arc<Image>>,
    swapchain: Arc<Swapchain>,
    device_queues: Vec<Arc<Queue>>,
    device: Arc<Device>,
    physical_device: Arc<PhysicalDevice>,
    surface: Arc<Surface>,
    debug_utils: Arc<Option<DebugUtils>>,
    instance: Arc<Instance>,
}

impl Renderer {
    pub fn new(config: &Config, window: &Window) -> Result<Self, Box<dyn Error>> {
        let instance = Arc::new(Instance::new(config, window.window())?);
        log::info!(
            "instance was created! Vulkan API version is {}",
            instance.version(),
        );
        let debug_utils = Arc::new(if instance::ENABLE_VALIDATION {
            log::info!("debug_utils was attached to instance");
            Some(DebugUtils::new(&instance)?)
        } else {
            None
        });
        let surface = Arc::new(Surface::new(&instance, window.window())?);

        let mut physical_devices: Vec<PhysicalDevice> =
            Instance::enumerate_physical_devices(&instance)?
                .into_iter()
                .filter(|item| {
                    let iter = surface.physical_device_queue_family_properties_support(item);
                    item.is_suitable()
                        && surface.is_suitable(item).unwrap_or(false)
                        && iter.peekable().peek().is_some()
                })
                .collect();
        log::info!(
            "enumerated {} suitable physical devices",
            physical_devices.len(),
        );
        physical_devices.sort_unstable();
        physical_devices.reverse();
        let physical_device = Arc::new(
            physical_devices
                .into_iter()
                .next()
                .ok_or_else(|| utils::make_error("no suitable physical devices were found"))?,
        );
        let device = Arc::new(Device::new(&surface, &physical_device)?);
        let device_queues = Device::enumerate_queues(&device)
            .into_iter()
            .map(|queue| Arc::new(queue))
            .collect();

        let swapchain = Arc::new(Swapchain::new(window, &device, &surface)?);
        let swapchain_images = Swapchain::enumerate_images(&swapchain)?
            .into_iter()
            .map(|image| Arc::new(image))
            .collect::<Vec<_>>();
        let swapchain_image_views = swapchain_images
            .iter()
            .map(|image| unsafe {
                let create_info = vk::ImageViewCreateInfo::builder()
                    .image(image.handle())
                    .view_type(vk::ImageViewType::TYPE_2D)
                    .format(swapchain.format().format)
                    .components(vk::ComponentMapping {
                        r: vk::ComponentSwizzle::IDENTITY,
                        g: vk::ComponentSwizzle::IDENTITY,
                        b: vk::ComponentSwizzle::IDENTITY,
                        a: vk::ComponentSwizzle::IDENTITY,
                    })
                    .subresource_range(vk::ImageSubresourceRange {
                        aspect_mask: vk::ImageAspectFlags::COLOR,
                        base_mip_level: 0,
                        level_count: 1,
                        base_array_layer: 0,
                        layer_count: 1,
                    });
                ImageView::new(image, &create_info).map(|image_view| Arc::new(image_view))
            })
            .collect::<Result<Vec<_>, _>>()?;

        Ok(Self {
            instance,
            debug_utils,
            surface,
            physical_device,
            device,
            device_queues,
            swapchain,
            swapchain_images,
            swapchain_image_views,
        })
    }

    pub fn render(&self) {
        log::trace!("rendering a frame!");
    }
}
