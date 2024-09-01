use glam::{Vec2, Vec3};

#[derive(Debug, Clone)]
pub struct RigidBody2D {
    pub position: Vec2,
    pub velocity: Vec2,
    pub mass: f32,
    pub fixed: bool,
}

#[derive(Debug, Clone)]
pub struct RigidBody3D {
    pub position: Vec3,
    pub velocity: Vec3,
    pub mass: f32,
    pub fixed: bool,
}

impl RigidBody2D {
    pub fn new(position: Vec2, mass: f32) -> Self {
        Self {
            position,
            velocity: Vec2::ZERO,
            mass,
            fixed: false,
        }
    }
}

#[derive(Debug, Clone)]
pub struct Spring {
    pub rb1: usize,
    pub rb2: usize,
    pub spring_stiffness: f32,
    pub spring_neutral_len: f32,
}
