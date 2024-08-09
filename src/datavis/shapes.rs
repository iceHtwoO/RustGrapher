use std::f64::consts::PI;

use super::Vertex;

pub fn create_cube(pos: [f64; 2], color: [f32; 4], s: f64) -> Vec<Vertex> {
    let mut shape = Vec::with_capacity(6);
    shape.push(Vertex {
        position: [pos[0] - s, pos[1] - s],
        color,
    });

    shape.push(Vertex {
        position: [pos[0] + s, pos[1] - s],
        color,
    });

    shape.push(Vertex {
        position: [pos[0] - s, pos[1] + s],
        color,
    });
    shape.push(Vertex {
        position: [pos[0] + s, pos[1] + s],
        color,
    });

    shape.push(Vertex {
        position: [pos[0] + s, pos[1] - s],
        color,
    });

    shape.push(Vertex {
        position: [pos[0] - s, pos[1] + s],
        color,
    });
    shape
}

pub fn create_circle(pos: [f64; 2], color: [f32; 4], r: f64, res: usize) -> Vec<Vertex> {
    let mut shape = Vec::with_capacity(3 * res);
    let a = 2.0 * PI / res as f64;

    for i in 0..res {
        let i = i as f64;
        shape.push(Vertex {
            position: pos,
            color,
        });
        shape.push(Vertex {
            position: [
                pos[0] + (r * f64::sin(a * i)),
                pos[1] + (r * f64::cos(a * i)),
            ],
            color,
        });
        shape.push(Vertex {
            position: [
                pos[0] + (r * f64::sin(a * (i + 1.0))),
                pos[1] + (r * f64::cos(a * (i + 1.0))),
            ],
            color,
        });
    }

    shape
}

pub fn line(p1: [f64; 2], p2: [f64; 2], color: [f32; 4]) -> Vec<Vertex> {
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
