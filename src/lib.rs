//! # Example
//! ```rust
//!let graph = petgraph_gen::barabasi_albert_graph(&mut rng, 1000, 1, None);
//!let simulator = SimulatorBuilder::default();
//!let renderer = Renderer::new(simulator);
//!renderer.create_window(graph.into());
//! ```

pub mod properties;
pub mod quadtree;
pub mod renderer;
pub mod simulator;
