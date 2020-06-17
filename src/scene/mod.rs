use std::fs;

use serde::Deserialize;
use serde_json::Value;

use crate::graphics::material::{Material, Optics};
use crate::graphics::shape::Shape;
use crate::graphics::{Color, Hittable};
use crate::graphics::{Hit, Object};
use crate::math::vector::Vector3f;
use crate::math::{sqr, FloatT, Ray, EPS, PI, ZERO};
use crate::utils::Image;

mod camera;

use crate::math::matrix::Matrix3;
pub use camera::*;
use rand::prelude::ThreadRng;
use rand::Rng;

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

    // n_stack: 折射率栈
    pub fn path_tracing(
        &self,
        ray: Ray,
        n_stack: Vec<FloatT>,
        depth: usize,
        max_depth: usize,
        rng: &mut ThreadRng,
    ) -> Color {
        if let Some(Hit {
            pos,
            mut normal,
            object,
        }) = self.hit(&ray, EPS)
        {
            let mut illumination = || {
                if depth == max_depth {
                    return self.env;
                }
                match &object.material.optics {
                    Optics::Diffuse => {
                        if Vector3f::dot(&normal, &ray.direction) > 0.0 {
                            // 调整为反射平面的法向量
                            normal = -normal;
                        }

                        // 以 normal 为 z 轴随便建个单位正交坐标系
                        let x = normal.get_orthogonal();
                        let y = Vector3f::cross(&normal, &x);
                        // 在半球面上选一个点
                        let theta = rng.gen_range(ZERO, 2.0 * PI);
                        let phi = rng.gen_range(ZERO, PI / 2.0);
                        let dir = Matrix3::from_vectors([x, y, normal], true)
                            * Vector3f::new([
                                phi.cos() * theta.cos(),
                                phi.cos() * theta.sin(),
                                phi.sin(),
                            ]);
                        assert!(Vector3f::dot(&normal, &dir) >= 0.0);
                        self.path_tracing(Ray::new(pos, dir), n_stack, depth + 1, max_depth, rng)
                    }
                    Optics::Specular => {
                        // 此时一定在物体外侧，因为不可能进入反射的材质
                        self.path_tracing(
                            Ray::new(
                                pos,
                                ray.direction
                                    - normal * 2.0 * Vector3f::dot(&normal, &ray.direction),
                            ),
                            n_stack.clone(),
                            depth + 1,
                            max_depth,
                            rng,
                        )
                    }
                    Optics::Refractive(nt) => {
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
                                println!("fuck");
                                let mut a = 1.0;
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
                            return self.path_tracing(
                                Ray::new(
                                    pos,
                                    ray.direction
                                        - normal * 2.0 * Vector3f::dot(&normal, &ray.direction),
                                ),
                                n_stack.clone(),
                                depth + 1,
                                max_depth,
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
                        r * self.path_tracing(
                            Ray::new(
                                pos,
                                ray.direction
                                    - normal * 2.0 * Vector3f::dot(&normal, &ray.direction),
                            ),
                            n_stack.clone(),
                            depth + 1,
                            max_depth,
                            rng,
                        ) + (1.0 - r) * {
                            let mut new_stack = n_stack.clone();
                            if inside {
                                new_stack.pop();
                            } else {
                                new_stack.push(nt);
                            }
                            self.path_tracing(
                                Ray::new(pos, t),
                                new_stack,
                                depth + 1,
                                max_depth,
                                rng,
                            )
                        }
                        // self.path_tracing(Ray::new(pos, t), n_stack, depth + 1, max_depth, rng)
                    }
                }
            };
            object.emission + object.color_at(pos) * illumination()
        } else {
            self.env
        }
    }
}

// shot scene with camera, save to save_path
pub struct Task {
    pub scene: Scene,
    pub camera: Camera,
    pub max_depth: usize,
    pub save_path: String,
}

impl Task {
    pub fn from_json(path: &str) -> Self {
        // let w = serde_json::from_value::<usize>(data["width"].take()).expect("Invalid width");
        // let h = serde_json::from_value::<usize>(data["height"].take()).expect("Invalid height");

        let data = fs::read_to_string(path).expect(&format!("Unable to read {}", path));
        let mut data = serde_json::from_str::<Value>(&data).expect("Cannot convert to json");

        let scene = serde_json::from_value::<Scene>(data["scene"].take()).expect("Invalid scene");
        println!("{:#?}", scene);
        let camera =
            serde_json::from_value::<Camera>(data["camera"].take()).expect("Invalid camera");

        let max_depth = serde_json::from_value::<usize>(data["max_depth"].take())
            .expect("Invalid maximum depth");
        let save_path =
            serde_json::from_value::<String>(data["save_path"].take()).expect("Invalid save path");

        Task {
            scene,
            camera,
            max_depth,
            save_path,
        }
    }
}
