use std::sync::Arc;

use palette::Srgba;
use thiserror::Error;
use ultraviolet::Vec3;
use vulkano::buffer::{BufferUsage, ImmutableBuffer};
use vulkano::command_buffer::{
    AutoCommandBufferBuilder, BuildError, CommandBufferUsage, DrawIndexedError, DynamicState,
    SecondaryAutoCommandBuffer,
};
use vulkano::descriptor_set::DescriptorSetsCollection;
use vulkano::device::Queue;
use vulkano::memory::DeviceMemoryAllocError;
use vulkano::pipeline::vertex::BuffersDefinition;
use vulkano::pipeline::viewport::Viewport;
use vulkano::pipeline::{GraphicsPipeline, GraphicsPipelineCreationError};
use vulkano::render_pass::Subpass;
use vulkano::sync::{FlushError, GpuFuture};
use vulkano::OomError;

use crate::graphics::vertex::Vertex;
use crate::window::Size;

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

        Ok(Self {
            graphics_queue,
            vertex_buffer,
            index_buffer,
            pipeline,
        })
    }

    /// Builds a secondary command buffer that draws game objects on the current subpass.
    pub fn draw<S, Pc>(
        &self,
        viewport_size: Size,
        descriptor_sets: S,
    ) -> Result<SecondaryAutoCommandBuffer, DrawError>
    where
        S: DescriptorSetsCollection,
    {
        let mut builder = AutoCommandBufferBuilder::secondary_graphics(
            self.graphics_queue.device().clone(),
            self.graphics_queue.family(),
            CommandBufferUsage::OneTimeSubmit,
            self.pipeline.clone().subpass(),
        )?;
        let dynamic_state = DynamicState {
            viewports: Some(vec![Viewport {
                origin: [0.0, 0.0],
                dimensions: [viewport_size.width as f32, viewport_size.height as f32],
                depth_range: 0.0..1.0,
            }]),
            ..DynamicState::none()
        };
        builder.draw_indexed(
            self.pipeline.clone(),
            &dynamic_state,
            self.vertex_buffer.clone(),
            self.index_buffer.clone(),
            descriptor_sets,
            (),
        )?;
        Ok(builder.build()?)
    }
}

#[derive(Debug, Error)]
pub enum ObjectDrawSystemCreationError {
    #[error("shader module allocation failure: {0}")]
    OutOfMemory(#[from] OomError),

    #[error("queue family must support graphics operations")]
    QueueFamilyNotSupported,

    #[error("graphics pipeline creation failure: {0}")]
    GraphicsPipelineCreation(#[from] GraphicsPipelineCreationError),

    #[error("vertex/index buffer creation failure: {0}")]
    BufferCreation(#[from] FlushError),

    #[error("vertex/index buffer allocation failure: {0}")]
    BufferAllocation(#[from] DeviceMemoryAllocError),
}

#[derive(Debug, Error)]
pub enum DrawError {
    #[error("command buffer allocation failure: {0}")]
    OutOfMemory(#[from] OomError),

    #[error("draw indexed command failure: {0}")]
    DrawIndexed(#[from] DrawIndexedError),

    #[error("draw command buffer build failure: {0}")]
    CommandBufferBuild(#[from] BuildError),
}
