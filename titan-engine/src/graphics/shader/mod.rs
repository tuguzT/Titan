//! Shader utilities of game engine.

pub mod default {
    //! Default shaders which are used in game engine.

    pub mod vertex {
        //! Default vertex shader utilities.

        vulkano_shaders::shader! {
            ty: "vertex",
            path: "src/graphics/shader/default.vert",
        }
    }

    pub mod fragment {
        //! Default fragment shader utilities.

        vulkano_shaders::shader! {
            ty: "fragment",
            path: "src/graphics/shader/default.frag",
        }
    }
}
