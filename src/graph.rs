use std::{
    clone,
    slice::{Iter, IterMut},
    vec,
};

use plotly::{Plot, Scatter};
use rand::Rng;

#[derive(Debug, Clone)]
pub enum GraphType {
    Directed,
    Undirected,
}

impl Default for GraphType {
    fn default() -> Self {
        GraphType::Undirected
    }
}

type DefaultIndex = usize;

#[derive(Debug, Clone)]
pub struct Node<T>
where
    T: PartialEq,
{
    pub data: T,
    pub position: [f64; 2],
    pub speed: [f64; 2],
    pub mass: f64,
    pub fixed: bool,
}

impl<T> PartialEq for Node<T>
where
    T: PartialEq,
{
    fn eq(&self, other: &Self) -> bool {
        self.data == other.data
    }
}

impl<T> Eq for Node<T> where T: PartialEq {}

impl<T> Node<T>
where
    T: PartialEq,
{
    pub fn new(data: T) -> Self {
        let x: f64 = rand::thread_rng().gen_range(-60.0..60.0);
        let y: f64 = rand::thread_rng().gen_range(-60.0..60.0);
        Self {
            data,
            position: [x, y],
            speed: [0.0, 0.0],
            mass: 1.0,
            fixed: false,
        }
    }
}
#[derive(Debug, Clone)]
pub struct Edge(pub DefaultIndex, pub DefaultIndex, u64);

#[derive(Debug, Clone, Default)]
pub struct Graph<T>
where
    T: PartialEq + Clone,
{
    nodes: Vec<Node<T>>,
    edges: Vec<Edge>,
    graph_type: GraphType,
    last_avg_pos: [f64; 2],
}

impl<T> Graph<T>
where
    T: PartialEq + Clone,
{
    pub fn new() -> Self {
        Self {
            nodes: vec![],
            edges: vec![],
            graph_type: GraphType::Undirected,
            last_avg_pos: [0.0, 0.0],
        }
    }

    pub fn add_node(&mut self, data: T) -> DefaultIndex {
        self.nodes.push(Node::new(data));
        self.nodes.len() - 1
    }

    pub fn add_node_pos(
        &mut self,
        data: T,
        position: [f64; 2],
        fixed: bool,
        mass: f64,
    ) -> DefaultIndex {
        self.nodes.push(Node {
            data,
            position,
            speed: [0.0, 0.0],
            mass,
            fixed,
        });
        self.nodes.len()
    }

    pub fn add_node_rand_pos(&mut self, data: T, fixed: bool, mass: f64) -> DefaultIndex {
        let x: f64 = rand::thread_rng().gen_range(-60.0..60.0);
        let y: f64 = rand::thread_rng().gen_range(-60.0..60.0);
        self.nodes.push(Node {
            data,
            position: [x, y],
            speed: [0.0, 0.0],
            mass,
            fixed,
        });
        self.nodes.len()
    }

    pub fn add_edge(&mut self, i1: DefaultIndex, i2: DefaultIndex, weight: u64) {
        self.edges.push(Edge(i1, i2, weight));
    }

    pub fn get_node_iter(&self) -> Iter<'_, Node<T>> {
        self.nodes.iter()
    }

    pub fn get_node_mut_iter(&mut self) -> IterMut<'_, Node<T>> {
        self.nodes.iter_mut()
    }

    pub fn get_node_by_index(&self, i1: DefaultIndex) -> &Node<T> {
        self.nodes.get(i1).unwrap()
    }

    pub fn get_node_by_index_mut(&mut self, i1: DefaultIndex) -> &mut Node<T> {
        self.nodes.get_mut(i1).unwrap()
    }

    pub fn get_node_count(&self) -> DefaultIndex {
        self.nodes.len()
    }

    pub fn get_edge_count(&self) -> DefaultIndex {
        self.edges.len()
    }

    pub fn contains_node(&self, node: &T) -> Option<DefaultIndex> {
        for (i, n) in self.get_node_iter().enumerate() {
            if n.data == *node {
                return Some(i);
            }
        }
        return None;
    }

    pub fn get_edge_iter(&self) -> Iter<'_, Edge> {
        self.edges.iter()
    }

    pub fn set_node_speed(&mut self, i: DefaultIndex, speed: [f64; 2]) {
        self.nodes.get_mut(i).unwrap().speed = speed;
    }

    pub fn get_incoming_count(&self, i: DefaultIndex) -> u32 {
        match self.graph_type {
            GraphType::Directed => {
                let mut c: u32 = 0;
                for e in self.get_edge_iter() {
                    if e.1 == i {
                        c += 1;
                    }
                }
                c
            }
            GraphType::Undirected => {
                let mut c: u32 = 0;
                for e in self.get_edge_iter() {
                    if e.1 == i {
                        c += 1;
                    }
                    if e.0 == i {
                        c += 1;
                    }
                }
                c
            }
        }
    }

    pub fn change_mass_based_on_incoming(&mut self) {
        let mut count = Vec::with_capacity(self.get_node_count());
        for (i, _) in self.get_node_iter().enumerate() {
            count.push(self.get_incoming_count(i));
        }

        for (i, node) in self.get_node_mut_iter().enumerate() {
            node.mass += count[i] as f64 * node.mass as f64;
        }
    }

    pub fn avg_pos(&self) -> [f64; 2] {
        let mut avg_pos = [0.0, 0.0];
        for n in self.get_node_iter() {
            avg_pos[0] += n.position[0];
            avg_pos[1] += n.position[1];
        }

        avg_pos[0] /= self.get_node_count() as f64;
        avg_pos[1] /= self.get_node_count() as f64;

        avg_pos
    }

    pub fn avg_avg_pos(&mut self) -> [f64; 2] {
        let mut avg_pos = self.avg_pos();
        avg_pos[0] = (avg_pos[0] + self.last_avg_pos[0]) / 2.0;
        avg_pos[1] = (avg_pos[1] + self.last_avg_pos[1]) / 2.0;
        self.last_avg_pos = avg_pos;
        avg_pos
    }
}
