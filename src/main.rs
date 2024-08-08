extern crate glium;
extern crate winit;

pub mod datavis;
pub mod graph;
pub mod simgraph;

use graph::Graph;

use rand::Rng;
use serde::Deserialize;
use std::fs::File;
use std::io::BufReader;
use std::io::Error;
use std::sync::Arc;
use std::sync::Mutex;

#[derive(Clone)]
struct Data {
    name: String,
}

impl Data {
    pub fn new(name: String) -> Self {
        Self { name }
    }
}

#[derive(Deserialize, Debug)]
struct WikiEntry {
    title: String,
    references: Vec<String>,
}

impl PartialEq for Data {
    fn eq(&self, other: &Self) -> bool {
        self.name == other.name
    }
}

fn main() {
    let mut g = Graph::<Data>::new();
    //rand_graph(&mut g);
    graph_wiki(&mut g);

    g.change_mass_based_on_incoming();
    let mut datavis = datavis::DataVis::new(g);
    datavis.create_window(&update);
}

fn graph_wiki(mut g: &mut Graph<Data>) {
    if let Ok(w) = load_wiki() {
        for e in w {
            println!("Node Count:{}", g.get_node_count());
            if (g.get_node_count() > 10000) {
                break;
            }
            let node_data = Data::new(e.title);
            let opt = g.contains_node(&node_data);
            let mut index = 0;
            if opt.is_none() {
                index = g.add_node(node_data);
            } else {
                index = opt.unwrap();
            }

            for r in e.references {
                let ref_data = Data::new(r);
                let opt_ref = g.contains_node(&ref_data);
                let mut index_ref = 0;
                if opt_ref.is_none() {
                    index_ref = g.add_node(ref_data);
                } else {
                    index_ref = opt_ref.unwrap();
                }
                g.add_edge(index, index_ref, 1);
            }
        }
    }

    println!("Nodes:{}", g.get_node_count());
    println!("Edges:{}", g.get_edge_count());
}

fn load_wiki() -> Result<Vec<WikiEntry>, Error> {
    let file = File::open("references.json")?;
    let reader = BufReader::new(file);

    let wiki: Vec<WikiEntry> = serde_json::from_reader(reader)?;
    Ok(wiki)
}

fn rand_graph(mut g: &mut Graph<Data>) {
    g.add_node_pos(Data::new("x".to_string()), [0.0, 0.0], true, 5.0);
    g.add_node_pos(Data::new("y".to_string()), [-10.0, 0.0], false, 5.0);
    g.add_edge(0, 1, 1);
}

fn update(graph: Arc<Mutex<Graph<Data>>>, fps: u128) {
    let time = 1.0 / fps as f32; // 0.1s
}

fn add_random_edge(graph: &mut Graph<Data>) {
    let c = graph.get_node_count();
    let start = rand::thread_rng().gen_range(0..c - 1);
    let end = rand::thread_rng().gen_range(0..c - 1);
    graph.add_edge(start, end, 1);
}
