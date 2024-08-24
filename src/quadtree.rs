const EPSILON: f32 = 1e-3;
#[derive(Debug)]
pub struct QuadTree<'a, T> {
    pub data: Option<&'a T>,
    pub children: Vec<Option<Self>>,
    pub boundary: BoundingBox2D,
    pub mass: f32,
    position: [f32; 2],
}

impl<'a, T> QuadTree<'a, T> {
    pub fn new(boundary: BoundingBox2D) -> Self {
        Self {
            data: None,
            children: vec![None, None, None, None],
            boundary: boundary,
            mass: 0.0,
            position: [0.0, 0.0],
        }
    }

    fn new_leaf(
        data: Option<&'a T>,
        position: [f32; 2],
        mass: f32,
        boundary: BoundingBox2D,
    ) -> Self {
        Self {
            data,
            children: vec![None, None, None, None],
            boundary,
            mass,
            position,
        }
    }

    pub fn get_position(&self) -> [f32; 2] {
        [self.position[0] / self.mass, self.position[1] / self.mass]
    }

    pub fn insert(&mut self, data: Option<&'a T>, position: [f32; 2], mass: f32) {
        let mut parent: &mut Self = self;

        if parent.mass == 0.0 {
            parent.mass = mass;
            parent.position = [position[0] * mass, position[1] * mass];
            parent.data = data;
            return;
        }

        // Search the lowest parent
        while !parent.is_leaf() {
            let quadrant = parent.boundary.get_section(&position);
            if parent.children[quadrant as usize].is_none() {
                break;
            }
            parent.update_mass(&position, &mass);
            parent = parent.children[quadrant as usize].as_mut().unwrap();
        }

        let mut quadrant = parent.boundary.get_section(&position);
        let mut new_bb = parent.boundary.get_sub_quadrant(quadrant);
        if parent.is_leaf() {
            let leaf_position = parent.position;
            let leaf_mass = parent.mass;
            let leaf_data = parent.data;
            let l_pos = [leaf_position[0] / leaf_mass, leaf_position[1] / leaf_mass];

            //Update the mass of the parent
            parent.update_mass(&position, &mass);

            let mut leaf_quadrant = parent.boundary.get_section(&l_pos);
            let mut leaf_new_bb = parent.boundary.get_sub_quadrant(leaf_quadrant);

            while quadrant == leaf_quadrant {
                // If child is too close, treat it as one
                if (leaf_position[0] - position[0]).abs() < EPSILON
                    && (leaf_position[1] - position[1]).abs() < EPSILON
                {
                    return;
                }

                parent.children[leaf_quadrant as usize] = Some(QuadTree::new_leaf(
                    None,
                    leaf_position,
                    leaf_mass,
                    leaf_new_bb,
                ));
                parent = parent.children[leaf_quadrant as usize].as_mut().unwrap();

                quadrant = parent.boundary.get_section(&position);
                new_bb = parent.boundary.get_sub_quadrant(quadrant);

                leaf_quadrant = parent.boundary.get_section(&l_pos);
                leaf_new_bb = parent.boundary.get_sub_quadrant(leaf_quadrant);
            }

            parent.children[leaf_quadrant as usize] = Some(Self::new_leaf(
                leaf_data,
                leaf_position,
                leaf_mass,
                leaf_new_bb,
            ));
        }
        parent.children[quadrant as usize] = Some(Self::new_leaf(
            data,
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

    pub fn get_stack(&'a self, position: &[f32; 2], theta: f32) -> Vec<&'a Self> {
        let mut nodes: Vec<&QuadTree<T>> = vec![];
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

    pub fn get_closest(&'a self, position: &[f32; 2]) -> &'a Self {
        let mut parent: &Self = self;
        let mut quadrant = parent.boundary.get_section(position);
        while parent.children[quadrant as usize].is_some() {
            parent = parent.children[quadrant as usize].as_ref().unwrap();
            quadrant = parent.boundary.get_section(position);
        }
        parent
    }
}

#[derive(Clone, Debug)]
pub struct BoundingBox2D {
    pub center: [f32; 2],
    pub width: f32,
    pub height: f32,
}

impl BoundingBox2D {
    pub fn new(center: [f32; 2], width: f32, height: f32) -> Self {
        Self {
            center,
            width,
            height,
        }
    }
    fn get_section(&self, loc: &[f32; 2]) -> u8 {
        let mut section = 0x00;

        if loc[1] > self.center[1] {
            section |= 0b10;
        }

        if loc[0] > self.center[0] {
            section |= 0b01;
        }

        section
    }

    pub fn get_sub_quadrant(&self, section: u8) -> Self {
        let mut shifted_x = self.center[0];
        let mut shifted_y = self.center[1];
        if section & 0b01 > 0 {
            shifted_x += 0.25 * self.width;
        } else {
            shifted_x -= 0.25 * self.width;
        }

        if section & 0b10 > 0 {
            shifted_y += 0.25 * self.height;
        } else {
            shifted_y -= 0.25 * self.height;
        }
        Self {
            center: [shifted_x, shifted_y],
            width: self.width * 0.5,
            height: self.height * 0.5,
        }
    }
}
