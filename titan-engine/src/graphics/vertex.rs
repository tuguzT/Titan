use palette::Srgba;
use ultraviolet::Vec3;
use vulkano::pipeline::vertex::{VertexMember, VertexMemberTy};

#[derive(Default, Copy, Clone)]
#[repr(transparent)]
struct Position(Vec3);

#[derive(Default, Copy, Clone)]
#[repr(transparent)]
struct Color(Srgba);

#[derive(Default, Copy, Clone)]
#[repr(C)]
pub struct Vertex {
    position: Position,
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
    pub fn new(position: Vec3, color: Srgba) -> Self {
        Self {
            position: Position(position),
            color: Color(color),
        }
    }

    pub fn position(&self) -> Vec3 {
        self.position.0
    }

    pub fn color(&self) -> Srgba {
        self.color.0
    }
}
