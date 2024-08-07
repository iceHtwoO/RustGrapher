use crate::graph::{Graph, Node};

pub struct SimGraph {
    spring_stiffness: f64,
    spring_default_len: f64,
    s_per_update: f64,
    f2c: f64,
    electric_repulsion: bool,
    electric_repulsion_const: f64,
    spring: bool,
    gravity: bool,
    damping: f64,
    max_force: f64,
}

impl SimGraph {
    pub fn new() -> Self {
        Self {
            spring_stiffness: 10.0,
            spring_default_len: 0.0,
            s_per_update: 0.005,
            f2c: 10.0,
            electric_repulsion: true,
            electric_repulsion_const: 5.5,
            spring: true,
            gravity: true,
            damping: 0.1,
            max_force: 50.0,
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
                let mut e_f: [f64; 2] = f_list[i];
                if self.electric_repulsion {
                    for (j, n2) in g.get_node_iter().enumerate().skip(i + 1) {
                        let (s_e_f, el_ke_s) = self.electric_repulsion(n1, n2);
                        e_f[0] += s_e_f[0].clamp(-self.max_force, self.max_force);
                        e_f[1] += s_e_f[1].clamp(-self.max_force, self.max_force);
                        f_list[j][0] -= s_e_f[0].clamp(-self.max_force, self.max_force);
                        f_list[j][1] -= s_e_f[1].clamp(-self.max_force, self.max_force);
                        el_ke += el_ke_s;
                    }
                }

                let mut grav_f = [0.0, 0.0];
                if self.gravity {
                    grav_f = self.center_grav(g, n1);
                }
                e_f[0] += grav_f[0];
                e_f[1] += grav_f[1];

                f_list[i][0] += e_f[0];
                f_list[i][1] += e_f[1];
            }
        }

        let mut s_ke = 0.0;

        if self.spring {
            s_ke = self.spring_force(g, &mut f_list);
        }

        self.update_node_position(g, &f_list);
        //println!("Energy: {}J", self.get_kinetic_energy(g) + s_ke);
        //println!("K Energy: {}J", self.get_kinetic_energy(g));
        //println!("SPR Energy: {}J", s_ke);
    }

    fn update_node_position<T>(&self, g: &mut Graph<T>, f_list: &Vec<[f64; 2]>)
    where
        T: PartialEq,
    {
        for (i, n1) in g.get_node_mut_iter().enumerate() {
            let mut node_force: [f64; 2] = f_list[i];

            if f_list[i][0].is_finite() {
                n1.speed[0] +=
                    f_list[i][0].signum() * (node_force[0] / n1.mass).abs() * self.s_per_update;
            }
            if f_list[i][1].is_finite() {
                n1.speed[1] +=
                    f_list[i][1].signum() * (node_force[1] / n1.mass).abs() * self.s_per_update;
            }
        }

        'damping: for n in g.get_node_mut_iter() {
            if n.fixed {
                n.speed[0] = 0.0;
                n.speed[1] = 0.0;
                continue 'damping;
            }

            n.speed[0] *= 1.0 - self.damping * self.s_per_update;
            n.speed[1] *= 1.0 - self.damping * self.s_per_update;

            n.position[0] += n.speed[0] * self.s_per_update;
            n.position[1] += n.speed[1] * self.s_per_update;
        }
    }

    fn get_kinetic_energy<T>(&self, g: &mut Graph<T>) -> f64
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

    pub fn calc_dist<T>(n1: &Node<T>, n2: &Node<T>) -> f64
    where
        T: PartialEq,
    {
        f64::sqrt(
            (n2.position[0] - n1.position[0]).powi(2) + (n2.position[1] - n1.position[1]).powi(2),
        )
    }

    fn calc_dist_x_y(n1: &[f64; 2], n2: &[f64; 2]) -> (f64, f64) {
        ((n2[0] - n1[0]), (n2[1] - n1[1]))
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

            let dist = Self::calc_dist(n1, n2);

            let force_magnitude = self.spring_stiffness * (dist - self.spring_default_len);

            let unit_vec = [vec[0] / dist, vec[1] / dist];

            ke += 0.5 * self.spring_stiffness * dist.powi(2);

            let force_x = -force_magnitude * unit_vec[0];
            let force_y = -force_magnitude * unit_vec[1];

            forcelist[edge.0][0] -= force_x;
            forcelist[edge.0][1] -= force_y;

            forcelist[edge.1][0] += force_x;
            forcelist[edge.1][1] += force_y;
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
        let dist = Self::calc_dist_x_y(&n1.position, &n2.position);
        let vec = Self::calc_dir_vec(n1, n2);
        let mut force = [0.0, 0.0];
        ke += (self.electric_repulsion_const * (n1.mass * n2.mass).abs()) / Self::calc_dist(n1, n2);

        let x_y_len = vec[0].abs() + vec[1].abs();
        if x_y_len == 0.0 {
            ([0.0, 0.0], 0);
        }

        if dist.0 != 0.0 {
            force[0] += vec[0].signum()
                * (self.electric_repulsion_const * -(n1.mass * n2.mass).abs())
                / (dist.0 * (vec[0].abs() / x_y_len)).powi(2);
        }
        if dist.1 != 0.0 {
            force[1] += vec[1].signum()
                * (self.electric_repulsion_const * -(n1.mass * n2.mass).abs())
                / (dist.1 * (vec[1].abs() / x_y_len)).powi(2);
        }
        (force, ke)
    }

    fn center_grav<T>(&self, g: &Graph<T>, n1: &Node<T>) -> [f64; 2]
    where
        T: PartialEq,
    {
        let avg_pos = g.avg_pos();
        let d = Self::calc_dist_x_y(&n1.position, &avg_pos);
        [
            self.f2c * (-n1.position[0]).signum().signum(),
            self.f2c * (-n1.position[1]).signum().signum(),
        ]
    }
}
