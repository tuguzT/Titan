//! Render utilities for graphics backend for game engine.

use std::collections::HashSet;
use std::iter;
use std::sync::Arc;

use egui::{ClippedMesh, Texture, TextureId};
use image::RgbaImage;
use vulkano::buffer::{BufferUsage, DeviceLocalBuffer};
use vulkano::command_buffer::{
    AutoCommandBufferBuilder, CommandBufferUsage, PrimaryAutoCommandBuffer,
};
use vulkano::device::physical::PhysicalDevice;
use vulkano::device::{Device, DeviceExtensions, Features, Queue};
use vulkano::format::Format;
use vulkano::image::view::ImageView;
use vulkano::image::{ImageDimensions, ImageUsage, ImmutableImage, MipmapsCount, SwapchainImage};
use vulkano::instance::debug::{DebugCallback, MessageSeverity, MessageType};
use vulkano::instance::Instance;
use vulkano::swapchain::{AcquireError, PresentMode, Surface, Swapchain};
use vulkano::sync::{FlushError, GpuFuture, SharingMode};
use vulkano::{swapchain, sync};
use vulkano_win::VkSurfaceBuild;
use winit::dpi::LogicalSize;
use winit::event_loop::EventLoop;
use winit::window::{Window, WindowBuilder};

pub use error::RendererCreationError;
use error::{ImageRegisterError, RenderError, ResizeError, TransferCommandBufferCreationError};

use crate::config::Config;

use super::{
    camera::CameraUBO,
    frame::{
        object_draw::ObjectDrawSystem,
        system::{FrameSystem, Pass},
        ui_draw::UiDrawSystem,
    },
    utils,
};

pub mod error;

/// System that renders all game objects and UI.
#[allow(dead_code)]
pub struct Renderer {
    previous_frame_end: Option<Box<dyn GpuFuture + Send + Sync>>,
    recreate_swapchain: bool,
    camera_ubo: CameraUBO,

    ui_draw_system: UiDrawSystem,
    object_draw_system: ObjectDrawSystem,
    frame_system: FrameSystem,
    uniform_buffers: Vec<Arc<DeviceLocalBuffer<CameraUBO>>>,

    swapchain_images: Vec<Arc<SwapchainImage<Window>>>,
    swapchain: Arc<Swapchain<Window>>,
    graphics_queue: Arc<Queue>,
    present_queue: Arc<Queue>,
    transfer_queue: Arc<Queue>,
    device: Arc<Device>,
    surface: Arc<Surface<Window>>,
    debug_callback: Option<DebugCallback>,
    instance: Arc<Instance>,
}

