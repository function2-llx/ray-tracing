use serde::Deserialize;

use crate::graphics::{Hit, HitTemp};
use crate::math::vector::{Vector2f, Vector3f};
use crate::math::{FloatT, Ray};

#[derive(Deserialize, Debug)]
pub struct Plane {
    pub normal: Vector3f,
    pub d: FloatT,
}

impl Plane {
    pub fn hit(&self, ray: &Ray, t_min: f64) -> Option<HitTemp> {
        let t = -(-self.d + Vector3f::dot(&self.normal, &ray.origin)) / Vector3f::dot(&self.normal, &ray.direction);
        if t > t_min {
            Some((t, self.normal))
        } else {
            None
        }
    }
    pub fn texture_map(&self, pos: Vector3f, w: usize, h: usize) -> (usize, usize) {
        unimplemented!()
    }
}
