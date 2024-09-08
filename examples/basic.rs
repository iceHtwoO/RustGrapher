extern crate glium;
extern crate grapher;
extern crate winit;

use grapher::renderer::Renderer;
use grapher::simulator::SimulatorBuilder;

fn main() {
    let mut rng = rand::thread_rng();
    let g = petgraph_gen::barabasi_albert_graph(&mut rng, 1000, 1, None);
    let simulator = SimulatorBuilder::new()
        .delta_time(0.01)
        .freeze_threshold(-1.0)
        .build();
    let renderer = Renderer::new(simulator);

    renderer.create_window(g.into());
}
