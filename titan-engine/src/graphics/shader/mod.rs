//! Shader utilities of game engine.

/// Default shaders which are used in game engine.
pub mod default {

    /// Default vertex shader utilities.
    pub mod vertex {
        vulkano_shaders::shader! {
            ty: "vertex",
            path: "src/graphics/shader/default.vert",
        }
    }

    /// Default fragment shader utilities.
    pub mod fragment {
        vulkano_shaders::shader! {
            ty: "fragment",
            path: "src/graphics/shader/default.frag",
        }
    }
}
