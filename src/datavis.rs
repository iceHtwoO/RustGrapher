use core::f32;
use std::{
    fmt::Debug,
    marker::PhantomData,
    sync::{Arc, Mutex, RwLock},
    thread,
    time::Instant,
};

use crate::{graph::Graph, simgraph::SimGraph, vectors::Vector3D};
use camera::Camera;
use cgmath::Deg;
use glium::{glutin::surface::WindowSurface, implement_vertex, uniform, Display, Frame, Surface};

use winit::{
    event::{Event, WindowEvent},
    event_loop::{ControlFlow, EventLoop},
    window::Window,
};

mod camera;
mod draw;
mod shapes;

const SCROLL_SENSITIVITY: f32 = 2.0;

#[derive(Copy, Clone, Debug)]
struct Vertex {
    position: [f32; 3],
    color: [f32; 4],
}
implement_vertex!(Vertex, position, color);

pub struct DataVis<T>
where
    T: PartialEq + Send + Sync + 'static + Clone,
{
    sim: SimGraph<T>,
    phantom: PhantomData<T>,
}

impl<T> DataVis<T>
where
    T: PartialEq + Send + Sync + 'static + Clone + Debug + Default,
{
    pub fn new() -> Self {
        Self {
            sim: SimGraph::new(),
            phantom: PhantomData,
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
        //Timing
        let mut last_redraw = Instant::now();
        let mut last_pause = Instant::now();
        // Camera
        let mut camera = Camera::new(Vector3D::new([0.0, 0.0, 5.0]));
        camera.look_at(&Vector3D::new([0.0, 0.0, 0.0]));
        let mut cursor = winit::dpi::PhysicalPosition::new(0.0, 0.0);

        //Config
        let toggle_sim = Arc::new(RwLock::new(false));
        let mut toggle_quadtree = false;

        let sim = self.sim.clone();

        let self_arc = Arc::new(Mutex::new(self));
        let display_arc = Arc::new(display);
        let graph = Arc::new(RwLock::new(graph));

        Self::spawn_simulation_thread(Arc::clone(&toggle_sim), sim, Arc::clone(&graph));

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
                                camera.position[2] -= SCROLL_SENSITIVITY;
                            } else if y > 0.0 {
                                camera.position[2] += SCROLL_SENSITIVITY;
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
                            let avg = graph.read().unwrap().avg_pos();
                            camera.position[0] = avg[0];
                            camera.position[1] = avg[1];
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
                last_redraw = Instant::now();

                let camera_factor: f32 = 1.0;
                if cursor.x < window.inner_size().width as f64 * 0.1 {
                    camera.position[0] -= camera_factor;
                } else if cursor.x > window.inner_size().width as f64 * 0.9 {
                    camera.position[0] += camera_factor;
                }
                if cursor.y < window.inner_size().height as f64 * 0.1 {
                    camera.position[1] += camera_factor;
                } else if cursor.y > window.inner_size().height as f64 * 0.9 {
                    camera.position[1] -= camera_factor;
                }

                self_mutex.draw_graph(&display_arc, Arc::clone(&graph), &camera, toggle_quadtree);
            }
        });
    }

    fn draw_graph(
        &mut self,
        display: &Display<WindowSurface>,
        graph: Arc<RwLock<Graph<T>>>,
        camera: &Camera,
        enable_quadtree: bool,
    ) {
        let mut target = display.draw();
        target.clear_color_and_depth((0.0, 0.0, 0.0, 1.0), 1.0);

        let max_mass = Self::find_max_mass(Arc::clone(&graph));

        let uniforms = uniform! {
            matrix: camera.matrix(),
            projection: build_perspective_matrix(&target)
        };

        let params = glium::DrawParameters {
            depth: glium::Depth {
                test: glium::draw_parameters::DepthTest::IfLess,
                write: true,
                ..Default::default()
            },
            ..Default::default()
        };

        draw::draw_edge(
            Arc::clone(&graph),
            &mut target,
            display,
            &max_mass,
            &uniforms,
            &params,
        );
        draw::draw_node(
            Arc::clone(&graph),
            &mut target,
            display,
            &max_mass,
            &uniforms,
            &params,
        );
        if enable_quadtree {
            draw::draw_quadtree(Arc::clone(&graph), &mut target, display, &uniforms, &params);
        }

        target.finish().unwrap();
    }

    fn spawn_simulation_thread(
        toggle_sim: Arc<RwLock<bool>>,
        mut sim: SimGraph<T>,
        graph: Arc<RwLock<Graph<T>>>,
    ) {
        thread::spawn(move || loop {
            let toggle_sim_read_guard = toggle_sim.read().unwrap();
            let sim_toggle = *toggle_sim_read_guard;
            drop(toggle_sim_read_guard);

            if sim_toggle {
                sim.simulation_step(Arc::clone(&graph));
            }
        });
    }

    fn find_max_mass(graph: Arc<RwLock<Graph<T>>>) -> f32 {
        let graph_read_guard = graph.read().unwrap();
        let mut max_m = 0.0;
        for n in graph_read_guard.get_node_iter() {
            max_m = n.mass.max(max_m);
        }
        max_m
    }
}

fn build_perspective_matrix(target: &Frame) -> [[f32; 4]; 4] {
    let (width, height) = target.get_dimensions();
    cgmath::perspective(Deg(45.0f32), width as f32 / height as f32, 0.1, 1000.0).into()
}
