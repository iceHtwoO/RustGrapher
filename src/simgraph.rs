use core::f32::INFINITY;
use std::{
    fmt::Debug,
    sync::{Arc, Mutex, RwLock},
    thread::{self, JoinHandle},
};

use rand::Rng;

use crate::{
    graph::{Graph, Node},
    quadtree::{self, QuadTree, Rectangle},
};

#[derive(Clone, Debug)]
pub struct SimGraph<T>
where
    T: PartialEq + Send + Sync + 'static + Clone,
{
    repel_force: bool,
    spring: bool,
    gravity: bool,
    spring_stiffness: f32,
    spring_default_len: f32,
    delta_time: f32,
    gravity_force: f32,
    repel_force_const: f32,
    damping: f32,
    phantom: std::marker::PhantomData<T>,
}

impl<T> SimGraph<T>
where
    T: PartialEq + Send + Sync + 'static + Clone,
{
    pub fn new() -> Self {
        Self {
            repel_force: true,
            spring: true,
            gravity: true,
            repel_force_const: 100.0,
            spring_stiffness: 100.0,
            spring_default_len: 2.0,
            gravity_force: 1.0,
            delta_time: 0.005,
            damping: 0.9,
            phantom: std::marker::PhantomData,
        }
    }

    pub fn new_config(repel_force: bool, spring: bool, gravity: bool) -> Self {
        Self {
            repel_force,
            spring,
            gravity,
            repel_force_const: 100.0,
            spring_stiffness: 100.0,
            spring_default_len: 2.0,
            gravity_force: 1.0,
            delta_time: 0.005,
            damping: 0.9,
            phantom: std::marker::PhantomData,
        }
    }

    pub fn simulation_step(&mut self, graph: Arc<RwLock<Graph<T>>>) {
        let f_vec = Arc::new(Mutex::new(vec![
            [0.0, 0.0];
            graph.read().unwrap().get_node_count()
        ]));

        self.calculate_forces(Arc::clone(&graph), Arc::clone(&f_vec));

        self.apply_node_force(Arc::clone(&graph), Arc::clone(&f_vec));
        self.update_node_position(Arc::clone(&graph));
    }

    fn calculate_forces(&mut self, g: Arc<RwLock<Graph<T>>>, f_vec: Arc<Mutex<Vec<[f32; 2]>>>) {
        if self.repel_force || self.gravity {
            let mut handles = vec![];

            let node_count = { g.read().unwrap().get_node_count() };
            let thread_count = usize::min(node_count, 16);
            let nodes_per_thread = node_count / thread_count;

            for thread in 0..thread_count {
                let mut extra = 0;

                if thread == thread_count - 1 {
                    extra = node_count % thread_count;
                }

                let handle = self.spawn_physic_thread(
                    thread * nodes_per_thread,
                    (thread + 1) * nodes_per_thread + extra,
                    node_count,
                    Arc::clone(&f_vec),
                    Arc::clone(&g),
                );

                handles.push(handle);
            }

            if self.spring {
                self.add_graph_spring_forces(Arc::clone(&g), Arc::clone(&f_vec));
            }

            for handle in handles {
                handle.join().unwrap();
            }
        }
    }

    fn spawn_physic_thread(
        &self,
        start_index: usize,
        end_index: usize,
        node_count: usize,
        vec_arc: Arc<Mutex<Vec<[f32; 2]>>>,
        graph: Arc<RwLock<Graph<T>>>,
    ) -> JoinHandle<()> {
        let repel_force_const = self.repel_force_const;
        let repel_force = self.repel_force;
        let gravity = self.gravity;
        let gravity_force = self.gravity_force;
        let quadtree = Self::build_quadtree(Arc::clone(&graph));

        let handle = thread::spawn(move || {
            let mut force_vec: Vec<[f32; 2]> = vec![[0.0, 0.0]; node_count];
            let graph_read_guard = graph.read().unwrap();

            for i in start_index..end_index {
                let n1 = graph_read_guard.get_node_by_index(i);
                if repel_force {
                    let node_approximations = quadtree.get_mass(&n1.position);
                    for node_approximation in node_approximations {
                        let node_approximation_particle = Node {
                            data: n1.data.clone(),
                            position: node_approximation.0,
                            velocity: [0.0, 0.0],
                            mass: node_approximation.1,
                            fixed: false,
                        };
                        let repel_force =
                            Self::repel_force(repel_force_const, &n1, &node_approximation_particle);

                        force_vec[i][0] += repel_force[0];
                        force_vec[i][1] += repel_force[1];
                    }
                }

                if gravity {
                    let gravity_force = Self::compute_center_gravity(gravity_force, &n1);
                    force_vec[i][0] += gravity_force[0];
                    force_vec[i][1] += gravity_force[1];
                }
            }

            {
                let mut l = vec_arc.lock().unwrap();

                for (i, force) in force_vec.into_iter().enumerate() {
                    l[i][0] += force[0];
                    l[i][1] += force[1];
                }
            }
        });

        handle
    }

    fn build_quadtree(graph: Arc<RwLock<Graph<T>>>) -> QuadTree<usize> {
        let mut quadtree = QuadTree::new();
        let graph_read_guard = graph.read().unwrap();

        let mut max_x = -INFINITY;
        let mut min_x = INFINITY;
        let mut max_y = -INFINITY;
        let mut min_y = INFINITY;

        for node in graph_read_guard.get_node_iter() {
            max_x = max_x.max(node.position[0]);
            max_y = max_y.max(node.position[1]);
            min_x = min_x.min(node.position[0]);
            min_y = min_y.min(node.position[1]);
        }

        let w = max_x - min_x;
        let h = max_y - min_y;
        let boundary = Rectangle::new([min_x + 0.5 * w, min_y + 0.5 * h], w, h);

        for (i, n) in graph_read_guard.get_node_iter().enumerate() {
            quadtree.add_node(i, n.position.clone(), n.mass, &boundary);
        }
        quadtree
    }

    fn apply_node_force(
        &self,
        graph: Arc<RwLock<Graph<T>>>,
        force_vec_arc: Arc<Mutex<Vec<[f32; 2]>>>,
    ) {
        let mut graph_write_guard = graph.write().unwrap();
        let force_vec = force_vec_arc.lock().unwrap();
        for (i, n1) in graph_write_guard.get_node_mut_iter().enumerate() {
            let node_force: [f32; 2] = force_vec[i];

            n1.velocity[0] +=
                force_vec[i][0].signum() * (node_force[0] / n1.mass).abs() * self.delta_time;
            n1.velocity[1] +=
                force_vec[i][1].signum() * (node_force[1] / n1.mass).abs() * self.delta_time;
        }
    }

    fn update_node_position(&self, graph: Arc<RwLock<Graph<T>>>) {
        let mut graph_write_guard = graph.write().unwrap();

        'damping: for n in graph_write_guard.get_node_mut_iter() {
            if n.fixed {
                n.velocity[0] = 0.0;
                n.velocity[1] = 0.0;
                continue 'damping;
            }

            n.velocity[0] *= self.damping;
            n.velocity[1] *= self.damping;

            n.position[0] += n.velocity[0] * self.delta_time;
            n.position[1] += n.velocity[1] * self.delta_time;
        }
    }

    fn compute_direction_vector(n1: &Node<T>, n2: &Node<T>) -> [f32; 2] {
        [
            n2.position[0] - n1.position[0],
            n2.position[1] - n1.position[1],
        ]
    }

    pub fn compute_distance(n1: &Node<T>, n2: &Node<T>) -> f32 {
        f32::sqrt(
            (n2.position[0] - n1.position[0]).powi(2) + (n2.position[1] - n1.position[1]).powi(2),
        )
    }

    pub fn compute_distance_to_center(n1: &Node<T>) -> f32 {
        f32::sqrt((n1.position[0]).powi(2) + (n1.position[1]).powi(2))
    }

    fn add_graph_spring_forces(
        &self,
        graph: Arc<RwLock<Graph<T>>>,
        force_vec_arc: Arc<Mutex<Vec<[f32; 2]>>>,
    ) {
        let mut force_vec = force_vec_arc.lock().unwrap();
        let graph_arc = Arc::clone(&graph);

        for edge in graph_arc.read().unwrap().get_edge_iter() {
            let g = graph_arc.read().unwrap();

            let n1 = g.get_node_by_index(edge.0);
            let n2 = g.get_node_by_index(edge.1);

            let spring_force = self.compute_spring_force(n1, n2);

            force_vec[edge.0][0] -= spring_force[0];
            force_vec[edge.0][1] -= spring_force[1];
            force_vec[edge.1][0] += spring_force[0];
            force_vec[edge.1][1] += spring_force[1];
        }
    }

    fn compute_spring_force(&self, n1: &Node<T>, n2: &Node<T>) -> [f32; 2] {
        let vec = Self::compute_direction_vector(n1, n2);
        let dist = Self::compute_distance(n1, n2);
        let force_magnitude = self.spring_stiffness * (dist - self.spring_default_len);
        let unit_vec = [vec[0] / dist, vec[1] / dist];

        let force_x = -force_magnitude * unit_vec[0];
        let force_y = -force_magnitude * unit_vec[1];

        [force_x, force_y]
    }

    fn repel_force(repel_force_const: f32, n1: &Node<T>, n2: &Node<T>) -> [f32; 2] {
        let vec = Self::compute_direction_vector(n1, n2);
        let dist = Self::compute_distance(n1, n2);
        let mut force = [0.0, 0.0];

        let x_y_len = vec[0].abs() + vec[1].abs();
        if x_y_len == 0.0 || Self::compute_distance(n1, n2) < 0.1 {
            return [
                rand::thread_rng().gen_range(-1..1) as f32,
                rand::thread_rng().gen_range(-1..1) as f32,
            ];
        }

        let f = -repel_force_const * (n1.mass * n2.mass).abs() / dist.powi(2);

        if vec[0] != 0.0 {
            force[0] += vec[0].signum() * f * (vec[0].abs() / x_y_len);
        }
        if vec[1] != 0.0 {
            force[1] += vec[1].signum() * f * (vec[1].abs() / x_y_len);
        }

        force
    }

    fn compute_center_gravity(gravity_force: f32, n1: &Node<T>) -> [f32; 2] {
        let mut force = [0.0, 0.0];
        if n1.position[0].is_finite() {
            force[0] = gravity_force * (-n1.position[0]) * n1.mass;
        }

        if n1.position[1].is_finite() {
            force[1] = gravity_force * (-n1.position[1]) * n1.mass;
        }
        force
    }
}
