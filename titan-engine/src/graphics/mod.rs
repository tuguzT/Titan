use std::collections::HashSet;
use std::sync::Arc;

use vulkano::buffer::{BufferUsage, ImmutableBuffer};
use vulkano::command_buffer::{
    AutoCommandBufferBuilder, CommandBufferUsage, DynamicState, PrimaryAutoCommandBuffer,
    SubpassContents,
};
use vulkano::device::physical::{PhysicalDevice, PhysicalDeviceType};
use vulkano::device::{Device, DeviceExtensions, Features, Queue};
use vulkano::format::{ClearValue, Format};
use vulkano::image::view::ImageView;
use vulkano::image::{ImageUsage, SwapchainImage};
use vulkano::instance::debug::{DebugCallback, MessageSeverity, MessageType};
use vulkano::instance::{ApplicationInfo, Instance};
use vulkano::pipeline::vertex::BuffersDefinition;
use vulkano::pipeline::viewport::Viewport;
use vulkano::pipeline::GraphicsPipeline;
use vulkano::render_pass::{Framebuffer, RenderPass, Subpass};
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
    math::{vec2, vec3},
};

use self::vertex::Vertex;

mod debug_callback;
mod shader;
mod utils;
mod vertex;

type SwapchainFramebuffer = Framebuffer<((), Arc<ImageView<Arc<SwapchainImage<Window>>>>)>;

const ENABLE_VALIDATION: bool = cfg!(debug_assertions);

lazy_static::lazy_static! {
    static ref VERTICES: [Vertex; 3] = [
        Vertex::new(vec2(0.0, -0.5), vec3(1.0, 0.0, 0.0)),
        Vertex::new(vec2(0.5, 0.5), vec3(0.0, 1.0, 0.0)),
        Vertex::new(vec2(-0.5, 0.5), vec3(0.0, 0.0, 1.0)),
    ];
}

pub struct Renderer {
    previous_frame_end: Option<Box<dyn GpuFuture>>,
    recreate_swapchain: bool,

    vertex_buffer: Arc<ImmutableBuffer<[Vertex]>>,
    framebuffers: Vec<Arc<SwapchainFramebuffer>>,
    dynamic_state: DynamicState,
    graphics_pipeline: Arc<GraphicsPipeline<BuffersDefinition>>,
    render_pass: Arc<RenderPass>,
    swapchain_images: Vec<Arc<SwapchainImage<Window>>>,
    swapchain: Arc<Swapchain<Window>>,
    graphics_queue: Arc<Queue>,
    present_queue: Arc<Queue>,
    device: Arc<Device>,
    surface: Arc<Surface<Window>>,
    _debug_callback: Option<DebugCallback>,
    _instance: Arc<Instance>,
}

