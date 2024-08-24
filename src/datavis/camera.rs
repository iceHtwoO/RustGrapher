use crate::vectors::{Vector2D, Vector3D};

pub struct Camera {
    pub position: Vector3D,
    pub direction: Vector3D,
    pub right: Vector3D,
    pub up: Vector3D,
}

impl Camera {
    pub fn new(position: Vector3D) -> Self {
        Self {
            position,
            direction: Vector3D::new([0.0, 0.0, 0.0]),
            right: Vector3D::new([0.0, 0.0, 0.0]),
            up: Vector3D::new([0.0, 0.0, 0.0]),
        }
    }

    pub fn look_at(&mut self, look_at: &Vector3D) {
        self.direction = self.position.clone();
        self.direction -= *look_at;
        self.direction = self.direction.normalize();

        self.right = Vector3D::new([0.0, 1.0, 0.0]).cross(&self.direction);
        self.up = self.direction.cross(&self.right);
    }

    pub fn matrix(&self) -> [[f32; 4]; 4] {
        let d = self.direction;
        let r = self.right;
        let u = self.up;
        let p = self.position;
        [
            [r[0], u[0], d[0], 0.0],
            [r[1], u[1], d[1], 0.0],
            [r[2], u[2], d[2], 0.0],
            [
                r[0] * -p[0] + r[1] * -p[1] + r[2] * -p[2],
                u[0] * -p[0] + u[1] * -p[1] + u[2] * -p[2],
                d[0] * -p[0] + d[1] * -p[1] + d[2] * -p[2],
                1.0,
            ],
        ]
    }

    pub fn matrix_2(&self) -> [[f32; 4]; 4] {
        [
            [1.0, 0.0, 0.0, 0.0],
            [0.0, 1.0, 0.0, 0.0],
            [0.0, 0.0, 1.0, 0.0],
            [0.0, 0.0, 0.0, 1.0f32],
        ]
    }
}
