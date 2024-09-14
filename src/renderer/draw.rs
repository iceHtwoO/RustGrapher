use core::f32;
use std::{f32::consts::PI, sync::Arc};

use crate::simulator::Simulator;
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
    sim: Arc<Simulator>,
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

    let spring_read_guard = sim.springs.read().unwrap();
    let rb_read_guard = sim.rigid_bodies.read().unwrap();

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
    sim: Arc<Simulator>,
    target: &mut Frame,
    display: &Display<WindowSurface>,
    max_m: &f32,
    uniform: &UniformsStorage<H, R>,
    params: &DrawParameters,
    highlight_index: Vec<u32>,
) where
    H: AsUniformValue,
    R: Uniforms,
{
    let program =
        glium::Program::from_source(display, VERTEX_SHADER_SRC, FRAGMENT_SHADER_SRC, None).unwrap();

    let mut shape: Vec<Vertex> = vec![];
    let graph_read_guard = sim.rigid_bodies.read().unwrap();

    for (e, rb) in graph_read_guard.iter().enumerate() {
        let pos = [rb.position[0], rb.position[1], 0.0];
        let r = f32::sqrt(rb.mass * PI) * 0.1;

        let mut rand = StdRng::seed_from_u64(e as u64);

        let mut highlight_mul = 1.0;

        if !highlight_index.is_empty() && !highlight_index.contains(&(e as u32)) {
            highlight_mul = 0.5;
        }

        let color = [
            (rand.gen_range(10..=100) as f32) / 100.0 * highlight_mul,
            (rand.gen_range(10..=100) as f32) / 100.0 * highlight_mul,
            (rand.gen_range(10..=100) as f32) / 100.0 * highlight_mul,
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
