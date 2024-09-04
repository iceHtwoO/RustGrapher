use core::panic;

use glam::Vec2;

const EPSILON: f32 = 1e-3;

/// Implementation of a quadtree for the barnes-hut algorithm.
/// An area gets split up into 4 sections and each can contain a leaf or another quadtree
/// This can be used to approximate far away nodes to reduce calculations.
#[derive(Debug)]
pub struct QuadTree<'a, T> {
    pub data: Option<&'a T>,
    pub children: Vec<Option<Self>>,
    pub boundary: BoundingBox2D,
    mass: f32,
    position: Vec2,
}

impl<'a, T> QuadTree<'a, T> {
    /// Creates a empty `QuadTree` with it's initial `BoundingBox2D`
    pub fn new(boundary: BoundingBox2D) -> Self {
        Self {
            data: None,
            children: vec![None, None, None, None],
            boundary,
            mass: 0.0,
            position: Vec2::ZERO,
        }
    }

    /// Returns the position of the node.
    /// If its an approximation its the average based on `mass`
    pub fn position(&self) -> Vec2 {
        self.position / self.mass
    }

    /// Returns the mass of the node
    pub fn mass(&self) -> f32 {
        self.mass
    }

    /// Inserts a node into the Quadtree and places it according to its relative position in the initial boundingBox
    pub fn insert(&mut self, data: Option<&'a T>, position: Vec2, mass: f32) {
        let mut parent: &mut Self = self;

        if mass == 0.0 {
            panic!("Mass in QuadTree may not be 0");
        }

        if parent.mass == 0.0 {
            parent.mass = mass;
            parent.position = position * mass;
            parent.data = data;
            return;
        }
        // Search the lowest parent
        while !parent.is_leaf() {
            let quadrant = parent.boundary.section(&position);
            if parent.children[quadrant as usize].is_none() {
                break;
            }
            parent.update_mass(&position, &mass);
            parent = parent.children[quadrant as usize].as_mut().unwrap();
        }

        let mut quadrant = parent.boundary.section(&position);
        let mut new_bb = parent.boundary.sub_quadrant(quadrant);

        // If the lowest member is a Leaf we create a new leaf and move the data down
        if parent.is_leaf() {
            let leaf_position = parent.position;
            let leaf_mass = parent.mass;
            let leaf_data = parent.data;
            parent.data = None;
            let l_pos = leaf_position / leaf_mass;

            //Update the mass of the parent
            parent.update_mass(&position, &mass);

            // If child is too close, treat it as one
            if position.distance(leaf_position / leaf_mass) < EPSILON {
                return;
            }

            let mut leaf_quadrant = parent.boundary.section(&l_pos);
            let mut leaf_new_bb = parent.boundary.sub_quadrant(leaf_quadrant);

            while quadrant == leaf_quadrant {
                // Create a new Quadrant and set it to parent
                parent.children[leaf_quadrant as usize] = Some(QuadTree::new_leaf(
                    None,
                    leaf_position,
                    leaf_mass,
                    leaf_new_bb,
                ));
                parent = parent.children[leaf_quadrant as usize].as_mut().unwrap();

                // Recalculate the position in the quadrant where the new and old data wil be placed.
                quadrant = parent.boundary.section(&position);
                new_bb = parent.boundary.sub_quadrant(quadrant);

                leaf_quadrant = parent.boundary.section(&l_pos);
                leaf_new_bb = parent.boundary.sub_quadrant(leaf_quadrant);
            }

            parent.children[leaf_quadrant as usize] = Some(Self::new_leaf(
                leaf_data,
                leaf_position,
                leaf_mass,
                leaf_new_bb,
            ));
        }
        parent.children[quadrant as usize] =
            Some(Self::new_leaf(data, position * mass, mass, new_bb));
    }

