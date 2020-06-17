use std::fs;

use serde::Deserialize;
use serde_json::Value;

use crate::graphics::material::{Material, Surface};
use crate::graphics::{Color, Hittable};
use crate::graphics::{Hit, Object};
use crate::math::vector::Vector3f;
use crate::math::{FloatT, Ray};
use crate::utils::Image;

mod camera;
mod renderer;

pub use camera::*;
pub use renderer::*;

#[derive(Deserialize, Debug)]
pub struct Scene {
    objects: Vec<Object>,
    /// 环境光
    env: Vector3f,
    /// 环境折射率
    n: FloatT,
}

impl Scene {
    pub fn hit(&self, ray: &Ray, t_min: FloatT) -> Option<Hit> {
        if let Some((object, t, normal)) = self
            .objects
            .iter()
            .filter_map(|object| {
                if let Some((t, normal)) = object.hit(ray, t_min) {
                    Some((object, t, normal))
                } else {
                    None
                }
            })
            .min_by(|(_, t1, _), (_, t2, _)| t1.partial_cmp(t2).unwrap())
        {
            Some(object.make_hit(ray.at(t), normal))
        } else {
            None
        }
    }
}
