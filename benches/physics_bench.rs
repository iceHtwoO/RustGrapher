use std::sync::{Arc, RwLock};

use criterion::{black_box, criterion_group, criterion_main, Criterion};
use grapher::{
    graph::Graph,
    quadtree::{BoundingBox2D, QuadTree},
    simgraph::SimGraph,
};
use rand::Rng;

const NODE: [u32; 13] = [
    1, 10, 50, 100, 250, 500, 750, 1000, 1250, 2500, 3250, 4000, 5000,
];

fn setup(g: &mut Graph<u32>, node_count: u32) {
    for i in 0..node_count {
        g.add_node(i);
    }
}

fn simulation_all_enabled(c: &mut Criterion) {
    let mut group = c.benchmark_group("Repel, Spring, Gravity");

    for i in NODE {
        let mut g = Graph::new();
        setup(&mut g, i);
        let g_arc = Arc::new(RwLock::new(g));
        let mut sim = SimGraph::new();

        group.bench_function(format!("{}", i), |b| {
            b.iter(|| sim.simulation_step(Arc::clone(&g_arc)));
        });
    }
}

fn simulation_repel(c: &mut Criterion) {
    let mut group = c.benchmark_group("Repel");

    for i in NODE {
        let mut g = Graph::new();
        setup(&mut g, i);

        let g_arc = Arc::new(RwLock::new(g));
        let mut sim = SimGraph::new_config(true, false, false);

        group.bench_function(format!("{}", i), |b| {
            b.iter(|| sim.simulation_step(Arc::clone(&g_arc)));
        });
    }
}

fn simulation_spring(c: &mut Criterion) {
    let mut group = c.benchmark_group("spring");

    for i in NODE {
        let mut g = Graph::new();
        setup(&mut g, i);
        let g_arc = Arc::new(RwLock::new(g));
        let mut sim = SimGraph::new_config(false, true, false);

        group.bench_function(format!("{}", i), |b| {
            b.iter(|| sim.simulation_step(Arc::clone(&g_arc)));
        });
    }
}

fn simulation_gravity(c: &mut Criterion) {
    let mut group = c.benchmark_group("gravity");

    for i in NODE {
        let mut g = Graph::new();
        setup(&mut g, i);
        let g_arc = Arc::new(RwLock::new(g));
        let mut sim = SimGraph::new_config(false, false, true);

        group.bench_function(format!("{}", i), |b| {
            b.iter(|| sim.simulation_step(Arc::clone(&g_arc)));
        });
    }
}

fn quadtree_insert(c: &mut Criterion) {
    let w = 1000.0;
    let bb = BoundingBox2D::new([0.0, 0.0], w, w);
    let mut qt = QuadTree::new(bb.clone());
    let mut rng = rand::thread_rng();
    c.bench_function("Quadtree insert", |b| {
        b.iter(|| {
            qt.insert(
                None,
                black_box([
                    rng.gen_range((-w / 2.0)..(w / 2.0)),
                    rng.gen_range((-w / 2.0)..(w / 2.0)),
                ]),
                rng.gen_range(0.0..2000.0),
            )
        });
    });
}

fn quadtree_get_stack(c: &mut Criterion) {
    let w = 1000.0;
    let bb = BoundingBox2D::new([0.0, 0.0], w, w);
    let mut rng = rand::thread_rng();
    let mut group = c.benchmark_group("QuadTree get");
    for i in NODE {
        let mut qt = QuadTree::new(bb.clone());
        for n in 0..i {
            qt.insert(
                None,
                black_box([
                    rng.gen_range((-w / 2.0)..(w / 2.0)),
                    rng.gen_range((-w / 2.0)..(w / 2.0)),
                ]),
                rng.gen_range(0.0..2000.0),
            )
        }
        group.bench_function(format!("Nodes: {}", i), |b| {
            b.iter(|| {
                qt.get_stack(
                    black_box(&[
                        rng.gen_range((-w / 2.0)..(w / 2.0)),
                        rng.gen_range((-w / 2.0)..(w / 2.0)),
                    ]),
                    0.75,
                )
            });
        });
    }
}

criterion_group!(
    simulation,
    simulation_all_enabled,
    simulation_repel,
    simulation_spring,
    simulation_gravity,
    quadtree_insert,
    quadtree_get_stack
);
criterion_main!(simulation);
