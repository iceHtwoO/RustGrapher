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

    pub fn rotate(&mut self, pivot: Vector2D, rad: f32) {
        let s = rad.sin();
        let c = rad.cos();
        self.position[0] -= pivot[0];
        self.position[2] -= pivot[1];

        let xnew = self.position[0] * c - self.position[2] * s;
        let znew = self.position[0] * s + self.position[2] * c;

        self.position = Vector3D::new([xnew + pivot[0], self.position[1], znew + pivot[1]])
    }

    pub fn matrix(&self) -> [[f32; 4]; 4] {
        let d = self.direction;
        let r = self.right;
        let u = self.up;
        let mut pp = self.position;
        pp.scalar(-1.0);
        let px = pp.dot(&r);
        let py = pp.dot(&u);
        let pz = pp.dot(&d);
        [
            [r[0], u[0], d[0], 0.0],
            [r[1], u[1], d[1], 0.0],
            [r[2], u[2], d[2], 0.0],
            [px, py, pz, 1.0],
        ]
    }

    pub fn ortho(l: f32, r: f32, t: f32, b: f32, f: f32, n: f32) -> [[f32; 4]; 4] {
        [
            [2.0 / (r - l), 0.0, 0.0, -((r + l) / (r - l))],
            [0.0, 2.0 / (t - b), 0.0, -((t + b) / (t - b))],
            [0.0, 0.0, 1.0, 0.0],
            [0.0, 0.0, 0.0, 1.0f32],
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
