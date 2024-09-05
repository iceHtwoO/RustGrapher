use std::borrow::BorrowMut;

use glam::Vec2;

use crate::quadtree::BoundingBox2D;

const EPSILON: f32 = 1e-3;

#[derive(Debug)]
pub struct QuadTreeVec {
    pub children: Vec<Node>,
    pub boundary: BoundingBox2D,
    pub root: u32,
}

#[derive(Debug)]
pub enum Node {
    Root {
        indices: [u32; 4],
        mass: f32,
        pos: Vec2,
    },
    Leaf {
        mass: f32,
        pos: Vec2,
    },
}

impl QuadTreeVec {
    pub fn new(boundary: BoundingBox2D) -> Self {
        Self {
            root: 0,
            boundary,
            children: Vec::new(),
        }
    }
    pub fn insert(&mut self, new_pos: Vec2, new_mass: f32) {
        self.children.push(Node::new_leaf(new_pos, new_mass));
        let new_index = self.children.len() as u32 - 1;

        // When only one node than there is no need to continue
        if self.children.len() == 1 {
            self.children.push(Node::new_leaf(new_pos, new_mass));
            return;
        }

        let mut bb = self.boundary.clone();
        let mut root_index = self.root;

        while let Node::Root { indices, mass, pos } =
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
        if let Node::Leaf { mass, pos } = self.children[root_index as usize] {
            if pos.distance(new_pos) < EPSILON {
                let m: f32 = mass + new_mass;
                self.children[root_index as usize] = Node::new_leaf(pos, m);
                return;
            }
        }

        // create new root until leaf and new leaf are in different sections
        while let Node::Leaf { mass, pos } = self.children[root_index as usize] {
            let mut fin = false;

            // Pushes the old leaf to the back of the vector and inserts its index into the index array of the new root
            let old_node = Node::new_leaf(pos, mass);
            self.children.push(old_node);
            let old_index = self.children.len() - 1;
            let section = bb.section(&pos);
            let mut ind = [u32::MAX, u32::MAX, u32::MAX, u32::MAX];
            ind[section as usize] = old_index as u32;

            let section = bb.section(&new_pos);

            // If section of the new root is empty we can set it and exit
            if ind[section as usize] == u32::MAX {
                ind[section as usize] = new_index;
                fin = true;
            }

            // sets the old leaf index to the new root
            let new_root = Node::new_root(pos + new_pos, mass + new_mass, ind);
            self.children[root_index as usize] = new_root;

            if fin {
                return;
            }

            root_index = old_index as u32;

            bb = bb.sub_quadrant(section);
        }
    }

    pub fn stack<'a>(&'a self, position: &Vec2, theta: f32) -> Vec<&'a Node> {
        let mut nodes: Vec<&Node> = vec![];

        let mut s: f32 = self.boundary.width.max(self.boundary.height);

        if self.children.is_empty() {
            return vec![];
        }

        let mut stack: Vec<u32> = vec![0];
        let mut new_stack: Vec<u32> = vec![];
        'outer: loop {
            while !stack.is_empty() {
                let parent = &self.children[stack.pop().unwrap() as usize];

                if let Node::Root { indices, mass, pos } = parent {
                    let center_mass = parent.get_center();
                    let dist = center_mass.distance(*position);
                    if s / dist < theta {
                        nodes.push(parent);
                    } else {
                        for i in indices {
                            if *i != u32::MAX {
                                new_stack.push(*i);
                            }
                        }
                    }
                }

                if let Node::Leaf { mass, pos } = parent {
                    nodes.push(parent);
                }
            }
            if new_stack.is_empty() {
                break 'outer;
            }
            s *= 0.5;
            stack = new_stack;
            new_stack = vec![];
        }
        nodes
    }
}

impl Node {
    fn new_leaf(pos: Vec2, mass: f32) -> Self {
        Self::Leaf { mass, pos }
    }
    fn new_root(pos: Vec2, mass: f32, indices: [u32; 4]) -> Self {
        Self::Root {
            indices,
            mass,
            pos: pos * mass,
        }
    }
    #[allow(dead_code)]
    fn is_leaf(&self) -> bool {
        matches!(self, Node::Leaf { .. })
    }

    #[allow(dead_code)]
    fn is_root(&self) -> bool {
        matches!(self, Node::Root { .. })
    }

    fn get_center(&self) -> Vec2 {
        match self {
            Node::Root { pos, mass, .. } => pos / mass,
            Node::Leaf { pos, .. } => *pos,
        }
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn test_quadtree_insert() {
        let mut qt: QuadTreeVec = QuadTreeVec::new(BoundingBox2D::new(Vec2::ZERO, 10.0, 10.0));
        // Insert first node
        let n1_mass = 5.0;
        qt.insert(Vec2::new(-1.0, -1.0), n1_mass);
        assert!(qt.children[0].is_leaf());
        if let Node::Leaf { mass, .. } = qt.children[0] {
            assert_eq!(mass, n1_mass);
        }

        // Insert second node in in the same quadrant but different sub quadrant
        //  N1-R-N2
        let n2_mass = 30.0;
        qt.insert(Vec2::new(1.0, 1.0), n2_mass);
        // check root node
        assert!(qt.children[0].is_root());
        if let Node::Root { indices, mass, .. } = qt.children[0] {
            assert_eq!(mass, n1_mass + n2_mass);

            // check node0
            assert_eq!(indices[0], 2);
            assert!(qt.children[1].is_leaf());

            // check node1
            assert_eq!(indices[3], 1);
            assert!(qt.children[2].is_leaf());
        }
    }
}
