use std::{
    fmt::Debug,
    sync::{Arc, Mutex, RwLock},
    thread::{self, JoinHandle},
};

use glam::{Vec2, Vec3, Vec3Swizzles};
use petgraph::{
    prelude::StableGraph,
    visit::{EdgeRef, IntoEdgeReferences},
};
use rand::Rng;

use crate::{
    properties::{RigidBody2D, Spring},
    quadtree::BoundingBox2D,
    quadtree::QuadTree,
};

#[derive(Clone, Debug)]
pub struct Simulator {
    pub rigid_bodies: Arc<RwLock<Vec<RigidBody2D>>>,
    pub springs: Arc<RwLock<Vec<Spring>>>,
    repel: bool,
    spring: bool,
    gravity: bool,
    spring_stiffness: f32,
    spring_neutral_length: f32,
    delta_time: f32,
    gravity_force: f32,
    repel_force_const: f32,
    damping: f32,
    quadtree_theta: f32,
    freeze_thresh: f32,
    max_threads: u32,
    simulation_thread_lock: Arc<RwLock<bool>>,
}

impl Simulator {
    pub fn builder() -> SimulatorBuilder {
        SimulatorBuilder::default()
    }

    pub fn average_node_position(&self) -> Vec2 {
        let rb_guard = self.rigid_bodies.read().unwrap();

        let mut avg = Vec2::ZERO;
        for rb in rb_guard.iter() {
            avg += rb.position;
        }

        avg / rb_guard.len() as f32
    }

    pub fn max_node_mass(&self) -> f32 {
        let graph_read_guard = self.rigid_bodies.read().unwrap();
        let mut max_m = 0.0;
        for rb in graph_read_guard.iter() {
            max_m = rb.mass.max(max_m);
        }
        max_m
    }

    pub fn insert_node(&self, vec: Vec3) {
        let _lock = self.simulation_thread_lock.write().unwrap();

        let mut rb = self.rigid_bodies.write().unwrap();
        rb.push(RigidBody2D::new(vec.xy(), 5.0));
    }

    pub fn simulation_step(&self) {
        // Lock so actions can only be performed when sim step has ended
        let _lock = self.simulation_thread_lock.write().unwrap();

        let f_vec = Arc::new(Mutex::new(vec![
            Vec2::ZERO;
            self.rigid_bodies.read().unwrap().len()
        ]));

        self.calculate_forces(Arc::clone(&f_vec));

        self.apply_node_force(Arc::clone(&f_vec));
        self.update_node_position();
    }

    fn calculate_forces(&self, f_vec: Arc<Mutex<Vec<Vec2>>>) {
        if self.repel || self.gravity {
            let node_count = { self.rigid_bodies.read().unwrap().len() };
            let thread_count = usize::min(node_count, self.max_threads as usize);

            let mut handles = Vec::with_capacity(thread_count);

            let nodes_per_thread = node_count / thread_count;

            let quadtree = Arc::new(build_quadtree(Arc::clone(&self.rigid_bodies)));
            for thread in 0..thread_count {
                let mut extra = 0;

                if thread == thread_count - 1 {
                    extra = node_count % thread_count;
                }

                let handle = self.spawn_physics_thread(
                    thread * nodes_per_thread,
                    (thread + 1) * nodes_per_thread + extra,
                    node_count,
                    Arc::clone(&f_vec),
                    Arc::clone(&self.rigid_bodies),
                    Arc::clone(&quadtree),
                );

                handles.push(handle);
            }

            if self.spring {
                self.compute_spring_forces_edges(Arc::clone(&f_vec));
            }

            for handle in handles {
                handle.join().unwrap();
            }
        }
    }

