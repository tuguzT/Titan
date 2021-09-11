use std::sync::Arc;

use vulkano::buffer::CpuBufferPool;
use vulkano::descriptor_set::DescriptorSet;
use vulkano::device::Queue;
use vulkano::pipeline::vertex::BuffersDefinition;
use vulkano::pipeline::GraphicsPipeline;
use vulkano::sampler::Sampler;

use crate::graphics::vertex::UiVertex;

pub struct UiDrawSystem {
    /// Queue to render.
    graphics_queue: Arc<Queue>,

    /// Buffer for all vertices of UI.
    vertex_buffer: Arc<CpuBufferPool<UiVertex>>,

    /// Buffer for all indices of vertices in UI element.
    index_buffer: Arc<CpuBufferPool<u32>>,

    /// Graphics pipeline used for rendering of UI.
    pipeline: Arc<GraphicsPipeline<BuffersDefinition>>,

    /// Version of `egui` base texture.
    texture_version: u64,

    /// Descriptor set for `egui` base texture that will be used by shader.
    texture_descriptor_set: Option<Arc<dyn DescriptorSet + Send + Sync>>,

    /// A sampler for textures used in UI rendering.
    sampler: Arc<Sampler>,
}
