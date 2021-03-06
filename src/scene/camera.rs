use std::sync::Mutex;

use pbr::ProgressBar;
use rand::{thread_rng, Rng};
use serde::{Deserialize, Deserializer};

use crate::math::matrix::Matrix3;
use crate::math::vector::Vector3f;
use crate::math::{FloatT, Ray, EPS, PI};
use rand::prelude::ThreadRng;

pub struct Camera {
    pub center: Vector3f,
    pub direction: Vector3f,  // z 轴
    pub up: Vector3f,         // y 轴
    pub horizontal: Vector3f, // x 轴
    pub rotate: Matrix3,
    /// 相机中心到成像平面的距离
    pub dis: FloatT,
    pub cx: FloatT,
    pub cy: FloatT,
    pub w: usize,
    pub h: usize,
    pub anti_alias: usize,
    pub focal: Option<FloatT>,
    pub r: FloatT,  // 镜头半径
}

impl<'de> Deserialize<'de> for Camera {
    fn deserialize<D>(deserializer: D) -> Result<Self, <D as Deserializer<'de>>::Error>
    where
        D: Deserializer<'de>,
    {
        #[derive(Deserialize)]
        struct CameraInfo {
            pub center: Vector3f,
            pub direction: Vector3f, // z 轴
            pub up: Vector3f,        // y 轴
            /// 相机中心到成像平面的距离
            pub dis: FloatT,
            pub w: usize,
            pub h: usize,
            pub anti_alias: usize,
            focal: Option<FloatT>,
            r: FloatT,  // aperture
        }

        let info = CameraInfo::deserialize(deserializer)?;
        let direction = info.direction.normalized();
        let up = info.up.normalized();
        let horizontal = Vector3f::cross(&direction, &up);
        assert!(
            Vector3f::dot(&direction, &up).abs() < EPS,
            "up and direction in camera must be orthogonal"
        );
        Ok(Camera {
            center: info.center,
            direction,
            up,
            horizontal,
            rotate: Matrix3::from_vectors([horizontal, up, direction], true),
            dis: info.dis,
            cx: info.w as FloatT / 2.0,
            cy: info.h as FloatT / 2.0,
            w: info.w,
            h: info.h,
            anti_alias: info.anti_alias,
            focal: info.focal,
            r: info.r,
        })
    }
}

impl Camera {
    // 在同一个像素内随机产生若干条光线
    pub fn gen(&self, x: usize, y: usize, rng: &mut ThreadRng) -> Vec<Ray> {
        let mut rays = vec![];
        for _ in 0..self.anti_alias {
            let x = x as FloatT + rng.gen_range(0.0, 1.0) - self.w as FloatT / 2.0;
            let y = y as FloatT + rng.gen_range(0.0, 1.0) - self.h as FloatT / 2.0;
            let dir = Vector3f::new([x, y, self.dis]).normalized();
            // 如果self.focal 不是 None 则为透镜，z 轴为主光轴；否则为小孔成像
            rays.push(if let Some(f) = self.focal {
                let f= self.center + f / dir.z() * dir; // 手动算汇聚点
                let r = rng.gen_range(0.0, self.r);
                let theta = rng.gen_range(0.0, 2.0 * PI);
                let center = self.center + r * Vector3f::new([theta.cos(), theta.sin(), 0.0]);
                Ray::new(
                    center,
                    self.rotate * (f - center).normalized()
                )
            } else {
                Ray::new(self.center, self.rotate * dir)
            });
        }
        rays
    }
}
