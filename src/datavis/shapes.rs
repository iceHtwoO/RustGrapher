use std::f32::consts::{PI, TAU};

use glium::implement_vertex;

#[derive(Copy, Clone, Debug)]
pub struct Vertex {
    pub position: [f32; 3],
    pub color: [f32; 4],
}
implement_vertex!(Vertex, position, color);

#[allow(dead_code)]
pub fn rectangle(pos: [f32; 3], color: [f32; 4], s: f32) -> Vec<Vertex> {
    let mut shape = Vec::with_capacity(6);
    shape.push(Vertex {
        position: [pos[0] - s, pos[1] - s, pos[2]],
        color,
    });

    shape.push(Vertex {
        position: [pos[0] + s, pos[1] - s, pos[2]],
        color,
    });

    shape.push(Vertex {
        position: [pos[0] - s, pos[1] + s, pos[2]],
        color,
    });
    shape.push(Vertex {
        position: [pos[0] + s, pos[1] + s, pos[2]],
        color,
    });

    shape.push(Vertex {
        position: [pos[0] + s, pos[1] - s, pos[2]],
        color,
    });

    shape.push(Vertex {
        position: [pos[0] - s, pos[1] + s, pos[2]],
        color,
    });
    shape
}

#[allow(dead_code)]
pub fn rectangle_lines(pos: [f32; 3], color: [f32; 4], x: f32, y: f32) -> Vec<Vertex> {
    let mut shape = Vec::with_capacity(6);
    shape.push(Vertex {
        position: [pos[0] - x, pos[1] - y, pos[2]],
        color,
    });

    shape.push(Vertex {
        position: [pos[0] + x, pos[1] - y, pos[2]],
        color,
    });

    shape.push(Vertex {
        position: [pos[0] + x, pos[1] - y, pos[2]],
        color,
    });
    shape.push(Vertex {
        position: [pos[0] + x, pos[1] + y, pos[2]],
        color,
    });

    shape.push(Vertex {
        position: [pos[0] + x, pos[1] + y, pos[2]],
        color,
    });
    shape.push(Vertex {
        position: [pos[0] - x, pos[1] + y, pos[2]],
        color,
    });

    shape.push(Vertex {
        position: [pos[0] - x, pos[1] + y, pos[2]],
        color,
    });

    shape.push(Vertex {
        position: [pos[0] - x, pos[1] - y, pos[2]],
        color,
    });

    shape
}

pub fn circle(pos: [f32; 3], color: [f32; 4], r: f32, res: usize) -> Vec<Vertex> {
    let mut shape = Vec::with_capacity(3 * res);
    let a = 2.0 * PI / res as f32;

    for i in 0..res {
        let i = i as f32;
        shape.push(Vertex {
            position: pos,
            color,
        });
        shape.push(Vertex {
            position: [
                pos[0] + (r * f32::sin(a * i)),
                pos[1] + (r * f32::cos(a * i)),
                pos[2],
            ],
            color,
        });
        shape.push(Vertex {
            position: [
                pos[0] + (r * f32::sin(a * (i + 1.0))),
                pos[1] + (r * f32::cos(a * (i + 1.0))),
                pos[2],
            ],
            color,
        });
    }

    shape
}

pub fn line(p1: [f32; 3], p2: [f32; 3], color: [f32; 4]) -> Vec<Vertex> {
    let mut shape = Vec::with_capacity(2);

    shape.push(Vertex {
        position: p1,
        color,
    });

    shape.push(Vertex {
        position: p2,
        color,
    });

    shape
}

#[allow(dead_code)]
pub fn sphere(p1: [f32; 3], color: [f32; 4], r: f32, res: u32) -> Vec<Vertex> {
    let mut shape = Vec::with_capacity(2);

    for i in 0..res {
        let lon = (TAU / res as f32 * i as f32) - PI;
        for j in 0..res {
            let lat = (PI / res as f32 * j as f32) - PI / 2.0;
            shape.push(Vertex {
                position: [
                    r * lon.sin() * lat.cos() + p1[0],
                    r * lon.sin() * lat.sin() + p1[1],
                    r * lon.cos() + p1[2],
                ],
                color,
            });
        }
    }

    shape
}
