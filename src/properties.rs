use crate::vectors::{Vector2D, Vector3D};

#[derive(Debug, Clone)]
pub struct RigidBody2D {
    pub position: Vector2D,
    pub velocity: Vector2D,
    pub mass: f32,
    pub fixed: bool,
}

#[derive(Debug, Clone)]
pub struct RigidBody3D {
    pub position: Vector3D,
    pub velocity: Vector3D,
    pub mass: f32,
    pub fixed: bool,
}

impl RigidBody2D {
    pub fn new(position: Vector2D, mass: f32) -> Self {
        Self {
            position,
            velocity: Vector2D::empty(),
            mass,
            fixed: false,
        }
    }
}