impl Renderer {
    pub fn new(config: &Config, event_loop: &EventLoop<()>) -> Result<Self> {
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
                MessageSeverity {
                    error: true,
                    warning: true,
                    information: true,
                    verbose: true,
                },
                MessageType::all(),
                debug_callback::callback,
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
        let (physical_device, graphics_family, present_family) = physical_devices
            .filter(|device: &PhysicalDevice| {
                let extensions = device.supported_extensions();
                // All required extensions are supported by device
                required_extensions.intersection(extensions) == required_extensions
            })
            .filter_map(|device| {
                let graphics_family = device
                    .queue_families()
                    .find(|&queue| queue.supports_graphics());
                let present_family = device
                    .queue_families()
                    .find(|&queue| surface.is_supported(queue).unwrap_or(false));
                match (graphics_family, present_family) {
                    (Some(graphics_family), Some(present_family)) => {
                        Some((device, graphics_family, present_family))
                    }
                    _ => None,
                }
            })
            .max_by_key(|(device, _, _)| {
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
            r#"using device "{}" (type "{:?}")"#,
            physical_device.properties().device_name,
            physical_device.properties().device_type,
        );

        let (device, mut queues) = {
            let priorities = 1.0;
            let unique_queue_families = {
                let unique_queue_families: HashSet<_> = [graphics_family.id(), present_family.id()]
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

        let (swapchain, swapchain_images) = {
            let caps = surface
                .capabilities(physical_device)
                .map_err(|err| Error::new("failed to get surface capabilities", err))?;
            let (format, color_space) = {
                let formats = caps.supported_formats;
                *formats
                    .iter()
                    .find(|(format, color_space)| {
                        *format == Format::B8G8R8A8Unorm
                            && *color_space == ColorSpace::SrgbNonLinear
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

        let render_pass = Arc::new(
            vulkano::single_pass_renderpass! {
                device.clone(),
                attachments: {
                    color: {
                        load: Clear,
                        store: Store,
                        format: swapchain.format(),
                        samples: 1,
                    }
                },
                pass: {
                    color: [color],
                    depth_stencil: {}
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
                .depth_clamp(false)
                .cull_mode_back()
                .front_face_clockwise()
                .render_pass(Subpass::from(render_pass.clone(), 0).unwrap())
                .build(device.clone())
                .map_err(|err| Error::new("graphics pipeline creation failure", err))?,
        );

        let mut dynamic_state = DynamicState::none();
        let framebuffers = Self::create_framebuffers(
            swapchain_images.as_slice(),
            render_pass.clone(),
            &mut dynamic_state,
        )?;

        let (vertex_buffer, future) = ImmutableBuffer::from_iter(
            VERTICES.iter().cloned(),
            BufferUsage::vertex_buffer(),
            graphics_queue.clone(),
        )
        .map_err(|err| Error::new("vertex buffer creation failure", err))?;
        future
            .flush()
            .map_err(|err| Error::new("vertex buffer creation failure", err))?;

        let previous_frame_end = Some(Box::new(sync::now(device.clone())) as Box<_>);
        Ok(Self {
            _instance: instance,
            _debug_callback: debug_callback,
            surface,
            device,
            graphics_queue,
            present_queue,
            swapchain,
            swapchain_images,
            render_pass,
            graphics_pipeline,
            dynamic_state,
            framebuffers,
            vertex_buffer,
            previous_frame_end,
            recreate_swapchain: false,
        })
    }

    fn create_framebuffers(
        images: &[Arc<SwapchainImage<Window>>],
        render_pass: Arc<RenderPass>,
        dynamic_state: &mut DynamicState,
    ) -> Result<Vec<Arc<SwapchainFramebuffer>>> {
        let dimensions = images[0].dimensions();

        let viewport = Viewport {
            origin: [0.0, 0.0],
            dimensions: [dimensions[0] as f32, dimensions[1] as f32],
            depth_range: 0.0..1.0,
        };
        dynamic_state.viewports = Some(vec![viewport]);

        images
            .iter()
            .map(|image| {
                let image_view = ImageView::new(image.clone())
                    .map_err(|err| Error::new("image view creation failure", err))?;
                let framebuffer = Framebuffer::start(render_pass.clone())
                    .add(image_view)
                    .map_err(|err| Error::new("failed to add an attachment to framebuffer", err))?
                    .build()
                    .map_err(|err| Error::new("framebuffer creation failure", err))?;
                Ok(Arc::new(framebuffer))
            })
            .collect()
    }

    pub fn window(&self) -> &Window {
        self.surface.window()
    }

    pub fn render(&mut self) -> Result<()> {
        self.previous_frame_end.as_mut().unwrap().cleanup_finished();

        if self.recreate_swapchain {
            let dimensions = self.window().inner_size().into();
            let (swapchain, swapchain_images) = self
                .swapchain
                .recreate()
                .dimensions(dimensions)
                .build()
                .map_err(|err| Error::new("failed to recreate swapchain", err))?;
            self.swapchain = swapchain;
            self.swapchain_images = swapchain_images;
            self.framebuffers = Self::create_framebuffers(
                self.swapchain_images.as_slice(),
                self.render_pass.clone(),
                &mut self.dynamic_state,
            )?;
            self.recreate_swapchain = false;
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
        let command_buffer: PrimaryAutoCommandBuffer = {
            let clear_values: Vec<ClearValue> = vec![[0.0, 0.0, 0.0, 1.0].into()];

            let mut builder = AutoCommandBufferBuilder::primary(
                self.device.clone(),
                self.graphics_queue.family(),
                CommandBufferUsage::OneTimeSubmit,
            )
            .map_err(|err| Error::new("command buffer creation failure", err))?;
            builder
                .begin_render_pass(framebuffer, SubpassContents::Inline, clear_values)
                .map_err(|err| Error::new("begin render pass failure", err))?
                .draw(
                    self.graphics_pipeline.clone(),
                    &self.dynamic_state,
                    self.vertex_buffer.clone(),
                    (),
                    (),
                )
                .map_err(|err| Error::new("draw command failure", err))?
                .end_render_pass()
                .map_err(|err| Error::new("end render pass failure", err))?;
            builder
                .build()
                .map_err(|err| Error::new("command buffer creation failure", err))?
        };

        let future = self
            .previous_frame_end
            .take()
            .unwrap()
            .join(acquire_future)
            .then_execute(self.graphics_queue.clone(), command_buffer)
            .map_err(|err| Error::new("command buffer execution failure", err))?
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
                Err(Error::new("failed to flush future", err))
            }
        }
    }
}
