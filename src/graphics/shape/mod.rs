use serde::Deserialize;

use crate::graphics::{Hit, HitTemp, Hittable, TextureMap};
use crate::math::matrix::Matrix3;
use crate::math::vector::Vector3f;
use crate::math::{FloatT, Ray, PI, ZERO};
pub use plane::*;
use rand::prelude::ThreadRng;
use rand::Rng;
pub use sphere::*;

mod plane;
mod sphere;

#[derive(Deserialize, Debug)]
pub enum Shape {
    Sphere(Sphere),
    Plane(Plane),
}

impl RandOut for Shape {
    fn rand_out(&self, rng: &mut ThreadRng) -> Ray {
        use Shape::*;
        match self {
            Sphere(sphere) => sphere.rand_out(rng),
            Plane(plane) => plane.rand_out(rng),
        }
    }
}

pub fn rand_sphere(rng: &mut ThreadRng) -> Vector3f {
    let theta = rng.gen_range(0.0, 2.0 * PI);
    let phi = rng.gen_range(0.0, PI);
    let sin_phi = phi.sin();
    Vector3f::new([sin_phi * theta.cos(), sin_phi * theta.sin(), phi.cos()])
}

// z: normal
pub fn rand_semisphere(z: &Vector3f, rng: &mut ThreadRng) -> Vector3f {
    // 以 normal 为 z 轴随便建个单位正交坐标系
    let x = z.get_orthogonal();
    let y = Vector3f::cross(z, &x);

    // 在半球面上选一个点
    let theta = rng.gen_range(ZERO, 2.0 * PI);
    let phi = rng.gen_range(ZERO, PI / 2.0);
    let sin_phi = phi.sin();
    Matrix3::from_vectors([x, y, *z], true)
        * Vector3f::new([sin_phi * theta.cos(), sin_phi * theta.sin(), phi.cos()])
}

pub trait RandOut {
    fn rand_out(&self, rng: &mut ThreadRng) -> Ray;
}

impl Hittable for Shape {
    fn hit(&self, r: &Ray, t_min: FloatT) -> Option<HitTemp> {
        use Shape::*;
        match self {
            Sphere(sphere) => sphere.hit(r, t_min),
            Plane(plane) => plane.hit(r, t_min),
        }
    }
}

impl TextureMap for Shape {
    fn texture_map(&self, pos: Vector3f, w: usize, h: usize) -> (usize, usize) {
        use Shape::*;
        match self {
            Sphere(sphere) => sphere.texture_map(pos, w, h),
            Plane(plane) => plane.texture_map(pos, w, h),
        }
    }
}
