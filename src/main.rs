extern crate glium;
extern crate winit;

pub mod datavis;
pub mod graph;
pub mod simgraph;

use core::time;
use std::thread::{self, sleep};
use std::time::Instant;

use graph::Graph;

use rand::Rng;

struct Data {
    name: String,
}

impl Data {
    pub fn new(name: String) -> Self {
        Self { name }
    }
}

impl PartialEq for Data {
    fn eq(&self, other: &Self) -> bool {
        self.name == other.name
    }
}

static mut changed_mass: bool = false;

fn main() {
    let mut g = Graph::<Data>::new();
    g.add_node(Data::new("x".to_string()));
    g.add_node(Data::new("x".to_string()));
    g.add_node(Data::new("x".to_string()));
    g.add_node(Data::new("x".to_string()));
    g.add_node(Data::new("x".to_string()));
    g.add_node(Data::new("x".to_string()));
    for i in 0..5 {
        g.add_node(Data::new("x".to_string()));
        add_random_edge(&mut g);
        add_random_edge(&mut g);
    }
    g.change_mass_based_on_incoming();

    let mut datavis = datavis::DataVis::new(&mut g);
    datavis.create_window(&update);
}

fn update(graph: &mut Graph<Data>, fps: u128) {
    let time = 1.0 / fps as f32; // 0.1s
    thread::sleep(time::Duration::from_nanos(
        ((0.017 - time) * 1_000_000_000.0) as u64,
    ));
}

fn add_random_edge(graph: &mut Graph<Data>) {
    let c = graph.get_node_count();
    let start = rand::thread_rng().gen_range(0..c - 1);
    let end = rand::thread_rng().gen_range(0..c - 1);
    graph.add_edge(start, end, 1);
}
