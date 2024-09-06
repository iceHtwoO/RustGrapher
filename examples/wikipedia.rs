extern crate glium;
extern crate grapher;
extern crate winit;

use grapher::renderer::Renderer;
use grapher::simulator::SimulatorBuilder;
use petgraph::Directed;
use petgraph::Graph;
use serde::Deserialize;
use std::collections::HashMap;
use std::fs::File;
use std::io::BufReader;
use std::io::Error;

#[derive(Clone, Debug, Default, Hash, PartialEq, Eq)]
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

fn main() {
    let mut g: Graph<Data, u32> = Graph::new();

    graph_wiki(&mut g);
    let simulator = SimulatorBuilder::new()
        .delta_time(0.01)
        .freeze_threshold(-1.0)
        .build();
    let datavis = Renderer::new(simulator);
    datavis.create_window(g);
}

fn graph_wiki(g: &mut Graph<Data, u32, Directed, u32>) {
    let mut hashmap = HashMap::new();
    if let Ok(w) = load_wiki() {
        for e in w {
            println!("Node Count:{}", g.node_count());
            if g.node_count() > 1000 {
                break;
            }
            let node_data = Data::new(e.title);

            let mut _index;
            let opt = hashmap.get(&node_data);
            if opt.is_none() {
                _index = g.add_node(node_data.clone());
                hashmap.insert(node_data, _index);
            } else {
                _index = *opt.unwrap();
            }

            for r in e.references {
                let ref_data = Data::new(r);
                let opt_ref = hashmap.get(&ref_data);
                let mut _index_ref;
                if opt_ref.is_none() {
                    _index_ref = g.add_node(ref_data.clone());
                    hashmap.insert(ref_data, _index);
                } else {
                    _index_ref = *opt_ref.unwrap();
                }
                g.add_edge(_index, _index_ref, 1);
            }
        }
    }

    println!("Nodes:{}", g.node_count());
    println!("Edges:{}", g.edge_count());
}

fn load_wiki() -> Result<Vec<WikiEntry>, Error> {
    let file = File::open("examples/reference.json")?;
    let reader = BufReader::new(file);

    let wiki: Vec<WikiEntry> = serde_json::from_reader(reader)?;
    Ok(wiki)
}
