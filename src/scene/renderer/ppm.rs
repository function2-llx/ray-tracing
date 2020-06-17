use std::sync::Mutex;

use pbr::ProgressBar;
use rand::prelude::*;
use rayon::prelude::*;
use serde::Deserialize;

use crate::graphics::material::Surface;
use crate::graphics::Hit;
use crate::math::matrix::Matrix3;
use crate::math::vector::Vector3f;
use crate::math::{sqr, FloatT, Ray, EPS, PI, ZERO};
use crate::scene::{Camera, Render, Scene};
use crate::utils::{Image, Positionable};

#[derive(Deserialize)]
pub struct PPM {
    /// 吸收率
    pub pa: FloatT,
}

struct ViewPoint {
    pub pos: Vector3f,
    /// 指向观察者的方向
    pub dir: Vector3f,
    pub pixel: (usize, usize),
}

impl Positionable for ViewPoint {
    fn pos(&self) -> Vector3f {
        self.pos
    }
}

impl PPM {
    // 尾递归
    fn ray_tracing(
        &self,
        scene: &Scene,
        ray: Ray,
        n_stack: &mut Vec<FloatT>,
        pixel: (usize, usize),
        rng: &mut ThreadRng,
    ) {
        // 俄罗斯赌轮，防止无限递归
        if rng.gen_range(0.0, 0.1) < self.pa {
            return;
        }
        if let Some(Hit {
            pos,
            mut normal,
            object,
        }) = scene.hit(&ray, EPS)
        {
            match &object.material.optics {
                Surface::Diffuse => {
                    if Vector3f::dot(&normal, &ray.direction) > 0.0 {
                        // 调整为反射平面的法向量
                        normal = -normal;
                    }

                    // // 以 normal 为 z 轴随便建个单位正交坐标系
                    // let x = normal.get_orthogonal();
                    // let y = Vector3f::cross(&normal, &x);
                    // // 在半球面上选一个点
                    // let theta = rng.gen_range(ZERO, 2.0 * PI);
                    // let phi = rng.gen_range(ZERO, PI / 2.0);
                    // let dir = Matrix3::from_vectors([x, y, normal], true)
                    //     * Vector3f::new([
                    //         phi.cos() * theta.cos(),
                    //         phi.cos() * theta.sin(),
                    //         phi.sin(),
                    //     ]);
                    // assert!(Vector3f::dot(&normal, &dir) >= 0.0);
                    // self.ray_tracing(scene, Ray::new(pos, dir), n_stack, pixel, rng)
                }
                Surface::Specular => {
                    // 此时一定在物体外侧，因为不可能进入反射的材质
                    self.ray_tracing(
                        scene,
                        Ray::new(
                            pos,
                            ray.direction - normal * 2.0 * Vector3f::dot(&normal, &ray.direction),
                        ),
                        n_stack,
                        pixel,
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
                    let (n, nt) = if inside {
                        let len = n_stack.len();
                        if len < 2 {
                            println!("abnormal");
                            let a = 1.0;
                            (a, a + 0.1)
                        } else {
                            (n_stack[len - 1], n_stack[len - 2])
                        }
                    } else {
                        (*n_stack.last().unwrap(), *nt)
                    };

                    let nt2 = nt * nt;
                    let dn = Vector3f::dot(&ray.direction, &normal);
                    let delta = { nt2 - n * n * (1.0 - dn * dn) };

                    // 全反射
                    if delta <= 0.0 {
                        return self.ray_tracing(
                            scene,
                            Ray::new(
                                pos,
                                ray.direction
                                    - normal * 2.0 * Vector3f::dot(&normal, &ray.direction),
                            ),
                            n_stack,
                            pixel,
                            rng,
                        );
                    }

                    // 折射光方向
                    let t = (n * (ray.direction - normal * dn) / nt
                        - normal * (delta / nt2).sqrt())
                    .normalized();
                    assert!(Vector3f::dot(&t, &normal) <= 0.0);

                    // 计算反射光强占比
                    let r = {
                        let r0 = sqr((nt - n) / (nt + n));
                        let c = if n <= nt {
                            Vector3f::dot(&ray.direction, &normal).abs()
                        } else {
                            Vector3f::dot(&t, &normal).abs()
                        };
                        r0 + (1.0 - r0) * (1.0 - c).powi(5)
                    };
                    if rng.gen_range(0.0, 1.0) < r {
                        self.ray_tracing(
                            scene,
                            Ray::new(
                                pos,
                                ray.direction
                                    - normal * 2.0 * Vector3f::dot(&normal, &ray.direction),
                            ),
                            n_stack,
                            pixel,
                            rng,
                        )
                    } else {
                        if inside {
                            n_stack.pop();
                        } else {
                            n_stack.push(nt);
                        }
                        self.ray_tracing(scene, Ray::new(pos, t), n_stack, pixel, rng)
                    }
                }
            }
        }
    }
}

impl Render for PPM {
    fn render(&self, scene: &Scene, camera: &Camera) -> Image {
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
            for ray in camera.gen(i, j, &mut rng) {
                self.ray_tracing(scene, ray, &mut vec![scene.n], (i, j), &mut rng);
            }

            // image.lock().unwrap().set(i, j, pixel);
            // progress_bar.lock().unwrap().inc();
        });
        progress_bar.lock().unwrap().finish_println("done\n");
        image.into_inner().unwrap()
    }
}
