//! Vertex utilities for game engine.

use std::ops::{Deref, DerefMut};

use palette::Srgba;
use ultraviolet::{Vec2, Vec3};
use vulkano::pipeline::vertex::{VertexMember, VertexMemberTy};

/// Wrapper for external 3-dimensional vector struct.
#[derive(Default, Copy, Clone)]
pub struct Position3(Vec3);

impl From<Vec3> for Position3 {
    fn from(vec3: Vec3) -> Self {
        Self(vec3)
    }
}

impl Deref for Position3 {
    type Target = Vec3;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl DerefMut for Position3 {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}

unsafe impl VertexMember for Position3 {
    fn format() -> (VertexMemberTy, usize) {
        (VertexMemberTy::F32, 3)
    }
}

/// Wrapper for external 2-dimensional vector struct.
#[derive(Default, Copy, Clone)]
pub struct Position2(Vec2);

impl From<Vec2> for Position2 {
    fn from(vec2: Vec2) -> Self {
        Self(vec2)
    }
}

impl Deref for Position2 {
    type Target = Vec2;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl DerefMut for Position2 {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}

unsafe impl VertexMember for Position2 {
    fn format() -> (VertexMemberTy, usize) {
        (VertexMemberTy::F32, 2)
    }
}

/// Wrapper for external color struct.
#[derive(Default, Copy, Clone)]
pub struct Color(Srgba);

impl From<Srgba> for Color {
    fn from(color: Srgba) -> Self {
        Self(color)
    }
}

impl Deref for Color {
    type Target = Srgba;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl DerefMut for Color {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}

unsafe impl VertexMember for Color {
    fn format() -> (VertexMemberTy, usize) {
        (VertexMemberTy::F32, 4)
    }
}

/// Vertex type which is used in vertex buffer.
#[derive(Default, Copy, Clone)]
#[repr(C)]
pub struct Vertex {
    /// Vertex position in the world.
    pub position: Position3,
    /// Color of this vertex.
    pub color: Color,
}

vulkano::impl_vertex!(Vertex, position, color);

impl Vertex {
    /// Creates new vertex with given position and color.
    pub fn new(position: Vec3, color: Srgba) -> Self {
        Self {
            position: Position3(position),
            color: Color(color),
        }
    }
}

/// Vertex type which is used in vertex buffer.
#[derive(Default, Copy, Clone)]
#[repr(C)]
pub struct UiVertex {
    /// Vertex position on the screen.
    pub position: Position2,
    /// UV position on the texture.
    pub uv: Position2,
    /// Color of this vertex.
    pub color: Color,
}

vulkano::impl_vertex!(UiVertex, position, uv, color);

impl UiVertex {
    /// Creates new vertex with given position and color.
    pub fn new(position: Vec2, uv: Vec2, color: Srgba) -> Self {
        Self {
            position: Position2(position),
            uv: Position2(uv),
            color: Color(color),
        }
    }
}
