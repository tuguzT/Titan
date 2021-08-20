//! Internal camera utilities for game engine.

#![allow(dead_code)]

use ultraviolet::Mat4;

/// Camera uniform buffer object (UBO) that will be passed into uniform buffer.
#[derive(Default, Copy, Clone)]
pub struct CameraUBO {
    /// Projection 4x4 matrix.
    projection: Mat4,
    /// Model 4x4 matrix.
    model: Mat4,
    /// View 4x4 matrix.
    view: Mat4,
}

impl CameraUBO {
    /// Creates new UBO that will be passed to shader.
    pub fn new(projection: Mat4, model: Mat4, view: Mat4) -> Self {
        Self {
            projection,
            model,
            view,
        }
    }

    /// Projection 4x4 matrix of camera.
    pub fn projection(&self) -> Mat4 {
        self.projection
    }

    /// Model 4x4 matrix of camera.
    pub fn model(&self) -> Mat4 {
        self.model
    }

    /// View 4x4 matrix of camera.
    pub fn view(&self) -> Mat4 {
        self.view
    }
}
