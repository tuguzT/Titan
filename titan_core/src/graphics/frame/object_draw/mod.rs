use std::sync::Arc;

use palette::Srgba;
use ultraviolet::Vec3;
use vulkano::buffer::{BufferUsage, ImmutableBuffer, TypedBufferAccess};
use vulkano::command_buffer::{
    AutoCommandBufferBuilder, CommandBufferUsage, DynamicState, SecondaryAutoCommandBuffer,
};
use vulkano::descriptor_set::FixedSizeDescriptorSetsPool;
use vulkano::device::Queue;
use vulkano::pipeline::vertex::BuffersDefinition;
use vulkano::pipeline::viewport::Viewport;
use vulkano::pipeline::{GraphicsPipeline, GraphicsPipelineAbstract};
use vulkano::render_pass::Subpass;
use vulkano::sync::GpuFuture;

use crate::{
    graphics::{
        camera::CameraUBO,
        frame::object_draw::error::{ObjectDrawError, ObjectDrawSystemCreationError},
        renderer::error::DescriptorSetCreationError,
        vertex::Vertex,
    },
    window::Size,
};

pub mod error;

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

/// System that contains the necessary facilities for rendering game objects.
pub struct ObjectDrawSystem {
    /// Queue to render.
    graphics_queue: Arc<Queue>,

    /// Buffer for all vertices of game objects.
    vertex_buffer: Arc<ImmutableBuffer<[Vertex]>>,

    /// Buffer for all indices of vertices in game object.
    index_buffer: Arc<ImmutableBuffer<[u32]>>,

    /// Graphics pipeline used for rendering of game objects.
    pipeline: Arc<GraphicsPipeline<BuffersDefinition>>,

    /// Pool of descriptor sets of uniform buffers with data for vertex shader.
    descriptor_set_pool: FixedSizeDescriptorSetsPool,
}

impl ObjectDrawSystem {
    /// Creates new object draw system.
    pub fn new(
        graphics_queue: Arc<Queue>,
        subpass: Subpass,
    ) -> Result<Self, ObjectDrawSystemCreationError> {
        // Check queue for graphics support.
        if !graphics_queue.family().supports_graphics() {
            return Err(ObjectDrawSystemCreationError::QueueFamilyNotSupported);
        }

        let pipeline = {
            use crate::graphics::shader::default::{fragment, vertex};

            let device = graphics_queue.device().clone();

            let vert_shader_module = vertex::Shader::load(device.clone())?;
            let frag_shader_module = fragment::Shader::load(device.clone())?;

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
                    .render_pass(subpass)
                    .build(device)?,
            )
        };

        let vertex_buffer = {
            let (vertex_buffer, future) = ImmutableBuffer::from_iter(
                self::vertices().iter().cloned(),
                BufferUsage::vertex_buffer(),
                graphics_queue.clone(),
            )?;
            future.flush()?;
            vertex_buffer
        };

        let index_buffer = {
            let (index_buffer, future) = ImmutableBuffer::from_iter(
                self::indices().iter().cloned(),
                BufferUsage::index_buffer(),
                graphics_queue.clone(),
            )?;
            future.flush()?;
            index_buffer
        };

        let descriptor_set_pool = {
            let layout = &pipeline.layout().descriptor_set_layouts()[0];
            FixedSizeDescriptorSetsPool::new(layout.clone())
        };

        Ok(Self {
            graphics_queue,
            vertex_buffer,
            index_buffer,
            pipeline,
            descriptor_set_pool,
        })
    }

    /// Builds a secondary command buffer that draws game objects on the current subpass.
    pub fn draw<B>(
        &mut self,
        viewport_size: Size,
        uniform_buffer: B,
    ) -> Result<SecondaryAutoCommandBuffer, ObjectDrawError>
    where
        B: TypedBufferAccess<Content = CameraUBO> + Send + Sync + 'static,
    {
        let mut builder = AutoCommandBufferBuilder::secondary_graphics(
            self.graphics_queue.device().clone(),
            self.graphics_queue.family(),
            CommandBufferUsage::OneTimeSubmit,
            self.pipeline.subpass().clone(),
        )?;

        let dynamic_state = DynamicState {
            viewports: Some(vec![Viewport {
                origin: [0.0, 0.0],
                dimensions: [viewport_size.width as f32, viewport_size.height as f32],
                depth_range: 0.0..1.0,
            }]),
            ..DynamicState::none()
        };

        let descriptor_set = {
            let descriptor_set = self
                .descriptor_set_pool
                .next()
                .add_buffer(uniform_buffer)
                .map_err(DescriptorSetCreationError::from)?
                .build()
                .map_err(DescriptorSetCreationError::from)?;
            Arc::new(descriptor_set)
        };

        builder.draw_indexed(
            self.pipeline.clone(),
            &dynamic_state,
            self.vertex_buffer.clone(),
            self.index_buffer.clone(),
            descriptor_set,
            (),
        )?;
        Ok(builder.build()?)
    }
}
