use glam::Mat4;

#[derive(Copy, Clone)]
pub struct CameraUBO {
    pub projection: Mat4,
    pub model: Mat4,
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

impl Default for CameraUBO {
    fn default() -> Self {
        Self::new(Mat4::IDENTITY, Mat4::IDENTITY, Mat4::IDENTITY)
    }
}
