use crate::graphics::shape::{rand_semisphere, Plane, RandOut};
use crate::graphics::{HitTemp, Hittable, TextureMap};
use crate::math::vector::Vector3f;
use crate::math::{FloatT, Ray};
use rand::prelude::ThreadRng;
use rand::Rng;
use serde::{Deserialize, Deserializer};
use serde_json::map::Entry::Vacant;
use std::path::Prefix::Verbatim;

#[derive(Debug)]
pub struct Rectangle {
    w: FloatT,
    h: FloatT,
    origin: Vector3f,
    normal: Vector3f,
    x: Vector3f, // x 轴
    y: Vector3f,
}

impl<'de> Deserialize<'de> for Rectangle {
    fn deserialize<D>(deserializer: D) -> Result<Self, <D as Deserializer<'de>>::Error>
    where
        D: Deserializer<'de>,
    {
        #[derive(Deserialize)]
        struct RectangleInfo {
            w: FloatT,
            h: FloatT,
            origin: Vector3f,
            normal: Vector3f,
            x: Vector3f, // x 轴
        }
        let info = RectangleInfo::deserialize(deserializer).unwrap();
        Ok(Rectangle::new(
            info.w,
            info.h,
            info.origin,
            info.normal,
            info.x,
        ))
    }
}

impl Rectangle {
    pub fn new(w: FloatT, h: FloatT, origin: Vector3f, normal: Vector3f, x: Vector3f) -> Rectangle {
        Self {
            w,
            h,
            origin,
            normal,
            x,
            y: Vector3f::cross(&normal, &x),
        }
    }
}

impl Hittable for Rectangle {
    fn hit(&self, ray: &Ray, t_min: f64) -> Option<HitTemp> {
        if let Some(HitTemp { t, normal, uv }) =
            Plane::new(self.normal, Vector3f::dot(&self.origin, &self.normal)).hit(ray, t_min)
        {
            let pos = ray.at(t);
            if Vector3f::dot(&(pos - self.origin), &self.x).abs() * 2.0 <= self.w
                && Vector3f::dot(&(pos - self.origin), &self.y).abs() * 2.0 <= self.h
            {
                Some(HitTemp { t, normal, uv })
            } else {
                None
            }
        } else {
            None
        }
    }
}

impl RandOut for Rectangle {
    fn rand_out(&self, rng: &mut ThreadRng) -> Ray {
        let pos = self.origin
            + (self.x * rng.gen_range(-self.w / 2.0, self.w / 2.0)
                + self.y * rng.gen_range(-self.h / 2.0, self.h / 2.0));
        Ray::new(pos, rand_semisphere(&self.normal, rng))
    }
}

impl TextureMap for Rectangle {
    fn texture_map(
        &self,
        pos: Vector3f,
        uv: Option<(f64, f64)>,
        w: usize,
        h: usize,
    ) -> (usize, usize) {
        let x = Vector3f::dot(&pos, &self.x) / self.w as FloatT + 0.5;
        let y = Vector3f::dot(&pos, &self.y) / self.h as FloatT + 0.5;

        ((x * w as FloatT) as usize, (y * h as FloatT) as usize)
    }
}
