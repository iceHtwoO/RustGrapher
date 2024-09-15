use grapher::renderer::Renderer;
use grapher::simulator::SimulatorBuilder;
use petgraph::Directed;

fn main() {
    // Build a PetGraph
    let mut rng = rand::thread_rng();
    let graph: petgraph::Graph<(), (), Directed> =
        petgraph_gen::barabasi_albert_graph(&mut rng, 1000, 1, None);

    // Configure the simulator
    let simulator = SimulatorBuilder::new()
        .delta_time(0.01)
        .freeze_threshold(-1.0)
        .build(graph.into());

    // Start the renderer
    let renderer = Renderer::new(simulator);
    renderer.create_window();
}
