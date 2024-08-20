#[derive(Debug)]
pub enum QuadTree<T> {
    Leaf {
        data: T,
        loc: [f32; 2],
        mass: f32,
        boundary: Rectangle,
    },
    Root {
        children: Vec<Box<QuadTree<T>>>,
        mass_location: [f32; 2],
        node_mass: f32,
        boundary: Rectangle,
    },
    Empty,
}

impl<T> QuadTree<T>
where
    T: Clone,
{
    pub fn new() -> Self {
        Self::Empty
    }

    fn new_root(loc: &[f32; 2], mass: &f32, boundary: Rectangle) -> Self {
        Self::Root {
            children: vec![
                Box::new(QuadTree::Empty),
                Box::new(QuadTree::Empty),
                Box::new(QuadTree::Empty),
                Box::new(QuadTree::Empty),
            ],
            mass_location: loc.clone(),
            node_mass: mass.clone(),
            boundary,
        }
    }

    pub fn add_node(
        &mut self,
        data_in: T,
        loc_in: [f32; 2],
        mass_in: f32,
        boundary_in: &Rectangle,
    ) {
        match self {
            Self::Empty => {
                *self = Self::Leaf {
                    data: data_in,
                    loc: loc_in,
                    mass: mass_in,
                    boundary: boundary_in.clone(),
                }
            }
            Self::Leaf {
                data,
                loc,
                mass,
                boundary,
            } => {
                let mut root = Self::new_root(loc, mass, boundary.clone());
                root.add_node(data_in, loc_in, mass_in, boundary);
                root.add_node(data.to_owned(), loc.to_owned(), mass.to_owned(), boundary);
                *self = root;
            }
            Self::Root {
                children,
                mass_location,
                node_mass,
                boundary,
            } => {
                let (small_boundary, rect) = boundary.get_section(loc_in);
                let child = &mut children[rect as usize];
                mass_location[0] += loc_in[0] * mass_in;
                mass_location[1] += loc_in[1] * mass_in;
                *node_mass += mass_in;
                child.add_node(data_in, loc_in, mass_in, &small_boundary);
            }
        }
    }

    pub fn get_mass(&self, loc: &[f32; 2]) -> Vec<([f32; 2], f32)> {
        match self {
            Self::Root {
                children,
                mass_location,
                node_mass,
                boundary,
            } => {
                let s = boundary.width.max(boundary.height);
                let center_mass = [mass_location[0] / node_mass, mass_location[1] / node_mass];
                let d = f32::sqrt(
                    (center_mass[0] - loc[0]).powi(2) + (center_mass[1] - loc[1]).powi(2),
                );

                let mut particles = Vec::new();
                if s / d > 0.5 {
                    // θ = 0.5
                    for child in children {
                        particles.append(&mut child.get_mass(loc));
                    }
                } else {
                    return vec![(
                        [mass_location[0] / node_mass, mass_location[1] / node_mass],
                        node_mass.clone(),
                    )];
                }
                particles
            }
            Self::Leaf {
                data,
                loc,
                mass,
                boundary,
            } => {
                vec![(loc.clone(), mass.clone())]
            }
            _ => vec![],
        }
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
    fn get_section(&self, loc: [f32; 2]) -> (Rectangle, RectangleSection) {
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

#[derive(Debug)]
enum RectangleSection {
    TL = 0,
    TR = 1,
    BL = 2,
    BR = 3,
}
