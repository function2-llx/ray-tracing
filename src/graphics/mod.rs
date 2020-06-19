use serde::Deserialize;

use crate::graphics::material::{Material, Texture};
use crate::graphics::shape::{RandOut, Shape};
use crate::math::vector::{Vector2f, Vector3f};
use crate::math::{FloatT, Ray};
use rand::prelude::ThreadRng;

mod bounding;
pub mod material;
pub mod shape;

pub use bounding::*;

pub type Color = Vector3f;

pub struct HitTemp {
    pub t: FloatT,
    pub normal: Vector3f,
    // 用于纹理映射
    pub uv: Option<(FloatT, FloatT)>,
}

pub struct Hit<'a> {
    pub pos: Vector3f,
    pub normal: Vector3f,
    pub uv: Option<(FloatT, FloatT)>,
    pub object: &'a Object,
}

// 统一接口
pub trait Hittable {
    fn hit(&self, ray: &Ray, t_min: FloatT) -> Option<HitTemp>;
}

trait TextureMap {
    /// map shape to w * h rectangle
    fn texture_map(
        &self,
        pos: Vector3f,
        uv: Option<(FloatT, FloatT)>,
        w: usize,
        h: usize,
    ) -> (usize, usize);
}

#[derive(Deserialize, Debug)]
pub struct Object {
    shape: Shape,
    pub material: Material,
    /// 物体自身发光
    pub flux: Color,
}

impl RandOut for Object {
    fn rand_out(&self, rng: &mut ThreadRng) -> Ray {
        self.shape.rand_out(rng)
    }
}

impl Hittable for Object {
    fn hit(&self, r: &Ray, t_min: f64) -> Option<HitTemp> {
        self.shape.hit(r, t_min)
    }
}

impl Object {
    pub fn make_hit(&self, pos: Vector3f, normal: Vector3f, uv: Option<(FloatT, FloatT)>) -> Hit {
        Hit {
            pos,
            normal,
            uv,
            object: self,
        }
    }

    pub fn color_at(&self, pos: Vector3f, uv: Option<(FloatT, FloatT)>) -> Color {
        match &self.material.texture {
            Texture::Pure(color) => *color,
            Texture::Image(image) => {
                let (x, y) = self.shape.texture_map(pos, uv, image.w, image.h);
                image.at(x, y)
            }
        }
    }
}