    fn spawn_physics_thread(
        &self,
        start_index: usize,
        end_index: usize,
        node_count: usize,
        force_vec_out: Arc<Mutex<Vec<Vec2>>>,
        rb_vec: Arc<RwLock<Vec<RigidBody2D>>>,
        quadtree: Arc<QuadTree>,
    ) -> JoinHandle<()> {
        let repel_force_const = self.repel_force_const;
        let repel_force = self.repel;
        let gravity = self.gravity;
        let gravity_force = self.gravity_force;
        let theta = self.quadtree_theta;

        let handle = thread::spawn(move || {
            let mut force_vec: Vec<Vec2> = vec![Vec2::ZERO; node_count];

            #[allow(clippy::needless_range_loop)]
            for i in start_index..end_index {
                let rb = &rb_vec.read().unwrap()[i];
                if rb.fixed {
                    continue;
                }
                if repel_force {
                    // Get node approximation from Quadtree
                    let node_approximations = quadtree.stack(&rb.position, theta);

                    // Calculate Repel Force
                    for node_approximation in node_approximations {
                        let node_approximation_particle = RigidBody2D::new(
                            node_approximation.position(),
                            node_approximation.mass(),
                        );
                        let repel_force =
                            Self::repel_force(repel_force_const, rb, &node_approximation_particle);

                        force_vec[i] += repel_force;
                    }
                }

                //Calculate Gravity Force
                if gravity {
                    let gravity_force = Self::compute_center_gravity(gravity_force, rb);
                    force_vec[i] += gravity_force;
                }
            }

            {
                let mut force_list = force_vec_out.lock().unwrap();

                for (i, force) in force_vec.into_iter().enumerate() {
                    force_list[i] += force;
                }
            }
        });

        handle
    }

    fn apply_node_force(&self, force_vec_arc: Arc<Mutex<Vec<Vec2>>>) {
        let mut graph_write_guard = self.rigid_bodies.write().unwrap();
        let force_vec = force_vec_arc.lock().unwrap();
        for (i, rb) in graph_write_guard.iter_mut().enumerate() {
            let node_force = force_vec[i];

            rb.velocity += node_force / rb.mass * self.delta_time;
        }
    }

    fn update_node_position(&self) {
        let mut graph_write_guard = self.rigid_bodies.write().unwrap();

        'damping: for rb in graph_write_guard.iter_mut() {
            if rb.fixed {
                rb.velocity = Vec2::ZERO;
                continue 'damping;
            }

            rb.velocity *= self.damping;

            rb.position += rb.velocity * self.delta_time;

            if self.freeze_thresh > rb.total_velocity() {
                rb.fixed = true;
            }
        }
    }

    fn compute_spring_forces_edges(&self, force_vec_arc: Arc<Mutex<Vec<Vec2>>>) {
        let mut force_vec = force_vec_arc.lock().unwrap();

        for spring in self.springs.read().unwrap().iter() {
            let g = self.rigid_bodies.read().unwrap();

            let rb1 = &g[spring.rb1];
            let rb2 = &g[spring.rb2];

            let spring_force: Vec2 = self.compute_spring_force(rb1, rb2);

            force_vec[spring.rb1] -= spring_force;
            force_vec[spring.rb2] += spring_force;
        }
    }

    fn compute_spring_force(&self, n1: &RigidBody2D, n2: &RigidBody2D) -> Vec2 {
        let direction_vec: Vec2 = n2.position - n1.position;
        let force_magnitude =
            self.spring_stiffness * (direction_vec.length() - self.spring_neutral_length);

        direction_vec.normalize_or(Vec2::ZERO) * -force_magnitude
    }

    fn repel_force(repel_force_const: f32, n1: &RigidBody2D, n2: &RigidBody2D) -> Vec2 {
        let dir_vec: Vec2 = n2.position - n1.position;

        if dir_vec.length_squared() == 0.0 {
            return Vec2::ZERO;
        }

        let f = -repel_force_const * (n1.mass * n2.mass).abs() / dir_vec.length_squared();

        let dir_vec_normalized = dir_vec.normalize_or(Vec2::ZERO);
        let force = dir_vec_normalized * f;

        force.clamp(
            Vec2::new(-100000.0, -100000.0),
            Vec2::new(100000.0, 100000.0),
        )
    }

    fn compute_center_gravity(gravity_force: f32, node: &RigidBody2D) -> Vec2 {
        -node.position * node.mass * gravity_force
    }

    pub fn find_closest_node_index(&self, loc: Vec3) -> Option<u32> {
        let rb_read = self.rigid_bodies.read().unwrap();
        let mut dist = f32::INFINITY;
        let mut index = 0;
        for (i, rb) in rb_read.iter().enumerate() {
            let new_dist = rb.position.distance(loc.xy());
            if new_dist < dist {
                dist = new_dist;
                index = i as u32;
            }
        }
        if dist.is_infinite() {
            None
        } else {
            Some(index)
        }
    }

    pub fn set_node_location_by_index(&self, loc: Vec3, index: u32) {
        let mut rb_write = self.rigid_bodies.write().unwrap();
        rb_write[index as usize].position = loc.xy();
    }
}

