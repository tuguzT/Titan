//! General graphics utilities for game engine.

use std::sync::Arc;

use vulkano::device::physical::{PhysicalDevice, PhysicalDeviceType, QueueFamily};
use vulkano::device::{DeviceExtensions, Features};
use vulkano::format::Format;
use vulkano::instance::{ApplicationInfo, Instance, InstanceCreationError};
use vulkano::swapchain::{Capabilities, ColorSpace, Surface};
use vulkano_win::required_extensions;
use winit::window::Window;

use crate::config::{Config, ENGINE_NAME, ENGINE_VERSION};

/// Convert [`semver::Version`] Version struct into [`vulkano::Version`] struct.
#[inline(always)]
const fn to_vk_version(version: &semver::Version) -> vulkano::Version {
    vulkano::Version {
        major: version.major as u32,
        minor: version.minor as u32,
        patch: version.patch as u32,
    }
}

/// Create instance of Vulkan (with low-level vkInstance handle).
///
/// Will enable `VK_EXT_debug_utils` extension if
/// validation is enabled by config.
///
pub fn create_instance(config: &Config) -> Result<Arc<Instance>, InstanceCreationError> {
    let info = ApplicationInfo {
        application_name: Some(config.name().into()),
        application_version: Some(self::to_vk_version(config.version())),
        engine_name: Some(ENGINE_NAME.into()),
        engine_version: Some(self::to_vk_version(&*ENGINE_VERSION)),
    };
    let extensions = {
        let mut extensions = self::required_extensions();
        if config.enable_validation() {
            extensions.ext_debug_utils = true;
        }
        extensions
    };
    let layers = config
        .enable_validation()
        .then(|| "VK_LAYER_KHRONOS_validation");

    Instance::new(Some(&info), vulkano::Version::V1_2, &extensions, layers)
}

/// Internal struct for representing suitable physical device with its queue families.
pub struct SuitablePhysicalDevice<'a> {
    pub physical_device: PhysicalDevice<'a>,
    pub graphics_family: QueueFamily<'a>,
    pub present_family: Option<QueueFamily<'a>>,
    pub transfer_family: Option<QueueFamily<'a>>,
}

/// Filter suitable physical device from all of them.
///
/// Will check for provided extensions and features support.
///
pub fn suitable_physical_device<'a>(
    physical_devices: impl ExactSizeIterator<Item = PhysicalDevice<'a>>,
    surface: &Arc<Surface<Window>>,
    required_extensions: &DeviceExtensions,
    required_features: &Features,
) -> Option<SuitablePhysicalDevice<'a>> {
    physical_devices
        .filter(|physical_device| {
            let extensions = physical_device.supported_extensions();
            let features = physical_device.supported_features();
            extensions.is_superset_of(required_extensions)
                && features.is_superset_of(required_features)
        })
        .filter_map(|physical_device| {
            let graphics_family = physical_device
                .queue_families()
                .find(QueueFamily::supports_graphics);
            let present_family = physical_device
                .queue_families()
                .find(|&queue| surface.is_supported(queue).unwrap_or(false));
            let transfer_family = physical_device
                .queue_families()
                .find(QueueFamily::explicitly_supports_transfers);
            match (graphics_family, present_family, transfer_family) {
                (Some(graphics_family), Some(present_family), Some(transfer_family)) => {
                    Some(SuitablePhysicalDevice {
                        physical_device,
                        graphics_family,
                        present_family: Some(present_family),
                        transfer_family: Some(transfer_family),
                    })
                }
                (Some(graphics_family), Some(present_family), None) => {
                    Some(SuitablePhysicalDevice {
                        physical_device,
                        graphics_family,
                        present_family: Some(present_family),
                        transfer_family: None,
                    })
                }
                (Some(graphics_family), None, None) => Some(SuitablePhysicalDevice {
                    physical_device,
                    graphics_family,
                    present_family: None,
                    transfer_family: None,
                }),
                _ => None,
            }
        })
        .max_by_key(|suitable| self::score(&suitable.physical_device))
}

/// Calculates internal score of given physical device.
fn score(physical_device: &PhysicalDevice) -> u32 {
    let properties = physical_device.properties();
    let mut score = match properties.device_type {
        PhysicalDeviceType::DiscreteGpu => 10000,
        PhysicalDeviceType::IntegratedGpu => 1000,
        PhysicalDeviceType::VirtualGpu => 100,
        PhysicalDeviceType::Cpu => 10,
        PhysicalDeviceType::Other => 0,
    };
    score += properties.max_image_dimension2_d;
    score
}

/// Depth stencil formats which are suitable for rendering backend.
pub const SUITABLE_DEPTH_STENCIL_FORMATS: [Format; 3] = [
    Format::D32Sfloat,
    Format::D32Sfloat_S8Uint,
    Format::D24Unorm_S8Uint,
];

/// Retrieves suitable depth stencil format (see [`SUITABLE_DEPTH_STENCIL_FORMATS`]),
/// if supported by physical device.
///
/// If none of suitable depth stencil formats are supported,
/// returns [`Format::D16Unorm`] which is guaranteed to be supported.
pub fn suitable_depth_stencil_format(physical_device: PhysicalDevice) -> Format {
    *SUITABLE_DEPTH_STENCIL_FORMATS
        .iter()
        .find(|format| {
            let properties = format.properties(physical_device);
            properties.optimal_tiling_features.depth_stencil_attachment
        })
        .unwrap_or(&Format::D16Unorm)
}

/// Image format which is suitable for rendering backend.
pub const SUITABLE_IMAGE_FORMAT: (Format, ColorSpace) =
    (Format::B8G8R8A8Srgb, ColorSpace::SrgbNonLinear);

/// Retrieves suitable image format if supported by physical device.
///
/// If none of suitable image formats are supported, returns first supported format.
pub fn suitable_image_format(capabilities: &Capabilities) -> (Format, ColorSpace) {
    let formats = &capabilities.supported_formats;
    *formats
        .iter()
        .find(|&&format| SUITABLE_IMAGE_FORMAT == format)
        .unwrap_or_else(|| &formats[0])
}
