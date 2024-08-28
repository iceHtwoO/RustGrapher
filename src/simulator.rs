use core::f32::INFINITY;
use std::{
    fmt::Debug,
    sync::{Arc, Mutex, RwLock},
    thread::{self, JoinHandle},
};

use rand::Rng;

use crate::{
    graph::Graph,
    properties::RigidBody2D,
    quadtree::{BoundingBox2D, QuadTree},
    vectors::Vector2D,
};

#[derive(Clone, Debug)]
pub struct Simulator<T>
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
    quadtree_theta: f32,
}

impl<T> Simulator<T>
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
            delta_time: 0.01,
            damping: 0.9,
            phantom: std::marker::PhantomData,
            quadtree_theta: 0.25,
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
            quadtree_theta: 0.5,
        }
    }

    pub fn simulation_step(&mut self, graph: Arc<RwLock<Graph<T>>>) {
        let f_vec = Arc::new(Mutex::new(vec![
            Vector2D::new([0.0, 0.0]);
            graph.read().unwrap().get_node_count()
        ]));

        self.calculate_forces(Arc::clone(&graph), Arc::clone(&f_vec));

        self.apply_node_force(Arc::clone(&graph), Arc::clone(&f_vec));
        self.update_node_position(Arc::clone(&graph));
    }

    fn calculate_forces(&mut self, g: Arc<RwLock<Graph<T>>>, f_vec: Arc<Mutex<Vec<Vector2D>>>) {
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
        vec_arc: Arc<Mutex<Vec<Vector2D>>>,
        graph: Arc<RwLock<Graph<T>>>,
    ) -> JoinHandle<()> {
        let repel_force_const = self.repel_force_const;
        let repel_force = self.repel_force;
        let gravity = self.gravity;
        let gravity_force = self.gravity_force;
        let quadtree = Self::build_quadtree(Arc::clone(&graph));
        let theta = self.quadtree_theta;

        let handle = thread::spawn(move || {
            let mut force_vec: Vec<Vector2D> = vec![Vector2D::new([0.0, 0.0]); node_count];
            let graph_read_guard = graph.read().unwrap();

            for i in start_index..end_index {
                let n1 = graph_read_guard
                    .get_node_by_index(i)
                    .rigidbody
                    .as_ref()
                    .unwrap();
                if repel_force {
                    // Get node approximation from Quadtree
                    let node_approximations =
                        quadtree.get_stack(&n1.position.get_position(), theta);

                    // Calculate Repel Force
                    for node_approximation in node_approximations {
                        let node_approximation_particle = RigidBody2D::new(
                            Vector2D::new(node_approximation.get_position_ref()),
                            node_approximation.mass,
                        );
                        let repel_force =
                            Self::repel_force(repel_force_const, &n1, &node_approximation_particle);

                        force_vec[i] += repel_force;
                    }
                }

                //Calculate Gravity Force
                if gravity {
                    let gravity_force = Self::compute_center_gravity(gravity_force, &n1);
                    force_vec[i] += gravity_force;
                }
            }

            {
                let mut force_list = vec_arc.lock().unwrap();

                for (i, force) in force_vec.into_iter().enumerate() {
                    force_list[i] += force;
                }
            }
        });

        handle
    }

    fn build_quadtree(graph: Arc<RwLock<Graph<T>>>) -> QuadTree<'static, u32> {
        let graph_read_guard = graph.read().unwrap();

        let mut max_x = -INFINITY;
        let mut min_x = INFINITY;
        let mut max_y = -INFINITY;
        let mut min_y = INFINITY;

        for node in graph_read_guard.get_node_iter() {
            let rb: &RigidBody2D = node.rigidbody.as_ref().unwrap();
            max_x = max_x.max(rb.position[0]);
            max_y = max_y.max(rb.position[1]);
            min_x = min_x.min(rb.position[0]);
            min_y = min_y.min(rb.position[1]);
        }

        let w = max_x - min_x;
        let h = max_y - min_y;
        let boundary = BoundingBox2D::new([min_x + 0.5 * w, min_y + 0.5 * h], w, h);
        let mut quadtree = QuadTree::new(boundary.clone());

        for n in graph_read_guard.get_node_iter() {
            let rb: &RigidBody2D = n.rigidbody.as_ref().unwrap();
            quadtree.insert(None, rb.position.get_position(), rb.mass);
        }
        quadtree
    }

    fn apply_node_force(
        &self,
        graph: Arc<RwLock<Graph<T>>>,
        force_vec_arc: Arc<Mutex<Vec<Vector2D>>>,
    ) {
        let mut graph_write_guard = graph.write().unwrap();
        let force_vec = force_vec_arc.lock().unwrap();
        for (i, n) in graph_write_guard.get_node_mut_iter().enumerate() {
            let node_force = force_vec[i];
            let rb = n.rigidbody.as_mut().unwrap();

            rb.velocity[0] +=
                force_vec[i][0].signum() * (node_force[0] / rb.mass).abs() * self.delta_time;
            rb.velocity[1] +=
                force_vec[i][1].signum() * (node_force[1] / rb.mass).abs() * self.delta_time;
        }
    }

    fn update_node_position(&self, graph: Arc<RwLock<Graph<T>>>) {
        let mut graph_write_guard = graph.write().unwrap();

        'damping: for n in graph_write_guard.get_node_mut_iter() {
            let rb = n.rigidbody.as_mut().unwrap();
            if rb.fixed {
                rb.velocity[0] = 0.0;
                rb.velocity[1] = 0.0;
                continue 'damping;
            }

            rb.velocity.scalar(self.damping);

            rb.position[0] += rb.velocity[0] * self.delta_time;
            rb.position[1] += rb.velocity[1] * self.delta_time;
        }
    }

    fn compute_direction_vector(n1: &RigidBody2D, n2: &RigidBody2D) -> [f32; 2] {
        [
            n2.position[0] - n1.position[0],
            n2.position[1] - n1.position[1],
        ]
    }

    pub fn compute_distance(n1: &RigidBody2D, n2: &RigidBody2D) -> f32 {
        f32::sqrt(
            (n2.position[0] - n1.position[0]).powi(2) + (n2.position[1] - n1.position[1]).powi(2),
        )
    }

    fn add_graph_spring_forces(
        &self,
        graph: Arc<RwLock<Graph<T>>>,
        force_vec_arc: Arc<Mutex<Vec<Vector2D>>>,
    ) {
        let mut force_vec = force_vec_arc.lock().unwrap();
        let graph_arc = Arc::clone(&graph);

        for edge in graph_arc.read().unwrap().get_edge_iter() {
            let g = graph_arc.read().unwrap();

            let n1 = g.get_node_by_index(edge.0).rigidbody.as_ref().unwrap();
            let n2 = g.get_node_by_index(edge.1).rigidbody.as_ref().unwrap();

            let spring_force = self.compute_spring_force(n1, n2);

            force_vec[edge.0] -= spring_force;
            force_vec[edge.1] += spring_force;
        }
    }

    fn compute_spring_force(&self, n1: &RigidBody2D, n2: &RigidBody2D) -> Vector2D {
        let vec = Self::compute_direction_vector(n1, n2);
        let dist = Self::compute_distance(n1, n2);

        let force_magnitude = self.spring_stiffness * (dist - self.spring_default_len);

        if dist == 0.0 {
            return Vector2D::new([force_magnitude, force_magnitude]);
        }

        let mut unit_vec = Vector2D::new([vec[0] / dist, vec[1] / dist]);

        unit_vec.scalar(-force_magnitude);
        unit_vec
    }

    fn repel_force(repel_force_const: f32, n1: &RigidBody2D, n2: &RigidBody2D) -> Vector2D {
        let vec = Self::compute_direction_vector(n1, n2);
        let mut dir_vec = Vector2D::new(vec);
        let mut force = Vector2D::new([0.0, 0.0]);

        let x_y_len = vec[0].abs() + vec[1].abs();
        if x_y_len == 0.0 || Self::compute_distance(n1, n2) < 0.1 {
            return Vector2D::new([
                rand::thread_rng().gen_range(-1..1) as f32,
                rand::thread_rng().gen_range(-1..1) as f32,
            ]);
        }

        let f = -repel_force_const * (n1.mass * n2.mass).abs() / (dir_vec.len() as f32).powi(2);

        if vec[0] != 0.0 {
            force[0] += f * (vec[0] / x_y_len);
        }
        if vec[1] != 0.0 {
            force[1] += f * (vec[1] / x_y_len);
        }

        dir_vec.scalar(f / x_y_len);

        dir_vec
    }

    fn compute_center_gravity(gravity_force: f32, n1: &RigidBody2D) -> Vector2D {
        let mut force = Vector2D::new([0.0, 0.0]);
        if n1.position[0].is_finite() {
            force[0] = gravity_force * (-n1.position[0]) * n1.mass;
        }

        if n1.position[1].is_finite() {
            force[1] = gravity_force * (-n1.position[1]) * n1.mass;
        }
        force
    }
}
