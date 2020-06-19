use std::sync::Mutex;

use pbr::ProgressBar;
use rand::prelude::*;
use rayon::prelude::*;
use serde::Deserialize;

use crate::graphics::material::Surface;
use crate::graphics::shape::{rand_semisphere, RandOut};
use crate::graphics::{Color, Hit};
use crate::math::matrix::Matrix3;
use crate::math::vector::Vector3f;
use crate::math::{sqr, FloatT, Ray, EPS, PI, ZERO};
use crate::scene::{renderer::get_n, Camera, Render, Scene};
use crate::utils::{kdtree, Image, Positionable};
use std::cmp::min;
use std::ptr::drop_in_place;
use std::time::Instant;

#[derive(Deserialize)]
pub struct PPM {
    /// 吸收率
    pa: FloatT,
    init_radius: FloatT,
    alpha: FloatT,
    photon_num: usize,
}

#[derive(Clone)]
struct ViewPoint {
    pub pos: Vector3f,
    /// 指向观察者的方向
    pub dir: Vector3f,
    pub pixel: (usize, usize),
    /// 光线从眼睛出发到此处（计算过 BRDF 后）的衰减系数
    pub weight: Color,
    /// 累计光子数
    pub n: usize,
    pub radius: FloatT,
    pub flux: Color,
}

impl ViewPoint {
    fn update(&mut self, photons: Vec<Photon>, alpha: FloatT) {
        let m = photons.len();
        if m == 0 {
            return;
        }
        let gain = alpha * m as FloatT;
        let ratio = (self.n as FloatT + gain as FloatT) / (self.n + m) as FloatT;
        self.n += gain as usize;
        self.radius *= ratio.sqrt();
        for photon in photons {
            self.flux += self.weight * photon.flux;
        }
        self.flux *= ratio;
    }
}

impl Positionable for ViewPoint {
    fn pos(&self) -> Vector3f {
        self.pos
    }
}

#[derive(Clone)]
struct Photon {
    pub pos: Vector3f,
    pub flux: Color,
    pub dir: Vector3f,
}

impl Positionable for Photon {
    fn pos(&self) -> Vector3f {
        self.pos
    }
}

impl PPM {
    // 返回值为所有直接光照
    fn ray_tracing(
        &self,
        scene: &Scene,
        ray: Ray,
        n_stack: Vec<FloatT>,
        pixel: (usize, usize),
        depth: usize,
        mut weight: Color,
        view_points: &mut Vec<ViewPoint>,
        rng: &mut ThreadRng,
    ) -> Color {
        // 俄罗斯赌轮，防止无限递归
        if depth > 4 {
            if rng.gen_range(0.0, 1.0) < self.pa {
                return scene.env;
            }
        }
        if let Some(Hit {
            pos,
            mut normal,
            uv,
            object,
        }) = scene.hit(&ray, EPS)
        {
            let color = object.color_at(pos, uv);
            weight *= color;
            object.flux
                + color
                    * match &object.material.surface {
                        Surface::Diffuse => {
                            if Vector3f::dot(&normal, &ray.direction) > 0.0 {
                                // 调整为反射平面的法向量
                                normal = -normal;
                            }
                            view_points.push(ViewPoint {
                                pos,
                                dir: -ray.direction,
                                pixel,
                                weight,
                                n: 0,
                                radius: self.init_radius,
                                flux: Color::empty(),
                            });
                            Color::empty()
                        }
                        Surface::Specular => {
                            // 此时一定在物体外侧，因为不可能进入反射的材质
                            self.ray_tracing(
                                scene,
                                Ray::new(
                                    pos,
                                    ray.direction
                                        - normal * 2.0 * Vector3f::dot(&normal, &ray.direction),
                                ),
                                n_stack.clone(),
                                pixel,
                                depth + 1,
                                weight,
                                view_points,
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
                                re * self.ray_tracing(
                                    scene,
                                    Ray::new(
                                        pos,
                                        ray.direction
                                            - normal * 2.0 * Vector3f::dot(&normal, &ray.direction),
                                    ),
                                    n_stack.clone(),
                                    pixel,
                                    depth + 1,
                                    weight * re,
                                    view_points,
                                    rng,
                                ) + tr * {
                                    let mut new_stack = n_stack.clone();
                                    if inside {
                                        new_stack.pop();
                                    } else {
                                        new_stack.push(nt);
                                    }
                                    self.ray_tracing(
                                        scene,
                                        Ray::new(pos, t),
                                        new_stack,
                                        pixel,
                                        depth + 1,
                                        weight * tr,
                                        view_points,
                                        rng,
                                    )
                                }
                            } else {
                                re * self.ray_tracing(
                                    scene,
                                    Ray::new(
                                        pos,
                                        ray.direction
                                            - normal * 2.0 * Vector3f::dot(&normal, &ray.direction),
                                    ),
                                    n_stack.clone(),
                                    pixel,
                                    depth + 1,
                                    weight * re,
                                    view_points,
                                    rng,
                                )
                            }
                        }
                    }
        } else {
            scene.env
        }
    }

