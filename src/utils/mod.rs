use crate::math::vector::Vector3f;
use crate::math::{clamp, FloatT};
use crate::scene::{Camera, Render, Renderer, Scene};
use serde::export::Formatter;
use serde::{Deserialize, Deserializer};
use std::fs;
use std::fs::File;
use std::io::Write;

pub fn trans(x: FloatT) -> u8 {
    (clamp(x).powf(1.0 / 2.2) * 255.0 + 0.5) as u8
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

impl<'de> Deserialize<'de> for Task {
    fn deserialize<D>(deserializer: D) -> Result<Self, <D as Deserializer<'de>>::Error> where
        D: Deserializer<'de>
    {
        #[derive(Deserialize)]
        struct TaskInfo {
            pub scene: Scene,
            pub camera: Camera,
            pub renderer: Renderer,
            pub num_threads: usize,
        }

        let info = TaskInfo::deserialize(deserializer).unwrap();
        Ok(Task {
            scene: info.scene,
            camera: info.camera,
            renderer: info.renderer,
            num_threads: info.num_threads,
            name: "".to_string(),
        })
    }
}

impl Task {
    pub fn from_json(name: &str) -> Self {
        let path = format!("task/{}.json", name);
        let data = fs::read_to_string(&path).expect(&format!("Unable to read {}", &path));
        let mut ret = serde_json::from_str::<Task>(&data).expect("Cannot convert to json");
        ret.name = name.to_string();
        ret
    }

    pub fn run(&self) {
        rayon::ThreadPoolBuilder::new()
            .num_threads(self.num_threads)
            .build()
            .unwrap()
            .install(|| {
                self.renderer
                    .render(&self.scene, &self.camera, &self.name);
            });
    }
}

pub trait Positionable {
    fn pos(&self) -> Vector3f;
}
