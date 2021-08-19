pub mod default {
    pub mod vertex {
        vulkano_shaders::shader! {
            ty: "vertex",
            path: "src/graphics/shader/default.vert",
        }
    }

    pub mod fragment {
        vulkano_shaders::shader! {
            ty: "fragment",
            path: "src/graphics/shader/default.frag",
        }
    }
}
