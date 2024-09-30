use core::f32;
use std::{
    f32::consts::PI,
    sync::{Arc, Mutex},
};

use glium::{
    glutin::surface::WindowSurface,
    implement_vertex,
    uniforms::{AsUniformValue, Uniforms, UniformsStorage},
    Display, DrawParameters, Frame, Surface,
};

use rand::{rngs::StdRng, Rng, SeedableRng};

use super::{shapes, SceneContext};

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

static INSTANCE_SHADER_SRC: &str = r#"
#version 150

in vec3 position;
in vec3 color;
in vec3 color_attr;
in vec3 world_position;
in float scale;

out vec4 vertex_color;

uniform mat4 projection;
uniform mat4 matrix;

void main() {
    vertex_color = vec4(color_attr, 1.0);
    gl_Position = projection * matrix * vec4((position*scale)+world_position, 1.0);
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

#[derive(Copy, Clone, Debug)]
pub struct Vertex {
    pub position: [f32; 3],
    pub color: [f32; 4],
}

implement_vertex!(Vertex, position, color);

#[derive(Copy, Clone)]
struct Attr {
    color_attr: [f32; 3],
    world_position: [f32; 3],
    scale: f32,
}
implement_vertex!(Attr, color_attr, world_position, scale);

pub fn draw_edge<H, R>(
    scene_context: Arc<Mutex<SceneContext>>,
    target: &mut Frame,
    display: &Display<WindowSurface>,
    uniform: &UniformsStorage<H, R>,
    params: &DrawParameters,
) where
    H: AsUniformValue,
    R: Uniforms,
{
    let scene_context = scene_context.lock().unwrap();

    let program =
        glium::Program::from_source(display, VERTEX_SHADER_SRC, FRAGMENT_SHADER_SRC, None).unwrap();

    let mut shape: Vec<Vertex> = vec![];

    let spring_read_guard = scene_context.simulator.springs.read().unwrap();
    let rb_read_guard = scene_context.simulator.rigid_bodies.read().unwrap();

    let mut longest_len = 0.0_f32;

    for edge in spring_read_guard.iter() {
        let rb1 = &rb_read_guard[edge.rb1];
        let rb2 = &rb_read_guard[edge.rb2];

        longest_len = longest_len.max(rb1.position.distance(rb2.position));
    }

    for edge in spring_read_guard.iter() {
        let rb1 = &rb_read_guard[edge.rb1];
        let rb2 = &rb_read_guard[edge.rb2];

        let dist = rb1.position.distance(rb2.position);

        let color = [dist / longest_len, 0.0, 0.0, 0.0];

        shape.append(&mut shapes::line(
            [rb1.position[0], rb1.position[1], -1.0],
            [rb2.position[0], rb2.position[1], -1.0],
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
    scene_context: Arc<Mutex<SceneContext>>,
    target: &mut Frame,
    display: &Display<WindowSurface>,
    uniform: &UniformsStorage<H, R>,
    params: &DrawParameters,
    highlight_index: &[u32],
) where
    H: AsUniformValue,
    R: Uniforms,
{
    let scene_context = scene_context.lock().unwrap();

    let program =
        glium::Program::from_source(display, INSTANCE_SHADER_SRC, FRAGMENT_SHADER_SRC, None)
            .unwrap();

    let mut shape: Vec<Vertex> = vec![];
    let graph_read_guard = scene_context.simulator.rigid_bodies.read().unwrap();

    shape.append(&mut shapes::circle(
        [0.0, 0.0, 0.0],
        [0.0, 0.0, 0.0, 0.0],
        1.0,
        10,
    ));

    let mut attr_list: Vec<Attr> = vec![];

    for (e, rb) in graph_read_guard.iter().enumerate() {
        let mut rand = StdRng::seed_from_u64(e as u64);
        let mut highlight_mul = 1.0;

        if !highlight_index.is_empty() && !highlight_index.contains(&(e as u32)) {
            highlight_mul = 0.5;
        }

        let color_attr = [
            (rand.gen_range(10..=100) as f32) / 100.0 * highlight_mul,
            (rand.gen_range(10..=100) as f32) / 100.0 * highlight_mul,
            (rand.gen_range(10..=100) as f32) / 100.0 * highlight_mul,
        ];

        attr_list.push(Attr {
            color_attr,
            world_position: [rb.position[0], rb.position[1], 0.0],
            scale: (rb.mass / PI).sqrt() / 2.0,
        })
    }

    let vertex_buffer = glium::VertexBuffer::new(display, &shape).unwrap();
    let instance_buffer = glium::vertex::VertexBuffer::dynamic(display, &attr_list).unwrap();
    let indices = glium::index::NoIndices(glium::index::PrimitiveType::TrianglesList);

    target
        .draw(
            (&vertex_buffer, instance_buffer.per_instance().unwrap()),
            indices,
            &program,
            uniform,
            params,
        )
        .unwrap();
}
