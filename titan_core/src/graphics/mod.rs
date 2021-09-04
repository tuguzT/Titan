//! Graphics utilities and backend based on Vulkan API for game engine.

use std::collections::HashSet;
use std::sync::Arc;

use egui::{ClippedMesh, Pos2, Texture};
use palette::Srgba;
use ultraviolet::Vec3;
use vulkano::buffer::{BufferUsage, CpuAccessibleBuffer, CpuBufferPool, ImmutableBuffer};
use vulkano::command_buffer::{
    AutoCommandBufferBuilder, CommandBufferUsage, DynamicState, PrimaryAutoCommandBuffer,
    SubpassContents,
};
use vulkano::descriptor_set::{DescriptorSet, PersistentDescriptorSet};
use vulkano::device::physical::PhysicalDevice;
use vulkano::device::{Device, DeviceExtensions, Features, Queue};
use vulkano::format::{ClearValue, Format};
use vulkano::image::view::ImageView;
use vulkano::image::{
    AttachmentImage, ImageAccess, ImageDimensions, ImageUsage, ImmutableImage, MipmapsCount,
    SwapchainImage,
};
use vulkano::instance::debug::{DebugCallback, MessageSeverity, MessageType};
use vulkano::instance::Instance;
use vulkano::pipeline::blend::{AttachmentBlend, BlendFactor};
use vulkano::pipeline::vertex::BuffersDefinition;
use vulkano::pipeline::viewport::{Scissor, Viewport};
use vulkano::pipeline::{GraphicsPipeline, GraphicsPipelineAbstract};
use vulkano::render_pass::{Framebuffer, FramebufferAbstract, RenderPass, Subpass};
use vulkano::sampler::{Filter, MipmapMode, Sampler, SamplerAddressMode};
use vulkano::swapchain::{AcquireError, ColorSpace, PresentMode, Surface, Swapchain};
use vulkano::sync::{FlushError, GpuFuture, SharingMode};
use vulkano::{swapchain, sync};
use vulkano_win::VkSurfaceBuild;
use winit::dpi::LogicalSize;
use winit::event_loop::EventLoop;
use winit::window::{Window, WindowBuilder};

use crate::config::Config;
use crate::error::{Error, Result};
use crate::graphics::vertex::UiVertex;

use self::camera::CameraUBO;
use self::vertex::Vertex;

pub(crate) mod camera;

mod debug_callback;
mod shader;
mod utils;
mod vertex;

