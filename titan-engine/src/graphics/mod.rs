//! Graphics utilities and backend based on Vulkan API for game engine.

use std::collections::HashSet;
use std::sync::Arc;

use palette::Srgba;
use ultraviolet::Vec3;
use vulkano::buffer::{BufferUsage, CpuAccessibleBuffer, ImmutableBuffer};
use vulkano::command_buffer::{
    AutoCommandBufferBuilder, CommandBufferUsage, DynamicState, PrimaryAutoCommandBuffer,
    SubpassContents,
};
use vulkano::descriptor_set::{DescriptorSet, PersistentDescriptorSet};
use vulkano::device::physical::PhysicalDevice;
use vulkano::device::{Device, DeviceExtensions, Features, Queue};
use vulkano::format::{ClearValue, Format};
use vulkano::image::view::ImageView;
use vulkano::image::{AttachmentImage, ImageAccess, ImageUsage, SwapchainImage};
use vulkano::instance::debug::{DebugCallback, MessageSeverity, MessageType};
use vulkano::instance::Instance;
use vulkano::pipeline::vertex::BuffersDefinition;
use vulkano::pipeline::viewport::Viewport;
use vulkano::pipeline::{GraphicsPipeline, GraphicsPipelineAbstract};
use vulkano::render_pass::{Framebuffer, FramebufferAbstract, RenderPass, Subpass};
use vulkano::swapchain::{AcquireError, ColorSpace, PresentMode, Surface, Swapchain};
use vulkano::sync::{FlushError, GpuFuture, SharingMode};
use vulkano::{swapchain, sync};
use vulkano_win::VkSurfaceBuild;
use winit::dpi::LogicalSize;
use winit::event_loop::EventLoop;
use winit::window::{Window, WindowBuilder};

use crate::config::Config;
use crate::error::{Error, Result};

use self::camera::CameraUBO;
use self::vertex::Vertex;

pub(crate) mod camera;

mod debug_callback;
mod shader;
mod utils;
mod vertex;

const fn indices() -> [u16; 12] {
    [0, 1, 2, 2, 3, 0, 4, 5, 6, 6, 7, 4]
}

fn vertices() -> [Vertex; 8] {
    [
        Vertex::new(Vec3::new(-0.5, -0.5, 0.0), Srgba::new(1.0, 0.0, 0.0, 1.0)),
        Vertex::new(Vec3::new(0.5, -0.5, 0.0), Srgba::new(0.0, 1.0, 0.0, 1.0)),
        Vertex::new(Vec3::new(0.5, 0.5, 0.0), Srgba::new(0.0, 0.0, 1.0, 1.0)),
        Vertex::new(Vec3::new(-0.5, 0.5, 0.0), Srgba::new(1.0, 1.0, 1.0, 1.0)),
        Vertex::new(Vec3::new(-0.5, -0.5, -0.5), Srgba::new(1.0, 0.0, 0.0, 1.0)),
        Vertex::new(Vec3::new(0.5, -0.5, -0.5), Srgba::new(0.0, 1.0, 0.0, 1.0)),
        Vertex::new(Vec3::new(0.5, 0.5, -0.5), Srgba::new(0.0, 0.0, 1.0, 1.0)),
        Vertex::new(Vec3::new(-0.5, 0.5, -0.5), Srgba::new(1.0, 1.0, 1.0, 1.0)),
    ]
}

/// System that renders all game objects and UI.
// TODO: UI rendering
pub struct Renderer {
    previous_frame_end: Option<Box<dyn GpuFuture + Send + Sync>>,
    recreate_swapchain: bool,
    camera_ubo: CameraUBO,

    descriptor_sets: Vec<Arc<dyn DescriptorSet + Send + Sync>>,
    uniform_buffers: Vec<Arc<CpuAccessibleBuffer<CameraUBO>>>,
    index_buffer: Arc<ImmutableBuffer<[u16]>>,
    vertex_buffer: Arc<ImmutableBuffer<[Vertex]>>,

    framebuffers: Vec<Arc<dyn FramebufferAbstract + Send + Sync>>,
    dynamic_state: DynamicState,
    graphics_pipeline: Arc<GraphicsPipeline<BuffersDefinition>>,
    render_pass: Arc<RenderPass>,
    depth_image: Arc<AttachmentImage>,
    swapchain_images: Vec<Arc<SwapchainImage<Window>>>,
    swapchain: Arc<Swapchain<Window>>,
    graphics_queue: Arc<Queue>,
    present_queue: Arc<Queue>,
    transfer_queue: Arc<Queue>,
    device: Arc<Device>,
    surface: Arc<Surface<Window>>,
    _debug_callback: Option<DebugCallback>,
    _instance: Arc<Instance>,
}