impl Renderer {
    /// Creates render system.
    pub fn new<T>(config: &Config, event_loop: &EventLoop<T>) -> Result<Self, RendererCreationError>
    where
        T: 'static,
    {
        let instance = utils::create_instance(config)?;
        log::info!(
            "max version of Vulkan instance is {}",
            instance.max_api_version(),
        );

        let debug_callback = config
            .enable_validation()
            .then(|| {
                use super::debug_callback::create_debug_callback as new;
                let debug_callback = new(&instance, MessageSeverity::all(), MessageType::all())?;
                log::info!("debug callback was attached to the instance");
                Result::<_, RendererCreationError>::Ok(debug_callback)
            })
            .transpose()?;

        let surface = WindowBuilder::new()
            .with_title(config.name())
            .with_min_inner_size(LogicalSize::new(250, 100))
            .with_visible(false)
            .build_vk_surface(event_loop, instance.clone())?;
        log::info!("window & surface initialized successfully");

        let physical_devices = PhysicalDevice::enumerate(&instance);
        log::info!("enumerated {} physical devices", physical_devices.len());

        let required_extensions = DeviceExtensions {
            khr_swapchain: true,
            ..DeviceExtensions::none()
        };
        let required_features = Features::none();
        let utils::SuitablePhysicalDevice {
            physical_device,
            graphics_family,
            present_family,
            transfer_family,
        } = utils::suitable_physical_device(
            physical_devices,
            &surface,
            &required_extensions,
            &required_features,
        )
        .ok_or_else(|| RendererCreationError::NoSuitablePhysicalDevice)?;
        log::info!(
            r#"using device "{}" of type "{:?}" with Vulkan version {}"#,
            physical_device.properties().device_name,
            physical_device.properties().device_type,
            physical_device.api_version(),
        );

        let (device, mut queues) = {
            let priorities = 1.0;
            let unique_queue_families = {
                let unique_queue_families: HashSet<_> = [
                    graphics_family.id(),
                    present_family.unwrap_or(graphics_family).id(),
                    transfer_family.unwrap_or(graphics_family).id(),
                ]
                .iter()
                .cloned()
                .collect();
                unique_queue_families.into_iter().map(|family| {
                    (
                        physical_device.queue_family_by_id(family).unwrap(),
                        priorities,
                    )
                })
            };
            let required_extensions = physical_device
                .required_extensions()
                .union(&required_extensions);
            Device::new(
                physical_device,
                &required_features,
                &required_extensions,
                unique_queue_families,
            )?
        };
        let graphics_queue = queues.next().unwrap();
        let present_queue = queues.next().unwrap_or_else(|| graphics_queue.clone());
        let transfer_queue = queues.next().unwrap_or_else(|| graphics_queue.clone());

        let (swapchain, swapchain_images) = {
            let capabilities = surface.capabilities(physical_device)?;
            let (format, color_space) = utils::suitable_image_format(&capabilities);
            let present_mode = capabilities
                .present_modes
                .iter()
                .find(|&mode| mode == PresentMode::Mailbox)
                .unwrap_or(PresentMode::Fifo);
            let dimensions = if let Some(current_extent) = capabilities.current_extent {
                current_extent
            } else {
                let window_size = surface.window().inner_size();
                let min_width = capabilities.min_image_extent[0];
                let max_width = capabilities.max_image_extent[0];
                let min_height = capabilities.min_image_extent[1];
                let max_height = capabilities.max_image_extent[1];
                [
                    window_size.width.clamp(min_width, max_width),
                    window_size.height.clamp(min_height, max_height),
                ]
            };
            let image_count = {
                let image_count = capabilities.min_image_count + 1;
                if let Some(max_image_count) = capabilities.max_image_count {
                    image_count.max(max_image_count)
                } else {
                    image_count
                }
            };
            let sharing_mode = present_family
                .as_ref()
                .map(|present_family| {
                    (present_family.id() != graphics_family.id()).then(|| {
                        let queues = [&graphics_queue, &present_queue];
                        SharingMode::from(&queues[..])
                    })
                })
                .flatten()
                .unwrap_or_else(|| SharingMode::from(&graphics_queue));
            Swapchain::start(device.clone(), surface.clone())
                .format(format)
                .color_space(color_space)
                .present_mode(present_mode)
                .dimensions(dimensions)
                .num_images(image_count)
                .transform(capabilities.current_transform)
                .sharing_mode(sharing_mode)
                .usage(ImageUsage::color_attachment())
                .build()?
        };

        let uniform_buffers = swapchain_images
            .iter()
            .map(|_| {
                DeviceLocalBuffer::new(
                    device.clone(),
                    BufferUsage::uniform_buffer_transfer_destination(),
                    iter::once(transfer_queue.family()),
                )
            })
            .collect::<Result<Vec<_>, _>>()?;

        let frame_system = FrameSystem::new(graphics_queue.clone(), swapchain.format())?;

        let object_draw_system =
            ObjectDrawSystem::new(graphics_queue.clone(), frame_system.object_subpass())?;

        let ui_draw_system = UiDrawSystem::new(graphics_queue.clone(), frame_system.ui_subpass())?;

        let previous_frame_end = Some(Box::new(sync::now(device.clone())) as Box<_>);
        Ok(Self {
            instance,
            debug_callback,
            surface,
            device,
            graphics_queue,
            present_queue,
            transfer_queue,
            swapchain,
            swapchain_images,
            uniform_buffers,
            frame_system,
            object_draw_system,
            ui_draw_system,
            camera_ubo: CameraUBO::default(),
            previous_frame_end,
            recreate_swapchain: false,
        })
    }

    /// Underlying window of render system.
    pub fn window(&self) -> &Window {
        self.surface.window()
    }

    /// Resize the underlying window and update Vulkan objects.
    pub fn resize(&mut self) -> Result<(), ResizeError> {
        let dimensions = self.window().inner_size().into();

        let (swapchain, swapchain_images) =
            self.swapchain.recreate().dimensions(dimensions).build()?;
        self.swapchain = swapchain;
        self.swapchain_images = swapchain_images;

        self.recreate_swapchain = false;
        Ok(())
    }

