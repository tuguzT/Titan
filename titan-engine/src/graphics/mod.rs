use std::any::Any;
use std::collections::HashSet;
use std::sync::Arc;
use std::time::Instant;

use glam::{Mat4, Vec3};
use palette::Srgba;
use vulkano::buffer::{BufferUsage, CpuAccessibleBuffer, ImmutableBuffer};
use vulkano::command_buffer::{
    AutoCommandBufferBuilder, CommandBufferUsage, DynamicState, SubpassContents,
};
use vulkano::descriptor_set::{DescriptorSet, PersistentDescriptorSet};
use vulkano::device::physical::{PhysicalDevice, PhysicalDeviceType};
use vulkano::device::{Device, DeviceExtensions, Features, Queue};
use vulkano::format::{ClearValue, Format};
use vulkano::image::view::ImageView;
use vulkano::image::{AttachmentImage, ImageAccess, ImageUsage, SwapchainImage};
use vulkano::instance::debug::{DebugCallback, MessageSeverity, MessageType};
use vulkano::instance::{ApplicationInfo, Instance};
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

use crate::{
    config::{Config, ENGINE_NAME, ENGINE_VERSION},
    error::{Error, Result},
};

use self::camera::CameraUBO;
use self::vertex::Vertex;

mod camera;
mod debug_callback;
mod shader;
mod utils;
mod vertex;

const ENABLE_VALIDATION: bool = cfg!(debug_assertions);

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

