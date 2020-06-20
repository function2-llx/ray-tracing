use serde::{Deserialize, Deserializer};

use crate::graphics::shape::{rand_sphere, RandOut};
use crate::graphics::{HitTemp, Hittable};
use crate::math::vector::{Vector2f, Vector3f};
use crate::math::{FloatT, Ray};
use rand::prelude::ThreadRng;
use rand::Rng;

#[derive(Debug)]
pub struct Plane {
    pub normal: Vector3f,
    pub d: FloatT,
    origin: Vector3f, // 等于 d * normal
    x: Vector3f,      // x, y: 根据 normal 建个系
    y: Vector3f,
}

impl Plane {
    pub fn new(normal: Vector3f, d: FloatT) -> Self {
        let origin = normal * d;
        let x = normal.get_orthogonal();
        let y = Vector3f::cross(&normal, &x);
        Plane {
            normal,
            d,
            origin,
            x,
            y,
        }
    }
}

impl RandOut for Plane {
    fn rand_out(&self, rng: &mut ThreadRng) -> Ray {
        Ray::new(
            self.origin
                + rng.gen_range(-1000.0, 1000.0) * self.x
                + rng.gen_range(-1000.0, 1000.0) * self.y,
            rand_sphere(rng),
        )
    }
}

impl<'de> Deserialize<'de> for Plane {
    fn deserialize<D>(deserializer: D) -> Result<Self, <D as Deserializer<'de>>::Error>
    where
        D: Deserializer<'de>,
    {
        #[derive(Deserialize)]
        struct PlaneInfo {
            normal: Vector3f,
            d: FloatT,
        }

        let info = PlaneInfo::deserialize(deserializer)?;
        Ok(Plane::new(info.normal, info.d))
    }
}

impl Hittable for Plane {
    fn hit(&self, ray: &Ray, t_min: FloatT) -> Option<HitTemp> {
        let t = -(-self.d + Vector3f::dot(&self.normal, &ray.origin))
            / Vector3f::dot(&self.normal, &ray.direction);
        if t > t_min {
            Some(HitTemp {
                t,
                normal: self.normal,
                uv: None,
            })
        } else {
            None
        }
    }
}

impl Plane {
    pub fn texture_map(
        &self,
        pos: Vector3f,
        uv: Option<(FloatT, FloatT)>,
        w: usize,
        h: usize,
    ) -> (usize, usize) {
        let w = w as isize;
        let h = h as isize;
        let pos = pos - self.origin;
        let x = (Vector3f::dot(&pos, &self.x) as isize % w + w * 3 / 2) % w;
        let y = (Vector3f::dot(&pos, &self.y) as isize % h + h * 3 / 2) % h;
        (x as usize, y as usize)
    }
}
