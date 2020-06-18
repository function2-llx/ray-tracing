use serde::Deserialize;

use crate::graphics::Color;
use crate::math::vector::Vector3f;
use crate::math::{FloatT, Ray};
use crate::utils::Image;

#[derive(Deserialize, Debug)]
pub enum Texture {
    Pure(Color),
    Image(Image),
}

// 表面光学特性
#[derive(Copy, Clone, Deserialize, Debug)]
pub enum Surface {
    Specular,           // 镜面
    Diffuse,            // 漫反射
    Refractive(FloatT), // 折射(折射率)
}

#[derive(Deserialize, Debug)]
pub struct Material {
    pub texture: Texture,
    pub surface: Surface,
}
