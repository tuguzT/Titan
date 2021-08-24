//! Vertex utilities for game engine.

use palette::Srgba;
use ultraviolet::Vec3;
use vulkano::pipeline::vertex::{VertexMember, VertexMemberTy};

/// Wrapper for external vector struct.
#[derive(Default, Copy, Clone)]
#[repr(transparent)]
struct Position(Vec3);

/// Wrapper for external color struct.
#[derive(Default, Copy, Clone)]
#[repr(transparent)]
struct Color(Srgba);

/// Vertex type which is used in vertex buffer.
#[derive(Default, Copy, Clone)]
#[repr(C)]
pub struct Vertex {
    /// Vertex position in the world.
    position: Position,
    /// Color of this vertex.
    color: Color,
}

unsafe impl VertexMember for Position {
    fn format() -> (VertexMemberTy, usize) {
        (VertexMemberTy::F32, 3)
    }
}

unsafe impl VertexMember for Color {
    fn format() -> (VertexMemberTy, usize) {
        (VertexMemberTy::F32, 4)
    }
}

vulkano::impl_vertex!(Vertex, position, color);

#[allow(dead_code)]
impl Vertex {
    /// Creates new vertex with given position and color.
    pub fn new(position: Vec3, color: Srgba) -> Self {
        Self {
            position: Position(position),
            color: Color(color),
        }
    }

    /// Vertex position in the world.
    pub fn position(&self) -> Vec3 {
        self.position.0
    }

    /// Color of this vertex.
    pub fn color(&self) -> Srgba {
        self.color.0
    }
}
