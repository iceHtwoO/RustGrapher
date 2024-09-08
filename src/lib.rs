//! # Example
//! ```no_run
//!use grapher::renderer::Renderer;
//!use grapher::simulator::SimulatorBuilder;
//!use petgraph::Directed;
//!
//!let mut rng = rand::thread_rng();
//!let graph: petgraph::Graph<(), (), Directed> =
//!    petgraph_gen::barabasi_albert_graph(&mut rng, 1000, 1, None);
//!
//!let simulator = SimulatorBuilder::new()
//!    .delta_time(0.01)
//!    .freeze_threshold(-1.0)
//!    .build(graph.into());
//!
//!let renderer = Renderer::new(simulator);
//!renderer.create_window();
//! ```

pub mod properties;
pub mod quadtree;
pub mod renderer;
pub mod simulator;
