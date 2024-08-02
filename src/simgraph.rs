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
            spring_stiffness: 10.0,
            spring_default_len: 0.25,
            s_per_update: 0.01,
            resistance: 0.05,
            f2c: 0.05,
            electric_repulsion: true,
            electric_repulsion_const: 0.0005,
            spring: true,
            gravity: true,
            res: true,
        }
    }

    pub fn sim<T>(&mut self, g: &mut Graph<T>, fps: u128)
    where
        T: PartialEq,
    {
        //self.s_per_update = 1.0 / fps as f64;
        let mut speedlist = vec![[0.0, 0.0]; g.get_node_count()];
        if self.electric_repulsion || self.gravity {
            for (i, n1) in g.get_node_iter().enumerate() {
                let mut speed: [f64; 2] = [0.0, 0.0];
                if self.electric_repulsion {
                    for n2 in g.get_node_iter() {
                        self.electric_repulsion(n1, n2, &mut speed);
                    }
                }

                if self.gravity {
                    self.center_grav(n1, &mut speed);
                }
                speedlist[i] = speed;
            }
        }

        if self.spring {
            self.spring_force(g, &mut speedlist);
        }

        for (i, n1) in g.get_node_mut_iter().enumerate() {
            n1.speed[0] += speedlist[i][0];
            n1.speed[1] += speedlist[i][1];

            if self.res {
                n1.speed[0] -= self.resistance * n1.speed[0];
                n1.speed[1] -= self.resistance * n1.speed[1];
            }
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
        let v = f64::sqrt(
            (n2.position[0] - n1.position[0]).powi(2) + (n2.position[1] - n1.position[1]).powi(2),
        );
        if v.is_nan() {
            println!("DDDDDDDDDDDDDDDDDDDDD");
            0.0
        } else {
            v
        }
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

    fn spring_force<T>(&self, g: &Graph<T>, speedlist: &mut Vec<[f64; 2]>)
    where
        T: PartialEq,
    {
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

            let mut force_x = self.spring_stiffness * diff * (vec[0].abs() / x_y_len);
            let mut force_y = self.spring_stiffness * diff * (vec[1].abs() / x_y_len);

            if force_x.is_nan() {
                println!("AAAAAAAAAAAAAAAAAAAAAAA");
                force_x = 0.0;
            }

            if force_y.is_nan() {
                println!("BBBBB");
                force_y = 0.0;
            }

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

    fn electric_repulsion<T>(&self, n1: &Node<T>, n2: &Node<T>, speed: &mut [f64; 2])
    where
        T: PartialEq,
    {
        let dist = Self::calc_dist_x_y(n1, n2);

        if dist.0 != 0.0 {
            if (dist.0.powi(2)).is_nan() || (dist.1.powi(2)).is_nan() {
                println!("CCCCCCCCCCCCCCCCCCCCCC");
                return;
            }
            let repx =
                (self.electric_repulsion_const * -(n1.mass * n2.mass).abs()) / dist.0.powi(2);
            let a: f64 = repx.signum() * repx.abs().min(1.0) / n1.mass * dist.0.signum();
            speed[0] += self.s_per_update * a;
        }
        if dist.1 != 0.0 {
            let repx =
                (self.electric_repulsion_const * -(n1.mass * n2.mass).abs()) / dist.1.powi(2);
            let a: f64 = repx.signum() * repx.abs().min(1.0) / n1.mass * dist.1.signum();
            speed[1] += self.s_per_update * a;
        }
    }

    fn center_grav<T>(&self, n1: &Node<T>, speed: &mut [f64; 2])
    where
        T: PartialEq,
    {
        let a_x: f64 = -self.f2c * n1.position[0].signum() * Self::calc_dist_2c(&n1);
        let a_y: f64 = -self.f2c * n1.position[1].signum() * Self::calc_dist_2c(&n1);
        speed[0] += self.s_per_update * a_x;
        speed[1] += self.s_per_update * a_y;
    }
}
