use crate::vectors::Vector3D;

pub struct Camera {
    pub position: Vector3D,
}

impl Camera {
    pub fn new(position: Vector3D) -> Self {
        Self { position }
    }
}
