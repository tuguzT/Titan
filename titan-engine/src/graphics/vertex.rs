use vulkano::pipeline::vertex::{VertexMember, VertexMemberTy};

#[derive(Default, Copy, Clone)]
#[repr(transparent)]
struct Vec2(glam::Vec2);

#[derive(Default, Copy, Clone)]
#[repr(transparent)]
struct Color(palette::Srgb);

#[derive(Copy, Clone, Default)]
#[repr(C)]
pub struct Vertex {
    position: Vec2,
    color: Color,
}

unsafe impl VertexMember for Vec2 {
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
    pub const fn new(position: glam::Vec2, color: palette::Srgb) -> Self {
        Self {
            position: Vec2(position),
            color: Color(color),
        }
    }
}
