use std::sync::Mutex;

use serde::Deserialize;
use pbr::ProgressBar;
use rand::{SeedableRng, thread_rng, Rng};

use crate::graphics::Color;
use crate::math::matrix::Matrix3;
use crate::math::vector::Vector3f;
use crate::math::{FloatT, Ray, EPS};
use crate::scene::Scene;
use crate::utils::Image;
use rayon::prelude::*;

#[derive(Deserialize)]
pub struct Camera {
    pub center: Vector3f,
    pub direction: Vector3f, // z 轴
    pub up: Vector3f,        // y 轴
    /// 相机中心到成像平面的距离
    pub dis: FloatT,
    /// 采样数
    pub samples: usize,
    pub w: usize,
    pub h: usize,
}

impl Camera {
    pub fn adjust(&mut self) {
        if Vector3f::dot(&self.direction, &self.up).abs() > EPS {
            panic!("up and direction in camera should be orthogonality");
        }
        self.direction = self.direction.normalized();
        self.up = self.up.normalized();
        // 保证是 4 的倍数
        self.samples = self.samples / 4 * 4;
    }

    pub fn shoot(&self, scene: &Scene, max_depth: usize) -> Image {
        rayon::ThreadPoolBuilder::new().num_threads(16).build().unwrap().install(|| {
            let (cx, cy) = (self.w as FloatT / 2.0, self.h as FloatT / 2.0);
            // x = y * z
            let horizontal = Vector3f::cross(&self.up, &self.direction);
            let rotate = Matrix3::from_vectors([horizontal, self.up, self.direction], true);
            let mut image = Mutex::new(Image::empty(self.w, self.h));
            let progress_bar = Mutex::new(ProgressBar::new((self.w * self.h) as u64));
            let mut pixels = vec![];
            for i in 0..self.w {
                for j in 0..self.h {
                    pixels.push((i, j));
                }
            }
            pixels.into_par_iter().for_each(|(i, j)| {
                let mut rng = thread_rng();
                let mut pixel = Color::empty();
                for sx in 0..2 {
                    for sy in 0..2 {
                        let ray = Ray::new(
                            self.center,
                            rotate
                                * Vector3f::new([
                                    i as FloatT - cx + 0.25 + 0.5 * sx as FloatT,
                                    j as FloatT - cy + 0.25 + 0.5 * sy as FloatT,
                                    self.dis,
                                ])
                                .normalized(),
                        );
                        for _ in 0..self.samples / 4 {
                            // 小孔成像
                            pixel +=
                                scene.path_tracing(ray, &mut vec![scene.n], 0, max_depth, &mut rng);
                        }
                    }
                }
                image.lock().unwrap().set(i, j, pixel / self.samples as FloatT);
                progress_bar.lock().unwrap().inc();
            });
            progress_bar.lock().unwrap().finish_println("done\n");
            image.into_inner().unwrap()
        })
    }
}
