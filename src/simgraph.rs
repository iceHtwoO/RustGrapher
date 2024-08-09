use core::panic;
use std::{
    fmt::Debug,
    sync::{Arc, Mutex},
    thread::{self, JoinHandle},
};

use crate::graph::{Graph, Node};

#[derive(Clone, Debug)]
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
    pub kinetic_energy: f64,
    pub spring_energy: f64,
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
            max_force: 100.0,
            kinetic_energy: 0.0,
            spring_energy: 0.0,
        }
    }

    pub fn sim<T>(mut self, g: Graph<T>, fps: u128) -> (Self, Graph<T>)
    where
        T: PartialEq + Send + Sync + 'static + Clone + Debug,
    {
        let g_arc = Arc::new(g);
        let f_list = Arc::new(Mutex::new(vec![[0.0, 0.0]; g_arc.get_node_count()]));

        self = self.calculate_physics(Arc::clone(&g_arc), Arc::clone(&f_list));

        let arc_mutex = Self::convert_arc_to_arc_mutex(g_arc);

        self.update_node_position(Arc::clone(&arc_mutex), Arc::clone(&f_list));
        self.kinetic_energy = self.get_kinetic_energy(Arc::clone(&arc_mutex));
        let mutex = Arc::into_inner(arc_mutex).unwrap();
        (self, mutex.into_inner().unwrap())
    }

    fn convert_arc_to_arc_mutex<T>(arc: Arc<T>) -> Arc<Mutex<T>>
    where
        T: std::fmt::Debug,
    {
        // Try to unwrap the Arc, consuming it if it's the only reference
        let value = Arc::try_unwrap(arc);
        if let Err(x) = value {
            panic!();
        }

        // Wrap the value in a Mutex
        let mutex = Mutex::new(value.unwrap());

        // Wrap the Mutex in a new Arc
        Arc::new(mutex)
    }

    fn calculate_physics<T>(
        mut self,
        grrr: Arc<Graph<T>>,
        f_list: Arc<Mutex<Vec<[f64; 2]>>>,
    ) -> Self
    where
        T: PartialEq + Send + Sync + 'static + Clone,
    {
        let self_arc = Arc::new(self);
        if self_arc.electric_repulsion || self_arc.gravity {
            let n_count = grrr.get_node_count();

            let thread_count = 16;
            let mut handles = vec![];

            let nodes_p_thread = n_count / thread_count;

            for thread in 0..thread_count {
                let mut extra = 0;

                if thread == thread_count - 1 {
                    extra = n_count % thread_count;
                }

                let handle = self_arc.spawn_electric_threads(
                    thread * nodes_p_thread,
                    (thread + 1) * nodes_p_thread + extra,
                    n_count,
                    Arc::clone(&f_list),
                    Arc::clone(&grrr),
                    Arc::clone(&self_arc),
                );

                handles.push(handle);
            }

            for handle in handles {
                handle.join().unwrap();
            }
        }

        self = Arc::try_unwrap(self_arc).unwrap();

        if self.spring {
            let s_ke = self.spring_force(Arc::clone(&grrr), Arc::clone(&f_list));
            self.spring_energy = s_ke;
        }
        self
    }

    fn spawn_electric_threads<T>(
        &self,
        start_index: usize,
        end_index: usize,
        n_count: usize,
        list_arc: Arc<Mutex<Vec<[f64; 2]>>>,
        g_arc: Arc<Graph<T>>,
        self_arc: Arc<Self>,
    ) -> JoinHandle<()>
    where
        T: PartialEq + Send + Sync + 'static + Clone,
    {
        let handle = thread::spawn(move || {
            for i in start_index..end_index {
                let n1 = g_arc.get_node_by_index(i);
                let mut j_force_list: Vec<(usize, [f64; 2])> = Vec::new();

                let mut e_f: [f64; 2] = [0.0, 0.0];
                if self_arc.electric_repulsion {
                    for j in (i + 1)..n_count {
                        let n2 = g_arc.get_node_by_index(j);
                        let s_e_f = self_arc.electric_repulsion(&n1, &n2);
                        e_f[0] += s_e_f[0].clamp(-self_arc.max_force, self_arc.max_force);
                        e_f[1] += s_e_f[1].clamp(-self_arc.max_force, self_arc.max_force);
                        let mut j_force = [0.0, 0.0];
                        j_force[0] -= s_e_f[0].clamp(-self_arc.max_force, self_arc.max_force);
                        j_force[1] -= s_e_f[1].clamp(-self_arc.max_force, self_arc.max_force);
                        j_force_list.push((j, j_force));
                    }
                }

                let mut grav_f = [0.0, 0.0];
                if self_arc.gravity {
                    grav_f = self_arc.center_grav(&n1);
                }
                e_f[0] += grav_f[0];
                e_f[1] += grav_f[1];

                let mut l = list_arc.lock().unwrap();

                l[i][0] += e_f[0];
                l[i][1] += e_f[1];
                for j_force in j_force_list {
                    l[j_force.0][0] -= j_force.1[0];
                    l[j_force.0][1] -= j_force.1[1];
                }
            }
        });
        handle
    }

    fn update_node_position<T>(
        &self,
        g: Arc<Mutex<Graph<T>>>,
        f_list_arc: Arc<Mutex<Vec<[f64; 2]>>>,
    ) where
        T: PartialEq + Clone,
    {
        let mut mutex = g.lock().unwrap();
        let f_list = f_list_arc.lock().unwrap();
        for (i, n1) in mutex.get_node_mut_iter().enumerate() {
            let node_force: [f64; 2] = f_list[i];

            if f_list[i][0].is_finite() {
                n1.speed[0] +=
                    f_list[i][0].signum() * (node_force[0] / n1.mass).abs() * self.s_per_update;
            }
            if f_list[i][1].is_finite() {
                n1.speed[1] +=
                    f_list[i][1].signum() * (node_force[1] / n1.mass).abs() * self.s_per_update;
            }
        }

        'damping: for n in mutex.get_node_mut_iter() {
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

    fn get_kinetic_energy<T>(&self, g: Arc<Mutex<Graph<T>>>) -> f64
    where
        T: PartialEq + Clone,
    {
        let mut ke = 0.0;
        let mutex = g.lock().unwrap();
        for n in mutex.get_node_iter() {
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

    fn spring_force<T>(&self, g: Arc<Graph<T>>, f_list_arc: Arc<Mutex<Vec<[f64; 2]>>>) -> f64
    where
        T: PartialEq + Clone,
    {
        let mut forcelist = f_list_arc.lock().unwrap();
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

    fn electric_repulsion<T>(&self, n1: &Node<T>, n2: &Node<T>) -> [f64; 2]
    where
        T: PartialEq,
    {
        if n1 == n2 {
            return [0.0, 0.0];
        }
        let dist = Self::calc_dist_x_y(&n1.position, &n2.position);
        let vec = Self::calc_dir_vec(n1, n2);
        let mut force = [0.0, 0.0];

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
        force
    }

    fn center_grav<T>(&self, n1: &Node<T>) -> [f64; 2]
    where
        T: PartialEq,
    {
        [
            self.f2c * (-n1.position[0]).signum().signum(),
            self.f2c * (-n1.position[1]).signum().signum(),
        ]
    }
}
