use core::f32;
use std::{
    f32::{consts::PI, INFINITY},
    sync::{Arc, RwLock},
};

use crate::{
    graph::{Graph, Node},
    quadtree::{BoundingBox2D, QuadTree},
};
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

pub fn draw_edge<T, H, R>(
    graph: Arc<RwLock<Graph<T>>>,
    target: &mut Frame,
    display: &Display<WindowSurface>,
    max_m: &f32,
    uniform: &UniformsStorage<H, R>,
    params: &DrawParameters,
) where
    H: AsUniformValue,
    R: Uniforms,
    T: std::cmp::PartialEq + std::clone::Clone,
{
    let program =
        glium::Program::from_source(display, VERTEX_SHADER_SRC, FRAGMENT_SHADER_SRC, None).unwrap();

    let mut shape: Vec<Vertex> = vec![];

    let graph_read_guard = graph.read().unwrap();

    for edge in graph_read_guard.get_edge_iter() {
        let n1 = graph_read_guard
            .get_node_by_index(edge.0)
            .rigidbody
            .as_ref()
            .unwrap();
        let n2 = graph_read_guard
            .get_node_by_index(edge.1)
            .rigidbody
            .as_ref()
            .unwrap();

        let min_m = n1.mass.min(n2.mass);
        let color = [
            (min_m / max_m) as f32 * 10.0,
            (min_m / max_m) as f32 * 6.0,
            (min_m / max_m) as f32 * 6.0,
            (min_m / max_m) as f32,
        ];

        shape.append(&mut shapes::line(
            [n1.position[0], n1.position[1], -0.1],
            [n2.position[0], n2.position[1], -0.1],
            color,
        ));
    }

    let vertex_buffer = glium::VertexBuffer::new(display, &shape).unwrap();
    let indices = glium::index::NoIndices(glium::index::PrimitiveType::LinesList);

    target
        .draw(&vertex_buffer, indices, &program, uniform, params)
        .unwrap();
}

pub fn draw_node<T, H, R>(
    graph: Arc<RwLock<Graph<T>>>,
    target: &mut Frame,
    display: &Display<WindowSurface>,
    max_m: &f32,
    uniform: &UniformsStorage<H, R>,
    params: &DrawParameters,
) where
    H: AsUniformValue,
    R: Uniforms,
    T: std::cmp::PartialEq + std::clone::Clone,
{
    let program =
        glium::Program::from_source(display, VERTEX_SHADER_SRC, FRAGMENT_SHADER_SRC, None).unwrap();

    let mut shape: Vec<Vertex> = vec![];
    let graph_read_guard = graph.read().unwrap();

    for (e, node) in graph_read_guard.get_node_iter().enumerate() {
        let rb = node.rigidbody.as_ref().unwrap();
        let pos = [rb.position[0], rb.position[1], 0.0];
        let r = f32::sqrt(rb.mass * PI) * 0.1;

        let mut rand = StdRng::seed_from_u64(e as u64);
        let color = [
            (rand.gen_range(10..=100) as f32) / 100.0,
            (rand.gen_range(10..=100) as f32) / 100.0,
            (rand.gen_range(10..=100) as f32) / 100.0,
            (rb.mass / max_m) as f32,
        ];

        shape.append(&mut shapes::circle(pos, color, r, 10));
    }

    let vertex_buffer = glium::VertexBuffer::new(display, &shape).unwrap();
    let indices = glium::index::NoIndices(glium::index::PrimitiveType::TrianglesList);

    target
        .draw(&vertex_buffer, indices, &program, uniform, params)
        .unwrap();
}

pub fn draw_quadtree<T, H, R>(
    graph: Arc<RwLock<Graph<T>>>,
    target: &mut Frame,
    display: &Display<WindowSurface>,
    uniform: &UniformsStorage<H, R>,
    params: &DrawParameters,
) where
    H: AsUniformValue,
    R: Uniforms,
    T: std::cmp::PartialEq + std::clone::Clone,
{
    let program =
        glium::Program::from_source(display, VERTEX_SHADER_SRC, FRAGMENT_SHADER_SRC, None).unwrap();

    let mut shape: Vec<Vertex> = vec![];

    let graph_read_guard = graph.read().unwrap();

    let mut max_x = -INFINITY;
    let mut min_x = INFINITY;
    let mut max_y = -INFINITY;
    let mut min_y = INFINITY;

    for node in graph_read_guard.get_node_iter() {
        let rb = node.rigidbody.as_ref().unwrap();
        max_x = max_x.max(rb.position[0]);
        max_y = max_y.max(rb.position[1]);
        min_x = min_x.min(rb.position[0]);
        min_y = min_y.min(rb.position[1]);
    }

    let w = max_x - min_x;
    let h = max_y - min_y;
    let boundary = BoundingBox2D::new([min_x + 0.5 * w, min_y + 0.5 * h], w, h);

    let mut quadtree = QuadTree::new(boundary.clone());

    for node in graph_read_guard.get_node_iter() {
        let rb = node.rigidbody.as_ref().unwrap();
        quadtree.insert(Some(node), rb.position.get_position(), rb.mass);
    }

    get_qt_vertex(&quadtree, &mut shape);

    let vertex_buffer = glium::VertexBuffer::new(display, &shape).unwrap();
    let indices = glium::index::NoIndices(glium::index::PrimitiveType::LinesList);

    target
        .draw(&vertex_buffer, indices, &program, uniform, params)
        .unwrap();
}

fn get_qt_vertex<T>(quadtree: &QuadTree<Node<T>>, shape: &mut Vec<Vertex>)
where
    T: std::cmp::PartialEq + std::clone::Clone,
{
    for child in quadtree.children.iter() {
        match child {
            Some(c) => get_qt_vertex(&c, shape),
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
