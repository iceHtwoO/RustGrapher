use core::f32;
use std::{
    fmt::Debug,
    marker::PhantomData,
    rc::Rc,
    sync::{Arc, Mutex, RwLock},
    thread,
    time::Instant,
};

use crate::{
    properties::{RigidBody2D, Spring},
    simulator::Simulator,
};
use camera::Camera;
use glam::{Mat4, Vec2, Vec3};
use glium::{glutin::surface::WindowSurface, implement_vertex, uniform, Display, Frame, Surface};

use petgraph::{Directed, Graph};
use rand::Rng;
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

pub struct DataVis<T, E>
where
    T: PartialEq + Send + Sync + 'static + Clone,
{
    simulator: Simulator,
    phantom: PhantomData<T>,
    phantom2: PhantomData<E>,
    mass_incoming: bool,
}

impl<T, E> DataVis<T, E>
where
    T: PartialEq + Send + Sync + 'static + Clone + Debug + Default,
    E: 'static,
{
    pub fn new(simulator: Simulator) -> Self {
        Self {
            simulator,
            phantom: PhantomData,
            phantom2: PhantomData,
            mass_incoming: true,
        }
    }

    pub fn create_window(self, g: Graph<T, E, Directed, u32>) {
        let event_loop = winit::event_loop::EventLoopBuilder::new().build();

        let (window, display) =
            glium::backend::glutin::SimpleWindowBuilder::new().build(&event_loop);

        self.run_render_loop(g, event_loop, display, window);
    }

    fn run_render_loop(
        self,
        graph: Graph<T, E, Directed, u32>,
        event_loop: EventLoop<()>,
        display: Display<WindowSurface>,
        window: Window,
    ) {
        //Timing
        let mut last_redraw = Instant::now();
        let mut last_pause = Instant::now();
        // Camera
        let mut camera = Camera::new(Vec3::new(0.0, 0.0, 5.0));
        camera.look_at(&Vec3::ZERO);

        let mut cursor = winit::dpi::PhysicalPosition::new(0.0, 0.0);

        //Config
        let toggle_sim = Arc::new(RwLock::new(false));
        let mut toggle_quadtree = false;

        let sim = self.simulator.clone();

        let (rb_v, spring_v) = self.build_rb_vec(graph);
        let rb_arc = Arc::new(RwLock::new(rb_v));
        let spring_arc = Arc::new(RwLock::new(spring_v));

        let self_arc = Arc::new(Mutex::new(self));
        let display_rc = Rc::new(display);

        Self::spawn_simulation_thread(
            Arc::clone(&toggle_sim),
            sim,
            Arc::clone(&rb_arc),
            Arc::clone(&spring_arc),
        );

        event_loop.run(move |event, _, control_flow| {
            *control_flow = ControlFlow::Poll;

            #[allow(clippy::single_match)]
            #[allow(clippy::collapsible_match)]
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
                            let avg = calc_average_position(Arc::clone(&rb_arc));
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

                self_mutex.draw_graph(
                    &display_rc,
                    Arc::clone(&rb_arc),
                    Arc::clone(&spring_arc),
                    &camera,
                );
            }
        });
    }

    fn draw_graph(
        &mut self,
        display: &Display<WindowSurface>,
        rb_v: Arc<RwLock<Vec<RigidBody2D>>>,
        spring_v: Arc<RwLock<Vec<Spring>>>,
        camera: &Camera,
    ) {
        let mut target = display.draw();
        target.clear_color_and_depth((0.0, 0.0, 0.0, 1.0), 1.0);

        let max_mass = Self::find_max_mass(Arc::clone(&rb_v));

        let uniforms = uniform! {
            matrix: camera.matrix(),
            projection: build_perspective_matrix(&target).to_cols_array_2d()
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
            Arc::clone(&rb_v),
            Arc::clone(&spring_v),
            &mut target,
            display,
            &max_mass,
            &uniforms,
            &params,
        );
        draw::draw_node(
            Arc::clone(&rb_v),
            &mut target,
            display,
            &max_mass,
            &uniforms,
            &params,
        );

        target.finish().unwrap();
    }

    fn spawn_simulation_thread(
        toggle_sim: Arc<RwLock<bool>>,
        mut sim: Simulator,
        rb: Arc<RwLock<Vec<RigidBody2D>>>,
        spring: Arc<RwLock<Vec<Spring>>>,
    ) {
        thread::spawn(move || loop {
            let toggle_sim_read_guard = toggle_sim.read().unwrap();
            let sim_toggle = *toggle_sim_read_guard;
            drop(toggle_sim_read_guard);

            if sim_toggle {
                let sim_start = Instant::now();
                sim.simulation_step(Arc::clone(&rb), Arc::clone(&spring));
                println!(
                    "Simulations Per Second: {}",
                    1.0 / sim_start.elapsed().as_secs_f32()
                )
            }
        });
    }

    fn find_max_mass(graph: Arc<RwLock<Vec<RigidBody2D>>>) -> f32 {
        let graph_read_guard = graph.read().unwrap();
        let mut max_m = 0.0;
        for rb in graph_read_guard.iter() {
            max_m = rb.mass.max(max_m);
        }
        max_m
    }

    fn build_rb_vec(&self, graph: Graph<T, E, Directed, u32>) -> (Vec<RigidBody2D>, Vec<Spring>) {
        let mut vec_rb = vec![];
        let mut vec_spring = vec![];

        let (nodes, edges) = graph.into_nodes_edges();

        for _ in nodes {
            vec_rb.push(RigidBody2D::new(
                Vec2::new(
                    rand::thread_rng().gen_range(-60.0..60.0),
                    rand::thread_rng().gen_range(-60.0..60.0),
                ),
                1.0,
            ));
        }

        for s in edges {
            if self.mass_incoming {
                vec_rb[s.target().index()].mass += 1.0;
                vec_rb[s.source().index()].mass += 1.0;
            }

            vec_spring.push(Spring {
                rb1: s.source().index(),
                rb2: s.target().index(),
                spring_neutral_len: 2.0,
                spring_stiffness: 1.0,
            })
        }

        (vec_rb, vec_spring)
    }
}

fn build_perspective_matrix(target: &Frame) -> Mat4 {
    let (width, height) = target.get_dimensions();
    Mat4::perspective_infinite_lh(0.8, width as f32 / height as f32, 0.1)
}

fn calc_average_position(rb_v: Arc<RwLock<Vec<RigidBody2D>>>) -> Vec2 {
    let rb_guard = rb_v.read().unwrap();

    let mut avg = Vec2::ZERO;
    for rb in rb_guard.iter() {
        avg += rb.position;
    }

    avg / rb_guard.len() as f32
}
