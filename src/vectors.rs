use core::f32;
use std::ops::{Add, AddAssign, Div, DivAssign, Index, IndexMut, Mul, MulAssign, Sub, SubAssign};

pub type Vector2D = Vector<2>;
pub type Vector3D = Vector<3>;

#[derive(Clone, Copy, Debug)]
pub struct Vector<const N: usize> {
    position: [f32; N],
}

impl<const N: usize> Vector<N>
where
    f32: Copy
        + Add<Output = f32>
        + Sub<Output = f32>
        + PartialOrd
        + Into<f32>
        + MulAssign
        + Mul
        + Mul<Output = f32>
        + AddAssign<f32>,
{
    pub fn new(position: [f32; N]) -> Self {
        Self { position }
    }

    pub fn set(&mut self, position: [f32; N]) {
        self.position = position;
    }

    pub fn get_position_ref(&self) -> &[f32; N] {
        &self.position
    }

    pub fn get_position(&self) -> [f32; N] {
        self.position
    }

    pub fn distance(&self, other: &Self) -> f32 {
        let mut sum: f32 = 0.0;
        for i in 0..N {
            let diff = self.position[i] - other.position[i];
            sum += diff * diff;
        }
        sum.sqrt()
    }

    pub fn scalar(&mut self, num: f32) {
        for i in 0..N {
            self.position[i] *= num;
        }
    }

    pub fn dot(&self, other: &Self) -> f32 {
        let mut sum: f32 = 0.0;
        for i in 0..N {
            sum += self.position[i] * other.position[i];
        }
        sum
    }

    pub fn len(&self) -> f32 {
        let mut sum: f32 = 0.0;
        for i in 0..N {
            sum += self.position[i] * self.position[i];
        }
        sum.sqrt()
    }

    pub fn normalize(mut self) -> Self {
        let len = self.len();
        if len == 0.0 {
            self.scalar(0.0);
        } else {
            self.scalar(1.0 / len);
        }

        self
    }
}

impl<const N: usize> Index<usize> for Vector<N> {
    type Output = f32;

    fn index(&self, index: usize) -> &Self::Output {
        &self.position[index]
    }
}

impl<const N: usize> IndexMut<usize> for Vector<N>
where
    f32: Copy + Add<Output = f32> + Sub<Output = f32> + PartialOrd + Into<f32>,
{
    fn index_mut(&mut self, index: usize) -> &mut Self::Output {
        &mut self.position[index]
    }
}

impl<const N: usize> Sub for Vector<N>
where
    f32: SubAssign + Clone + Copy,
{
    type Output = Self;

    fn sub(mut self, rhs: Self) -> Self::Output {
        for i in 0..N {
            self.position[i] -= rhs.position[i];
        }
        self
    }
}

impl<const N: usize> SubAssign for Vector<N>
where
    f32: SubAssign + Clone + Copy,
{
    fn sub_assign(&mut self, rhs: Self) {
        for i in 0..N {
            self.position[i] -= rhs.position[i];
        }
    }
}

impl<const N: usize> Add for Vector<N>
where
    f32: AddAssign + Clone + Copy,
{
    type Output = Self;

    fn add(mut self, rhs: Self) -> Self::Output {
        for i in 0..N {
            self.position[i] += rhs.position[i];
        }
        self
    }
}

impl<const N: usize> AddAssign for Vector<N>
where
    f32: AddAssign + Clone + Copy,
{
    fn add_assign(&mut self, rhs: Self) {
        for i in 0..N {
            self.position[i] += rhs.position[i];
        }
    }
}

impl<const N: usize> Mul for Vector<N>
where
    f32: MulAssign + Clone + Copy,
{
    type Output = Self;

    fn mul(mut self, rhs: Self) -> Self::Output {
        for i in 0..N {
            self.position[i] *= rhs.position[i];
        }
        self
    }
}

impl<const N: usize> MulAssign for Vector<N>
where
    f32: MulAssign + Clone + Copy,
{
    fn mul_assign(&mut self, rhs: Self) {
        for i in 0..N {
            self.position[i] *= rhs.position[i];
        }
    }
}

impl<const N: usize> Div for Vector<N>
where
    f32: DivAssign + Clone + Copy,
{
    type Output = Self;

    fn div(mut self, rhs: Self) -> Self::Output {
        for i in 0..N {
            self.position[i] /= rhs.position[i];
        }
        self
    }
}

impl<const N: usize> DivAssign for Vector<N>
where
    f32: DivAssign + Clone + Copy,
{
    fn div_assign(&mut self, rhs: Self) {
        for i in 0..N {
            self.position[i] /= rhs.position[i];
        }
    }
}

impl Vector2D {
    pub fn to_3d(self) -> Vector3D {
        Vector3D::new([self.position[0], self.position[1], 0.0])
    }

    pub fn empty() -> Vector2D {
        Self {
            position: [0.0, 0.0],
        }
    }
}

impl Vector3D {
    pub fn cross(&self, b_vec: &Vector3D) -> Vector3D {
        let a = self.position;
        let b = b_vec.position;
        Self {
            position: [
                a[1] * b[2] - a[2] * b[1],
                a[2] * b[0] - a[0] * b[2],
                a[0] * b[1] - a[1] * b[0],
            ],
        }
    }

    pub fn empty() -> Vector3D {
        Self {
            position: [0.0, 0.0, 0.0],
        }
    }
}
