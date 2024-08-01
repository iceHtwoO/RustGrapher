use std::time::{Instant, SystemTime, UNIX_EPOCH};

use crate::{simgraph::SimGraph, Graph};
use glium::{glutin::surface::WindowSurface, implement_vertex, Display, Frame, Surface};

use winit::{
    event::{Event, WindowEvent},
    event_loop::ControlFlow,
    platform::run_return::EventLoopExtRunReturn,
};

static VERTEX_SHADER_SRC: &str = r#"

    in vec2 position;

    void main() {
        gl_Position = vec4(position, 0.0, 1.0);
    }
"#;

static FRAGMENT_SHADER_SRC: &str = r#"

    out vec4 color;

    void main() {
        color = vec4(1.0, 0.0, 0.0, 1.0);
    }
"#;

static FRAGMENT_SHADER_LINE_SRC: &str = r#"

    out vec4 color;

    void main() {
        color = vec4(0.0, 1.0, 0.0, 1.0);
    }
"#;

#[derive(Copy, Clone)]
struct Vertex {
    position: [f32; 2],
}
implement_vertex!(Vertex, position);

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
            //println!("{}", (fps + lastfps) / 2);
            lastfps = fps;
        });
    }

    fn draw_graph(&self, display: &Display<WindowSurface>) {
        let mut target = display.draw();
        target.clear_color(1.0, 1.0, 1.0, 1.0);
        self.draw_edge(&mut target, display);
        self.draw_node(&mut target, display);

        target.finish().unwrap();
    }

    fn draw_edge(&self, target: &mut Frame, display: &Display<WindowSurface>) {
        let program =
            glium::Program::from_source(display, VERTEX_SHADER_SRC, FRAGMENT_SHADER_LINE_SRC, None)
                .unwrap();

        let mut shape: Vec<Vertex> = vec![];

        for edge in self.graph.get_edge_iter() {
            let n1 = self.graph.get_node_by_index(edge.0);
            let n2 = self.graph.get_node_by_index(edge.1);
            shape.push(Vertex {
                position: n1.position,
            });

            shape.push(Vertex {
                position: n2.position,
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

    fn draw_node(&self, target: &mut Frame, display: &Display<WindowSurface>) {
        let program =
            glium::Program::from_source(display, VERTEX_SHADER_SRC, FRAGMENT_SHADER_SRC, None)
                .unwrap();

        let mut shape: Vec<Vertex> = vec![];

        for node in self.graph.get_node_iter() {
            let pos = node.position;
            let s = 0.005 * node.mass / 2.0;
            shape.push(Vertex {
                position: [pos[0] - s, pos[1]],
            });

            shape.push(Vertex {
                position: [pos[0] + s, pos[1]],
            });

            shape.push(Vertex {
                position: [pos[0], pos[1] + s],
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
