use std::f64::consts::E;

use crate::graph::{Graph, Node};

pub struct SimGraph {
    spring_stiffness: f64,
    spring_default_len: f64,
    s_per_update: f64,
    resistance: f64,
    f2c: f64,
    electric_repulsion: bool,
    electric_repulsion_const: f64,
    spring: bool,
    gravity: bool,
    res: bool,
}

impl SimGraph {
    pub fn new() -> Self {
        Self {
            spring_stiffness: 0.4,
            spring_default_len: 1.0,
            s_per_update: 0.5,
            resistance: 0.003,
            f2c: 0.001,
            electric_repulsion: true,
            electric_repulsion_const: 10.1,
            spring: true,
            gravity: false,
            res: false,
        }
    }

    pub fn sim<T>(&mut self, g: &mut Graph<T>, fps: u128)
    where
        T: PartialEq,
    {
        //self.s_per_update = 1.0 / fps as f64;
        let mut f_list = vec![[0.0, 0.0]; g.get_node_count()];

        let mut el_ke: f64 = 0.0;
        if self.electric_repulsion || self.gravity {
            for (i, n1) in g.get_node_iter().enumerate() {
                let mut e_f: [f64; 2] = [0.0, 0.0];
                if self.electric_repulsion {
                    for n2 in g.get_node_iter() {
                        let (s_e_f, el_ke_s) = self.electric_repulsion(n1, n2);
                        e_f[0] += s_e_f[0];
                        e_f[1] += s_e_f[1];
                        el_ke += el_ke_s;
                    }
                }

                let mut grav_f = [0.0, 0.0];
                if self.gravity {
                    grav_f = self.center_grav(n1);
                }
                e_f[0] += grav_f[0];
                e_f[1] += grav_f[1];

                f_list[i] = e_f;
            }
        }

        let mut s_ke = 0.0;

        if self.spring {
            s_ke = self.spring_force(g, &mut f_list);
        }

        self.clac_pos(g, &f_list);
        println!("Energy: {}J", self.ke(g) + s_ke + el_ke / 2.0);
    }

    fn clac_pos<T>(&self, g: &mut Graph<T>, f_list: &Vec<[f64; 2]>)
    where
        T: PartialEq,
    {
        for (i, n1) in g.get_node_mut_iter().enumerate() {
            n1.speed[0] = n1.speed[0] * 0.8 + (f_list[i][0] / n1.mass) * self.s_per_update;
            n1.speed[1] = n1.speed[1] * 0.8 + (f_list[i][1] / n1.mass) * self.s_per_update;
            n1.speed[0] = n1.speed[0].signum() * n1.speed[0].abs().min(10.0);
            n1.speed[1] = n1.speed[1].signum() * n1.speed[1].abs().min(10.0);
        }

        for n in g.get_node_mut_iter() {
            if n.fixed {
                continue;
            }

            n.position[0] += n.speed[0] * self.s_per_update;
            n.position[1] += n.speed[1] * self.s_per_update;

            if n.position[0].is_nan() {
                println!("XXX");
                loop {}
            }
        }
    }

    fn ke<T>(&self, g: &mut Graph<T>) -> f64
    where
        T: PartialEq,
    {
        let mut ke = 0.0;
        for n in g.get_node_iter() {
            ke += 0.5 * n.speed[0].powi(2) * n.mass;
            ke += 0.5 * n.speed[1].powi(2) * n.mass;
        }
        ke
    }

    fn calc_dir_vec<T>(n1: &Node<T>, n2: &Node<T>) -> [f64; 2]
    where
        T: PartialEq,
    {
        [
            n2.position[0] - n1.position[0],
            n2.position[1] - n1.position[1],
        ]
    }

    fn calc_dist<T>(n1: &Node<T>, n2: &Node<T>) -> f64
    where
        T: PartialEq,
    {
        f64::sqrt(
            (n2.position[0] - n1.position[0]).powi(2) + (n2.position[1] - n1.position[1]).powi(2),
        )
    }

    fn calc_dist_2c<T>(n1: &Node<T>) -> f64
    where
        T: PartialEq,
    {
        f64::sqrt(n1.position[0].powi(2) + n1.position[1].powi(2))
    }

    fn calc_dist_x_y<T>(n1: &Node<T>, n2: &Node<T>) -> (f64, f64)
    where
        T: PartialEq,
    {
        (
            (n2.position[0] - n1.position[0]),
            (n2.position[1] - n1.position[1]),
        )
    }

    fn spring_force<T>(&self, g: &Graph<T>, forcelist: &mut Vec<[f64; 2]>) -> f64
    where
        T: PartialEq,
    {
        let mut ke = 0.0;
        for (i, edge) in g.get_edge_iter().enumerate() {
            let n1 = g.get_node_by_index(edge.0);
            let n2 = g.get_node_by_index(edge.1);
            let vec = Self::calc_dir_vec(n1, n2);
            // instead of vec calculate the length offset based on both cords

            let diff = self.spring_default_len - Self::calc_dist(n1, n2);

            let x_y_len = (vec[0].abs() + vec[1].abs());
            if x_y_len == 0.0 {
                continue;
            }

            ke += 0.5 * self.spring_stiffness * diff.powi(2);

            let force_x = self.spring_stiffness * diff * (vec[0].abs() / x_y_len);
            let force_y = self.spring_stiffness * diff * (vec[1].abs() / x_y_len);

            forcelist[edge.0][0] += force_x * -vec[0].signum();
            forcelist[edge.0][1] += force_y * -vec[1].signum();

            forcelist[edge.1][0] += force_x * vec[0].signum();
            forcelist[edge.1][1] += force_y * vec[1].signum();
        }
        ke
    }

    fn electric_repulsion<T>(&self, n1: &Node<T>, n2: &Node<T>) -> ([f64; 2], f64)
    where
        T: PartialEq,
    {
        if n1 == n2 {
            return ([0.0, 0.0], 0.0);
        }
        let mut ke = 0.0;
        let dist = Self::calc_dist_x_y(n1, n2);
        let vec = Self::calc_dir_vec(n1, n2);
        let mut force = [0.0, 0.0];
        ke += (self.electric_repulsion_const * (n1.mass * n2.mass).abs()) / Self::calc_dist(n1, n2);
        if dist.0 != 0.0 {
            force[0] += vec[0].signum()
                * (self.electric_repulsion_const * -(n1.mass * n2.mass).abs())
                / dist.0.powi(2);
        }
        if dist.1 != 0.0 {
            force[1] = vec[1].signum()
                * (self.electric_repulsion_const * -(n1.mass * n2.mass).abs())
                / dist.1.powi(2);
        }
        (force, ke)
    }

    fn center_grav<T>(&self, n1: &Node<T>) -> [f64; 2]
    where
        T: PartialEq,
    {
        [
            -self.f2c * n1.position[0].signum(),
            -self.f2c * n1.position[1].signum(),
        ]
    }
}
