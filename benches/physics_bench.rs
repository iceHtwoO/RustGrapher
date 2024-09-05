use criterion::{black_box, criterion_group, criterion_main, BenchmarkId, Criterion};
use glam::Vec2;
use grapher::{quadtree::BoundingBox2D, quadtree::QuadTree};
use rand::Rng;

const NODE: [u32; 10] = [10, 100, 500, 1000, 2500, 3250, 4000, 5000, 10000, 30000];
fn quadtree_insert(c: &mut Criterion) {
    let w = 1000.0;
    let bb = BoundingBox2D::new(Vec2::ZERO, w, w);
    let mut qt: QuadTree = QuadTree::new(bb.clone());
    let mut rng = rand::thread_rng();
    let mut group = c.benchmark_group("QuadTree insert");

    group.bench_function("Vector Quadtree", |b| {
        b.iter(|| {
            qt.insert(
                black_box(Vec2::new(
                    rng.gen_range((-w / 2.0)..(w / 2.0)),
                    rng.gen_range((-w / 2.0)..(w / 2.0)),
                )),
                rng.gen_range(1.0..2000.0),
            )
        });
    });
}

fn quadtree_get_stack(c: &mut Criterion) {
    let w = 1000.0;
    let bb = BoundingBox2D::new(Vec2::ZERO, w, w);
    let mut rng = rand::thread_rng();
    let mut group = c.benchmark_group("QuadTree get");
    for i in NODE {
        let mut qt: QuadTree = QuadTree::new(bb.clone());
        for _ in 0..i {
            let v = Vec2::new(
                rng.gen_range((-w / 2.0)..(w / 2.0)),
                rng.gen_range((-w / 2.0)..(w / 2.0)),
            );
            qt.insert(black_box(v), rng.gen_range(1.0..2000.0));
        }
        group.throughput(criterion::Throughput::Elements(i as u64));
        group.bench_function(BenchmarkId::new("Quadtree Stack Vector", i), |b| {
            b.iter(|| {
                qt.stack(
                    black_box(&Vec2::new(
                        rng.gen_range((-w / 2.0)..(w / 2.0)),
                        rng.gen_range((-w / 2.0)..(w / 2.0)),
                    )),
                    0.75,
                )
            });
        });
    }
}

criterion_group!(simulation, quadtree_insert, quadtree_get_stack);
criterion_main!(simulation);
