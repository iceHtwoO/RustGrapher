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

    // Run 10k simulation steps
    for _ in 0..10000 {
        simulator.simulation_step();
    }
}
