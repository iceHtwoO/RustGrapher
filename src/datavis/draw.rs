use core::f32;
use std::{
    f32::consts::PI,
    sync::{Arc, RwLock},
};

use crate::{
    properties::{RigidBody2D, Spring},
    quadtree::{BoundingBox2D, QuadTree},
};
use glam::Vec2;
use glium::{
    glutin::surface::WindowSurface,
    uniforms::{AsUniformValue, Uniforms, UniformsStorage},
    Display, DrawParameters, Frame, Surface,
};

use rand::{rngs::StdRng, Rng, SeedableRng};

use super::{shapes, Vertex};

static VERTEX_SHADER_SRC: &str = r#"
#version 150

in vec3 position;
in vec4 color;
out vec4 vertex_color;

uniform mat4 projection;
uniform mat4 matrix;

void main() {
    vertex_color = color;
    gl_Position = projection * matrix * vec4(position, 1.0);
}
"#;

static FRAGMENT_SHADER_SRC: &str = r#"
#version 140

in vec4 vertex_color;
out vec4 color;

void main() {
    color = vec4(vertex_color);
}
"#;

pub fn draw_edge<H, R>(
    rb_v: Arc<RwLock<Vec<RigidBody2D>>>,
    spring_v: Arc<RwLock<Vec<Spring>>>,
    target: &mut Frame,
    display: &Display<WindowSurface>,
    max_m: &f32,
    uniform: &UniformsStorage<H, R>,
    params: &DrawParameters,
) where
    H: AsUniformValue,
    R: Uniforms,
{
    let program =
        glium::Program::from_source(display, VERTEX_SHADER_SRC, FRAGMENT_SHADER_SRC, None).unwrap();

    let mut shape: Vec<Vertex> = vec![];

    let spring_read_guard = spring_v.read().unwrap();
    let rb_read_guard = rb_v.read().unwrap();

    for edge in spring_read_guard.iter() {
        let rb1 = &rb_read_guard[edge.rb1];
        let rb2 = &rb_read_guard[edge.rb2];

        let min_m = rb1.mass.min(rb2.mass);
        let color = [
            min_m / max_m * 10.0,
            min_m / max_m * 6.0,
            min_m / max_m * 6.0,
            min_m / max_m,
        ];

        shape.append(&mut shapes::line(
            [rb1.position[0], rb1.position[1], -0.1],
            [rb2.position[0], rb2.position[1], -0.1],
            color,
        ));
    }

    let vertex_buffer = glium::VertexBuffer::new(display, &shape).unwrap();
    let indices = glium::index::NoIndices(glium::index::PrimitiveType::LinesList);

    target
        .draw(&vertex_buffer, indices, &program, uniform, params)
        .unwrap();
}

pub fn draw_node<H, R>(
    rb_v: Arc<RwLock<Vec<RigidBody2D>>>,
    target: &mut Frame,
    display: &Display<WindowSurface>,
    max_m: &f32,
    uniform: &UniformsStorage<H, R>,
    params: &DrawParameters,
) where
    H: AsUniformValue,
    R: Uniforms,
{
    let program =
        glium::Program::from_source(display, VERTEX_SHADER_SRC, FRAGMENT_SHADER_SRC, None).unwrap();

    let mut shape: Vec<Vertex> = vec![];
    let graph_read_guard = rb_v.read().unwrap();

    for (e, rb) in graph_read_guard.iter().enumerate() {
        let pos = [rb.position[0], rb.position[1], -0.2];
        let r = f32::sqrt(rb.mass * PI) * 0.1;

        let mut rand = StdRng::seed_from_u64(e as u64);
        let color = [
            (rand.gen_range(10..=100) as f32) / 100.0,
            (rand.gen_range(10..=100) as f32) / 100.0,
            (rand.gen_range(10..=100) as f32) / 100.0,
            rb.mass / max_m,
        ];

        shape.append(&mut shapes::circle(pos, color, r, 10));
    }

    let vertex_buffer = glium::VertexBuffer::new(display, &shape).unwrap();
    let indices = glium::index::NoIndices(glium::index::PrimitiveType::TrianglesList);

    target
        .draw(&vertex_buffer, indices, &program, uniform, params)
        .unwrap();
}

pub fn draw_quadtree<H, R>(
    rb_v: Arc<RwLock<Vec<RigidBody2D>>>,
    target: &mut Frame,
    display: &Display<WindowSurface>,
    uniform: &UniformsStorage<H, R>,
    params: &DrawParameters,
) where
    H: AsUniformValue,
    R: Uniforms,
{
    let program =
        glium::Program::from_source(display, VERTEX_SHADER_SRC, FRAGMENT_SHADER_SRC, None).unwrap();

    let mut shape: Vec<Vertex> = vec![];

    let graph_read_guard = rb_v.read().unwrap();

    let mut max_x = -f32::INFINITY;
    let mut min_x = f32::INFINITY;
    let mut max_y = -f32::INFINITY;
    let mut min_y = f32::INFINITY;

    for rb in graph_read_guard.iter() {
        max_x = max_x.max(rb.position[0]);
        max_y = max_y.max(rb.position[1]);
        min_x = min_x.min(rb.position[0]);
        min_y = min_y.min(rb.position[1]);
    }

    let w = max_x - min_x;
    let h = max_y - min_y;
    let boundary = BoundingBox2D::new(Vec2::new(min_x + 0.5 * w, min_y + 0.5 * h), w, h);

    let mut quadtree = QuadTree::new(boundary.clone());

    for rb in graph_read_guard.iter() {
        quadtree.insert(Some(rb), rb.position, rb.mass);
    }

    qt_vertex(&quadtree, &mut shape);

    let vertex_buffer = glium::VertexBuffer::new(display, &shape).unwrap();
    let indices = glium::index::NoIndices(glium::index::PrimitiveType::LinesList);

    target
        .draw(&vertex_buffer, indices, &program, uniform, params)
        .unwrap();
}

fn qt_vertex(quadtree: &QuadTree<RigidBody2D>, shape: &mut Vec<Vertex>) {
    for child in quadtree.children.iter() {
        match child {
            Some(c) => qt_vertex(c, shape),
            None => (),
        }
    }
    let boundary = quadtree.boundary.clone();

    let pos = [boundary.center[0], boundary.center[1], 0.1];

    shape.append(&mut shapes::rectangle_lines(
        pos,
        [0.0, 0.0, 1.0, 1.0],
        boundary.width * 0.5,
        boundary.height * 0.5,
    ));
}
