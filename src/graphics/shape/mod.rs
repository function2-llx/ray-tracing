use serde::Deserialize;

use crate::graphics::{Hit, HitTemp, Hittable, TextureMap};
use crate::math::vector::Vector3f;
use crate::math::{FloatT, Ray};
pub use plane::*;
pub use sphere::*;

mod plane;
mod sphere;

#[derive(Deserialize, Debug)]
pub enum Shape {
    Sphere(Sphere),
    Plane(Plane),
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
