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
    scene_context: Arc<Mutex<SceneContext>>,
}

impl Renderer {
    /// Creates a instance of the `Renderer`
    pub fn new(simulator: Simulator) -> Self {
        let scene_context = SceneContext::new(simulator);
        Self {
            scene_context: Arc::new(Mutex::new(scene_context)),
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

        let scene_context_arc: Arc<Mutex<SceneContext>> = Arc::clone(&self.scene_context);
        self.spawn_simulation_thread();

        let display_rc = Rc::new(display);

        event_loop.run(move |event, _, control_flow| {
            *control_flow = ControlFlow::Poll;

            #[allow(clippy::collapsible_match)]
            if let Event::WindowEvent { event, .. } = &event {
                match event {
                    WindowEvent::CloseRequested | WindowEvent::Destroyed => {
                        *control_flow = ControlFlow::Exit;
                    }
                    _ => (),
                }
            }

            events(&event, Arc::clone(&scene_context_arc));

            let mut scene_context = scene_context_arc.lock().unwrap();

            let delta_time = last_event_cycle.elapsed().as_secs_f32();
            last_event_cycle = Instant::now();

            let mut highlight_index = vec![];

            camera_movement(&mut scene_context, delta_time);

            if let Some(event) = scene_context
                .event_manager
                .get_key_event_mut(&winit::event::VirtualKeyCode::P)
            {
                if event.is_initial_check() {
                    scene_context.place_mode = !scene_context.place_mode;
                }
            }

            if scene_context
                .event_manager
                .contains_mouse_button(&winit::event::MouseButton::Left)
            {
                let sim = Arc::clone(&scene_context.simulator);

                let vector = cursor_pos_to_world_vec(
                    &window,
                    &scene_context.camera,
                    &scene_context.cursor_pos,
                );
                let intersection_point = vector_plane_intersection(
                    vector,
                    scene_context.camera.position,
                    Vec4::new(0.0, 0.0, 1.0, 0.0),
                    2,
                );

                let is_initial;
                let time_engaged;
                {
                    let event = scene_context
                        .event_manager
                        .get_mouse_button_event_mut(&winit::event::MouseButton::Left)
                        .unwrap();
                    is_initial = event.is_initial_check();
                    time_engaged = event.time_engaged();
                }

                if !scene_context.place_mode {
                    let selected_node = &mut scene_context.selected_node_index;

                    if is_initial {
                        *selected_node = sim.find_closest_node_index(intersection_point);
                    }

                    if let Some(index) = *selected_node {
                        sim.set_node_location_by_index(intersection_point, index);
                        highlight_index.push(index);
                    }
                } else if (is_initial || time_engaged.as_secs_f32() > 0.5)
                    && !*scene_context.toggle_sim.read().unwrap()
                {
                    scene_context
                        .event_manager
                        .get_mouse_button_event_mut(&winit::event::MouseButton::Left)
                        .unwrap()
                        .reset_timer();
                    sim.insert_node(intersection_point);
                }
            }

            drop(scene_context);

            if last_redraw.elapsed().as_millis() >= 34 {
                last_redraw = Instant::now();
                draw_graph(
                    Arc::clone(&scene_context_arc),
                    &display_rc,
                    &window,
                    &highlight_index,
                );
            }
        });
    }

    fn spawn_simulation_thread(&self) {
        let sim;
        let toggle_sim;
        {
            let scene_context = self.scene_context.lock().unwrap();
            sim = Arc::clone(&scene_context.simulator);
            toggle_sim = Arc::clone(&scene_context.toggle_sim);
        }

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

struct SceneContext {
    camera: Camera,
    event_manager: EventManager,
    cursor_pos: Vec2,
    selected_node_index: Option<u32>,
    simulator: Arc<Simulator>,
    last_pause: Instant,

    toggle_sim: Arc<RwLock<bool>>,
    place_mode: bool,
}

impl SceneContext {
    pub fn new(simulator: Simulator) -> Self {
        let mut camera = Camera::new(Vec3::new(0.0, 0.0, 150.0));
        camera.look_at(&Vec3::ZERO);

        Self {
            camera,
            event_manager: EventManager::new(),
            cursor_pos: Vec2::ZERO,
            selected_node_index: None,
            simulator: Arc::new(simulator),
            last_pause: Instant::now(),
            toggle_sim: Arc::new(RwLock::new(false)),
            place_mode: false,
        }
    }
}

fn draw_graph(
    scene_context: Arc<Mutex<SceneContext>>,
    display: &Display<WindowSurface>,
    window: &Window,
    highlight_index: &[u32],
) {
    let mut target = display.draw();
    target.clear_color_and_depth((0.0, 0.0, 0.0, 1.0), 1.0);

    let uniforms = uniform! {
        matrix: scene_context.lock().unwrap().camera.matrix().to_cols_array_2d(),
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
        Arc::clone(&scene_context),
        &mut target,
        display,
        &uniforms,
        &params,
    );
    draw::draw_node(
        Arc::clone(&scene_context),
        &mut target,
        display,
        &uniforms,
        &params,
        highlight_index,
    );

    target.finish().unwrap();
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

fn camera_movement(scene_context: &mut SceneContext, delta_time: f32) {
    let event_manager = &scene_context.event_manager;
    let camera = &mut scene_context.camera;

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

fn events(event: &Event<'_, ()>, scene_context: Arc<Mutex<SceneContext>>) {
    let mut scene_context = scene_context.lock().unwrap();

    #[allow(clippy::collapsible_match)]
    if let Event::WindowEvent { event, .. } = event {
        match event {
            WindowEvent::MouseWheel { delta, .. } => {
                if let winit::event::MouseScrollDelta::LineDelta(_, y) = delta {
                    if *y < 0.0 {
                        scene_context.camera.position[2] -= SCROLL_SENSITIVITY;
                    } else if *y > 0.0 {
                        scene_context.camera.position[2] += SCROLL_SENSITIVITY;
                    }
                }
            }
            WindowEvent::MouseInput { state, button, .. } => {
                let event_manager = &mut scene_context.event_manager;
                match state {
                    ElementState::Pressed => {
                        event_manager.insert_mouse_button(*button);
                    }
                    ElementState::Released => {
                        event_manager.remove_mouse_button(button);
                    }
                }
            }
            WindowEvent::CursorMoved { position, .. } => {
                scene_context.cursor_pos[0] = position.x as f32;
                scene_context.cursor_pos[1] = position.y as f32;
            }
            WindowEvent::KeyboardInput { input, .. } => match input.virtual_keycode {
                Some(winit::event::VirtualKeyCode::Space) => {
                    if scene_context.last_pause.elapsed().as_millis() >= 400 {
                        {
                            let mut toggle_sim_write_guard =
                                scene_context.toggle_sim.write().unwrap();
                            *toggle_sim_write_guard = !(*toggle_sim_write_guard);
                        }
                        scene_context.last_pause = Instant::now();
                    }
                }
                Some(winit::event::VirtualKeyCode::Return) => {
                    let avg = scene_context.simulator.average_node_position();
                    scene_context.camera.position[0] = avg[0];
                    scene_context.camera.position[1] = avg[1];
                }
                Some(keycode) => {
                    let event_manager = &mut scene_context.event_manager;
                    match input.state {
                        ElementState::Pressed => {
                            event_manager.insert_key(keycode);
                        }
                        ElementState::Released => {
                            event_manager.remove_key(&keycode);
                        }
                    }
                }
                None => (),
            },
            _ => (),
        }
    }
}
