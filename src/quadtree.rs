use std::borrow::BorrowMut;

#[derive(Debug, PartialEq)]
pub enum QuadTree<T> {
    Leaf {
        data: T,
        loc: [f32; 2],
        mass: f32,
        boundary: Rectangle,
    },
    Root {
        children: [Box<QuadTree<T>>; 4],
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

    fn new_root(boundary: &Rectangle) -> Self {
        Self::Root {
            children: [
                Box::new(QuadTree::Empty),
                Box::new(QuadTree::Empty),
                Box::new(QuadTree::Empty),
                Box::new(QuadTree::Empty),
            ],
            mass_location: [0.0, 0.0],
            node_mass: 0.0,
            boundary: boundary.clone(),
        }
    }

    pub fn add_node(
        self: &mut Box<Self>,
        data_in: T,
        loc_in: &[f32; 2],
        mass_in: &f32,
        boundary_in: &Rectangle,
        depth: &mut u32,
    ) {
        match self.as_mut() {
            Self::Empty => {
                *self = Box::new(Self::Leaf {
                    data: data_in,
                    loc: *loc_in,
                    mass: *mass_in,
                    boundary: boundary_in.clone(),
                })
            }
            Self::Leaf {
                data,
                loc,
                mass,
                boundary,
            } => {
                let mut root = Box::new(Self::new_root(boundary));
                root.add_node(data.to_owned(), loc, mass, &boundary, depth);
                root.add_node(data_in.to_owned(), loc_in, mass_in, &boundary, depth);
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
                println!("{}", depth);
                *depth += 1;
                if *depth < 200 {
                    child.add_node(data_in, loc_in, mass_in, &small_boundary, depth);
                }
            }
        }
    }

    /*pub fn add_node_iter(
        &mut self,
        data_in: T,
        loc_in: &[f32; 2],
        mass_in: &f32,
        boundary_in: &Rectangle,
        depth: &mut u32,
    ) {
        let mut current_node = self;
        let mut second = None;
        loop {
            match current_node {
                QuadTree::Empty => {
                    *self = Self::Leaf {
                        data: data_in.clone(),
                        loc: *loc_in,
                        mass: *mass_in,
                        boundary: boundary_in.clone(),
                    };
                    return;
                }
                Self::Leaf {
                    data,
                    loc,
                    mass,
                    boundary,
                } => {
                    let mut root_node = QuadTree::new_root(&boundary);

                    if let QuadTree::Root {
                        children,
                        mass_location,
                        node_mass,
                        boundary,
                    } = root_node.borrow_mut()
                    {
                        let (small_boundary_prev, rect_prev) = boundary.get_section(&loc);
                        let (small_boundary, rect) = boundary.get_section(loc_in);
                        if rect_prev == rect {
                            second = Some((data.to_owned(), *loc, *mass, boundary_in.clone()));
                        } else {
                            mass_location[0] += loc[0] * *mass;
                            mass_location[1] += loc[1] * *mass;
                            *node_mass += *mass;

                            mass_location[0] += loc_in[0] * mass_in;
                            mass_location[1] += loc_in[1] * mass_in;
                            *node_mass += mass_in;

                            children[rect_prev as usize] = Box::new(Self::Leaf {
                                data: data.to_owned(),
                                loc: *loc,
                                mass: *mass,
                                boundary: small_boundary_prev,
                            });
                            children[rect as usize] = Box::new(Self::Leaf {
                                data: data_in.clone(),
                                loc: *loc_in,
                                mass: *mass_in,
                                boundary: small_boundary,
                            });
                        }
                    }
                    current_node = &mut root_node;
                }
                Self::Root {
                    children,
                    mass_location,
                    node_mass,
                    boundary,
                } => {
                    mass_location[0] += loc_in[0] * mass_in;
                    mass_location[1] += loc_in[1] * mass_in;
                    *node_mass += mass_in;

                    let (small_boundary, rect) = boundary.get_section(loc_in);
                    let mut out = current_node;

                    match *children[rect as usize] {
                        Self::Empty => {
                            children[rect as usize] = Box::new(Self::Leaf {
                                data: data_in.clone(),
                                loc: *loc_in,
                                mass: *mass_in,
                                boundary: small_boundary,
                            });
                        }
                        Self::Leaf {
                            data,
                            loc,
                            mass,
                            boundary,
                        } => {
                            current_node = &mut children[rect as usize];
                        }
                        QuadTree::Root {
                            children,
                            mass_location,
                            node_mass,
                            boundary,
                        } => {}
                    }
                }
            }
        }
    }*/

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
                if s / d > 0.75 {
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

#[derive(Clone, Debug, PartialEq)]
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

#[derive(Debug, PartialEq, Clone, Copy)]
enum RectangleSection {
    TL = 0,
    TR = 1,
    BL = 2,
    BR = 3,
}