fn build_quadtree(rb_vec_arc: Arc<RwLock<Vec<RigidBody2D>>>) -> QuadTree {
    let rb_vec_guard = rb_vec_arc.read().unwrap();

    let mut min = Vec2::INFINITY;
    let mut max = Vec2::NEG_INFINITY;

    for rb in rb_vec_guard.iter() {
        min = min.min(rb.position);
        max = max.max(rb.position);
    }
    let dir = max - min;

    let boundary = BoundingBox2D::new((dir / 2.0) + min, dir[0], dir[1]);
    let mut quadtree = QuadTree::with_capacity(boundary.clone(), rb_vec_guard.len());

    for rb in rb_vec_guard.iter() {
        quadtree.insert(rb.position, rb.mass);
    }
    quadtree
}

fn build_property_vec<T, E, D>(
    graph: StableGraph<T, E, D, u32>,
    edge_based_mass: bool,
) -> (Vec<RigidBody2D>, Vec<Spring>)
where
    D: petgraph::EdgeType,
{
    let mut vec_rb = vec![];
    let mut vec_spring = vec![];

    for _ in 0..graph.node_count() {
        vec_rb.push(RigidBody2D::new(
            Vec2::new(
                rand::thread_rng().gen_range(-60.0..60.0),
                rand::thread_rng().gen_range(-60.0..60.0),
            ),
            1.0,
        ));
    }

    let edges = graph.edge_references();

    for s in edges {
        if edge_based_mass {
            vec_rb[s.target().index()].mass += 1.0;
            vec_rb[s.source().index()].mass += 1.0;
        }

        vec_spring.push(Spring {
            rb1: s.source().index(),
            rb2: s.target().index(),
            spring_neutral_len: 2.0,
            spring_stiffness: 1.0,
        })
    }

    (vec_rb, vec_spring)
}

/// Builder for `Simulator`
pub struct SimulatorBuilder {
    repel: bool,
    spring: bool,
    gravity: bool,
    spring_stiffness: f32,
    spring_neutral_length: f32,
    delta_time: f32,
    gravity_force: f32,
    repel_force_const: f32,
    damping: f32,
    quadtree_theta: f32,
    freeze_thresh: f32,
    max_threads: u32,
    edge_based_mass: bool,
}

impl SimulatorBuilder {
    /// Get a Instance of `SimulatorBuilder` with default values
    pub fn new() -> Self {
        Self::default()
    }

    /// If nodes should repel from each other
    ///
    /// Default: `true`
    pub fn repel(mut self, repel: bool) -> Self {
        self.repel = repel;
        self
    }

    /// If edges should act as springs and pull the nodes together
    ///
    /// Default: `true`
    pub fn spring(mut self, spring: bool) -> Self {
        self.spring = spring;
        self
    }

    /// If nodes should be globally pulled to the center of the canvas
    ///
    /// Default: `true`
    pub fn gravity(mut self, gravity: bool) -> Self {
        self.gravity = gravity;
        self
    }

    /// How strong the spring force should be
    ///
    /// Default: `100.0`
    pub fn spring_stiffness(mut self, spring_stiffness: f32) -> Self {
        self.spring_stiffness = spring_stiffness;
        self
    }

    /// Length of a edge in neutral position.
    ///
    /// If edge is shorter it pushers apart.
    /// If edge is longer it pulls together.
    ///
    /// Set to `0` if edges should always pull apart.
    ///
    /// Default: `2.0`
    pub fn spring_neutral_length(mut self, neutral_length: f32) -> Self {
        self.spring_neutral_length = neutral_length;
        self
    }

