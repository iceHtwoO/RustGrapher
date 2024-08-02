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

fn main() {
    let mut g = Graph::<Data>::new();
    g.add_node_pos(Data::new("x".to_string()), [0.0, 0.0], false, 5.0);
    g.add_node_pos(Data::new("x".to_string()), [0.5, 0.0], false, 5.0);
    g.add_edge(0, 1, 1);
    for i in 0..100 {
        g.add_node(Data::new("x".to_string()));
        add_random_edge(&mut g);
        add_random_edge(&mut g);
        add_random_edge(&mut g);
        add_random_edge(&mut g);
        g.add_edge(i, 0, 1);
    }
    for i in 100..140 {
        g.add_node(Data::new("x".to_string()));
        add_random_edge(&mut g);
        add_random_edge(&mut g);
        g.add_edge(i, 3, 1);
    }
    for i in 120..300 {
        g.add_node(Data::new("x".to_string()));
        add_random_edge(&mut g);
        add_random_edge(&mut g);
        g.add_edge(i, 1, 1);
    }
    g.add_edge(1, 0, 1);
    for i in 300..500 {
        g.add_node(Data::new("x".to_string()));
        add_random_edge(&mut g);
        add_random_edge(&mut g);
        g.add_edge(i, 5, 6);
    }

    g.change_mass_based_on_incoming();

    let mut datavis = datavis::DataVis::new(&mut g);
    datavis.create_window(&update);
}

fn update(graph: &mut Graph<Data>, fps: u128) {
    let time = 1.0 / fps as f32; // 0.1s
}

fn add_random_edge(graph: &mut Graph<Data>) {
    let c = graph.get_node_count();
    let start = rand::thread_rng().gen_range(0..c - 1);
    let end = rand::thread_rng().gen_range(0..c - 1);
    graph.add_edge(start, end, 1);
}
