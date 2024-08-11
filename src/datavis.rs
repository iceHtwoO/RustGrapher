use core::f32;
use std::{
    f32::{consts::PI, INFINITY},
    fmt::Debug,
    marker::PhantomData,
    sync::{Arc, Mutex},
    time::Instant,
};

use crate::{simgraph::SimGraph, Graph};
use glium::{glutin::surface::WindowSurface, implement_vertex, Display, Frame, Surface};

use plotly::{layout::Axis, Layout, Plot, Scatter};
use rand::{rngs::StdRng, Rng, SeedableRng};
use winit::{
    event::{Event, WindowEvent},
    event_loop::{ControlFlow, EventLoop},
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
    sim: SimGraph,
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

    pub fn create_window(mut self, g: Graph<T>) {
        let event_loop = winit::event_loop::EventLoopBuilder::new().build();

        let (_window, display) =
            glium::backend::glutin::SimpleWindowBuilder::new().build(&event_loop);

        self.run_render_loop(g, event_loop, display);
    }

    fn run_render_loop(
        mut self,
        mut graph: Graph<T>,
        mut event_loop: EventLoop<()>,
        display: Display<WindowSurface>,
    ) {
        let mut now = Instant::now();
        let mut last_redraw = Instant::now();
        let mut last_pause = Instant::now();
        let mut scroll_scale: f32 = 1.0;
        let mut lastfps = 0;
        let mut toggle_sim = false;
        let self_arc = Arc::new(Mutex::new(self));
        let display_arc = Arc::new(display);

        event_loop.run(move |event, _, control_flow| {
            *control_flow = ControlFlow::Poll;

            let fps = 1_000_000_000 / now.elapsed().as_nanos();
            now = Instant::now();
            match event {
                Event::WindowEvent { event, .. } => match event {
                    WindowEvent::CloseRequested | WindowEvent::Destroyed => {
                        *control_flow = ControlFlow::Exit;
                    }
                    WindowEvent::MouseWheel { delta, .. } => match delta {
                        winit::event::MouseScrollDelta::LineDelta(_, y) => {
                            scroll_scale += y as f32 * 0.5;
                        }
                        _ => (),
                    },
                    WindowEvent::KeyboardInput { input, .. } => match input.virtual_keycode {
                        Some(winit::event::VirtualKeyCode::Space) => {
                            if last_pause.elapsed().as_millis() >= 100 {
                                toggle_sim = !toggle_sim;
                                last_pause = Instant::now();
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
            let x = self_mutex.sim.s_per_update > 0.0;
            if toggle_sim && x {
                let mut g_taken = std::mem::take(&mut graph);
                g_taken = self_mutex.sim.sim(g_taken);
                graph = g_taken;

                let kinetic_energy = self_mutex.sim.kinetic_energy;
                let spring_energy = self_mutex.sim.spring_energy;
                let repulsion_energy = self_mutex.sim.repulsion_energy;
                let pot_energy = self_mutex.sim.pot_energy;
                self_mutex.energy.push(Energy {
                    kinetic: kinetic_energy,
                    spring: spring_energy,
                    repulsion_energy,
                    pot_energy,
                });
                self_mutex.plot_data();
            }
            drop(self_mutex);

            let self_arc_clone = Arc::clone(&self_arc);
            let mut self_mutex = self_arc_clone.lock().unwrap();

            if last_redraw.elapsed().as_millis() >= 34 {
                self_mutex.draw_graph(&display_arc, &graph, &scroll_scale);
                println!("FPS{}", (fps + lastfps) / 2);
                last_redraw = Instant::now();
            }

            lastfps = fps;
        });
    }

    fn draw_graph(&mut self, display: &Display<WindowSurface>, g: &Graph<T>, scroll_scale: &f32) {
        let mut target = display.draw();
        target.clear_color(0.0, 0.0, 0.0, 1.0);

        let mut max: f32 = -INFINITY;
        let mut min: f32 = INFINITY;
        let avg_pos = g.avg_pos();
        for n in g.get_node_iter() {
            max = max.max(n.position[0]);
            min = min.min(n.position[0]);
        }

        let scale = 2.0 / (scroll_scale * 20.0);

        let mut max_m = 0.0;
        for n in g.get_node_iter() {
            max_m = n.mass.max(max_m);
        }
        self.draw_edge(g, &mut target, display, &scale, &max_m, &avg_pos);
        self.draw_node(g, &mut target, display, &scale, &max_m, &avg_pos);

        target.finish().unwrap();
    }

    fn draw_edge(
        &self,
        g: &Graph<T>,
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

        shape.append(&mut shapes::line(
            [-1.0, -0.9],
            [(1.0 * scale) - 1.0, -0.9],
            [0.0, 1.0, 0.0, 1.0],
        ));

        for edge in g.get_edge_iter() {
            let n1 = g.get_node_by_index(edge.0);
            let n2 = g.get_node_by_index(edge.1);
            let p1 = [
                (n1.position[0] - avg[0]) * scale,
                (n1.position[1] - avg[1]) * scale,
            ];
            let p2 = [
                (n2.position[0] - avg[0]) * scale,
                (n2.position[1] - avg[1]) * scale,
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
        g: &Graph<T>,
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

        for (e, node) in g.get_node_iter().enumerate() {
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
