extern crate glium;
extern crate winit;

pub mod datavis;
pub mod graph;

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
    g.add_node(Data::new("Hello".to_string()));
    g.add_node(Data::new("World".to_string()));
    g.add_node(Data::new("!".to_string()));
    g.add_edge(0, 1, 2);

    let mut datavis = datavis::DataVis::new(&mut g);
    datavis.create_window(&update);
}

fn update(graph: &mut Graph<Data>) {
    if graph.get_node_count() < 60 {
        graph.add_node(Data::new("x".to_string()));
        add_random_edge(graph);
        add_random_edge(graph);
    }
}

fn add_random_edge(graph: &mut Graph<Data>) {
    let c = graph.get_node_count();
    let start = rand::thread_rng().gen_range(0..c - 1);
    let end = rand::thread_rng().gen_range(0..c - 1);
    graph.add_edge(start, end, 1);
}
