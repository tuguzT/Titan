//! Internal camera utilities for game engine.

use ultraviolet::Mat4;

/// Camera uniform buffer object (UBO) that will be passed into uniform buffer.
#[derive(Default, Copy, Clone)]
pub struct CameraUBO {
    /// Projection 4x4 matrix.
    pub projection: Mat4,
    /// Model 4x4 matrix.
    pub model: Mat4,
    /// View 4x4 matrix.
    pub view: Mat4,
}

impl CameraUBO {
    pub fn new(projection: Mat4, model: Mat4, view: Mat4) -> Self {
        Self {
            projection,
            model,
            view,
        }
    }
}
