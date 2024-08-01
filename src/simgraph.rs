use core::prelude;
use std::path::StripPrefixError;

use crate::graph::{Graph, Node};

pub struct SimGraph {
    spring_stiffness: f32,
    spring_default_len: f32,
    s_per_update: f32,
    resistance: f32,
    f2c: f32,
    electric_repulsion_aoe: f32,
}

impl SimGraph {
    pub fn new() -> Self {
        Self {
            spring_stiffness: 1.0,
            spring_default_len: 0.1,
            s_per_update: 0.001,
            resistance: 0.01,
            f2c: 0.05,
            electric_repulsion_aoe: 0.4,
        }
    }

    pub fn sim<T>(&mut self, g: &mut Graph<T>, fps: u128)
    where
        T: PartialEq,
    {
        //self.s_per_update = 1.0 / fps as f32;
        let mut speedlist = vec![[0.0, 0.0]; g.get_node_count()];
        for (i, n1) in g.get_node_iter().enumerate() {
            let mut speed: [f32; 2] = [0.0, 0.0];
            for n2 in g.get_node_iter() {
                //self.electric_repulsion(n1, n2, &mut speed);
            }

            //self.center_grav(n1, &mut speed);

            speedlist[i] = speed;
        }

        self.spring_force(g, &mut speedlist);

        for (i, n1) in g.get_node_mut_iter().enumerate() {
            n1.speed[0] += speedlist[i][0];
            n1.speed[1] += speedlist[i][1];

            n1.speed[0] -= self.resistance * n1.speed[0];
            n1.speed[1] -= self.resistance * n1.speed[1];
        }

        for n in g.get_node_mut_iter() {
            if n.fixed {
                continue;
            }
            n.position[0] += n.speed[0] * self.s_per_update;

            n.position[1] += n.speed[1] * self.s_per_update;
            if n.position[0] > 1.0 {
                n.position[0] = 1.0;
                n.speed[0] = 0.0;
            }
            if n.position[0] < -1.0 {
                n.position[0] = -1.0;
                n.speed[0] = 0.0;
            }
            if n.position[1] > 1.0 {
                n.position[1] = 1.0;
                n.speed[1] = 0.0;
            }
            if n.position[1] < -1.0 {
                n.position[1] = -1.0;
                n.speed[1] = 0.0;
            }
        }
    }

    fn calc_dir_vec<T>(n1: &Node<T>, n2: &Node<T>) -> [f32; 2]
    where
        T: PartialEq,
    {
        [
            n2.position[0] - n1.position[0],
            n2.position[1] - n1.position[1],
        ]
    }

    fn calc_dist<T>(n1: &Node<T>, n2: &Node<T>) -> f32
    where
        T: PartialEq,
    {
        f32::sqrt(
            (n2.position[0] - n1.position[0]).powf(2.0)
                + f32::exp2(n2.position[1] - n1.position[1]).powf(2.0),
        )
    }

    fn calc_dist_2c<T>(n1: &Node<T>) -> f32
    where
        T: PartialEq,
    {
        f32::sqrt(f32::exp2(n1.position[0]) + f32::exp2(n1.position[1]))
    }

    fn calc_dist_x_y<T>(n1: &Node<T>, n2: &Node<T>) -> (f32, f32)
    where
        T: PartialEq,
    {
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

    fn spring_force<T>(&self, g: &Graph<T>, speedlist: &mut Vec<[f32; 2]>)
    where
        T: PartialEq,
    {
        for edge in g.get_edge_iter() {
            let n1 = g.get_node_by_index(edge.0);
            let n2 = g.get_node_by_index(edge.1);
            let vec = Self::calc_dir_vec(n1, n2);
            // instead of vec calculate the length offset based on both cords

            let diff = self.spring_default_len - Self::calc_dist(n1, n2);

            let force_x =
                dbg!(self.spring_stiffness * diff * (vec[0].abs() / (vec[0].abs() + vec[1].abs())));
            let force_y =
                self.spring_stiffness * diff * (vec[1].abs() / (vec[0].abs() + vec[1].abs()));

            let a_x_n1 = (force_x * -vec[0].signum()) / n1.mass;
            let a_y_n1 = (force_y * -vec[1].signum()) / n1.mass;
            let a_x_n2 = (force_x * vec[0].signum()) / n2.mass;
            let a_y_n2 = (force_y * vec[1].signum()) / n2.mass;

            speedlist[edge.0][0] += self.s_per_update * a_x_n1;
            speedlist[edge.0][1] += self.s_per_update * a_y_n1;

            speedlist[edge.1][0] += self.s_per_update * a_x_n2;
            speedlist[edge.1][1] += self.s_per_update * a_y_n2;
        }
    }

    fn electric_repulsion<T>(&self, n1: &Node<T>, n2: &Node<T>, speed: &mut [f32; 2])
    where
        T: PartialEq,
    {
        let dist = Self::calc_dist_x_y(n1, n2);

        if dist.0.clone().abs() < self.electric_repulsion_aoe && dist.0 != 0.0 {
            let repx = -(n1.mass * n2.mass).abs() / dist.0.exp2();
            let a: f32 = repx / n1.mass * dist.0.signum();
            speed[0] += self.s_per_update * a;
        }
        if dist.1.clone().abs() < self.electric_repulsion_aoe && dist.1 != 0.0 {
            let repx = -(n1.mass * n2.mass).abs() / dist.1.exp2();
            let a: f32 = repx / n1.mass * dist.1.signum();
            speed[1] += self.s_per_update * a;
        }
    }

    fn center_grav<T>(&self, n1: &Node<T>, speed: &mut [f32; 2])
    where
        T: PartialEq,
    {
        let a_x: f32 = -self.f2c * n1.mass * n1.position[0].signum() * Self::calc_dist_2c(n1).abs();
        let a_y: f32 = -self.f2c * n1.mass * n1.position[1].signum() * Self::calc_dist_2c(n1).abs();
        speed[0] += self.s_per_update * a_x;
        speed[1] += self.s_per_update * a_y;
    }
}