const fn indices() -> [u32; 12] {
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
pub struct Renderer {
    previous_frame_end: Option<Box<dyn GpuFuture + Send + Sync>>,
    recreate_swapchain: bool,
    camera_ubo: CameraUBO,

    ub_descriptor_sets: Vec<Arc<dyn DescriptorSet + Send + Sync>>,
    uniform_buffers: Vec<Arc<CpuAccessibleBuffer<CameraUBO>>>,
    index_buffer: Arc<ImmutableBuffer<[u32]>>,
    vertex_buffer: Arc<ImmutableBuffer<[Vertex]>>,
    ui_vertex_buffer: Arc<CpuBufferPool<UiVertex>>,
    ui_index_buffer: Arc<CpuBufferPool<u32>>,

    framebuffers: Vec<Arc<dyn FramebufferAbstract + Send + Sync>>,
    dynamic_state: DynamicState,
    graphics_pipeline: Arc<GraphicsPipeline<BuffersDefinition>>,
    ui_pipeline: Arc<GraphicsPipeline<BuffersDefinition>>,

    egui_texture_version: u64,
    egui_texture_descriptor_set: Option<Arc<dyn DescriptorSet + Send + Sync>>,
    egui_sampler: Arc<Sampler>,

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
            vulkano::ordered_passes_renderpass! {
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
                passes: [
                    { color: [color], depth_stencil: {depth}, input: [] },
                    { color: [color], depth_stencil: {}, input: [] }
                ]
            }
            .map_err(|err| Error::new("render pass creation failure", err))?,
        );
        let graphics_subpass = Subpass::from(render_pass.clone(), 0).unwrap();
        let ui_subpass = Subpass::from(render_pass.clone(), 1).unwrap();

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
                    .render_pass(graphics_subpass)
                    .build(device.clone())
                    .map_err(|err| Error::new("graphics pipeline creation failure", err))?,
            )
        };

        let ui_pipeline = {
            use self::shader::ui::{fragment, vertex};

            let vert_shader_module = vertex::Shader::load(device.clone())
                .map_err(|err| Error::new("vertex shader module creation failure", err))?;
            let frag_shader_module = fragment::Shader::load(device.clone())
                .map_err(|err| Error::new("fragment shader module creation failure", err))?;

            let blend = AttachmentBlend {
                color_source: BlendFactor::One,
                ..AttachmentBlend::alpha_blending()
            };

            Arc::new(
                GraphicsPipeline::start()
                    .vertex_input_single_buffer::<UiVertex>()
                    .vertex_shader(vert_shader_module.main_entry_point(), ())
                    .fragment_shader(frag_shader_module.main_entry_point(), ())
                    .triangle_list()
                    .viewports_scissors_dynamic(1)
                    .cull_mode_disabled()
                    .blend_collective(blend)
                    .render_pass(ui_subpass)
                    .build(device.clone())
                    .map_err(|err| Error::new("UI pipeline creation failure", err))?,
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
        let ui_vertex_buffer = Arc::new(CpuBufferPool::vertex_buffer(device.clone()));

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
        let ui_index_buffer = Arc::new(CpuBufferPool::new(
            device.clone(),
            BufferUsage::index_buffer(),
        ));
        let egui_sampler = Sampler::new(
            device.clone(),
            Filter::Linear,
            Filter::Linear,
            MipmapMode::Linear,
            SamplerAddressMode::ClampToEdge,
            SamplerAddressMode::ClampToEdge,
            SamplerAddressMode::ClampToEdge,
            0.0,
            1.0,
            0.0,
            0.0,
        )
        .map_err(|err| Error::new("sample creation error", err))?;

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
            ui_pipeline,
            dynamic_state,
            framebuffers,
            vertex_buffer,
            ui_vertex_buffer,
            index_buffer,
            ui_index_buffer,
            uniform_buffers,
            ub_descriptor_sets: descriptor_sets,
            camera_ubo: CameraUBO::default(),
            previous_frame_end,
            recreate_swapchain: false,
            egui_texture_version: 0,
            egui_texture_descriptor_set: None,
            egui_sampler,
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
        builder
            .build()
            .map_err(|err| Error::new("transfer command buffer creation failure", err))
    }

    /// Create command buffer for actual rendering operations.
    fn draw_cb(
        &mut self,
        image_index: usize,
        ui: Option<(Vec<ClippedMesh>, Arc<Texture>)>,
    ) -> Result<PrimaryAutoCommandBuffer> {
        let framebuffer = self.framebuffers[image_index].clone();
        let dimensions = framebuffer.dimensions();
        let clear_values = [
            ClearValue::Float([0.0, 0.0, 0.0, 1.0]),
            ClearValue::Depth(1.0),
        ];
        let descriptor_set = self.ub_descriptor_sets[image_index].clone();

        let mut builder = AutoCommandBufferBuilder::primary(
            self.device.clone(),
            self.graphics_queue.family(),
            CommandBufferUsage::OneTimeSubmit,
        )
        .map_err(|err| Error::new("draw command buffer creation failure", err))?;
        builder
            .begin_render_pass(framebuffer, SubpassContents::Inline, clear_values)
            .map_err(|err| Error::new("begin render pass failure", err))?;
        builder
            .draw_indexed(
                self.graphics_pipeline.clone(),
                &self.dynamic_state,
                self.vertex_buffer.clone(),
                self.index_buffer.clone(),
                descriptor_set,
                (),
            )
            .map_err(|err| Error::new("draw command failure", err))?;
        builder
            .next_subpass(SubpassContents::Inline)
            .map_err(|err| Error::new("next subpass failure", err))?;
        if let Some((ui_meshes, ui_texture)) = ui {
            use self::shader::ui::vertex;

            if ui_texture.version != self.egui_texture_version {
                self.egui_texture_version = ui_texture.version;
                let layout = self.ui_pipeline.layout().descriptor_set_layouts()[0].clone();
                let image = {
                    let dimensions = ImageDimensions::Dim2d {
                        width: ui_texture.width as u32,
                        height: ui_texture.height as u32,
                        array_layers: 1,
                    };
                    let data: Vec<_> = ui_texture
                        .pixels
                        .iter()
                        .flat_map(|&r| vec![r, r, r, r])
                        .collect();

                    let (image, image_future) = ImmutableImage::from_iter(
                        data.into_iter(),
                        dimensions,
                        MipmapsCount::One,
                        Format::R8G8B8A8Unorm,
                        self.graphics_queue.clone(),
                    )
                    .map_err(|err| Error::new("UI texture creation failure", err))?;

                    image_future
                        .flush()
                        .map_err(|err| Error::new("UI texture creation failure", err))?;
                    image
                };

                let view = ImageView::new(image)
                    .map_err(|err| Error::new("UI texture image view creation failure", err))?;
                let set = Arc::new(
                    PersistentDescriptorSet::start(layout)
                        .add_sampled_image(view, self.egui_sampler.clone())
                        .map_err(|err| Error::new("sampled image addition failure", err))?
                        .build()
                        .map_err(|err| {
                            let message = "UI texture descriptor set creation failure";
                            Error::new(message, err)
                        })?,
                );
                self.egui_texture_descriptor_set = Some(set);
            }

            let width = dimensions[0] as f32;
            let height = dimensions[1] as f32;
            let scale_factor = self.window().scale_factor() as f32;
            let push_constants = vertex::ty::PushConstants {
                screen_size: [width / scale_factor, height / scale_factor],
            };
            for ClippedMesh(rect, mesh) in ui_meshes {
                // Nothing to draw if we don't have vertices & indices
                if mesh.vertices.is_empty() || mesh.indices.is_empty() {
                    continue;
                }
                let scissor = {
                    let min = rect.min;
                    let min = Pos2 {
                        x: min.x * scale_factor,
                        y: min.y * scale_factor,
                    };
                    let min = Pos2 {
                        x: min.x.clamp(0.0, width),
                        y: min.y.clamp(0.0, height),
                    };
                    let max = rect.max;
                    let max = Pos2 {
                        x: max.x * scale_factor,
                        y: max.y * scale_factor,
                    };
                    let max = Pos2 {
                        x: max.x.clamp(min.x, width),
                        y: max.y.clamp(min.y, height),
                    };
                    Scissor {
                        origin: [min.x.round() as i32, min.y.round() as i32],
                        dimensions: [
                            (max.x.round() - min.x) as u32,
                            (max.y.round() - min.y) as u32,
                        ],
                    }
                };
                self.dynamic_state.scissors = Some(vec![scissor]);

                let chunk = mesh.vertices.into_iter().map(|vertex| vertex.into());
                let vertex_buffer = self
                    .ui_vertex_buffer
                    .chunk(chunk)
                    .map_err(|err| Error::new("UI vertex buffer allocation failure", err))?;

                let chunk = mesh.indices.into_iter();
                let index_buffer = self
                    .ui_index_buffer
                    .chunk(chunk)
                    .map_err(|err| Error::new("UI index buffer allocation failure", err))?;

                builder
                    .draw_indexed(
                        self.ui_pipeline.clone(),
                        &self.dynamic_state,
                        vertex_buffer,
                        index_buffer,
                        self.egui_texture_descriptor_set.as_ref().unwrap().clone(),
                        push_constants,
                    )
                    .map_err(|err| Error::new("UI draw command failure", err))?;
            }
            self.dynamic_state.scissors = None;
        }
        builder
            .end_render_pass()
            .map_err(|err| Error::new("end render pass failure", err))?;
        builder
            .build()
            .map_err(|err| Error::new("draw command buffer creation failure", err))
    }

    /// Render new frame into the underlying window.
    pub fn render(&mut self, ui: Option<(Vec<ClippedMesh>, Arc<Texture>)>) -> Result<()> {
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
        let draw_command_buffer = self.draw_cb(image_index, ui)?;
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
