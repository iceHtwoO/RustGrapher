use std::{f32::MIN, hash::Hash, slice::Iter, vec};

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
    speed: [f32; 2],
    mass: f32,
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
            speed: [0.0, 0.0],
            mass: 1.0,
        }
    }
}
pub struct Edge(pub DefaultIndex, pub DefaultIndex, u64);

pub struct Graph<T>
where
    T: PartialEq,
{
    nodes: Vec<Node<T>>,
    edges: Vec<Edge>,
    graph_type: GraphType,
    spring_stiffness: f32,
    spring_default_len: f32,
    s_per_update: f32,
    resistance: f32,
    f2c: f32,
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
            spring_stiffness: 1.0,
            spring_default_len: 0.25,
            s_per_update: 0.001,
            resistance: 0.6,
            f2c: 1.0,
        }
    }

    pub fn add_node(&mut self, data: T) -> DefaultIndex {
        self.nodes.push(Node::new(data));
        self.nodes.len()
    }

    pub fn add_edge(&mut self, i1: DefaultIndex, i2: DefaultIndex, weight: u64) {
        self.edges.push(Edge(i1, i2, weight));
    }

    pub fn get_node_iter(&self) -> Iter<'_, Node<T>> {
        self.nodes.iter()
    }

    pub fn get_node_by_index(&self, i1: DefaultIndex) -> &Node<T> {
        self.nodes.get(i1).unwrap()
    }

    pub fn get_node_count(&self) -> DefaultIndex {
        self.nodes.len()
    }

    pub fn get_edge_iter(&self) -> Iter<'_, Edge> {
        self.edges.iter()
    }

    pub fn sim(&mut self) {
        let mut speedlist = vec![];
        for n1 in self.nodes.iter() {
            let mut speed = [0.0, 0.0];
            for n2 in self.nodes.iter() {
                let dist = Self::calc_dist_x_y(n1, n2);

                if dist.0.clone().abs() < 0.1 && dist.0 != 0.0 {
                    let repx = -Self::repulsion(&dist.0).min(1.0);
                    let a: f32 = repx / n1.mass * dist.0.signum();
                    speed[0] += self.s_per_update * a;
                }
                if dist.1.clone().abs() < 0.1 && dist.1 != 0.0 {
                    let repx = -Self::repulsion(&dist.1).min(1.0);
                    let a: f32 = repx / n1.mass * dist.1.signum();
                    speed[1] += self.s_per_update * a;
                }
            }

            let a_x: f32 = -self.f2c / n1.mass * n1.speed[0].signum();
            let a_y: f32 = -self.f2c / n1.mass * n1.speed[1].signum();
            speed[0] += self.s_per_update * a_x;
            speed[0] += self.s_per_update * a_y;

            speedlist.push(speed);
        }
        for (i, n1) in self.nodes.iter_mut().enumerate() {
            n1.speed[0] += speedlist[i][0];
            n1.speed[1] += speedlist[i][1];
        }

        for edge in self.edges.iter() {
            let n1 = self.nodes.get(edge.0).unwrap();
            let n2 = self.nodes.get(edge.1).unwrap();
            let vec = Self::calc_dir_vec(n1, n2);
            // instead of vec calculate the length offset based on both cords
            let force_x = self.spring_stiffness * (self.spring_default_len - vec[0]);
            let force_y = self.spring_stiffness * (self.spring_default_len - vec[1]);

            let res_x_n1 = -self.resistance * n1.speed[0];
            let res_y_n1 = -self.resistance * n1.speed[1];

            let res_x_n2 = -self.resistance * n2.speed[0];
            let res_y_n2 = -self.resistance * n2.speed[1];

            let a_x_n1 = -force_x / n1.mass + res_x_n1 / n1.mass;
            let a_y_n1 = -force_y / n1.mass + res_y_n1 / n1.mass;
            let a_x_n2 = force_x / n2.mass + res_x_n2 / n2.mass;
            let a_y_n2 = force_y / n2.mass + res_y_n2 / n2.mass;
            let n1 = self.nodes.get_mut(edge.0).unwrap();
            n1.speed[0] += self.s_per_update * a_x_n1;
            n1.speed[1] += self.s_per_update * a_y_n1;
            let n2 = self.nodes.get_mut(edge.1).unwrap();
            n2.speed[0] += self.s_per_update * a_x_n2;
            n2.speed[1] += self.s_per_update * a_y_n2;
        }

        for n in self.nodes.iter_mut() {
            n.position[0] += n.speed[0] * self.s_per_update;
            n.position[1] += n.speed[1] * self.s_per_update;
            if n.position[0] > 1.0 {
                n.position[0] = 1.0;
            }
            if n.position[0] < -1.0 {
                n.position[0] = -1.0;
            }
            if n.position[1] > 1.0 {
                n.position[1] = 1.0;
            }
            if n.position[1] < -1.0 {
                n.position[1] = -1.0;
            }
        }
    }

    fn calc_dir_vec(n1: &Node<T>, n2: &Node<T>) -> [f32; 2] {
        [
            n2.position[0] - n1.position[0],
            n2.position[1] - n1.position[1],
        ]
    }

    fn calc_dist(n1: &Node<T>, n2: &Node<T>) -> f32 {
        f32::sqrt(
            f32::exp2(n2.position[0] - n1.position[0]) + f32::exp2(n2.position[1] - n1.position[1]),
        )
    }

    fn calc_dist_x_y(n1: &Node<T>, n2: &Node<T>) -> (f32, f32) {
        (
            (n2.position[0] - n1.position[0]),
            (n2.position[1] - n1.position[1]),
        )
    }

    fn sign(val: &f32) -> f32 {
        if val < &0.0 {
            -1.0
        } else if val > &0.0 {
            1.0
        } else {
            0.0
        }
    }

    fn repulsion(val: &f32) -> f32 {
        1.0 / val.clone().abs().powf(1.0 / 7.0) - 1.0
    }
}
