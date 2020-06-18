use serde::Deserialize;

use crate::graphics::material::{Material, Surface, Texture};
use crate::graphics::shape::{rand_sphere, RandOut};
use crate::graphics::{Hit, HitTemp, Hittable, Shape, TextureMap};
use crate::math::vector::{Vector2f, Vector3f};
use crate::math::{FloatT, Ray};
use rand::prelude::ThreadRng;

#[derive(Deserialize, Debug)]
pub struct Sphere {
    pub center: Vector3f,
    pub radius: FloatT,
}

impl RandOut for Sphere {
    fn rand_out(&self, rng: &mut ThreadRng) -> Ray {
        let dir = rand_sphere(rng);
        Ray::new(self.center + self.radius * dir, dir)
    }
}

impl Hittable for Sphere {
    fn hit(&self, ray: &Ray, t_min: f64) -> Option<HitTemp> {
        let b = Vector3f::dot(&ray.direction.normalized(), &(ray.origin - self.center));
        let c = Vector3f::dot(&(ray.origin - self.center), &(ray.origin - self.center))
            - self.radius * self.radius;
        let mut d = b * b - c;
        if d < 0.0 {
            None
        } else {
            d = d.sqrt();
            let mut t = (-b - d) / ray.direction.length();
            if t > t_min {
                Some((t, (ray.at(t) - self.center) / self.radius))
            } else {
                t = (-b + d) / ray.direction.length();
                if t > t_min {
                    Some((t, (ray.at(t) - self.center) / self.radius))
                } else {
                    None
                }
            }
        }
    }
}

impl TextureMap for Sphere {
    fn texture_map(&self, pos: Vector3f, w: usize, h: usize) -> (usize, usize) {
        unimplemented!()
    }
}

impl Sphere {
    pub fn contains(&self, p: Vector3f) -> bool {
        (self.center - p).length2() <= self.radius * self.radius
    }
}
