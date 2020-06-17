use std::sync::Mutex;

use rand::thread_rng;
use rayon::prelude::*;
use serde::Deserialize;

use crate::graphics::Color;
use crate::math::matrix::Matrix3;
use crate::math::vector::Vector3f;
use crate::math::{clamp, FloatT, Ray};
use crate::scene::{Camera, Render, Scene};
use crate::utils::Image;
use pbr::ProgressBar;

#[derive(Deserialize)]
pub struct PT {
    /// 采样数
    pub samples: usize,
    pub max_depth: usize,
}

impl Render for PT {
    fn render(&self, scene: &Scene, camera: &Camera) -> Image {
        rayon::ThreadPoolBuilder::new()
            .num_threads(16)
            .build()
            .unwrap()
            .install(|| {
                let image = Mutex::new(Image::empty(camera.w, camera.h));
                let progress_bar = Mutex::new(ProgressBar::new((camera.w * camera.h) as u64));
                let mut pixels = vec![];
                for i in 0..camera.w {
                    for j in 0..camera.h {
                        pixels.push((i, j));
                    }
                }
                pixels.into_par_iter().for_each(|(i, j)| {
                    let mut rng = thread_rng();
                    let rays = camera.gen(i, j, &mut rng);
                    let pixel = rays
                        .iter()
                        .map(|ray| {
                            let mut color = Color::empty();
                            for _ in 0..self.samples {
                                color += scene.path_tracing(
                                    ray.clone(),
                                    vec![scene.n],
                                    0,
                                    self.max_depth,
                                    &mut rng,
                                );
                            }
                            color /= self.samples as FloatT;
                            Vector3f::new([clamp(color[0]), clamp(color[1]), clamp(color[2])])
                        })
                        .sum::<Color>()
                        / rays.len() as FloatT;
                    image.lock().unwrap().set(i, j, pixel);
                    progress_bar.lock().unwrap().inc();
                });
                progress_bar.lock().unwrap().finish_println("done\n");
                image.into_inner().unwrap()
            })
    }
}
