use crate::graphics::shape::{rand_semisphere, RandOut};
use crate::graphics::{HitTemp, Hittable};
use crate::math::vector::Vector3f;
use crate::math::{sqr, FloatT, Ray, PI};
use rand::prelude::ThreadRng;
use rand::Rng;
use serde::{Deserialize, Deserializer};

#[derive(Debug)]
pub struct Circle {
    origin: Vector3f,
    normal: Vector3f,
    radius: FloatT,
    x: Vector3f,
    y: Vector3f,
}

impl<'de> Deserialize<'de> for Circle {
    fn deserialize<D>(deserializer: D) -> Result<Self, <D as Deserializer<'de>>::Error>
    where
        D: Deserializer<'de>,
    {
        #[derive(Deserialize)]
        struct CircleInfo {
            pub origin: Vector3f,
            pub normal: Vector3f,
            pub radius: FloatT,
        }
        let info = CircleInfo::deserialize(deserializer).unwrap();
        Ok(Circle::new(info.origin, info.normal, info.radius))
    }
}

impl Circle {
    pub fn new(origin: Vector3f, normal: Vector3f, radius: FloatT) -> Self {
        let x = normal.get_orthogonal();
        let y = Vector3f::cross(&normal, &x);
        Self {
            origin,
            normal,
            radius,
            x,
            y,
        }
    }
}

impl Hittable for Circle {
    fn hit(&self, ray: &Ray, t_min: f64) -> Option<HitTemp> {
        let t = -(-Vector3f::dot(&self.origin, &self.normal)
            + Vector3f::dot(&self.normal, &ray.origin))
            / Vector3f::dot(&self.normal, &ray.direction);
        if t > t_min {
            let pos = ray.at(t);
            if (pos - self.origin).length2() <= sqr(self.radius) {
                Some(HitTemp {
                    t,
                    normal: self.normal,
                    uv: None,
                })
            } else {
                None
            }
        } else {
            None
        }
    }
}

impl RandOut for Circle {
    fn rand_out(&self, rng: &mut ThreadRng) -> Ray {
        let theta = rng.gen_range(0.0, 2.0 * PI);
        let r = rng.gen_range(0.0, self.radius);
        let pos = r * (self.x * theta.cos() + self.y * theta.sin()) + self.origin;
        Ray::new(pos, rand_semisphere(&self.normal, rng))
    }
}
