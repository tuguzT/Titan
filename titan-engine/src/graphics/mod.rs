use std::error::Error;
use std::sync::Arc;

use device::{Device, PhysicalDevice};
use ext::{DebugUtils, Swapchain};
use instance::Instance;
use surface::Surface;

use super::config::Config;
use super::impl_window::Window;

mod device;
mod ext;
mod instance;
mod surface;
mod utils;

pub struct Renderer {
    swapchain: Arc<Swapchain>,
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
        let swapchain = Arc::new(Swapchain::new(window, &device, &surface)?);

        Ok(Self {
            instance,
            debug_utils,
            surface,
            physical_device,
            device,
            swapchain,
        })
    }

    pub fn render(&self) {
        log::trace!("rendering a frame!");
    }
}
