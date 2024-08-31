use std::{
    fmt::Debug,
    sync::{Arc, Mutex, RwLock},
    thread::{self, JoinHandle},
};

use glam::Vec2;

use crate::{
    graph::Graph,
    properties::RigidBody2D,
    quadtree::{BoundingBox2D, QuadTree},
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
    quadtree_theta: f32,
}

impl<T> Default for SimGraph<T>
where
    T: PartialEq + Send + Sync + 'static + Clone,
{
    fn default() -> Self {
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
            quadtree_theta: 0.25,
        }
    }
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
            Vec2::ZERO;
            graph.read().unwrap().get_node_count()
        ]));

        self.calculate_forces(Arc::clone(&graph), Arc::clone(&f_vec));

        self.apply_node_force(Arc::clone(&graph), Arc::clone(&f_vec));
        self.update_node_position(Arc::clone(&graph));
    }

    fn calculate_forces(&mut self, g: Arc<RwLock<Graph<T>>>, f_vec: Arc<Mutex<Vec<Vec2>>>) {
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
        vec_arc: Arc<Mutex<Vec<Vec2>>>,
        graph: Arc<RwLock<Graph<T>>>,
    ) -> JoinHandle<()> {
        let repel_force_const = self.repel_force_const;
        let repel_force = self.repel_force;
        let gravity = self.gravity;
        let gravity_force = self.gravity_force;
        let quadtree = Self::build_quadtree(Arc::clone(&graph));
        let theta = self.quadtree_theta;

        let handle = thread::spawn(move || {
            let mut force_vec: Vec<Vec2> = vec![Vec2::ZERO; node_count];
            let graph_read_guard = graph.read().unwrap();

            #[allow(clippy::needless_range_loop)]
            for i in start_index..end_index {
                let n1 = graph_read_guard
                    .get_node_by_index(i)
                    .rigidbody
                    .as_ref()
                    .unwrap();
                if repel_force {
                    // Get node approximation from Quadtree
                    let node_approximations = quadtree.get_stack(&n1.position, theta);

                    // Calculate Repel Force
                    for node_approximation in node_approximations {
                        let node_approximation_particle = RigidBody2D::new(
                            node_approximation.get_position_ref(),
                            node_approximation.mass,
                        );
                        let repel_force =
                            Self::repel_force(repel_force_const, n1, &node_approximation_particle);

                        force_vec[i] += repel_force;
                    }
                }

                //Calculate Gravity Force
                if gravity {
                    let gravity_force = Self::compute_center_gravity(gravity_force, n1);
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

        let mut max_x = -f32::INFINITY;
        let mut min_x = f32::INFINITY;
        let mut max_y = -f32::INFINITY;
        let mut min_y = f32::INFINITY;

        for node in graph_read_guard.get_node_iter() {
            let rb: &RigidBody2D = node.rigidbody.as_ref().unwrap();
            max_x = max_x.max(rb.position[0]);
            max_y = max_y.max(rb.position[1]);
            min_x = min_x.min(rb.position[0]);
            min_y = min_y.min(rb.position[1]);
        }

        let w = max_x - min_x;
        let h = max_y - min_y;
        let boundary = BoundingBox2D::new(Vec2::new(min_x + 0.5 * w, min_y + 0.5 * h), w, h);
        let mut quadtree = QuadTree::new(boundary.clone());

        for n in graph_read_guard.get_node_iter() {
            let rb: &RigidBody2D = n.rigidbody.as_ref().unwrap();
            quadtree.insert(None, rb.position, rb.mass);
        }
        quadtree
    }

    fn apply_node_force(&self, graph: Arc<RwLock<Graph<T>>>, force_vec_arc: Arc<Mutex<Vec<Vec2>>>) {
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

            rb.velocity *= self.damping;

            rb.position += rb.velocity * self.delta_time;
        }
    }

    fn add_graph_spring_forces(
        &self,
        graph: Arc<RwLock<Graph<T>>>,
        force_vec_arc: Arc<Mutex<Vec<Vec2>>>,
    ) {
        let mut force_vec = force_vec_arc.lock().unwrap();
        let graph_arc = Arc::clone(&graph);

        for edge in graph_arc.read().unwrap().get_edge_iter() {
            let g = graph_arc.read().unwrap();

            let n1 = g.get_node_by_index(edge.0).rigidbody.as_ref().unwrap();
            let n2 = g.get_node_by_index(edge.1).rigidbody.as_ref().unwrap();

            let spring_force: Vec2 = self.compute_spring_force(n1, n2);

            force_vec[edge.0] -= spring_force;
            force_vec[edge.1] += spring_force;
        }
    }

    fn compute_spring_force(&self, n1: &RigidBody2D, n2: &RigidBody2D) -> Vec2 {
        let direction_vec: Vec2 = n2.position - n1.position;
        let force_magnitude =
            self.spring_stiffness * (direction_vec.length() - self.spring_default_len);

        direction_vec.normalize_or(Vec2::ZERO) * -force_magnitude
    }

    fn repel_force(repel_force_const: f32, n1: &RigidBody2D, n2: &RigidBody2D) -> Vec2 {
        let dir_vec: Vec2 = n2.position - n1.position;

        let dir_vec_normalized = dir_vec.normalize_or(Vec2::ZERO);
        if dir_vec.length_squared() == 0.0 {
            return Vec2::ZERO;
        }

        let f = -repel_force_const * (n1.mass * n2.mass).abs() / dir_vec.length_squared();

        let force = dir_vec_normalized * f;

        force.clamp(
            Vec2::new(-100000.0, -100000.0),
            Vec2::new(100000.0, 100000.0),
        )
    }

    fn compute_center_gravity(gravity_force: f32, node: &RigidBody2D) -> Vec2 {
        -node.position * node.mass * gravity_force
    }
}
