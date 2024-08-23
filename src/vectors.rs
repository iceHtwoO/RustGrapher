use core::f64;
use std::ops::{Add, AddAssign, Index, IndexMut, Mul, MulAssign, Sub, SubAssign};

pub type Vector2D = Vector<f32, 2>;
pub type Vector3D = Vector<f32, 3>;

#[derive(Clone, Copy, Debug)]
pub struct Vector<T, const N: usize> {
    position: [T; N],
}

impl<T, const N: usize> Vector<T, N>
where
    T: Copy
        + Add<Output = T>
        + Sub<Output = T>
        + PartialOrd
        + Into<f64>
        + MulAssign
        + Mul
        + Mul<Output = T>
        + AddAssign<T>,
{
    pub fn new(position: [T; N]) -> Self {
        Self { position }
    }

    pub fn get_position(&self) -> &[T; N] {
        &self.position
    }

    pub fn distance(&self, other: &Self) -> f64 {
        let mut sum = 0.0;
        for i in 0..N {
            let diff = (self.position[i] - other.position[i]).into();
            sum += diff * diff;
        }
        sum.sqrt()
    }

    pub fn scalar(&mut self, num: T) {
        for i in 0..N {
            self.position[i] *= num;
        }
    }

    pub fn len(&self) -> f64 {
        let mut sum: f64 = 0.0;
        for i in 0..N {
            sum += (self.position[i] * self.position[i]).into();
        }
        sum.sqrt()
    }
}

impl<T, const N: usize> Index<usize> for Vector<T, N> {
    type Output = T;

    fn index(&self, index: usize) -> &Self::Output {
        &self.position[index]
    }
}

impl<T, const N: usize> IndexMut<usize> for Vector<T, N>
where
    T: Copy + Add<Output = T> + Sub<Output = T> + PartialOrd + Into<f64>,
{
    fn index_mut(&mut self, index: usize) -> &mut Self::Output {
        &mut self.position[index]
    }
}

impl<T, const N: usize> Sub for Vector<T, N>
where
    T: SubAssign + Clone + Copy,
{
    type Output = Self;

    fn sub(mut self, rhs: Self) -> Self::Output {
        for i in 0..N {
            self.position[i] -= rhs.position[i];
        }
        self
    }
}

impl<T, const N: usize> SubAssign for Vector<T, N>
where
    T: SubAssign + Clone + Copy,
{
    fn sub_assign(&mut self, rhs: Self) {
        for i in 0..N {
            self.position[i] -= rhs.position[i];
        }
    }
}

impl<T, const N: usize> Add for Vector<T, N>
where
    T: AddAssign + Clone + Copy,
{
    type Output = Self;

    fn add(mut self, rhs: Self) -> Self::Output {
        for i in 0..N {
            self.position[i] += rhs.position[i];
        }
        self
    }
}

impl<T, const N: usize> AddAssign for Vector<T, N>
where
    T: AddAssign + Clone + Copy,
{
    fn add_assign(&mut self, rhs: Self) {
        for i in 0..N {
            self.position[i] += rhs.position[i];
        }
    }
}

impl<T, const N: usize> Mul for Vector<T, N>
where
    T: MulAssign + Clone + Copy,
{
    type Output = Self;

    fn mul(mut self, rhs: Self) -> Self::Output {
        for i in 0..N {
            self.position[i] *= rhs.position[i];
        }
        self
    }
}

impl<T, const N: usize> MulAssign for Vector<T, N>
where
    T: MulAssign + Clone + Copy,
{
    fn mul_assign(&mut self, rhs: Self) {
        for i in 0..N {
            self.position[i] *= rhs.position[i];
        }
    }
}
