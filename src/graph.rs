use std::{hash::Hash, slice::Iter, vec};

use rand::Rng;

pub enum GraphType {
    Directed,
    Undirected,
}

type DefaultIndex = usize;

pub struct Node<T>
where
    T: PartialEq,
{
    pub data: T,
    pub position: [f32; 2],
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
        let x: f32 = rand::thread_rng().gen_range(-1.0..1.0);
        let y: f32 = rand::thread_rng().gen_range(-1.0..1.0);
        Self {
            data,
            position: [x, y],
        }
    }
}
pub struct Edge(pub DefaultIndex, pub DefaultIndex, u64);

pub struct Graph<T>
where
    T: PartialEq,
{
    nodes: Vec<Option<Node<T>>>,
    edges: Vec<Edge>,
    graph_type: GraphType,
}

impl<T> Graph<T>
where
    T: PartialEq,
{
    pub fn new() -> Self {
        Self {
            nodes: vec![],
            edges: vec![],
            graph_type: GraphType::Directed,
        }
    }

    pub fn add_node(&mut self, data: T) -> DefaultIndex {
        self.nodes.push(Some(Node::new(data)));
        self.nodes.len()
    }

    pub fn add_edge(&mut self, i1: DefaultIndex, i2: DefaultIndex, weight: u64) {
        self.edges.push(Edge(i1, i2, weight));
    }

    pub fn get_node_iter(&self) -> Iter<'_, Option<Node<T>>> {
        self.nodes.iter()
    }

    pub fn get_node_by_index(&self, i1: DefaultIndex) -> &Option<Node<T>> {
        self.nodes.get(i1).unwrap()
    }

    pub fn get_node_count(&self) -> DefaultIndex {
        self.nodes.len()
    }

    pub fn get_edge_iter(&self) -> Iter<'_, Edge> {
        self.edges.iter()
    }
}
