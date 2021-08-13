use vulkano::pipeline::vertex::{VertexMember, VertexMemberTy};

use crate::math;

#[derive(Default, Copy, Clone)]
struct Vec2(math::Vec2);

#[derive(Default, Copy, Clone)]
struct Vec3(math::Vec3);

#[derive(Copy, Clone, Default)]
pub struct Vertex {
    position: Vec2,
    color: Vec3,
}

unsafe impl VertexMember for Vec2 {
    fn format() -> (VertexMemberTy, usize) {
        (VertexMemberTy::F32, 2)
    }
}

unsafe impl VertexMember for Vec3 {
    fn format() -> (VertexMemberTy, usize) {
        (VertexMemberTy::F32, 3)
    }
}

vulkano::impl_vertex!(Vertex, position, color);

impl Vertex {
    pub const fn new(position: math::Vec2, color: math::Vec3) -> Self {
        Self {
            position: Vec2(position),
            color: Vec3(color),
        }
    }
}
