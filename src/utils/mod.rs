use crate::math::vector::Vector3f;
use crate::math::FloatT;
use crate::scene::{Camera, Render, Renderer, Scene};
use serde::{Deserialize, Deserializer};
use std::fs;
use std::io::Write;

// 伽马修正
pub fn trans(x: FloatT) -> u8 {
    (::image::math::utils::clamp(x, 0.0, 1.0).powf(1.0 / 2.2) * 255.0 + 0.5) as u8
}

mod image;
pub mod kdtree;

pub use self::image::*;

// shoot scene with camera (render with renderer)
pub struct Task {
    pub scene: Scene,
    pub camera: Camera,
    pub renderer: Renderer,
    pub num_threads: usize,
    pub name: String,
}

impl Task {
    pub fn from_json(name: &str) -> Self {
        let path = format!("task/{}.json", name);
        let data = fs::read_to_string(&path).expect(&format!("Unable to read {}", &path));
        #[derive(Deserialize)]
        struct TaskInfo {
            pub scene: Scene,
            pub camera: Camera,
            pub renderer: Renderer,
            pub num_threads: usize,
        }
        let mut info = serde_json::from_str::<TaskInfo>(&data).expect("Cannot convert to json");
        Task {
            scene: info.scene,
            camera: info.camera,
            renderer: info.renderer,
            num_threads: info.num_threads,
            name: name.to_string(),
        }
    }

    pub fn run(&self) {
        rayon::ThreadPoolBuilder::new()
            .num_threads(self.num_threads)
            .build()
            .unwrap()
            .install(|| {
                self.renderer.render(&self.scene, &self.camera, &self.name);
            });
    }
}

pub trait Positionable {
    fn pos(&self) -> Vector3f;
}
