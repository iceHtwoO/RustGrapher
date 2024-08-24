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
use camera::Camera;
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

mod camera;
mod draw;
mod shapes;

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
        let mut last_redraw = Instant::now();
        let mut last_pause = Instant::now();
        let scroll_scale: f32 = 1.0;
        let mut camera = Camera::new(Vector3D::new([0.0, 0.0, 0.0]));
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
                                camera.position[2] -= 0.25;
                            } else if y > 0.0 {
                                camera.position[2] += 0.25;
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
                            camera.position.set(
                                Vector2D::new(graph.read().unwrap().avg_pos())
                                    .to_3d()
                                    .get_position(),
                            );
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
                /*if cursor.x < window.inner_size().width as f64 * 0.1 {
                    camera.position[0] -= camera_factor;
                } else if cursor.x > window.inner_size().width as f64 * 0.9 {
                    camera.position[0] += camera_factor;
                }
                if cursor.y < window.inner_size().height as f64 * 0.1 {
                    camera.position[1] += camera_factor;
                } else if cursor.y > window.inner_size().height as f64 * 0.9 {
                    camera.position[1] -= camera_factor;
                }*/
                camera.look_at(&Vector3D::new([0.0, 0.0, -1.0]));

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

        let cpos = camera.position;

        let perspective = {
            let (width, height) = target.get_dimensions();
            let aspect_ratio = height as f32 / width as f32;

            let fov: f32 = 3.141592 / 3.0;
            let zfar = 1024.0;
            let znear = 0.1;

            let f = 1.0 / (fov / 2.0).tan();

            [
                [f * aspect_ratio, 0.0, 0.0, 0.0],
                [0.0, f, 0.0, 0.0],
                [0.0, 0.0, (zfar + znear) / (zfar - znear), 1.0],
                [0.0, 0.0, -(2.0 * zfar * znear) / (zfar - znear), 0.0],
            ]
        };

        let uniforms = uniform! {
            matrix: camera.matrix(),
            perspective: perspective
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
            &max_m,
            &uniforms,
            &params,
        );
        draw::draw_node(
            Arc::clone(&graph),
            &mut target,
            display,
            &max_m,
            &uniforms,
            &params,
        );
        if enable_quadtree {
            draw::draw_quadtree(Arc::clone(&graph), &mut target, display, &uniforms, &params);
        }

        target.finish().unwrap();
    }
}
