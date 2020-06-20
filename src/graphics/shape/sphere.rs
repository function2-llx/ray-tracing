use serde::Deserialize;

use crate::graphics::material::{Material, Surface, Texture};
use crate::graphics::shape::{rand_semisphere, rand_sphere, RandOut};
use crate::graphics::{Hit, HitTemp, Hittable, Shape, TextureMap};
use crate::math::vector::{Vector2f, Vector3f};
use crate::math::{FloatT, Ray, sqr};
use rand::prelude::ThreadRng;

#[derive(Deserialize, Debug)]
pub struct Sphere {
    pub center: Vector3f,
    pub radius: FloatT,
}

impl RandOut for Sphere {
    fn rand_out(&self, rng: &mut ThreadRng) -> Ray {
        let normal = rand_sphere(rng);
        let pos = self.center + self.radius * normal;
        Ray::new(pos, rand_semisphere(&normal, rng))
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
                Some(HitTemp {
                    t,
                    normal: (ray.at(t) - self.center) / self.radius,
                    uv: None,
                })
            } else {
                t = (-b + d) / ray.direction.length();
                if t > t_min {
                    Some(HitTemp {
                        t,
                        normal: (ray.at(t) - self.center) / self.radius,
                        uv: None,
                    })
                } else {
                    None
                }
            }
        }
    }
}

impl TextureMap for Sphere {
    fn texture_map(
        &self,
        mut pos: Vector3f,
        uv: Option<(FloatT, FloatT)>,
        w: usize,
        h: usize,
    ) -> (usize, usize) {
        // pos /= self.radius;
        let pos = (pos - self.center) / self.radius;
        let theta = (pos.x() / (sqr(pos.x()) + sqr(pos.y())).sqrt()).acos();
        let phi = pos.z().acos();
        ((theta * w as FloatT) as usize % w, (phi * h as FloatT) as usize % h)
    }
}

impl Sphere {
    pub fn contains(&self, p: Vector3f) -> bool {
        (self.center - p).length2() <= self.radius * self.radius
    }
}
