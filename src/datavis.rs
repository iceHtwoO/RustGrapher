use core::f64;
use std::{
    f64::{consts::PI, INFINITY},
    sync::{Arc, Mutex},
    time::{Instant, SystemTime, UNIX_EPOCH},
};

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
in vec4 color;
out vec4 vertex_color;

void main() {
    vertex_color = color;
    gl_Position =  vec4(position, 0.0, 1.0);
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
struct Vertex {
    position: [f64; 2],
    color: [f32; 4],
}
implement_vertex!(Vertex, position, color);

pub struct DataVis<T>
where
    T: PartialEq + Send + Sync + 'static + Clone,
{
    graph: Arc<Mutex<Graph<T>>>,
    sim: SimGraph,
}

impl<T> DataVis<T>
where
    T: PartialEq + Send + Sync + 'static + Clone,
{
    pub fn new(graph: Graph<T>) -> Self {
        Self {
            graph: Arc::new(Mutex::new(graph)),
            sim: SimGraph::new(),
        }
    }

    pub fn create_window<'b>(&mut self, update: &dyn Fn(Arc<Mutex<Graph<T>>>, u128)) {
        let mut lastfps = 0;

        let mut event_loop = winit::event_loop::EventLoopBuilder::new().build();

        let (window, display) =
            glium::backend::glutin::SimpleWindowBuilder::new().build(&event_loop);

        let mut now = Instant::now();
        let mut last_redraw = Instant::now();

        let mut scroll_scale: f64 = 1.0;

        event_loop.run_return(|event, _, control_flow| {
            *control_flow = ControlFlow::Poll;

            let fps = 1_000_000_000 / now.elapsed().as_nanos();
            now = Instant::now();
            match event {
                Event::WindowEvent { event, .. } => match event {
                    WindowEvent::CloseRequested | WindowEvent::Destroyed => {
                        *control_flow = ControlFlow::Exit;
                    }
                    WindowEvent::MouseWheel {
                        device_id,
                        delta,
                        phase,
                        modifiers,
                    } => match delta {
                        winit::event::MouseScrollDelta::LineDelta(x, y) => {
                            scroll_scale += y as f64 * 0.5;
                        }
                        _ => (),
                    },
                    _ => (),
                },
                _ => (),
            }

            update(Arc::clone(&self.graph), fps);

            self.sim.clone().sim(Arc::clone(&self.graph), fps);
            println!("FIN");
            if last_redraw.elapsed().as_millis() >= 17 {
                println!("DRAW");
                self.draw_graph(&display, &scroll_scale);
                println!("DRAW DONE");
                println!("FPS{}", (fps + lastfps) / 2);
                last_redraw = Instant::now();
            }

            lastfps = fps;
        });
    }

    fn draw_graph(&mut self, display: &Display<WindowSurface>, scroll_scale: &f64) {
        let mut target = display.draw();
        target.clear_color(1.0, 1.0, 1.0, 1.0);

        let mut max: f64 = -INFINITY;
        let mut min: f64 = INFINITY;
        println!("GET LOCK");
        let mutex = self.graph.lock().unwrap();
        println!("GOT LOCK");
        let avg_pos = mutex.avg_pos();
        for (i, n) in mutex.get_node_iter().enumerate() {
            max = max.max(n.position[0]);
            min = min.min(n.position[0]);
        }

        let scale = 2.0 / (((max - min).abs()).max(100.0) + 0.01) + 0.001;

        //println!("SCALE{}", 2.0 / scale);

        let mut max_m = 0.0;
        for n in mutex.get_node_iter() {
            max_m = n.mass.max(max_m);
        }
        drop(mutex);

        println!("EDGE");
        self.draw_edge(&mut target, display, &scale, &max_m, &avg_pos);
        println!("NODE");
        self.draw_node(&mut target, display, &scale, &max_m, &avg_pos);
        println!("DONE DRAW");

        target.finish().unwrap();

        let image: glium::texture::RawImage2d<'_, u8> = display.read_front_buffer().unwrap();
        let image =
            image::ImageBuffer::from_raw(image.width, image.height, image.data.into_owned())
                .unwrap();
        let image = image::DynamicImage::ImageRgba8(image).into_rgba16();
        image.save("glium-example-screenshot.png").unwrap();
    }

    fn draw_edge(
        &self,
        target: &mut Frame,
        display: &Display<WindowSurface>,
        scale: &f64,
        max_m: &f64,
        avg: &[f64; 2],
    ) {
        let program =
            glium::Program::from_source(display, VERTEX_SHADER_SRC, FRAGMENT_SHADER_SRC, None)
                .unwrap();

        let mut shape: Vec<Vertex> = vec![];

        let graph_mutex = self.graph.lock().unwrap();

        for edge in graph_mutex.get_edge_iter() {
            let n1 = graph_mutex.get_node_by_index(edge.0);
            let n2 = graph_mutex.get_node_by_index(edge.1);
            let p1 = [
                (n1.position[0] - avg[0]) * scale,
                (n1.position[1] - avg[1]) * scale,
            ];
            let p2 = [
                (n2.position[0] - avg[0]) * scale,
                (n2.position[1] - avg[1]) * scale,
            ];

            let min_m = n1.mass.min(n2.mass);

            shape.push(Vertex {
                position: p1,
                color: [
                    1.0,
                    1.0 - (min_m / max_m) as f32 * 6.0,
                    1.0 - (min_m / max_m) as f32 * 6.0,
                    (min_m / max_m) as f32,
                ],
            });

            shape.push(Vertex {
                position: p2,
                color: [
                    1.0,
                    1.0 - (min_m / max_m) as f32 * 6.0,
                    1.0 - (min_m / max_m) as f32 * 6.0,
                    (min_m / max_m) as f32,
                ],
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

    fn draw_node(
        &self,
        target: &mut Frame,
        display: &Display<WindowSurface>,
        scale: &f64,
        max_m: &f64,
        avg: &[f64; 2],
    ) {
        let program =
            glium::Program::from_source(display, VERTEX_SHADER_SRC, FRAGMENT_SHADER_SRC, None)
                .unwrap();

        let mut shape: Vec<Vertex> = vec![];

        let graph_mutex = self.graph.lock().unwrap();

        for (e, node) in graph_mutex.get_node_iter().enumerate() {
            let mut pos = node.position;
            let r = (f64::sqrt(node.mass * PI) * scale) * 0.1;

            pos[0] -= avg[0];
            pos[1] -= avg[1];

            pos[0] *= scale;
            pos[1] *= scale;

            let mut rand = StdRng::seed_from_u64(e as u64);
            let color = [
                (rand.gen_range(0..=100) as f32) / 100.0,
                (rand.gen_range(0..=100) as f32) / 100.0,
                (rand.gen_range(0..=100) as f32) / 100.0,
                (node.mass / max_m) as f32,
            ];
            let mut v = create_circle(pos, color, r, 30);
            shape.append(&mut v);
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

fn create_cube(pos: [f64; 2], color: [f32; 4], s: f64) -> Vec<Vertex> {
    let mut shape = Vec::with_capacity(6);
    shape.push(Vertex {
        position: [pos[0] - s, pos[1] - s],
        color,
    });

    shape.push(Vertex {
        position: [pos[0] + s, pos[1] - s],
        color,
    });

    shape.push(Vertex {
        position: [pos[0] - s, pos[1] + s],
        color,
    });
    shape.push(Vertex {
        position: [pos[0] + s, pos[1] + s],
        color,
    });

    shape.push(Vertex {
        position: [pos[0] + s, pos[1] - s],
        color,
    });

    shape.push(Vertex {
        position: [pos[0] - s, pos[1] + s],
        color,
    });
    shape
}

fn create_circle(pos: [f64; 2], color: [f32; 4], r: f64, res: usize) -> Vec<Vertex> {
    let mut shape = Vec::with_capacity(3 * res);
    let a = 2.0 * PI / res as f64;

    for i in 0..res {
        let i = i as f64;
        shape.push(Vertex {
            position: pos,
            color,
        });
        shape.push(Vertex {
            position: [
                pos[0] + (r * f64::sin(a * i)),
                pos[1] + (r * f64::cos(a * i)),
            ],
            color,
        });
        shape.push(Vertex {
            position: [
                pos[0] + (r * f64::sin(a * (i + 1.0))),
                pos[1] + (r * f64::cos(a * (i + 1.0))),
            ],
            color,
        });
    }

    shape
}