    /// Returns a Vector filled with `QuadTree` according to the barnes-hut algorithm
    /// Far away nodes get approximated
    /// Higher `theta` values result in more approximations.
    /// If `theta` is 0, all nodes are returned without summarizing.
    pub fn stack(&'a self, position: &Vec2, theta: f32) -> Vec<&'a Self> {
        let mut nodes: Vec<&QuadTree<T>> = vec![];
        let mut stack = vec![self];
        while !stack.is_empty() {
            let parent = match stack.pop() {
                Some(p) => p,
                None => break,
            };
            let s = parent.boundary.width.max(parent.boundary.height);
            let center_mass = parent.position / parent.mass;
            let dist = center_mass.distance(*position);

            // We check if dist ist bigger than EPSILON, so we don't add interactions with itself!
            if (s / dist < theta || parent.is_leaf()) && dist > EPSILON {
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

    /// Returns the node that is closest to given position
    pub fn closest(&'a self, position: &Vec2) -> &'a Self {
        let mut parent: &Self = self;
        let mut quadrant = parent.boundary.section(position);
        while parent.children[quadrant as usize].is_some() {
            parent = parent.children[quadrant as usize].as_ref().unwrap();
            quadrant = parent.boundary.section(position);
        }
        parent
    }

    fn update_mass(&mut self, position: &Vec2, mass: &f32) {
        self.position += position * mass;
        self.mass += mass;
    }

    fn is_leaf(&self) -> bool {
        for child in self.children.iter() {
            if child.is_some() {
                return false;
            }
        }
        true
    }

    fn new_leaf(data: Option<&'a T>, position: Vec2, mass: f32, boundary: BoundingBox2D) -> Self {
        Self {
            data,
            children: vec![None, None, None, None],
            boundary,
            mass,
            position,
        }
    }
}

#[derive(Clone, Debug, PartialEq)]
pub struct BoundingBox2D {
    pub center: Vec2,
    pub width: f32,
    pub height: f32,
}

impl BoundingBox2D {
    pub fn new(center: Vec2, width: f32, height: f32) -> Self {
        Self {
            center,
            width,
            height,
        }
    }
    pub fn section(&self, loc: &Vec2) -> u8 {
        let mut section = 0x00;

        if loc[1] > self.center[1] {
            section |= 0b10;
        }

        if loc[0] > self.center[0] {
            section |= 0b01;
        }

        section
    }

    pub fn sub_quadrant(&self, section: u8) -> Self {
        let mut shift = self.center;
        if section & 0b01 > 0 {
            shift[0] += 0.25 * self.width;
        } else {
            shift[0] -= 0.25 * self.width;
        }

        if section & 0b10 > 0 {
            shift[1] += 0.25 * self.height;
        } else {
            shift[1] -= 0.25 * self.height;
        }
        Self {
            center: shift,
            width: self.width * 0.5,
            height: self.height * 0.5,
        }
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn test_bounding_box_section() {
        let bb: BoundingBox2D = BoundingBox2D::new(Vec2::ZERO, 10.0, 10.0);
        assert_eq!(bb.section(&Vec2::new(-1.0, -1.0)), 0);
        assert_eq!(bb.section(&Vec2::new(1.0, -1.0)), 1);
        assert_eq!(bb.section(&Vec2::new(-1.0, 1.0)), 2);
        assert_eq!(bb.section(&Vec2::new(1.0, 1.0)), 3);
    }

    #[test]
    fn test_bounding_box_sub_quadrant() {
        let bb: BoundingBox2D = BoundingBox2D::new(Vec2::ZERO, 10.0, 10.0);
        assert_eq!(
            bb.sub_quadrant(0),
            BoundingBox2D::new(Vec2::new(-2.5, -2.5), 5.0, 5.0)
        );
        assert_eq!(
            bb.sub_quadrant(1),
            BoundingBox2D::new(Vec2::new(2.5, -2.5), 5.0, 5.0)
        );
        assert_eq!(
            bb.sub_quadrant(2),
            BoundingBox2D::new(Vec2::new(-2.5, 2.5), 5.0, 5.0)
        );
        assert_eq!(
            bb.sub_quadrant(3),
            BoundingBox2D::new(Vec2::new(2.5, 2.5), 5.0, 5.0)
        );
    }

    #[test]
    fn test_quadtree_insert() {
        let mut qt: QuadTree<u32> = QuadTree::new(BoundingBox2D::new(Vec2::ZERO, 10.0, 10.0));
        // Insert first node
        let mass = 5.0;
        let d = 0;
        qt.insert(Some(&d), Vec2::new(-1.0, -1.0), mass);
        assert_eq!(qt.mass, mass);
        assert_eq!(qt.data, Some(&d));

        // Insert second node in in the same quadrant but different sub quadrant
        //  N1-R-N2
        let mass1 = 30.0;
        let d1 = 1;
        qt.insert(Some(&d1), Vec2::new(1.0, 1.0), mass1);
        // check root node
        assert!(qt.data.is_none());
        assert_eq!(qt.mass, mass1 + mass);

        // check node0
        assert!(qt.children[0].is_some());
        let node = qt.children[0].as_ref().unwrap();
        assert!(node.data.is_some());
        assert_eq!(node.data, Some(&d));
        assert_eq!(node.mass, mass);

        // check node1
        assert!(qt.children[3].is_some());
        let node = qt.children[3].as_ref().unwrap();
        assert!(node.data.is_some());
        assert_eq!(node.data, Some(&d1));
        assert_eq!(node.mass, mass1);

        let mass2 = 60.0;
        let d2 = 2;
        // Insert on same position
        qt.insert(Some(&d2), Vec2::new(1.0, 1.0), mass2);
        // Node0 should be unchanged
        assert!(qt.children[0].is_some());
        let node = qt.children[0].as_ref().unwrap();
        assert!(node.data.is_some());
        assert_eq!(node.data, Some(&d));
        assert_eq!(node.mass, mass);

        // Mass should have been updated of node1
        assert!(qt.children[3].is_some());
        let node = qt.children[3].as_ref().unwrap();
        assert!(node.data.is_none());
        assert_eq!(node.mass, mass1 + mass2);
    }
}
