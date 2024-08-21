use std::marker::PhantomData;

#[derive(Debug)]

pub struct QuadTree {
    pub children: Vec<Option<Self>>,
    pub boundary: Rectangle,
    pub mass: f32,
    position: [f32; 2],
}

impl QuadTree {
    pub fn new(boundary: Rectangle) -> Self {
        Self {
            children: vec![None, None, None, None],
            boundary: boundary,
            mass: 0.0,
            position: [0.0, 0.0],
        }
    }

    fn new_leaf(position: [f32; 2], mass: f32, boundary: Rectangle) -> Self {
        Self {
            children: vec![None, None, None, None],
            boundary,
            mass,
            position,
        }
    }

    pub fn get_position(&self) -> [f32; 2] {
        [self.position[0] / self.mass, self.position[1] / self.mass]
    }

    pub fn insert(&mut self, position: [f32; 2], mass: f32, boundary: &Rectangle) {
        let mut parent: &mut Self = self;

        if parent.mass == 0.0 {
            parent.mass = mass;
            parent.position = [position[0] * mass, position[1] * mass];
            parent.boundary = boundary.clone();
            return;
        }

        // Search the lowest parent
        while !parent.is_leaf() {
            let (_, quadrent) = parent.boundary.get_section(&position);
            if parent.children[quadrent as usize].is_none() {
                break;
            }
            parent.update_mass(&position, &mass);
            parent = parent.children[quadrent as usize].as_mut().unwrap();
        }

        let (mut new_bb, mut quadrent) = parent.boundary.get_section(&position);
        if parent.is_leaf() {
            let leaf_position = parent.position;
            let leaf_mass = parent.mass;
            let l_pos = [leaf_position[0] / leaf_mass, leaf_position[1] / leaf_mass];

            //Update the mass of the parent
            parent.update_mass(&position, &mass);

            let (mut new_bb_leaf, mut quadrent_leaf) = parent.boundary.get_section(&l_pos);

            while quadrent == quadrent_leaf {
                //TODO: Handle Case where both are on same x and y
                parent.children[quadrent_leaf as usize] =
                    Some(QuadTree::new_leaf(leaf_position, leaf_mass, new_bb_leaf));
                parent = parent.children[quadrent_leaf as usize].as_mut().unwrap();

                (new_bb, quadrent) = parent.boundary.get_section(&position);
                (new_bb_leaf, quadrent_leaf) = parent.boundary.get_section(&l_pos);
            }

            parent.children[quadrent_leaf as usize] =
                Some(Self::new_leaf(leaf_position, leaf_mass, new_bb_leaf));
        }
        parent.children[quadrent as usize] = Some(Self::new_leaf(
            [position[0] * mass, position[1] * mass],
            mass,
            new_bb,
        ));
    }

    fn update_mass(&mut self, position: &[f32; 2], mass: &f32) {
        self.position[0] += position[0] * mass;
        self.position[1] += position[1] * mass;
        self.mass += mass;
    }

    fn is_leaf(&self) -> bool {
        for child in self.children.iter() {
            if child.is_some() {
                return false;
            }
        }
        return true;
    }

    pub fn get_stack<'a>(&'a self, position: &[f32; 2], theta: f32) -> Vec<&'a Self> {
        let mut nodes: Vec<&QuadTree> = vec![];
        let mut stack = vec![self];

        while !stack.is_empty() {
            let parent = match stack.pop() {
                Some(p) => p,
                None => break,
            };
            let s = parent.boundary.width.max(parent.boundary.height);
            let center_mass = [
                parent.position[0] / parent.mass,
                parent.position[1] / parent.mass,
            ];
            let d = f32::sqrt(
                (center_mass[0] - position[0]).powi(2) + (center_mass[1] - position[1]).powi(2),
            );

            if s / d < theta || parent.is_leaf() {
                nodes.push(parent);
            } else {
                for child in parent.children.iter() {
                    match child {
                        Some(c) => stack.push(c),
                        None => (),
                    }
                }
            }
        }
        nodes
    }
}

#[derive(Clone, Debug)]
pub struct Rectangle {
    pub center: [f32; 2],
    pub width: f32,
    pub height: f32,
}

impl Rectangle {
    pub fn new(center: [f32; 2], width: f32, height: f32) -> Self {
        Self {
            center,
            width,
            height,
        }
    }
    fn get_section(&self, loc: &[f32; 2]) -> (Rectangle, RectangleSection) {
        let sect;
        let mut newx = self.center[0];
        let mut newy = self.center[1];
        if loc[1] < self.center[1] {
            if loc[0] < self.center[0] {
                sect = RectangleSection::TL;
            } else {
                sect = RectangleSection::TR;
            }
            newy -= 0.25 * self.height;
        } else {
            if loc[0] < self.center[0] {
                sect = RectangleSection::BL;
            } else {
                sect = RectangleSection::BR;
            }
            newy += 0.25 * self.height;
        }

        if loc[0] < self.center[0] {
            newx -= 0.25 * self.width;
        } else {
            newx += 0.25 * self.width;
        }

        (
            Rectangle {
                center: [newx, newy],
                width: self.width * 0.5,
                height: self.height * 0.5,
            },
            sect,
        )
    }
}

#[derive(Debug, Clone, Copy, PartialEq)]
enum RectangleSection {
    TL = 0,
    TR = 1,
    BL = 2,
    BR = 3,
}
