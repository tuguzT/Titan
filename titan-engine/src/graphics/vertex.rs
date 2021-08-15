use glam::Vec2;
use palette::Srgb;
use vulkano::pipeline::vertex::{VertexMember, VertexMemberTy};

#[derive(Default, Copy, Clone)]
#[repr(transparent)]
struct Position(Vec2);

#[derive(Default, Copy, Clone)]
#[repr(transparent)]
struct Color(Srgb);

#[derive(Default, Copy, Clone)]
#[repr(C)]
pub struct Vertex {
    position: Position,
    color: Color,
}

unsafe impl VertexMember for Position {
    fn format() -> (VertexMemberTy, usize) {
        (VertexMemberTy::F32, 2)
    }
}

unsafe impl VertexMember for Color {
    fn format() -> (VertexMemberTy, usize) {
        (VertexMemberTy::F32, 3)
    }
}

vulkano::impl_vertex!(Vertex, position, color);

impl Vertex {
    pub const fn new(position: Vec2, color: Srgb) -> Self {
        Self {
            position: Position(position),
            color: Color(color),
        }
    }
}
