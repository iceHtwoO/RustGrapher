use std::{
    slice::{Iter, IterMut},
    vec,
};

use rand::{rngs::StdRng, Rng, SeedableRng};

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
    pub position: [f32; 2],
    pub velocity: [f32; 2],
    pub mass: f32,
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
        let mut rng = StdRng::seed_from_u64(0 as u64);
        let x: f32 = rng.gen_range(-60.0..60.0);
        let y: f32 = rng.gen_range(-60.0..60.0);
        Self {
            data,
            position: [x, y],
            velocity: [0.0, 0.0],
            mass: 1.0,
            fixed: false,
        }
    }
    pub fn new_seeded(data: T, seed: u64) -> Self {
        let mut rng = StdRng::seed_from_u64(seed);
        let x: f32 = rng.gen_range(-60.0..60.0);
        let y: f32 = rng.gen_range(-60.0..60.0);
        Self {
            data,
            position: [x, y],
            velocity: [0.0, 0.0],
            mass: 1.0,
            fixed: false,
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
        self.nodes.push(Node::new_seeded(data, self.seed));
        self.seed += 1;
        self.nodes.len() - 1
    }

    pub fn add_node_pos(
        &mut self,
        data: T,
        position: [f32; 2],
        fixed: bool,
        mass: f32,
    ) -> DefaultIndex {
        self.nodes.push(Node {
            data,
            position,
            velocity: [0.0, 0.0],
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

    pub fn set_node_velocity(&mut self, i: DefaultIndex, velocity: [f32; 2]) {
        self.nodes.get_mut(i).unwrap().velocity = velocity;
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
            node.mass += count[i] as f32 * node.mass as f32;
        }
    }

    pub fn avg_pos(&self) -> [f32; 2] {
        let mut avg_pos = [0.0, 0.0];
        for n in self.get_node_iter() {
            avg_pos[0] += n.position[0];
            avg_pos[1] += n.position[1];
        }

        avg_pos[0] /= self.get_node_count() as f32;
        avg_pos[1] /= self.get_node_count() as f32;

        avg_pos
    }
}
