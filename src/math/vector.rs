use std::fmt::{Display, Formatter};

use crate::math::FloatT;
use serde::Deserialize;
use std::iter::Sum;
use std::ops::{Add, AddAssign, Deref, DerefMut, Div, DivAssign, Mul, Neg, Sub};

#[derive(Copy, Clone, Deserialize)]
#[repr(C)]
pub struct Vector2f([FloatT; 2]);

impl Deref for Vector2f {
    type Target = [FloatT; 2];

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

#[derive(Copy, Clone, Deserialize, Debug)]
#[repr(C)]
pub struct Vector3f([FloatT; 3]);

impl Sum for Vector3f {
    fn sum<I: Iterator<Item = Vector3f>>(iter: I) -> Self {
        iter.fold(Vector3f::empty(), |sum, x| sum + x)
    }
}

impl Display for Vector3f {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        f.write_str(format!("({}, {}, {})", self[0], self[1], self[2]).as_str())
    }
}

impl Vector3f {
    pub fn new(x: [FloatT; 3]) -> Self {
        Vector3f(x)
    }
    pub fn empty() -> Self {
        Vector3f::full(0.0)
    }

    pub fn full(x: FloatT) -> Self {
        Vector3f::new([x, x, x])
    }

    pub fn dot(a: &Vector3f, b: &Vector3f) -> FloatT {
        a[0] * b[0] + a[1] * b[1] + a[2] * b[2]
    }
    pub fn cross(a: &Vector3f, b: &Vector3f) -> Vector3f {
        Vector3f::new([
            a[1] * b[2] - a[2] * b[1],
            a[2] * b[0] - a[0] * b[2],
            a[0] * b[1] - a[1] * b[0],
        ])
    }

    pub fn length2(&self) -> FloatT {
        Vector3f::dot(self, self)
    }

    pub fn length(&self) -> FloatT {
        self.length2().sqrt()
    }

    pub fn normalized(&self) -> Self {
        *self / self.length() as FloatT
    }

    // pub fn standardized(&self) -> Self {
    //     let p = self[0].max(self[1]).max(self[2]);
    //     self / p
    // }

    /// 随便返回一个和自己正交的向量
    pub fn get_orthogonal(&self) -> Self {
        if self[0].abs() < self[1].abs().min(self[2].abs()) {
            Self::new([0.0, -self[2], self[1]])
        } else if self[1].abs() < self[2].abs() {
            Self::new([-self[2], 0.0, self[0]])
        } else {
            Self::new([-self[1], self[0], 0.0])
        }
    }
}

impl Deref for Vector3f {
    type Target = [FloatT; 3];

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl DerefMut for Vector3f {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}

impl Add for Vector3f {
    type Output = Vector3f;

    fn add(self, rhs: Vector3f) -> Self::Output {
        Vector3f::new([self[0] + rhs[0], self[1] + rhs[1], self[2] + rhs[2]])
    }
}

impl AddAssign for Vector3f {
    fn add_assign(&mut self, rhs: Self) {
        self[0] += rhs[0];
        self[1] += rhs[1];
        self[2] += rhs[2];
    }
}

impl Sub for Vector3f {
    type Output = Vector3f;

    fn sub(self, rhs: Self) -> Self::Output {
        Vector3f::new([self[0] - rhs[0], self[1] - rhs[1], self[2] - rhs[2]])
    }
}

impl Mul for Vector3f {
    type Output = Vector3f;

    fn mul(self, rhs: Self) -> Self::Output {
        Vector3f::new([self[0] * rhs[0], self[1] * rhs[1], self[2] * rhs[2]])
    }
}

impl Mul<FloatT> for Vector3f {
    type Output = Vector3f;

    fn mul(self, rhs: f64) -> Self::Output {
        self * Vector3f::full(rhs)
    }
}

impl Mul<Vector3f> for FloatT {
    type Output = Vector3f;

    fn mul(self, rhs: Vector3f) -> Self::Output {
        Vector3f::full(self) * rhs
    }
}

impl Div for Vector3f {
    type Output = Vector3f;

    fn div(self, rhs: Self) -> Self::Output {
        Vector3f::new([self[0] / rhs[0], self[1] / rhs[1], self[2] / rhs[2]])
    }
}

impl DivAssign<FloatT> for Vector3f {
    fn div_assign(&mut self, rhs: FloatT) {
        self[0] /= rhs;
        self[1] /= rhs;
        self[2] /= rhs;
    }
}

impl Div<FloatT> for Vector3f {
    type Output = Vector3f;

    fn div(self, rhs: FloatT) -> Self::Output {
        self / Vector3f::full(rhs)
    }
}

impl Neg for Vector3f {
    type Output = Self;
    fn neg(self) -> Self {
        Self::new([-self[0], -self[1], -self[2]])
    }
}
