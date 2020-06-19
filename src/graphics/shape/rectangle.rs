use crate::math::{FloatT, Ray};
use crate::math::vector::Vector3f;
use crate::graphics::{Hittable, HitTemp};

struct Rectangle {
    a: FloatT,
    b: FloatT,
    normal: Vector3f,
}

impl Hittable for Rectangle {
    fn hit(&self, ray: &Ray, t_min: f64) -> Option<HitTemp> {
        unimplemented!()
    }
}