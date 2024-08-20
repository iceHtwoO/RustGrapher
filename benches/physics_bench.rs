use std::sync::{Arc, RwLock};

use criterion::{criterion_group, criterion_main, Criterion};
use grapher::{graph::Graph, simgraph::SimGraph};

const NODE: [u32; 7] = [1, 10, 100, 500, 1000, 2500, 5000];

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

criterion_group!(
    simulation,
    simulation_all_enabled,
    simulation_repel,
    simulation_spring,
    simulation_gravity
);
criterion_main!(simulation);
