pub mod default {
    pub mod vertex {
        vulkano_shaders::shader! {
            ty: "vertex",
            path: "res/shaders/default.vert",
        }
    }

    pub mod fragment {
        vulkano_shaders::shader! {
            ty: "fragment",
            path: "res/shaders/default.frag",
        }
    }
}