    pub fn set_camera_ubo(&mut self, ubo: CameraUBO) {
        self.camera_ubo = ubo;
    }

    /// Create command buffer for transfer operations which will be executed
    /// before actual rendering.
    fn transfer_cb(
        &self,
        image_index: usize,
    ) -> Result<PrimaryAutoCommandBuffer, TransferCommandBufferCreationError> {
        let uniform_buffer = self.uniform_buffers[image_index].clone();

        let mut builder = AutoCommandBufferBuilder::primary(
            self.device.clone(),
            self.transfer_queue.family(),
            CommandBufferUsage::OneTimeSubmit,
        )?;
        builder.update_buffer(uniform_buffer, Box::new(self.camera_ubo))?;
        Ok(builder.build()?)
    }

    pub fn register_ui_image(
        &mut self,
        image: &RgbaImage,
    ) -> Result<TextureId, ImageRegisterError> {
        let pixels: Vec<_> = image.pixels().flat_map(|p| p.0).collect();
        let (image, future) = ImmutableImage::from_iter(
            pixels,
            ImageDimensions::Dim2d {
                width: image.width(),
                height: image.height(),
                array_layers: 1,
            },
            MipmapsCount::One,
            Format::R8G8B8A8_SRGB, // todo: remove hardcoded format
            self.transfer_queue.clone(),
        )?;
        future.flush()?;
        let image_view = ImageView::new(image)?;
        Ok(self.ui_draw_system.register_texture(image_view)?)
    }

    /// Render new frame into the underlying window.
    pub fn render(
        &mut self,
        mut ui: Option<(Vec<ClippedMesh>, Arc<Texture>)>,
    ) -> Result<(), RenderError> {
        self.previous_frame_end.as_mut().unwrap().cleanup_finished();
        if self.recreate_swapchain {
            self.resize()?;
        }

        let (image_index, suboptimal, acquire_future) =
            match swapchain::acquire_next_image(self.swapchain.clone(), None) {
                Ok(r) => r,
                Err(AcquireError::OutOfDate) => {
                    self.recreate_swapchain = true;
                    return Ok(());
                }
                Err(err) => return Err(RenderError::AcquireNextImage(err)),
            };
        self.recreate_swapchain = suboptimal;

        let transfer_command_buffer = self.transfer_cb(image_index)?;
        let previous_frame_end = self.previous_frame_end.take().unwrap();
        let before_future = previous_frame_end
            .join(acquire_future)
            .then_execute(self.transfer_queue.clone(), transfer_command_buffer)?
            .then_signal_semaphore();

        let scale_factor = self.window().scale_factor() as f32;
        let graphics_future = {
            let mut frame = self
                .frame_system
                .frame(before_future, self.swapchain_images[image_index].clone())?;
            let mut graphics_future = Box::new(sync::now(self.device.clone())) as Box<_>;
            while let Some(next_pass) = frame.next_pass()? {
                match next_pass {
                    Pass::Deferred(mut draw_pass) => {
                        let uniform_buffer = self.uniform_buffers[image_index].clone();
                        let command_buffer = self
                            .object_draw_system
                            .draw(draw_pass.viewport_size(), uniform_buffer)?;
                        draw_pass.execute(command_buffer)?;
                    }
                    Pass::UI(mut ui_pass) => {
                        if let Some((meshes, texture)) = ui.take() {
                            let command_buffer = self.ui_draw_system.draw(
                                ui_pass.viewport_size(),
                                scale_factor,
                                meshes,
                                texture,
                            )?;
                            ui_pass.execute(command_buffer)?;
                        }
                    }
                    Pass::Finished(future) => {
                        graphics_future = future;
                    }
                }
            }
            graphics_future
        };

        let future = graphics_future
            .then_swapchain_present(
                self.present_queue.clone(),
                self.swapchain.clone(),
                image_index,
            )
            .then_signal_fence_and_flush();
        match future {
            Ok(future) => {
                self.previous_frame_end = Some(Box::new(future));
                Ok(())
            }
            Err(FlushError::OutOfDate) => {
                self.recreate_swapchain = true;
                self.previous_frame_end = Some(Box::new(sync::now(self.device.clone())));
                Ok(())
            }
            Err(err) => {
                self.previous_frame_end = Some(Box::new(sync::now(self.device.clone())));
                Err(RenderError::SubmitQueue(err))
            }
        }
    }
}
