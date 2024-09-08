use core::f32;
use std::{
    collections::HashSet,
    fmt::Debug,
    rc::Rc,
    sync::{Arc, Mutex, RwLock},
    thread,
    time::Instant,
};

use crate::simulator::Simulator;
use camera::Camera;
use glam::{Mat4, Vec3};
use glium::{glutin::surface::WindowSurface, implement_vertex, uniform, Display, Frame, Surface};

use winit::{
    event::{ElementState, Event, WindowEvent},
    event_loop::{ControlFlow, EventLoop},
    window::Window,
};

mod camera;
mod draw;
mod shapes;

const SCROLL_SENSITIVITY: f32 = 2.0;
const CAMERA_MOVEMENT_SENSITIVITY: f32 = 40.0;

#[derive(Copy, Clone, Debug)]
struct Vertex {
    position: [f32; 3],
    color: [f32; 4],
}
implement_vertex!(Vertex, position, color);

/// Renders a petgraph `StableGraph`
pub struct Renderer {
    simulator: Arc<Simulator>,
}

impl Renderer {
    /// Creates a instance of the `Renderer`
    pub fn new(simulator: Simulator) -> Self {
        Self {
            simulator: Arc::new(simulator),
        }
    }

    /// Creates a window and renders all the nodes and edges of given stable graph
    pub fn create_window(self) {
        let event_loop = winit::event_loop::EventLoopBuilder::new().build();

        let (window, display) =
            glium::backend::glutin::SimpleWindowBuilder::new().build(&event_loop);

        self.run_render_loop(event_loop, display, window);
    }

    #[allow(unused_variables)]
    fn run_render_loop(
        self,
        event_loop: EventLoop<()>,
        display: Display<WindowSurface>,
        window: Window,
    ) {
        //Timing
        let mut last_redraw = Instant::now();
        let mut last_event_cycle = Instant::now();
        let mut last_pause = Instant::now();
        // Camera
        let mut camera = Camera::new(Vec3::new(0.0, 0.0, 5.0));
        camera.look_at(&Vec3::ZERO);

        //Config
        let toggle_sim = Arc::new(RwLock::new(false));

        let sim: Arc<Simulator> = Arc::clone(&self.simulator);
        self.spawn_simulation_thread(Arc::clone(&toggle_sim));

        let self_arc = Arc::new(Mutex::new(self));
        let display_rc = Rc::new(display);

        let mut keys_held = HashSet::new();

        event_loop.run(move |event, _, control_flow| {
            *control_flow = ControlFlow::Poll;

            let delta_time = last_event_cycle.elapsed().as_secs_f32();
            last_event_cycle = Instant::now();

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
                    WindowEvent::KeyboardInput { input, .. } => match input.virtual_keycode {
                        Some(winit::event::VirtualKeyCode::Space) => {
                            if last_pause.elapsed().as_millis() >= 400 {
                                let mut toggle_sim_write_guard = toggle_sim.write().unwrap();
                                *toggle_sim_write_guard = !(*toggle_sim_write_guard);
                                last_pause = Instant::now();
                            }
                        }
                        Some(winit::event::VirtualKeyCode::Return) => {
                            let avg = sim.average_node_position();
                            camera.position[0] = avg[0];
                            camera.position[1] = avg[1];
                        }
                        Some(keycode) => match input.state {
                            ElementState::Pressed => {
                                keys_held.insert(keycode);
                            }
                            ElementState::Released => {
                                keys_held.remove(&keycode);
                            }
                        },
                        None => (),
                    },
                    _ => (),
                },
                _ => (),
            }

            let self_arc_clone = Arc::clone(&self_arc);
            let mut self_mutex = self_arc_clone.lock().unwrap();

            // Camera movement
            if keys_held.contains(&winit::event::VirtualKeyCode::W) {
                camera.position[1] += CAMERA_MOVEMENT_SENSITIVITY * delta_time;
            }
            if keys_held.contains(&winit::event::VirtualKeyCode::S) {
                camera.position[1] -= CAMERA_MOVEMENT_SENSITIVITY * delta_time;
            }
            if keys_held.contains(&winit::event::VirtualKeyCode::A) {
                camera.position[0] -= CAMERA_MOVEMENT_SENSITIVITY * delta_time;
            }
            if keys_held.contains(&winit::event::VirtualKeyCode::D) {
                camera.position[0] += CAMERA_MOVEMENT_SENSITIVITY * delta_time;
            }

            if last_redraw.elapsed().as_millis() >= 34 {
                last_redraw = Instant::now();

                self_mutex.draw_graph(&display_rc, Arc::clone(&sim), &camera);
            }
        });
    }

    fn draw_graph(
        &mut self,
        display: &Display<WindowSurface>,
        sim: Arc<Simulator>,
        camera: &Camera,
    ) {
        let mut target = display.draw();
        target.clear_color_and_depth((0.0, 0.0, 0.0, 1.0), 1.0);

        let max_mass = { sim.max_node_mass() };

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
            Arc::clone(&sim),
            &mut target,
            display,
            &max_mass,
            &uniforms,
            &params,
        );
        draw::draw_node(
            Arc::clone(&sim),
            &mut target,
            display,
            &max_mass,
            &uniforms,
            &params,
        );

        target.finish().unwrap();
    }

    fn spawn_simulation_thread(&self, toggle_sim: Arc<RwLock<bool>>) {
        let sim = Arc::clone(&self.simulator);

        thread::spawn(move || loop {
            let toggle_sim_read_guard = toggle_sim.read().unwrap();
            let sim_toggle = *toggle_sim_read_guard;
            drop(toggle_sim_read_guard);

            if sim_toggle {
                sim.simulation_step();
            }
        });
    }
}

fn build_perspective_matrix(target: &Frame) -> Mat4 {
    let (width, height) = target.get_dimensions();
    Mat4::perspective_infinite_lh(0.8, width as f32 / height as f32, 0.1)
}
