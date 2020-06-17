use serde::{Deserialize, Deserializer};

use crate::graphics::{Hit, HitTemp};
use crate::math::vector::{Vector2f, Vector3f};
use crate::math::{FloatT, Ray};
use crate::utils::modf;

#[derive(Debug)]
pub struct Plane {
    pub normal: Vector3f,
    pub d: FloatT,
    origin: Vector3f, // 等于 d * normal
    x: Vector3f,      // x, y: 根据 normal 建个系
    y: Vector3f,
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
        let origin = info.normal * info.d;
        let x = info.normal.get_orthogonal();
        let y = Vector3f::cross(&info.normal, &x);
        Ok(Plane {
            normal: info.normal,
            d: info.d,
            origin,
            x,
            y,
        })
    }
}

impl Plane {
    pub fn hit(&self, ray: &Ray, t_min: f64) -> Option<HitTemp> {
        let t = -(-self.d + Vector3f::dot(&self.normal, &ray.origin))
            / Vector3f::dot(&self.normal, &ray.direction);
        if t > t_min {
            Some((t, self.normal))
        } else {
            None
        }
    }
    pub fn texture_map(&self, pos: Vector3f, w: usize, h: usize) -> (usize, usize) {
        let w = w as isize;
        let h = h as isize;
        let pos = pos - self.origin;
        let x = (Vector3f::dot(&pos, &self.x) as isize % w + w * 3 / 2) % w;
        let y = (Vector3f::dot(&pos, &self.y) as isize % h + h * 3 / 2) % h;
        (x as usize, y as usize)
    }
}
