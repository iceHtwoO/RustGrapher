use core::f32;
use std::{
    f32::{consts::PI, INFINITY},
    fmt::Debug,
    marker::PhantomData,
    sync::{Arc, Mutex, RwLock},
    thread,
    time::Instant,
};

use crate::{
    graph::{Graph, Node},
    quadtree::{BoundingBox2D, QuadTree},
    simgraph::SimGraph,
    vectors::{Vector2D, Vector3D},
};
use glium::{
    glutin::surface::WindowSurface,
    implement_vertex, uniform,
    uniforms::{AsUniformValue, Uniforms, UniformsStorage},
    Display, DrawParameters, Frame, Surface,
};

use rand::{rngs::StdRng, Rng, SeedableRng};
use winit::{
    event::{Event, WindowEvent},
    event_loop::{ControlFlow, EventLoop},
    window::Window,
};

use super::{shapes, Vertex};

static VERTEX_SHADER_SRC: &str = r#"
#version 150

in vec3 position;
in vec4 color;
out vec4 vertex_color;

uniform mat4 perspective;
uniform mat4 matrix;

void main() {
    vertex_color = color;
    gl_Position = perspective* matrix * vec4(position, 1.0);
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
        let n1 = graph_read_guard.get_node_by_index(edge.0);
        let n2 = graph_read_guard.get_node_by_index(edge.1);

        let min_m = n1.mass.min(n2.mass);
        let color = [
            (min_m / max_m) as f32 * 10.0,
            (min_m / max_m) as f32 * 6.0,
            (min_m / max_m) as f32 * 6.0,
            (min_m / max_m) as f32,
        ];

        shape.append(&mut shapes::line(
            [n1.position[0], n1.position[1], 0.1],
            [n2.position[0], n2.position[1], 0.1],
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
        let pos = [node.position[0], node.position[1], 0.1];
        let r = f32::sqrt(node.mass * PI) * 0.1;

        let mut rand = StdRng::seed_from_u64(e as u64);
        let color = [
            (rand.gen_range(10..=100) as f32) / 100.0,
            (rand.gen_range(10..=100) as f32) / 100.0,
            (rand.gen_range(10..=100) as f32) / 100.0,
            (node.mass / max_m) as f32,
        ];

        shape.append(&mut shapes::circle(pos, color, r, 5));
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
        max_x = max_x.max(node.position[0]);
        max_y = max_y.max(node.position[1]);
        min_x = min_x.min(node.position[0]);
        min_y = min_y.min(node.position[1]);
    }

    let w = max_x - min_x;
    let h = max_y - min_y;
    let boundary = BoundingBox2D::new([min_x + 0.5 * w, min_y + 0.5 * h], w, h);

    let mut quadtree = QuadTree::new(boundary.clone());

    for node in graph_read_guard.get_node_iter() {
        quadtree.insert(Some(node), node.position, node.mass);
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

    let pos = [boundary.center[0], boundary.center[1], 0.0];

    shape.append(&mut shapes::rectangle_lines(
        pos,
        [0.0, 0.0, 1.0, 1.0],
        boundary.width * 0.5,
        boundary.height * 0.5,
    ));
}
