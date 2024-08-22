use core::f32;
use std::{
    f32::{consts::PI, INFINITY},
    fmt::Debug,
    marker::PhantomData,
    sync::{Arc, Mutex, RwLock},
    thread,
    time::{Duration, Instant},
};

use crate::{
    graph::Graph,
    quadtree::{self, QuadTree, Rectangle},
    simgraph::SimGraph,
};
use glium::{glutin::surface::WindowSurface, implement_vertex, Display, Frame, Surface};

use plotly::{layout::Axis, Layout, Plot, Scatter};
use rand::{rngs::StdRng, Rng, SeedableRng};
use winit::{
    event::{Event, WindowEvent},
    event_loop::{ControlFlow, EventLoop},
    window::Window,
};

mod shapes;

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
    position: [f32; 2],
    color: [f32; 4],
}

struct Energy {
    kinetic: f32,
    spring: f32,
    repulsion_energy: f32,
    pot_energy: f32,
}

implement_vertex!(Vertex, position, color);

pub struct DataVis<T>
where
    T: PartialEq + Send + Sync + 'static + Clone,
{
    sim: SimGraph<T>,
    phantom: PhantomData<T>,
    energy: Vec<Energy>,
}

impl<T> DataVis<T>
where
    T: PartialEq + Send + Sync + 'static + Clone + Debug + Default,
{
    pub fn new() -> Self {
        Self {
            sim: SimGraph::new(),
            phantom: PhantomData,
            energy: vec![],
        }
    }

    pub fn create_window(self, g: Graph<T>) {
        let event_loop = winit::event_loop::EventLoopBuilder::new().build();

        let (window, display) =
            glium::backend::glutin::SimpleWindowBuilder::new().build(&event_loop);

        self.run_render_loop(g, event_loop, display, window);
    }

    fn run_render_loop(
        self,
        graph: Graph<T>,
        event_loop: EventLoop<()>,
        display: Display<WindowSurface>,
        window: Window,
    ) {
        let mut last_redraw = Instant::now();
        let mut last_pause = Instant::now();
        let mut scroll_scale: f32 = 1.0;
        let mut camera = [0.0, 0.0];
        let mut cursor = winit::dpi::PhysicalPosition::new(0.0, 0.0);

        let toggle_sim = Arc::new(RwLock::new(false));
        let mut toggle_quadtree = false;
        let mut sim = self.sim.clone();

        let self_arc = Arc::new(Mutex::new(self));
        let display_arc = Arc::new(display);
        let graph = Arc::new(RwLock::new(graph));

        let graph_clone = Arc::clone(&graph);
        let toggle_sim_clone = Arc::clone(&toggle_sim);
        thread::spawn(move || loop {
            let toggle_sim_read_guard = toggle_sim_clone.read().unwrap();
            let sim_toggle = *toggle_sim_read_guard;
            drop(toggle_sim_read_guard);
            if sim_toggle {
                sim.simulation_step(Arc::clone(&graph_clone));
            }
        });

        event_loop.run(move |event, _, control_flow| {
            *control_flow = ControlFlow::Poll;
            match event {
                Event::WindowEvent { event, .. } => match event {
                    WindowEvent::CloseRequested | WindowEvent::Destroyed => {
                        *control_flow = ControlFlow::Exit;
                    }
                    WindowEvent::MouseWheel { delta, .. } => match delta {
                        winit::event::MouseScrollDelta::LineDelta(_, y) => {
                            if y < 0.0 {
                                scroll_scale *= 0.75;
                            } else if y > 0.0 {
                                scroll_scale *= 1.25;
                            }
                        }
                        _ => (),
                    },
                    WindowEvent::CursorMoved { position, .. } => {
                        cursor = position;
                    }
                    WindowEvent::KeyboardInput { input, .. } => match input.virtual_keycode {
                        Some(winit::event::VirtualKeyCode::Space) => {
                            if last_pause.elapsed().as_millis() >= 400 {
                                let mut toggle_sim_write_guard = toggle_sim.write().unwrap();
                                *toggle_sim_write_guard = !(*toggle_sim_write_guard);
                                last_pause = Instant::now();
                            }
                        }
                        Some(winit::event::VirtualKeyCode::Return) => {
                            camera = graph.read().unwrap().avg_pos();
                        }
                        Some(winit::event::VirtualKeyCode::Q) => {
                            if last_pause.elapsed().as_millis() >= 400 {
                                toggle_quadtree = !toggle_quadtree;
                                last_pause = Instant::now()
                            }
                        }
                        _ => (),
                    },
                    _ => (),
                },
                _ => (),
            }

            let self_arc_clone = Arc::clone(&self_arc);
            let mut self_mutex = self_arc_clone.lock().unwrap();

            if last_redraw.elapsed().as_millis() >= 34 {
                let scale = 2.0 / (scroll_scale * 20.0);
                last_redraw = Instant::now();

                let camera_factor = 0.05 * scroll_scale * 20.0;
                if cursor.x < window.inner_size().width as f64 * 0.1 {
                    camera[0] -= camera_factor;
                } else if cursor.x > window.inner_size().width as f64 * 0.9 {
                    camera[0] += camera_factor;
                }
                if cursor.y < window.inner_size().height as f64 * 0.1 {
                    camera[1] += camera_factor;
                } else if cursor.y > window.inner_size().height as f64 * 0.9 {
                    camera[1] -= camera_factor;
                }
                self_mutex.draw_graph(
                    &display_arc,
                    Arc::clone(&graph),
                    &scale,
                    &camera,
                    toggle_quadtree,
                );
            }
        });
    }

    fn draw_graph(
        &mut self,
        display: &Display<WindowSurface>,
        graph: Arc<RwLock<Graph<T>>>,
        scale: &f32,
        camera: &[f32; 2],
        enable_quadtree: bool,
    ) {
        let mut target = display.draw();
        target.clear_color(0.0, 0.0, 0.0, 1.0);

        let mut max: f32 = -INFINITY;
        let mut min: f32 = INFINITY;
        let graph_read_guard = graph.read().unwrap();
        for n in graph_read_guard.get_node_iter() {
            max = max.max(n.position[0]);
            min = min.min(n.position[0]);
        }

        let mut max_m = 0.0;
        for n in graph_read_guard.get_node_iter() {
            max_m = n.mass.max(max_m);
        }
        drop(graph_read_guard);
        self.draw_edge(
            Arc::clone(&graph),
            &mut target,
            display,
            &scale,
            &max_m,
            &camera,
        );
        self.draw_node(
            Arc::clone(&graph),
            &mut target,
            display,
            &scale,
            &max_m,
            &camera,
        );
        if enable_quadtree {
            self.draw_quadtree(
                Arc::clone(&graph),
                &mut target,
                display,
                &scale,
                &max_m,
                &camera,
            );
        }

        target.finish().unwrap();
    }

    fn draw_edge(
        &self,
        graph: Arc<RwLock<Graph<T>>>,
        target: &mut Frame,
        display: &Display<WindowSurface>,
        scale: &f32,
        max_m: &f32,
        camera: &[f32; 2],
    ) {
        let program =
            glium::Program::from_source(display, VERTEX_SHADER_SRC, FRAGMENT_SHADER_SRC, None)
                .unwrap();

        let mut shape: Vec<Vertex> = vec![];

        shape.append(&mut shapes::line(
            [-1.0, -0.9],
            [(1.0 * scale) - 1.0, -0.9],
            [0.0, 1.0, 0.0, 1.0],
        ));

        let graph_read_guard = graph.read().unwrap();

        for edge in graph_read_guard.get_edge_iter() {
            let n1 = graph_read_guard.get_node_by_index(edge.0);
            let n2 = graph_read_guard.get_node_by_index(edge.1);
            let p1 = [
                (n1.position[0] - camera[0]) * scale,
                (n1.position[1] - camera[1]) * scale,
            ];
            let p2 = [
                (n2.position[0] - camera[0]) * scale,
                (n2.position[1] - camera[1]) * scale,
            ];

            let min_m = n1.mass.min(n2.mass);
            let color = [
                (min_m / max_m) as f32 * 10.0,
                (min_m / max_m) as f32 * 6.0,
                (min_m / max_m) as f32 * 6.0,
                (min_m / max_m) as f32,
            ];

            shape.append(&mut shapes::line(p1, p2, color));
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
        graph: Arc<RwLock<Graph<T>>>,
        target: &mut Frame,
        display: &Display<WindowSurface>,
        scale: &f32,
        max_m: &f32,
        avg: &[f32; 2],
    ) {
        let program =
            glium::Program::from_source(display, VERTEX_SHADER_SRC, FRAGMENT_SHADER_SRC, None)
                .unwrap();

        let mut shape: Vec<Vertex> = vec![];
        let graph_read_guard = graph.read().unwrap();

        for (e, node) in graph_read_guard.get_node_iter().enumerate() {
            let mut pos = node.position;
            let r = (f32::sqrt(node.mass * PI) * scale) * 0.1;

            pos[0] -= avg[0];
            pos[1] -= avg[1];

            pos[0] *= scale;
            pos[1] *= scale;

            let mut rand = StdRng::seed_from_u64(e as u64);
            let color = [
                (rand.gen_range(10..=100) as f32) / 100.0,
                (rand.gen_range(10..=100) as f32) / 100.0,
                (rand.gen_range(10..=100) as f32) / 100.0,
                (node.mass / max_m) as f32,
            ];

            shape.append(&mut shapes::circle(pos, color, r, 30));
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

    fn draw_quadtree(
        &self,
        graph: Arc<RwLock<Graph<T>>>,
        target: &mut Frame,
        display: &Display<WindowSurface>,
        scale: &f32,
        max_m: &f32,
        camera: &[f32; 2],
    ) {
        let program =
            glium::Program::from_source(display, VERTEX_SHADER_SRC, FRAGMENT_SHADER_SRC, None)
                .unwrap();

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
        let boundary = Rectangle::new([min_x + 0.5 * w, min_y + 0.5 * h], w, h);

        let mut quadtree = QuadTree::new(boundary.clone());

        for node in graph_read_guard.get_node_iter() {
            quadtree.insert(node.position, node.mass, &boundary);
        }
        Self::get_qt_vertex(&quadtree, &mut shape, camera, scale);

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

    fn get_qt_vertex(quadtree: &QuadTree, shape: &mut Vec<Vertex>, camera: &[f32; 2], scale: &f32) {
        for child in quadtree.children.iter() {
            match child {
                Some(c) => Self::get_qt_vertex(&c, shape, camera, scale),
                None => (),
            }
        }
        let boundary = quadtree.boundary.clone();

        let mut pos = boundary.center;

        pos[0] -= camera[0];
        pos[1] -= camera[1];

        pos[0] *= scale;
        pos[1] *= scale;
        shape.append(&mut shapes::rectangle_lines(
            pos,
            [0.0, 0.0, 1.0, 1.0],
            boundary.width * scale * 0.5,
            boundary.height * scale * 0.5,
        ));
    }

    pub fn plot_data(&self) {
        let mut x_axis = Vec::with_capacity(self.energy.len());
        let mut k_energy = Vec::with_capacity(self.energy.len());
        let mut s_energy = Vec::with_capacity(self.energy.len());
        let mut r_energy = Vec::with_capacity(self.energy.len());
        let mut pot_energy = Vec::with_capacity(self.energy.len());
        let mut sys_energy = Vec::with_capacity(self.energy.len());
        for (i, e) in self.energy.iter().enumerate() {
            x_axis.push(i);
            k_energy.push(e.kinetic);
            s_energy.push(e.spring);
            r_energy.push(e.repulsion_energy);
            pot_energy.push(e.pot_energy);
            sys_energy.push(e.repulsion_energy + e.kinetic + e.spring + e.pot_energy);
        }

        let mut plot = Plot::new();
        plot.set_layout(
            Layout::new()
                .x_axis(Axis::new().title("Frame"))
                .y_axis(Axis::new().title("J")),
        );

        let k_trace = Scatter::new(x_axis.clone(), k_energy)
            .connect_gaps(true)
            .name("Kinetic energy");

        let s_trace = Scatter::new(x_axis.clone(), s_energy)
            .connect_gaps(true)
            .name("Spring energy");

        let e_trace = Scatter::new(x_axis.clone(), r_energy)
            .connect_gaps(true)
            .name("Repulsion energy");

        let pot_trace = Scatter::new(x_axis.clone(), pot_energy)
            .connect_gaps(true)
            .name("Pot energy");

        let sys_trace = Scatter::new(x_axis, sys_energy)
            .connect_gaps(true)
            .name("System energy");

        plot.add_trace(k_trace);
        plot.add_trace(s_trace);
        plot.add_trace(e_trace);
        plot.add_trace(pot_trace);
        plot.add_trace(sys_trace);

        plot.write_html("out.html");
    }
}
