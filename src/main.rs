extern crate glium;
extern crate winit;

use grapher::datavis::DataVis;
use grapher::graph::Graph;
use serde::Deserialize;
use std::fs::File;
use std::io::BufReader;
use std::io::Error;

#[derive(Clone, Debug, Default)]
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
    //let mut g = Graph::<Data>::new(0);

    //graph_wiki(&mut g);
    let mut g = Graph::<u32>::new(0);
    g.add_node_pos(1, [0.0, 0.0], true, 2.0);

    g.change_mass_based_on_incoming();
    let datavis = DataVis::new();
    datavis.create_window(g);
}

fn graph_wiki(g: &mut Graph<Data>) {
    if let Ok(w) = load_wiki() {
        for e in w {
            println!("Node Count:{}", g.get_node_count());
            if g.get_node_count() > 1000 {
                break;
            }
            let node_data = Data::new(e.title);
            let opt = g.contains_node(&node_data);
            let mut _index = 0;
            if opt.is_none() {
                _index = g.add_node(node_data);
            } else {
                _index = opt.unwrap();
            }

            for r in e.references {
                let ref_data = Data::new(r);
                let opt_ref = g.contains_node(&ref_data);
                let mut _index_ref = 0;
                if opt_ref.is_none() {
                    _index_ref = g.add_node(ref_data);
                } else {
                    _index_ref = opt_ref.unwrap();
                }
                g.add_edge(_index, _index_ref, 1);
            }
        }
    }

    println!("Nodes:{}", g.get_node_count());
    println!("Edges:{}", g.get_edge_count());
}

fn load_wiki() -> Result<Vec<WikiEntry>, Error> {
    let file = File::open("reference.json")?;
    let reader = BufReader::new(file);

    let wiki: Vec<WikiEntry> = serde_json::from_reader(reader)?;
    Ok(wiki)
}
