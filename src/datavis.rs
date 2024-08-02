use std::time::{Instant, SystemTime, UNIX_EPOCH};

use crate::{simgraph::SimGraph, Graph};
use glium::{glutin::surface::WindowSurface, implement_vertex, Display, Frame, Surface};

use rand::{rngs::StdRng, Rng, SeedableRng};
use winit::{
    event::{Event, WindowEvent},
    event_loop::{self, ControlFlow},
    platform::run_return::EventLoopExtRunReturn,
};

static VERTEX_SHADER_SRC: &str = r#"
#version 140

in vec2 position;
in vec3 color;
out vec3 vertex_color;

void main() {
    vertex_color = color;
    gl_Position =  vec4(position, 0.0, 1.0);
}
"#;

static FRAGMENT_SHADER_SRC: &str = r#"
#version 140

in vec3 vertex_color;
out vec4 color;

void main() {
    color = vec4(vertex_color, 1.0);
}
"#;
#[derive(Copy, Clone)]
struct Vertex {
    position: [f64; 2],
    color: [f32; 3],
}
implement_vertex!(Vertex, position, color);

pub struct DataVis<'a, T>
where
    T: PartialEq,
{
    graph: &'a mut Graph<T>,
    sim: SimGraph,
}

impl<'a, T> DataVis<'a, T>
where
    T: PartialEq,
{
    pub fn new(graph: &'a mut Graph<T>) -> Self {
        Self {
            graph,
            sim: SimGraph::new(),
        }
    }

    pub fn create_window<'b>(&mut self, update: &dyn Fn(&mut Graph<T>, u128)) {
        let mut lastfps = 0;

        let mut event_loop = winit::event_loop::EventLoopBuilder::new().build();

        let (window, display) =
            glium::backend::glutin::SimpleWindowBuilder::new().build(&event_loop);

        let mut now = Instant::now();

        event_loop.run_return(|event, _, control_flow| {
            *control_flow = ControlFlow::Poll;

            match event {
                Event::WindowEvent { event, .. } => match event {
                    WindowEvent::CloseRequested | WindowEvent::Destroyed => {
                        *control_flow = ControlFlow::Exit;
                    }
                    _ => (),
                },
                _ => (),
            }

            let fps = 1_000_000_000 / now.elapsed().as_nanos();
            now = Instant::now();
            update(self.graph, fps);
            self.sim.sim(self.graph, fps);
            self.draw_graph(&display);
            println!("{}", (fps + lastfps) / 2);
            lastfps = fps;
        });
    }

    fn draw_graph(&self, display: &Display<WindowSurface>) {
        let mut target = display.draw();
        target.clear_color(1.0, 1.0, 1.0, 1.0);

        let mut x_max: f64 = 0.1;
        let mut y_max: f64 = 0.1;
        for n in self.graph.get_node_iter() {
            x_max = x_max.max(n.position[0].abs());
            y_max = y_max.max(n.position[1].abs());
        }

        let scale = 1.0 / x_max.max(y_max);
        println!("Scale:{}", x_max.max(y_max));

        self.draw_edge(&mut target, display, &scale);
        self.draw_node(&mut target, display, &scale);

        target.finish().unwrap();
    }

    fn draw_edge(&self, target: &mut Frame, display: &Display<WindowSurface>, scale: &f64) {
        let program =
            glium::Program::from_source(display, VERTEX_SHADER_SRC, FRAGMENT_SHADER_SRC, None)
                .unwrap();

        let mut shape: Vec<Vertex> = vec![];

        for edge in self.graph.get_edge_iter() {
            let n1 = self.graph.get_node_by_index(edge.0);
            let n2 = self.graph.get_node_by_index(edge.1);

            let p1 = [n1.position[0] * scale, n1.position[1] * scale];
            let p2 = [n2.position[0] * scale, n2.position[1] * scale];

            shape.push(Vertex {
                position: p1,
                color: [1.0, 0.0, 0.0],
            });

            shape.push(Vertex {
                position: p2,
                color: [1.0, 0.0, 0.0],
            });
        }

        let vertex_buffer = glium::VertexBuffer::new(display, &shape).unwrap();
        let indices = glium::index::NoIndices(glium::index::PrimitiveType::LinesList);

        target
            .draw(
                &vertex_buffer,
                indices,
                &program,
                &glium::uniforms::EmptyUniforms,
                &Default::default(),
            )
            .unwrap();
    }

    fn draw_node(&self, target: &mut Frame, display: &Display<WindowSurface>, scale: &f64) {
        let program =
            glium::Program::from_source(display, VERTEX_SHADER_SRC, FRAGMENT_SHADER_SRC, None)
                .unwrap();

        let mut shape: Vec<Vertex> = vec![];

        for (e, node) in self.graph.get_node_iter().enumerate() {
            let pos = node.position;
            let s = (0.01 * node.mass * scale) / 2.0;

            if pos[0].abs() > 1.0 || pos[1].abs() > 1.0 {
                continue;
            }

            let mut r = StdRng::seed_from_u64(e as u64);
            let color = [
                (r.gen_range(0..=100) as f32) / 100.0,
                (r.gen_range(0..=100) as f32) / 100.0,
                (r.gen_range(0..=100) as f32) / 100.0,
            ];
            shape.push(Vertex {
                position: [(pos[0] - s) * scale, pos[1] * scale],
                color,
            });

            shape.push(Vertex {
                position: [(pos[0] + s) * scale, pos[1] * scale],
                color,
            });

            shape.push(Vertex {
                position: [pos[0] * scale, (pos[1] + s) * scale],
                color,
            });
        }

        let vertex_buffer = glium::VertexBuffer::new(display, &shape).unwrap();
        let indices = glium::index::NoIndices(glium::index::PrimitiveType::TrianglesList);

        target
            .draw(
                &vertex_buffer,
                indices,
                &program,
                &glium::uniforms::EmptyUniforms,
                &Default::default(),
            )
            .unwrap();
    }
}
