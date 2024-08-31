use std::f32::consts::PI;

use super::Vertex;

#[allow(dead_code)]
pub fn rectangle(pos: [f32; 3], color: [f32; 4], s: f32) -> Vec<Vertex> {
    vec![
        Vertex {
            position: [pos[0] - s, pos[1] - s, pos[2]],
            color,
        },
        Vertex {
            position: [pos[0] + s, pos[1] - s, pos[2]],
            color,
        },
        Vertex {
            position: [pos[0] - s, pos[1] + s, pos[2]],
            color,
        },
        Vertex {
            position: [pos[0] + s, pos[1] + s, pos[2]],
            color,
        },
        Vertex {
            position: [pos[0] + s, pos[1] - s, pos[2]],
            color,
        },
        Vertex {
            position: [pos[0] - s, pos[1] + s, pos[2]],
            color,
        },
    ]
}

#[allow(dead_code)]
pub fn rectangle_lines(pos: [f32; 3], color: [f32; 4], x: f32, y: f32) -> Vec<Vertex> {
    vec![
        Vertex {
            position: [pos[0] - x, pos[1] - y, pos[2]],
            color,
        },
        Vertex {
            position: [pos[0] + x, pos[1] - y, pos[2]],
            color,
        },
        Vertex {
            position: [pos[0] + x, pos[1] - y, pos[2]],
            color,
        },
        Vertex {
            position: [pos[0] + x, pos[1] + y, pos[2]],
            color,
        },
        Vertex {
            position: [pos[0] + x, pos[1] + y, pos[2]],
            color,
        },
        Vertex {
            position: [pos[0] - x, pos[1] + y, pos[2]],
            color,
        },
        Vertex {
            position: [pos[0] - x, pos[1] + y, pos[2]],
            color,
        },
        Vertex {
            position: [pos[0] - x, pos[1] - y, pos[2]],
            color,
        },
    ]
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
    vec![
        Vertex {
            position: p1,
            color,
        },
        Vertex {
            position: p2,
            color,
        },
    ]
}
