use std::borrow::BorrowMut;

use glam::Vec3;

const EPSILON: f32 = 1e-3;

#[derive(Debug)]
pub struct OctaTree {
    pub children: Vec<Node3D>,
    pub boundary: BoundingBox3D,
    pub root: u32,
}

#[derive(Debug)]
pub enum Node3D {
    Root {
        indices: [u32; 8],
        mass: f32,
        pos: Vec3,
    },
    Leaf {
        mass: f32,
        pos: Vec3,
    },
}

impl OctaTree {
    pub fn new(boundary: BoundingBox3D) -> Self {
        Self {
            root: 0,
            boundary,
            children: Vec::new(),
        }
    }

    pub fn with_capacity(boundary: BoundingBox3D, capacity: usize) -> Self {
        Self {
            root: 0,
            boundary,
            children: Vec::with_capacity(capacity),
        }
    }

    pub fn insert(&mut self, new_pos: Vec3, new_mass: f32) {
        self.children.push(Node3D::new_leaf(new_pos, new_mass));
        let new_index = self.children.len() as u32 - 1;

        // When only one Node3D than there is no need to continue
        if self.children.len() == 1 {
            return;
        }

        let mut bb = self.boundary.clone();
        let mut root_index = self.root;

        while let Node3D::Root { indices, mass, pos } =
            self.children[root_index as usize].borrow_mut()
        {
            // Update mass and Pos of root
            *mass += new_mass;
            *pos += new_pos * new_mass;

            let section = bb.section(&new_pos);
            // If section not set: create new leaf and exit
            if indices[section as usize] == u32::MAX {
                indices[section as usize] = new_index;
                break;
            }

            root_index = indices[section as usize];
            bb = bb.sub_quadrant(section);
        }

        // if new leaf is too close to current leaf we merge
        // TODO: in this case we will have a "dead" leaf
        if let Node3D::Leaf { mass, pos } = self.children[root_index as usize] {
            if pos.distance(new_pos) < EPSILON {
                let m: f32 = mass + new_mass;
                self.children[root_index as usize] = Node3D::new_leaf(pos, m);
                return;
            }
        }

        // create new root until leaf and new leaf are in different sections
        while let Node3D::Leaf { mass, pos } = self.children[root_index as usize] {
            let mut fin = false;

            // Pushes the old leaf to the back of the vector and inserts its index into the index array of the new root
            let old_node = Node3D::new_leaf(pos, mass);
            self.children.push(old_node);
            let old_index = self.children.len() - 1;
            let section = bb.section(&pos);
            let mut ind = [
                u32::MAX,
                u32::MAX,
                u32::MAX,
                u32::MAX,
                u32::MAX,
                u32::MAX,
                u32::MAX,
                u32::MAX,
            ];
            ind[section as usize] = old_index as u32;

            let section = bb.section(&new_pos);

            // If section of the new root is empty we can set it and exit
            if ind[section as usize] == u32::MAX {
                ind[section as usize] = new_index;
                fin = true;
            }

            // sets the old leaf index to the new root
            let new_root = Node3D::new_root(pos * mass + new_pos * new_mass, mass + new_mass, ind);
            self.children[root_index as usize] = new_root;

            if fin {
                return;
            }

            root_index = old_index as u32;

            bb = bb.sub_quadrant(section);
        }
    }

    pub fn stack<'a>(&'a self, position: &Vec3, theta: f32) -> Vec<&'a Node3D> {
        let mut node: Vec<&Node3D> =
            Vec::with_capacity((self.children.len() as f32).log2() as usize);

        let mut s: f32 = self
            .boundary
            .width
            .max(self.boundary.height)
            .max(self.boundary.depth);

        if self.children.is_empty() {
            return vec![];
        }

        let mut stack: Vec<u32> = vec![0];
        let mut new_stack: Vec<u32> = vec![];
        'outer: loop {
            for node_index in stack {
                let parent = &self.children[node_index as usize];

                if let Node3D::Root { indices, .. } = parent {
                    let center_mass = parent.position();
                    let dist = center_mass.distance(*position);
                    if s / dist < theta {
                        if node.capacity() == node.len() {
                            node.reserve((node.len() as f32 * 0.1) as usize);
                        }
                        node.push(parent);
                    } else {
                        for i in indices {
                            if *i != u32::MAX {
                                new_stack.push(*i);
                            }
                        }
                    }
                }

                if let Node3D::Leaf { .. } = parent {
                    if node.capacity() == node.len() {
                        node.reserve((node.len() as f32 * 0.1) as usize);
                    }
                    node.push(parent);
                }
            }
            if new_stack.is_empty() {
                break 'outer;
            }
            s *= 0.5;
            stack = new_stack;
            new_stack = Vec::with_capacity(stack.len() * 2);
        }
        node
    }
}

impl Node3D {
    fn new_leaf(pos: Vec3, mass: f32) -> Self {
        Self::Leaf { mass, pos }
    }
    fn new_root(pos: Vec3, mass: f32, indices: [u32; 8]) -> Self {
        Self::Root { indices, mass, pos }
    }
    #[allow(dead_code)]
    pub fn is_leaf(&self) -> bool {
        matches!(self, Node3D::Leaf { .. })
    }

    #[allow(dead_code)]
    pub fn is_root(&self) -> bool {
        matches!(self, Node3D::Root { .. })
    }

    pub fn position(&self) -> Vec3 {
        match self {
            Node3D::Root { pos, mass, .. } => pos / mass,
            Node3D::Leaf { pos, .. } => *pos,
        }
    }

    pub fn mass(&self) -> f32 {
        match self {
            Node3D::Root { mass, .. } => *mass,
            Node3D::Leaf { mass, .. } => *mass,
        }
    }
}

#[derive(Clone, Debug, PartialEq)]
pub struct BoundingBox3D {
    pub center: Vec3,
    pub width: f32,
    pub height: f32,
    pub depth: f32,
}

impl BoundingBox3D {
    pub fn new(center: Vec3, width: f32, height: f32, depth: f32) -> Self {
        Self {
            center,
            width,
            depth,
            height,
        }
    }
    pub fn section(&self, loc: &Vec3) -> u8 {
        let mut section = 0x000;

        if loc[2] > self.center[2] {
            section |= 0b100;
        }

        if loc[1] > self.center[1] {
            section |= 0b010;
        }

        if loc[0] > self.center[0] {
            section |= 0b001;
        }

        section
    }

    pub fn sub_quadrant(&self, section: u8) -> Self {
        let mut shift = self.center;
        if section & 0b01 > 0 {
            shift[0] += 0.25 * self.width
        } else {
            shift[0] -= 0.25 * self.width;
        }

        if section & 0b10 > 0 {
            shift[1] += 0.25 * self.height;
        } else {
            shift[1] -= 0.25 * self.height;
        }

        if section & 0b100 > 0 {
            shift[2] += 0.25 * self.depth;
        } else {
            shift[2] -= 0.25 * self.depth;
        }
        Self {
            center: shift,
            width: self.width * 0.5,
            height: self.height * 0.5,
            depth: self.depth * 0.5,
        }
    }
}