    fn photon_tracing(
        &self,
        scene: &Scene,
        ray: Ray,
        n_stack: Vec<FloatT>,
        mut flux: Color,
        depth: usize,
        photons: &mut Vec<Photon>,
        rng: &mut ThreadRng,
    ) {
        if depth > 2 {
            if rng.gen_range(0.0, 1.0) < self.pa {
                return;
            }
        }
        if let Some(Hit {
            pos,
            mut normal,
            uv,
            object,
        }) = scene.hit(&ray, EPS)
        {
            match &object.material.surface {
                Surface::Diffuse => {
                    if Vector3f::dot(&normal, &ray.direction) > 0.0 {
                        // 调整为反射平面的法向量
                        normal = -normal;
                    }
                    photons.push(Photon {
                        pos,
                        flux,
                        dir: ray.direction,
                    });
                    self.photon_tracing(
                        scene,
                        Ray::new(pos, rand_semisphere(&normal, rng)),
                        n_stack.clone(),
                        flux * object.color_at(pos, uv),
                        depth + 1,
                        photons,
                        rng,
                    );
                }
                Surface::Specular => {
                    self.photon_tracing(
                        scene,
                        Ray::new(
                            pos,
                            ray.direction - normal * 2.0 * Vector3f::dot(&normal, &ray.direction),
                        ),
                        n_stack.clone(),
                        flux * object.color_at(pos, uv),
                        depth + 1,
                        photons,
                        rng,
                    );
                }
                Surface::Refractive(nt) => {
                    flux *= object.color_at(pos, uv);
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
                        self.photon_tracing(
                            scene,
                            Ray::new(
                                pos,
                                ray.direction
                                    - normal * 2.0 * Vector3f::dot(&normal, &ray.direction),
                            ),
                            n_stack.clone(),
                            flux * re,
                            depth + 1,
                            photons,
                            rng,
                        );
                        let mut new_stack = n_stack.clone();
                        if inside {
                            new_stack.pop();
                        } else {
                            new_stack.push(nt);
                        }
                        self.photon_tracing(
                            scene,
                            Ray::new(pos, t),
                            new_stack,
                            flux * tr,
                            depth + 1,
                            photons,
                            rng,
                        );
                    } else {
                        self.photon_tracing(
                            scene,
                            Ray::new(
                                pos,
                                ray.direction
                                    - normal * 2.0 * Vector3f::dot(&normal, &ray.direction),
                            ),
                            n_stack.clone(),
                            flux * re,
                            depth + 1,
                            photons,
                            rng,
                        );
                    }
                }
            }
        }
    }
}

impl Render for PPM {
    fn render(&self, scene: &Scene, camera: &Camera, name: &str) {
        let mut pixels = vec![];
        for i in 0..camera.w {
            for j in 0..camera.h {
                pixels.push((i, j));
            }
        }
        let view_points = Mutex::new(Vec::<ViewPoint>::new());
        println!("eye pass");
        // 处理光源的颜色
        let direct = Mutex::new(Image::empty(camera.w, camera.h));
        pixels.into_par_iter().for_each(|(i, j)| {
            let mut cur = Vec::new();
            let mut rng = thread_rng();
            let color = camera
                .gen(i, j, &mut rng)
                .iter()
                .map(|ray| {
                    self.ray_tracing(
                        scene,
                        *ray,
                        vec![scene.n],
                        (i, j),
                        0,
                        Color::new([1.0, 1.0, 1.0]),
                        &mut cur,
                        &mut rng,
                    )
                })
                .sum::<Color>()
                / camera.anti_alias as FloatT;
            direct.lock().unwrap().set(i, j, color);
            view_points.lock().unwrap().extend(cur);
        });
        println!("view points num: {}", view_points.lock().unwrap().len());
        let direct = direct.into_inner().unwrap();
        for iter in 1.. {
            let now = Instant::now();
            println!("iteration {}", iter);
            let energy = scene
                .objects
                .iter()
                .map(|object| object.flux.norm1())
                .collect::<Vec<_>>();
            let tot_energy = energy.iter().sum::<FloatT>();
            println!("energy: {:?}", energy);
            let photons = Mutex::new(Vec::<Photon>::new());
            println!("building photon map...");
            for (object, energy) in scene.objects.iter().zip(energy) {
                let photon_num = (self.photon_num as FloatT * energy / tot_energy + 0.5) as usize;
                let flux = object.flux / photon_num as FloatT;
                (0..photon_num)
                    .into_par_iter()
                    .chunks(100)
                    .for_each(|chunk| {
                        let chunk: Vec<usize> = chunk;
                        let mut rng = thread_rng();
                        let mut cur = Vec::new();
                        for _ in chunk.iter() {
                            let ray = object.rand_out(&mut rng);
                            // println!("ray: {:?}", ray);
                            self.photon_tracing(
                                scene,
                                ray,
                                vec![scene.n],
                                flux,
                                0,
                                &mut cur,
                                &mut rng,
                            );
                        }
                        photons.lock().unwrap().extend(cur);
                    });
            }
            let photons = photons.into_inner().unwrap();
            let photon_map = {
                let len = photons.len();
                let ret = kdtree::new(photons);
                println!("photon map size: {}", len);
                ret
            };

            println!("updating...");
            let image = Mutex::new(direct.clone());
            view_points
                .lock()
                .unwrap()
                .par_iter_mut()
                .for_each(|view_point| {
                    let view_point: &mut ViewPoint = view_point;
                    // let photons = photon_map.within(&view_point.pos, view_point.radius);
                    view_point.update(
                        photon_map.within(&view_point.pos, view_point.radius),
                        self.alpha,
                    );
                    image.lock().unwrap().add(
                        view_point.pixel.0,
                        view_point.pixel.1,
                        view_point.flux
                            / ((iter * camera.anti_alias) as FloatT * sqr(view_point.radius))
                                as FloatT,
                    );
                });

            image.lock().unwrap().dump(name, true);
            println!("image updated");
            println!("iteration elapsed: {}ms", now.elapsed().as_millis());
        }
    }
}
