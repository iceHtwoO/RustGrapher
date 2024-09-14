use core::f32;
use std::{
    fmt::Debug,
    rc::Rc,
    sync::{Arc, Mutex, RwLock},
    thread,
    time::Instant,
};

use crate::simulator::Simulator;
use camera::Camera;
use event::EventManager;
use glam::{Mat4, Vec2, Vec3, Vec4, Vec4Swizzles};
use glium::{glutin::surface::WindowSurface, implement_vertex, uniform, Display, Surface};

use rand::Rng;
use winit::{
    event::{ElementState, Event, WindowEvent},
    event_loop::{ControlFlow, EventLoop},
    window::Window,
};

mod camera;
mod draw;
mod event;
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
        // Timing
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

        let mut event_manager = EventManager::new();

        let mut cursor_pos = Vec2::ZERO;

        let mut selected_node: Option<u32> = None;

        let mut place_mode = false;

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
                    WindowEvent::MouseInput {
                        device_id,
                        state,
                        button,
                        ..
                    } => match state {
                        ElementState::Pressed => {
                            event_manager.insert_mouse_button(button);
                        }
                        ElementState::Released => {
                            event_manager.remove_mouse_button(&button);
                        }
                    },
                    WindowEvent::CursorMoved { position, .. } => {
                        cursor_pos[0] = position.x as f32;
                        cursor_pos[1] = position.y as f32;
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
                            let avg = sim.average_node_position();
                            camera.position[0] = avg[0];
                            camera.position[1] = avg[1];
                        }
                        Some(keycode) => match input.state {
                            ElementState::Pressed => {
                                event_manager.insert_key(keycode);
                            }
                            ElementState::Released => {
                                event_manager.remove_key(&keycode);
                            }
                        },
                        None => (),
                    },
                    _ => (),
                },
                _ => (),
            }

            let mut highlight_index = vec![];

            camera_movement(&mut event_manager, &mut camera, delta_time);

            if let Some(event) = event_manager.get_key_event(&winit::event::VirtualKeyCode::P) {
                if event.is_initial_check() {
                    place_mode = !place_mode;
                }
            }

            if let Some(input_event) =
                event_manager.get_mouse_button_event(&winit::event::MouseButton::Left)
            {
                let vector = cursor_pos_to_world_vec(&window, &camera, &cursor_pos);
                let intersection_point = vector_plane_intersection(
                    vector,
                    camera.position,
                    Vec4::new(0.0, 0.0, 1.0, 0.0),
                    2,
                );

                if !place_mode {
                    if (*input_event).is_initial_check() {
                        selected_node = sim.find_closest_node_index(intersection_point);
                    }

                    if let Some(index) = selected_node {
                        sim.set_node_location_by_index(intersection_point, index);
                        highlight_index.push(index);
                    }
                } else if (input_event.is_initial_check()
                    || input_event.time_engaged().as_secs_f32() > 0.5)
                    && !*toggle_sim.read().unwrap()
                {
                    input_event.reset_timer();
                    sim.insert_node(intersection_point);
                }
            }

            if last_redraw.elapsed().as_millis() >= 34 {
                let mut self_mutex = self_arc.lock().unwrap();
                last_redraw = Instant::now();
                self_mutex.draw_graph(
                    &display_rc,
                    Arc::clone(&sim),
                    &camera,
                    &window,
                    highlight_index,
                );
            }
        });
    }

    fn draw_graph(
        &mut self,
        display: &Display<WindowSurface>,
        sim: Arc<Simulator>,
        camera: &Camera,
        window: &Window,
        highlight_index: Vec<u32>,
    ) {
        let mut target = display.draw();
        target.clear_color_and_depth((0.0, 0.0, 0.0, 1.0), 1.0);

        let max_mass = { sim.max_node_mass() };

        let uniforms = uniform! {
            matrix: camera.matrix().to_cols_array_2d(),
            projection: build_perspective_matrix(window).to_cols_array_2d()
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
            &uniforms,
            &params,
            highlight_index,
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

fn build_perspective_matrix(window: &Window) -> Mat4 {
    let width = window.inner_size().width;
    let height = window.inner_size().height;
    Mat4::perspective_infinite_rh(0.8, width as f32 / height as f32, 0.1)
}

fn vector_plane_intersection(vec: Vec3, off: Vec3, plane: Vec4, accuracy: u32) -> Vec3 {
    let f = |r: f32| (plane.xyz() * (vec * r + off)).element_sum() - plane.w;
    let f_d = || (plane.xyz() * vec).element_sum();

    let mut r_approx = rand::thread_rng().gen_range(-100.0..100.0);

    loop {
        let r_before = r_approx;

        r_approx = r_approx - (f(r_approx) / f_d());

        if (r_approx * 10.0_f32.powi(accuracy as i32)).round()
            == (r_before * 10.0_f32.powi(accuracy as i32)).round()
        {
            return -vec * r_approx + off;
        }
    }
}

fn cursor_pos_to_world_vec(window: &Window, camera: &Camera, view_space_coordinate: &Vec2) -> Vec3 {
    let clip_ray = calculate_mouse_ray(window, view_space_coordinate);
    let mut x = build_perspective_matrix(window).inverse() * clip_ray;
    x[2] = 1.0;
    x[3] = 0.0;
    x = camera.matrix().inverse() * x;
    x.xyz() * -1.0
}

fn calculate_mouse_ray(window: &Window, view_space_coordinate: &Vec2) -> Vec4 {
    let normalized_view_space = normalize_view_space(window, view_space_coordinate);
    Vec4::new(
        normalized_view_space[0],
        -normalized_view_space[1],
        1.0,
        1.0,
    )
}

// -1 <= 2x/xm-1 <= 1
fn normalize_view_space(window: &Window, view_space_coordinate: &Vec2) -> Vec2 {
    let width = window.inner_size().width;
    let height = window.inner_size().height;

    let mut normalized_view_space = view_space_coordinate * 2.0;

    normalized_view_space[0] /= width as f32;
    normalized_view_space[1] /= height as f32;

    normalized_view_space - 1.0
}

fn camera_movement(event_manager: &mut EventManager, camera: &mut Camera, delta_time: f32) {
    // Camera movement
    if event_manager.contains_key(&winit::event::VirtualKeyCode::W) {
        camera.position[1] += CAMERA_MOVEMENT_SENSITIVITY * delta_time;
    }
    if event_manager.contains_key(&winit::event::VirtualKeyCode::S) {
        camera.position[1] -= CAMERA_MOVEMENT_SENSITIVITY * delta_time;
    }
    if event_manager.contains_key(&winit::event::VirtualKeyCode::A) {
        camera.position[0] -= CAMERA_MOVEMENT_SENSITIVITY * delta_time;
    }
    if event_manager.contains_key(&winit::event::VirtualKeyCode::D) {
        camera.position[0] += CAMERA_MOVEMENT_SENSITIVITY * delta_time;
    }
}
