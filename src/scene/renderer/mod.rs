use serde::Deserialize;

use crate::scene::{Camera, Scene};
use crate::utils::Image;

mod ppm;
mod pt;

use crate::math::vector::Vector3f;
use crate::math::{sqr, FloatT, Ray, EPS};
pub use ppm::*;
pub use pt::*;
use rand::prelude::*;

pub trait Render {
    fn render(&self, scene: &Scene, camera: &Camera, path: &str);
    fn refractive(
        &self,
        i: &Vector3f,
        normal: &Vector3f,
        n: FloatT,
        nt: FloatT,
    ) -> (FloatT, Option<(FloatT, Vector3f)>) {
        let nt2 = nt * nt;
        let dn = Vector3f::dot(i, normal);
        let delta = { nt2 - n * n * (1.0 - dn * dn) };

        // 全反射
        if delta <= 0.0 {
            (1.0, None)
        } else {
            // 折射光方向
            let t = (n * (*i - *normal * dn) / nt - *normal * (delta / nt2).sqrt()).normalized();
            assert!(Vector3f::dot(&t, normal) <= 0.0);

            // 计算反射光强占比
            let r = {
                let r0 = sqr((nt - n) / (nt + n));
                let c = if n <= nt {
                    Vector3f::dot(i, normal).abs()
                } else {
                    Vector3f::dot(&t, &normal).abs()
                };
                r0 + (1.0 - r0) * (1.0 - c).powi(5)
            };
            (r, Some((1.0 - r, t)))
        }
    }
}

#[derive(Deserialize)]
pub enum Renderer {
    PT(PT),
    PPM(PPM),
}

impl Renderer {
    pub fn render(&self, scene: &Scene, camera: &Camera, path: &str) {
        use Renderer::*;
        match self {
            PT(pt) => pt.render(scene, camera, path),
            PPM(ppm) => ppm.render(scene, camera, path),
        }
    }
}

// 返回当前介质和下一个介质的折射率
fn get_n(inside: bool, n_stack: &Vec<FloatT>, nt: &FloatT) -> (FloatT, FloatT) {
    if inside {
        let len = n_stack.len();
        if len < 2 {
            println!("yabaidesune!");
            let a = 1.0;
            (a, a + 0.1)
        } else {
            (n_stack[len - 1], n_stack[len - 2])
        }
    } else {
        (*n_stack.last().unwrap(), *nt)
    }
}
