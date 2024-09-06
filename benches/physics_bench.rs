use criterion::{black_box, criterion_group, criterion_main, BenchmarkId, Criterion};
use glam::{Vec2, Vec3};
use grapher::{
    octatree::{BoundingBox3D, OctaTree},
    quadtree::{BoundingBox2D, QuadTree},
};
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

fn octatree_insert(c: &mut Criterion) {
    let w = 1000.0;
    let bb = BoundingBox3D::new(Vec3::ZERO, w, w, w);
    let mut ot: OctaTree = OctaTree::new(bb.clone());
    let mut rng = rand::thread_rng();
    let mut group = c.benchmark_group("Octatree insert");

    group.bench_function("Vector Octatree", |b| {
        b.iter(|| {
            ot.insert(
                black_box(Vec3::new(
                    rng.gen_range((-w / 2.0)..(w / 2.0)),
                    rng.gen_range((-w / 2.0)..(w / 2.0)),
                    rng.gen_range((-w / 2.0)..(w / 2.0)),
                )),
                rng.gen_range(1.0..2000.0),
            )
        });
    });
}

fn octatree_get_stack(c: &mut Criterion) {
    let w = 1000.0;
    let bb = BoundingBox3D::new(Vec3::ZERO, w, w, w);
    let mut rng = rand::thread_rng();
    let mut group = c.benchmark_group("Octatree get");
    for i in NODE {
        let mut ot: OctaTree = OctaTree::new(bb.clone());
        for _ in 0..i {
            let v = Vec3::new(
                rng.gen_range((-w / 2.0)..(w / 2.0)),
                rng.gen_range((-w / 2.0)..(w / 2.0)),
                rng.gen_range((-w / 2.0)..(w / 2.0)),
            );
            ot.insert(black_box(v), rng.gen_range(1.0..2000.0));
        }
        group.throughput(criterion::Throughput::Elements(i as u64));
        group.bench_function(BenchmarkId::new("Octatree Stack Vector", i), |b| {
            b.iter(|| {
                ot.stack(
                    black_box(&Vec3::new(
                        rng.gen_range((-w / 2.0)..(w / 2.0)),
                        rng.gen_range((-w / 2.0)..(w / 2.0)),
                        rng.gen_range((-w / 2.0)..(w / 2.0)),
                    )),
                    0.75,
                )
            });
        });
    }
}

fn bounding_box_2d_sub_quadrant(c: &mut Criterion) {
    let bb = BoundingBox2D::new(Vec2::ZERO, 1000.0, 1000.0);
    let mut group = c.benchmark_group("BB 2D Sub_quadrant");
    group.bench_function(BenchmarkId::new("BB 2D Sub_quadrant", 0), |b| {
        b.iter(|| {
            bb.sub_quadrant(black_box(rand::thread_rng().gen_range(0..=3)));
        });
    });
}

fn bounding_box_3d_sub_quadrant(c: &mut Criterion) {
    let bb = BoundingBox3D::new(Vec3::ZERO, 1000.0, 1000.0, 1000.0);
    let mut group = c.benchmark_group("BB 3D Sub_quadrant");
    group.bench_function(BenchmarkId::new("BB 3D Sub_quadrant", 0), |b| {
        b.iter(|| {
            bb.sub_quadrant(black_box(rand::thread_rng().gen_range(0..=3)));
        });
    });
}

criterion_group!(
    simulation,
    quadtree_insert,
    quadtree_get_stack,
    octatree_insert,
    octatree_get_stack,
    bounding_box_2d_sub_quadrant,
    bounding_box_3d_sub_quadrant,
);
criterion_main!(simulation);