pub struct Renderer {
    previous_frame_end: Option<Box<dyn GpuFuture + Send + Sync>>,
    recreate_swapchain: bool,
    start_time: Instant,

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
    pub fn new(config: &Config, event_loop: &EventLoop<impl Any>) -> Result<Self> {
        let instance = {
            let info = ApplicationInfo {
                application_name: Some(config.name().into()),
                application_version: Some(utils::to_vk_version(config.version())),
                engine_name: Some(ENGINE_NAME.into()),
                engine_version: Some(utils::to_vk_version(&*ENGINE_VERSION)),
            };
            let extensions = {
                let mut extensions = vulkano_win::required_extensions();
                if ENABLE_VALIDATION {
                    extensions.ext_debug_utils = true;
                }
                extensions
            };
            let layers = ENABLE_VALIDATION.then(|| "VK_LAYER_KHRONOS_validation");
            Instance::new(Some(&info), vulkano::Version::V1_2, &extensions, layers)
                .map_err(|err| Error::new("instance creation failure", err))
        }?;
        log::info!(
            "max version of Vulkan instance is {}",
            instance.max_api_version(),
        );

        let debug_callback = if ENABLE_VALIDATION {
            let debug_callback = DebugCallback::new(
                &instance,
                MessageSeverity::all(),
                MessageType::all(),
                self::debug_callback::callback,
            )
            .map_err(|err| Error::new("debug callback creation failure", err))?;
            log::info!("debug callback was attached to the instance");
            Some(debug_callback)
        } else {
            None
        };

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
        let (physical_device, graphics_family, present_family, transfer_family) = physical_devices
            .filter(|device: &PhysicalDevice| {
                let extensions = device.supported_extensions();
                extensions.is_superset_of(&required_extensions)
            })
            .filter_map(|device| {
                let graphics_family = device
                    .queue_families()
                    .find(|queue| queue.supports_graphics());
                let present_family = device
                    .queue_families()
                    .find(|&queue| surface.is_supported(queue).unwrap_or(false));
                let transfer_family = device
                    .queue_families()
                    .find(|queue| queue.explicitly_supports_transfers());
                match (graphics_family, present_family, transfer_family) {
                    (Some(graphics_family), Some(present_family), Some(transfer_family)) => {
                        Some((device, graphics_family, present_family, transfer_family))
                    }
                    _ => None,
                }
            })
            .max_by_key(|(device, _, _, _)| {
                let mut score = match device.properties().device_type {
                    PhysicalDeviceType::DiscreteGpu => 10000,
                    PhysicalDeviceType::IntegratedGpu => 1000,
                    PhysicalDeviceType::VirtualGpu => 100,
                    PhysicalDeviceType::Cpu => 10,
                    PhysicalDeviceType::Other => 0,
                };
                score += device.properties().max_image_dimension2_d;
                score
            })
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
                    present_family.id(),
                    transfer_family.id(),
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
            Device::new(
                physical_device,
                &required_features,
                &physical_device
                    .required_extensions()
                    .union(&required_extensions),
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
                [
                    window_size
                        .width
                        .clamp(caps.min_image_extent[0], caps.max_image_extent[0]),
                    window_size
                        .height
                        .clamp(caps.min_image_extent[1], caps.max_image_extent[1]),
                ]
            };
            let image_count = {
                let mut image_count = caps.min_image_count + 1;
                if caps.max_image_count.is_some() && image_count > caps.max_image_count.unwrap() {
                    image_count = caps.max_image_count.unwrap();
                }
                image_count
            };
            let sharing_mode: SharingMode = if graphics_family != present_family {
                vec![&graphics_queue, &present_queue].as_slice().into()
            } else {
                (&graphics_queue).into()
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

        use self::shader::default as shader;
        let vert_shader_module = shader::vertex::Shader::load(device.clone())
            .map_err(|err| Error::new("vertex shader module creation failure", err))?;
        let frag_shader_module = shader::fragment::Shader::load(device.clone())
            .map_err(|err| Error::new("fragment shader module creation failure", err))?;
        let graphics_pipeline = Arc::new(
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
        );

        let mut dynamic_state = DynamicState::none();
        let framebuffers = Self::create_framebuffers(
            swapchain_images.as_slice(),
            render_pass.clone(),
            &mut dynamic_state,
            &depth_image,
        )?;

        let vertex_buffer = {
            let (vertex_buffer, future) = ImmutableBuffer::from_iter(
                vertices().iter().cloned(),
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
                indices().iter().cloned(),
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
        let start_time = Instant::now();
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
            start_time,
            previous_frame_end,
            recreate_swapchain: false,
        })
    }

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

    pub fn window(&self) -> &Window {
        self.surface.window()
    }

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

        let framebuffer = self.framebuffers[image_index].clone();

        let transfer_command_buffer = {
            let uniform_buffer = self.uniform_buffers[image_index].clone();
            let ubo = {
                let duration = Instant::now().duration_since(self.start_time);
                let elapsed = duration.as_millis();
                Box::new(CameraUBO {
                    projection: {
                        let mut projection = Mat4::perspective_rh(
                            45f32.to_radians(),
                            (framebuffer.width() as f32) / (framebuffer.height() as f32),
                            1.0,
                            10.0,
                        );
                        projection.y_axis.y *= -1f32;
                        projection
                    },
                    model: Mat4::from_rotation_z((elapsed as f32) * 0.1f32.to_radians()),
                    view: Mat4::look_at_rh(Vec3::new(2.0, 2.0, 2.0), Vec3::ZERO, Vec3::Z),
                })
            };

            let mut builder = AutoCommandBufferBuilder::primary(
                self.device.clone(),
                self.transfer_queue.family(),
                CommandBufferUsage::OneTimeSubmit,
            )
            .map_err(|err| Error::new("transfer command buffer creation failure", err))?;
            builder
                .update_buffer(uniform_buffer, ubo)
                .map_err(|err| Error::new("update buffer command creation failure", err))?;
            builder
                .build()
                .map_err(|err| Error::new("transfer command buffer creation failure", err))?
        };
        let draw_command_buffer = {
            let clear_values = vec![
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
            builder
                .build()
                .map_err(|err| Error::new("draw command buffer creation failure", err))?
        };

        let future = self
            .previous_frame_end
            .take()
            .unwrap()
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
