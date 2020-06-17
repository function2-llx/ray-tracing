use serde::Deserialize;

use crate::math::vector::Vector3f;
use std::ops::Deref;

pub mod matrix;
pub mod vector;

pub type FloatT = f64;
pub const EPS: FloatT = 1e-8;
pub const ZERO: FloatT = 0.0;
pub const PI: FloatT = std::f64::consts::PI;
pub const INF: FloatT = 1e30;

pub fn sqr(x: FloatT) -> FloatT {
    x * x
}

#[derive(Copy, Clone, Deserialize)]
pub struct Ray {
    pub origin: Vector3f,
    pub direction: Vector3f,
}

impl Ray {
    pub fn new(origin: Vector3f, direct: Vector3f) -> Self {
        Self {
            origin,
            direction: direct,
        }
    }
    pub fn at(&self, t: FloatT) -> Vector3f {
        self.origin + t * self.direction
    }
}

pub fn clamp(x: FloatT) -> FloatT {
    if x < 0.0 {
        0.0
    } else if x > 1.0 {
        1.0
    } else {
        x
    }
}
