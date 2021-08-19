#![allow(dead_code)]

use ultraviolet::Mat4;

#[derive(Default, Copy, Clone)]
pub struct CameraUBO {
    projection: Mat4,
    model: Mat4,
    view: Mat4,
}

impl CameraUBO {
    pub fn new(projection: Mat4, model: Mat4, view: Mat4) -> Self {
        Self {
            projection,
            model,
            view,
        }
    }

    pub fn projection(&self) -> Mat4 {
        self.projection
    }

    pub fn model(&self) -> Mat4 {
        self.model
    }

    pub fn view(&self) -> Mat4 {
        self.view
    }
}