impl Renderer {
    /// Creates render system.
    pub fn new<T>(config: &Config, event_loop: &EventLoop<T>) -> Result<Self>
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
                use self::debug_callback::create_debug_callback as new;
                let debug_callback = new(&instance, MessageSeverity::all(), MessageType::all())?;
                log::info!("debug callback was attached to the instance");
                Result::Ok(debug_callback)
            })
            .transpose()?;

        let surface = WindowBuilder::new()
            .with_title(config.name())
            .with_min_inner_size(LogicalSize::new(250, 100))
            .with_visible(false)
            .build_vk_surface(event_loop, instance.clone())
            .map_err(|err| Error::new("surface creation failure", err))?;
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
        .ok_or_else(|| Error::from("no suitable physical device were found"))?;
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
            )
            .map_err(|err| Error::new("device creation failure", err))?
        };
        let graphics_queue = queues.next().unwrap();
        let present_queue = queues.next().unwrap_or_else(|| graphics_queue.clone());
        let transfer_queue = queues.next().unwrap_or_else(|| graphics_queue.clone());

        let (swapchain, swapchain_images) = {
            let caps = surface
                .capabilities(physical_device)
                .map_err(|err| Error::new("failed to get surface capabilities", err))?;
            let (format, color_space) = {
                let formats = caps.supported_formats;
                *formats
                    .iter()
                    .find(|(format, color_space)| {
                        *format == Format::B8G8R8A8Srgb && *color_space == ColorSpace::SrgbNonLinear
                    })
                    .unwrap_or_else(|| &formats[0])
            };
            let present_mode = caps
                .present_modes
                .iter()
                .find(|&mode| mode == PresentMode::Mailbox)
                .unwrap_or(PresentMode::Fifo);
            let dimensions = if let Some(current_extent) = caps.current_extent {
                current_extent
            } else {
                let window_size = surface.window().inner_size();
                let min_width = caps.min_image_extent[0];
                let max_width = caps.max_image_extent[0];
                let min_height = caps.min_image_extent[1];
                let max_height = caps.max_image_extent[1];
                [
                    window_size.width.clamp(min_width, max_width),
                    window_size.height.clamp(min_height, max_height),
                ]
            };
            let image_count = {
                let image_count = caps.min_image_count + 1;
                if let Some(max_image_count) = caps.max_image_count {
                    image_count.max(max_image_count)
                } else {
                    image_count
                }
            };
            let sharing_mode = if present_family.is_some() {
                let queues = [&graphics_queue, &present_queue];
                SharingMode::from(&queues[..])
            } else {
                SharingMode::from(&graphics_queue)
            };
            Swapchain::start(device.clone(), surface.clone())
                .format(format)
                .color_space(color_space)
                .present_mode(present_mode)
                .dimensions(dimensions)
                .num_images(image_count)
                .transform(caps.current_transform)
                .sharing_mode(sharing_mode)
                .usage(ImageUsage::color_attachment())
                .build()
                .map_err(|err| Error::new("swapchain creation failure", err))?
        };

        let depth_format = {
            let suitable_formats = [
                Format::D32Sfloat,
                Format::D32Sfloat_S8Uint,
                Format::D24Unorm_S8Uint,
            ];
            *suitable_formats
                .iter()
                .find(|format| {
                    let properties = format.properties(physical_device);
                    properties.optimal_tiling_features.depth_stencil_attachment
                })
                .unwrap_or(&Format::D16Unorm)
        };
        let depth_image = AttachmentImage::with_usage(
            device.clone(),
            swapchain.dimensions(),
            depth_format,
            ImageUsage::depth_stencil_attachment(),
        )
        .map_err(|err| Error::new("depth image creation failure", err))?;

        let render_pass = Arc::new(
            vulkano::single_pass_renderpass! {
                device.clone(),
                attachments: {
                    color: {
                        load: Clear,
                        store: Store,
                        format: swapchain.format(),
                        samples: 1,
                    },
                    depth: {
                        load: Clear,
                        store: DontCare,
                        format: depth_image.format(),
                        samples: 1,
                        initial_layout: ImageLayout::Undefined,
                        final_layout: ImageLayout::DepthStencilAttachmentOptimal,
                    }
                },
                pass: {
                    color: [color],
                    depth_stencil: {depth}
                }
            }
            .map_err(|err| Error::new("render pass creation failure", err))?,
        );

        let graphics_pipeline = {
            use self::shader::default::{fragment, vertex};

            let vert_shader_module = vertex::Shader::load(device.clone())
                .map_err(|err| Error::new("vertex shader module creation failure", err))?;
            let frag_shader_module = fragment::Shader::load(device.clone())
                .map_err(|err| Error::new("fragment shader module creation failure", err))?;

            Arc::new(
                GraphicsPipeline::start()
                    .vertex_input_single_buffer::<Vertex>()
                    .vertex_shader(vert_shader_module.main_entry_point(), ())
                    .fragment_shader(frag_shader_module.main_entry_point(), ())
                    .triangle_list()
                    .primitive_restart(false)
                    .viewports_dynamic_scissors_irrelevant(1)
                    .depth_stencil_simple_depth()
                    .cull_mode_back()
                    .render_pass(Subpass::from(render_pass.clone(), 0).unwrap())
                    .build(device.clone())
                    .map_err(|err| Error::new("graphics pipeline creation failure", err))?,
            )
        };

        let mut dynamic_state = DynamicState::none();
        let framebuffers = Self::create_framebuffers(
            swapchain_images.as_slice(),
            render_pass.clone(),
            &mut dynamic_state,
            &depth_image,
        )?;

        let vertex_buffer = {
            let (vertex_buffer, future) = ImmutableBuffer::from_iter(
                self::vertices().iter().cloned(),
                BufferUsage::vertex_buffer(),
                graphics_queue.clone(),
            )
            .map_err(|err| Error::new("vertex buffer creation failure", err))?;
            future
                .flush()
                .map_err(|err| Error::new("vertex buffer creation failure", err))?;
            vertex_buffer
        };

        let index_buffer = {
            let (index_buffer, future) = ImmutableBuffer::from_iter(
                self::indices().iter().cloned(),
                BufferUsage::index_buffer(),
                graphics_queue.clone(),
            )
            .map_err(|err| Error::new("index buffer creation failure", err))?;
            future
                .flush()
                .map_err(|err| Error::new("index buffer creation failure", err))?;
            index_buffer
        };

        let uniform_buffers = swapchain_images
            .iter()
            .map(|_| {
                CpuAccessibleBuffer::from_data(
                    device.clone(),
                    BufferUsage::uniform_buffer_transfer_destination(),
                    false,
                    CameraUBO::default(),
                )
                .map_err(|err| Error::new("uniform buffer creation failure", err))
            })
            .collect::<Result<Vec<_>>>()?;
        let descriptor_sets = uniform_buffers
            .iter()
            .map(|uniform_buffer| {
                let layout = &graphics_pipeline.layout().descriptor_set_layouts()[0];
                Ok(Arc::new(
                    PersistentDescriptorSet::start(layout.clone())
                        .add_buffer(uniform_buffer.clone())
                        .map_err(|err| Error::new("descriptor set creation failure", err))?
                        .build()
                        .map_err(|err| Error::new("descriptor set creation failure", err))?,
                ) as Arc<_>)
            })
            .collect::<Result<Vec<_>>>()?;

        let previous_frame_end = Some(Box::new(sync::now(device.clone())) as Box<_>);
        Ok(Self {
            _instance: instance,
            _debug_callback: debug_callback,
            surface,
            device,
            graphics_queue,
            present_queue,
            transfer_queue,
            swapchain,
            swapchain_images,
            depth_image,
            render_pass,
            graphics_pipeline,
            dynamic_state,
            framebuffers,
            vertex_buffer,
            index_buffer,
            uniform_buffers,
            descriptor_sets,
            camera_ubo: CameraUBO::default(),
            previous_frame_end,
            recreate_swapchain: false,
        })
    }

    /// (Re)create framebuffers in which game content will be rendered.
    fn create_framebuffers(
        images: &[Arc<SwapchainImage<Window>>],
        render_pass: Arc<RenderPass>,
        dynamic_state: &mut DynamicState,
        depth_image: &Arc<AttachmentImage>,
    ) -> Result<Vec<Arc<dyn FramebufferAbstract + Send + Sync>>> {
        let dimensions = images[0].dimensions();

        let viewport = Viewport {
            origin: [0.0, 0.0],
            dimensions: [dimensions.width() as f32, dimensions.height() as f32],
            depth_range: 0.0..1.0,
        };
        dynamic_state.viewports = Some(vec![viewport]);

        let depth_image_view = ImageView::new(depth_image.clone())
            .map_err(|err| Error::new("depth image view creation failure", err))?;
        images
            .iter()
            .map(|image| {
                let image_view = ImageView::new(image.clone())
                    .map_err(|err| Error::new("image view creation failure", err))?;
                let framebuffer = Framebuffer::start(render_pass.clone())
                    .add(image_view)
                    .map_err(|err| Error::new("failed to add an attachment to framebuffer", err))?
                    .add(depth_image_view.clone())
                    .map_err(|err| Error::new("failed to add a depth image to framebuffer", err))?
                    .build()
                    .map_err(|err| Error::new("framebuffer creation failure", err))?;
                Ok(Arc::new(framebuffer) as Arc<_>)
            })
            .collect()
    }

    /// Underlying window of render system.
    pub fn window(&self) -> &Window {
        self.surface.window()
    }

    /// Resize the underlying window and update Vulkan objects.
    pub fn resize(&mut self) -> Result<()> {
        let dimensions = self.window().inner_size().into();

        let (swapchain, swapchain_images) = self
            .swapchain
            .recreate()
            .dimensions(dimensions)
            .build()
            .map_err(|err| Error::new("failed to recreate swapchain", err))?;
        self.swapchain = swapchain;
        self.swapchain_images = swapchain_images;

        self.depth_image = AttachmentImage::with_usage(
            self.device.clone(),
            self.swapchain.dimensions(),
            self.depth_image.format(),
            ImageUsage::depth_stencil_attachment(),
        )
        .map_err(|err| Error::new("depth image creation failure", err))?;
        self.framebuffers = Self::create_framebuffers(
            self.swapchain_images.as_slice(),
            self.render_pass.clone(),
            &mut self.dynamic_state,
            &self.depth_image,
        )?;

        self.recreate_swapchain = false;
        Ok(())
    }

    pub fn set_camera_ubo(&mut self, ubo: CameraUBO) {
        self.camera_ubo = ubo;
    }

    /// Create command buffer for transfer operations which will be executed
    /// before actual rendering.
    fn transfer_cb(&self, image_index: usize) -> Result<PrimaryAutoCommandBuffer> {
        let uniform_buffer = self.uniform_buffers[image_index].clone();

        let mut builder = AutoCommandBufferBuilder::primary(
            self.device.clone(),
            self.transfer_queue.family(),
            CommandBufferUsage::OneTimeSubmit,
        )
        .map_err(|err| Error::new("transfer command buffer creation failure", err))?;
        builder
            .update_buffer(uniform_buffer, Box::new(self.camera_ubo))
            .map_err(|err| Error::new("update buffer command creation failure", err))?;
        Ok(builder
            .build()
            .map_err(|err| Error::new("transfer command buffer creation failure", err))?)
    }

    /// Create command buffer for actual rendering operations.
    fn draw_cb(&self, image_index: usize) -> Result<PrimaryAutoCommandBuffer> {
        let framebuffer = self.framebuffers[image_index].clone();
        let clear_values = [
            ClearValue::Float([0.0, 0.0, 0.0, 1.0]),
            ClearValue::Depth(1.0),
        ];
        let descriptor_set = self.descriptor_sets[image_index].clone();

        let mut builder = AutoCommandBufferBuilder::primary(
            self.device.clone(),
            self.graphics_queue.family(),
            CommandBufferUsage::OneTimeSubmit,
        )
        .map_err(|err| Error::new("draw command buffer creation failure", err))?;
        builder
            .begin_render_pass(framebuffer, SubpassContents::Inline, clear_values)
            .map_err(|err| Error::new("begin render pass failure", err))?
            .draw_indexed(
                self.graphics_pipeline.clone(),
                &self.dynamic_state,
                self.vertex_buffer.clone(),
                self.index_buffer.clone(),
                descriptor_set,
                (),
            )
            .map_err(|err| Error::new("draw command failure", err))?
            .end_render_pass()
            .map_err(|err| Error::new("end render pass failure", err))?;
        Ok(builder
            .build()
            .map_err(|err| Error::new("draw command buffer creation failure", err))?)
    }

    /// Render new frame into the underlying window.
    pub fn render(&mut self) -> Result<()> {
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
                Err(err) => return Err(Error::new("failed to acquire next image", err)),
            };
        self.recreate_swapchain = suboptimal;

        let transfer_command_buffer = self.transfer_cb(image_index)?;
        let draw_command_buffer = self.draw_cb(image_index)?;
        let previous_frame_end = self.previous_frame_end.take().unwrap();

        let future = previous_frame_end
            .join(acquire_future)
            .then_execute(self.transfer_queue.clone(), transfer_command_buffer)
            .map_err(|err| Error::new("transfer command buffer execution failure", err))?
            .then_signal_semaphore()
            .then_execute(self.graphics_queue.clone(), draw_command_buffer)
            .map_err(|err| Error::new("draw command buffer execution failure", err))?
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
                Err(Error::new("failed to submit commands", err))
            }
        }
    }
}