    /// How strong the pull to the center should be.
    ///
    /// Default: `1.0`
    pub fn gravity_force(mut self, gravity_force: f32) -> Self {
        self.gravity_force = gravity_force;
        self
    }

    /// How strong nodes should push others away.
    ///
    /// Default: `100.0`
    pub fn repel_force(mut self, repel_force_const: f32) -> Self {
        self.repel_force_const = repel_force_const;
        self
    }

    /// Amount of damping that should be applied to the nodes movement
    ///
    /// `1.0` -> No Damping
    ///
    /// `0.0` -> No Movement
    ///
    /// Default: `0.9`
    pub fn damping(mut self, damping: f32) -> Self {
        self.damping = damping;
        self
    }

    /// To improve performance(`n log(n)`) we use a Quadtree to approximate forces form far away nodes.
    /// Higher numbers result in more approximations but faster calculations.
    ///
    /// Value should be between 0.0 and 1.0.
    ///
    /// `0.0` -> No approximation -> n^2 brute force
    ///
    /// Default: `0.75`
    pub fn quadtree_accuracy(mut self, theta: f32) -> Self {
        self.quadtree_theta = theta;
        self
    }

    /// Freeze nodes when their velocity falls below `freeze_thresh`.
    /// Set to `-1` to disable
    ///
    /// Default: `1e-2`
    pub fn freeze_threshold(mut self, freeze_thresh: f32) -> Self {
        self.freeze_thresh = freeze_thresh;
        self
    }

    /// How much time a simulation step should simulate. (euler method)
    ///
    /// Bigger time steps result in faster simulations, but less accurate or even wrong simulations.
    ///
    /// `delta_time` is in seconds
    ///
    /// Panics when delta time is `0` or below
    ///
    /// Default: `0.005`
    pub fn delta_time(mut self, delta_time: f32) -> Self {
        if delta_time <= 0.0 {
            panic!("delta_time may not be 0 or below!");
        }
        self.delta_time = delta_time;
        self
    }

    /// How many CPU threads should be used to calculate physics
    ///
    /// Panics when thread count is `0`
    ///
    /// Default: `16`
    pub fn max_threads(mut self, max_threads: u32) -> Self {
        if max_threads == 0 {
            panic!("Threads may not be 0!");
        }
        self.max_threads = max_threads;
        self
    }

    /// If nodes mass should be increased based on the edges connected to a node
    ///
    /// Default: `true`
    pub fn edge_based_mass(mut self, edge_based_mass: bool) -> Self {
        self.edge_based_mass = edge_based_mass;
        self
    }

    /// Constructs a instance of `Simulator`
    pub fn build<T, E, D>(self, graph: StableGraph<T, E, D, u32>) -> Simulator
    where
        D: petgraph::EdgeType,
    {
        let (rigid_bodies, springs) = build_property_vec(graph, self.edge_based_mass);
        Simulator {
            simulation_thread_lock: Arc::new(RwLock::new(true)),
            repel: self.repel,
            spring: self.spring,
            gravity: self.gravity,
            repel_force_const: self.repel_force_const,
            spring_stiffness: self.spring_stiffness,
            spring_neutral_length: self.spring_neutral_length,
            gravity_force: self.gravity_force,
            delta_time: self.delta_time,
            damping: self.damping,
            quadtree_theta: self.quadtree_theta,
            freeze_thresh: self.freeze_thresh,
            max_threads: self.max_threads,
            rigid_bodies: Arc::new(RwLock::new(rigid_bodies)),
            springs: Arc::new(RwLock::new(springs)),
        }
    }
}

impl Default for SimulatorBuilder {
    /// Get a Instance of `SimulatorBuilder` with default values
    fn default() -> Self {
        Self {
            repel: true,
            spring: true,
            gravity: true,
            repel_force_const: 100.0,
            spring_stiffness: 100.0,
            spring_neutral_length: 2.0,
            gravity_force: 1.0,
            delta_time: 0.005,
            damping: 0.9,
            quadtree_theta: 0.75,
            freeze_thresh: 1e-2,
            max_threads: 16,
            edge_based_mass: true,
        }
    }
}
