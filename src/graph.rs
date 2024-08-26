use std::{
    slice::{Iter, IterMut},
    vec,
};

use rand::{rngs::StdRng, Rng, SeedableRng};

use crate::{properties::RigidBody2D, vectors::Vector2D};

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
    pub rigidbody: Option<RigidBody2D>,
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
        let mut rng = StdRng::seed_from_u64(0 as u64);
        let x: f32 = rng.gen_range(-60.0..60.0);
        let y: f32 = rng.gen_range(-60.0..60.0);
        Self {
            data,
            rigidbody: Some(RigidBody2D::new(Vector2D::new([x, y]), 1.0)),
        }
    }
    pub fn new_rb(data: T, seed: u64, rb: RigidBody2D) -> Self {
        let mut rng = StdRng::seed_from_u64(seed);
        let x: f32 = rng.gen_range(-60.0..60.0);
        let y: f32 = rng.gen_range(-60.0..60.0);
        Self {
            data,
            rigidbody: Some(rb),
        }
    }
}
#[allow(dead_code)]
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
    seed: u64,
}

impl<T> Graph<T>
where
    T: PartialEq + Clone,
{
    pub fn new(seed: u64) -> Self {
        Self {
            nodes: vec![],
            edges: vec![],
            graph_type: GraphType::Undirected,
            seed,
        }
    }

    pub fn add_node(&mut self, data: T) -> DefaultIndex {
        self.nodes.push(Node::new(data));
        self.seed += 1;
        self.nodes.len() - 1
    }

    pub fn add_node_rb(&mut self, data: T, rb: RigidBody2D) -> DefaultIndex {
        self.nodes.push(Node::new_rb(data, 0, rb));
        self.seed += 1;
        self.nodes.len() - 1
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
            let rb = node.rigidbody.as_mut().unwrap();
            rb.mass += count[i] as f32 * rb.mass as f32;
        }
    }

    pub fn avg_pos(&self) -> [f32; 2] {
        let mut avg_pos = [0.0, 0.0];
        for n in self.get_node_iter() {
            let rb = n.rigidbody.as_ref().unwrap();
            avg_pos[0] += rb.position[0];
            avg_pos[1] += rb.position[1];
        }

        avg_pos[0] /= self.get_node_count() as f32;
        avg_pos[1] /= self.get_node_count() as f32;

        avg_pos
    }
}
