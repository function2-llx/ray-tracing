use std::sync::Mutex;

use rand::{thread_rng, Rng};
use rayon::prelude::*;
use serde::Deserialize;

use crate::graphics::material::Surface;
use crate::graphics::shape::rand_semisphere;
use crate::graphics::{Color, Hit};
use crate::math::matrix::Matrix3;
use crate::math::vector::Vector3f;
use crate::math::{sqr, FloatT, Ray, EPS, PI, ZERO};
use crate::scene::{renderer::get_n, Camera, Render, Scene};
use crate::utils::Image;
use image::math::utils::clamp;
use pbr::ProgressBar;
use rand::prelude::ThreadRng;

#[derive(Deserialize)]
pub struct PT {
    /// 采样数
    pub samples: usize,
    pub max_depth: usize,
}

impl PT {
    // n_stack: 折射率栈
    pub fn path_tracing(
        &self,
        scene: &Scene,
        ray: Ray,
        n_stack: Vec<FloatT>,
        depth: usize,
        rng: &mut ThreadRng,
    ) -> Color {
        if let Some(Hit {
            pos,
            mut normal,
            uv,
            object,
        }) = scene.hit(&ray, EPS)
        {
            let illumination = || {
                if depth == self.max_depth {
                    return scene.env;
                }
                match &object.material.surface {
                    Surface::Diffuse => {
                        if Vector3f::dot(&normal, &ray.direction) > 0.0 {
                            // 调整为反射平面的法向量
                            normal = -normal;
                        }
                        let dir = rand_semisphere(&normal, rng);
                        assert!(Vector3f::dot(&normal, &dir) >= 0.0);
                        self.path_tracing(scene, Ray::new(pos, dir), n_stack, depth + 1, rng)
                    }
                    Surface::Specular => {
                        // 此时一定在物体外侧，因为不可能进入反射的材质
                        self.path_tracing(
                            scene,
                            Ray::new(
                                pos,
                                ray.direction
                                    - normal * 2.0 * Vector3f::dot(&normal, &ray.direction),
                            ),
                            n_stack.clone(),
                            depth + 1,
                            rng,
                        )
                    }
                    Surface::Refractive(nt) => {
                        let inside = if Vector3f::dot(&normal, &ray.direction) > 0.0 {
                            // (-normal, true)
                            normal = -normal;
                            true
                        } else {
                            false
                        };

                        // 当前折射率和即将进入的介质的折射率
                        let (n, nt) = get_n(inside, &n_stack, nt);

                        let (re, tr) = self.refractive(&ray.direction, &normal, n, nt);
                        if let Some((tr, t)) = tr {
                            assert!(Vector3f::dot(&t, &normal) <= 0.0);
                            re * self.path_tracing(
                                scene,
                                Ray::new(
                                    pos,
                                    ray.direction
                                        - normal * 2.0 * Vector3f::dot(&normal, &ray.direction),
                                ),
                                n_stack.clone(),
                                depth + 1,
                                rng,
                            ) + tr * {
                                let mut new_stack = n_stack.clone();
                                if inside {
                                    new_stack.pop();
                                } else {
                                    new_stack.push(nt);
                                }
                                self.path_tracing(
                                    scene,
                                    Ray::new(pos, t),
                                    new_stack,
                                    depth + 1,
                                    rng,
                                )
                            }
                        } else {
                            // 全反射
                            re * self.path_tracing(
                                scene,
                                Ray::new(
                                    pos,
                                    ray.direction
                                        - normal * 2.0 * Vector3f::dot(&normal, &ray.direction),
                                ),
                                n_stack.clone(),
                                depth + 1,
                                rng,
                            )
                        }
                    }
                }
            };
            object.flux + object.color_at(pos, uv) * illumination()
        } else {
            scene.env
        }
    }
}

impl Render for PT {
    fn render(&self, scene: &Scene, camera: &Camera, name: &str) {
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
                        // 光源和相机只能处在环境中 T_T
                        color += self.path_tracing(scene, ray.clone(), vec![scene.n], 0, &mut rng);
                    }
                    color /= self.samples as FloatT;
                    Vector3f::new([
                        clamp(color[0], 0.0, 1.0),
                        clamp(color[1], 0.0, 1.0),
                        clamp(color[2], 0.0, 1.0),
                    ])
                })
                .sum::<Color>()
                / rays.len() as FloatT;
            image.lock().unwrap().set(i, j, pixel);
            progress_bar.lock().unwrap().inc();
        });
        progress_bar.lock().unwrap().finish_println("done\n");
        image.lock().unwrap().dump(name, true);
    }
}
