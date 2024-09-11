use glam::{Mat4, Vec3};

pub struct Camera {
    pub position: Vec3,
    pub direction: Vec3,
    pub right: Vec3,
    pub up: Vec3,
}

impl Camera {
    pub fn new(position: Vec3) -> Self {
        Self {
            position,
            direction: Vec3::ZERO,
            right: Vec3::ZERO,
            up: Vec3::ZERO,
        }
    }

    pub fn look_at(&mut self, look_at: &Vec3) {
        self.direction = (self.position - look_at).normalize();
        self.right = Vec3::new(0.0, 1.0, 0.0).cross(self.direction);
        self.up = self.direction.cross(self.right);
    }

    pub fn matrix(&self) -> Mat4 {
        let d = self.direction;
        let r = self.right;
        let u = self.up;
        let pp = self.position * -1.0;
        let px = pp.dot(r);
        let py = pp.dot(u);
        let pz = pp.dot(d);
        Mat4::from_cols_array_2d(&[
            [r[0], u[0], d[0], 0.0],
            [r[1], u[1], d[1], 0.0],
            [r[2], u[2], d[2], 0.0],
            [px, py, pz, 1.0],
        ])
    }

    pub fn rotate_y_around_point(&mut self, point: Vec3, angle_in_rads: f32) {
        let s = angle_in_rads.sin();
        let c = angle_in_rads.cos();
        self.position -= point;

        let new_pos = Vec3::new(
            self.position.x * c - self.position.z * s,
            self.position.y,
            self.position.x * c + self.position.z * s,
        );
        self.position = new_pos + point;
    }
}
