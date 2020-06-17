use crate::graphics::Color;
use crate::math::vector::{Vector2f, Vector3f};
use crate::math::{clamp, FloatT};
use serde::export::fmt::Debug;
use serde::export::Formatter;
use serde::{Deserialize, Deserializer};
use std::fs::File;
use std::io::Write;

pub fn trans(x: FloatT) -> u8 {
    (clamp(x).powf(1.0 / 2.2) * 255.0 + 0.5) as u8
}
pub fn modf(f: FloatT, m: usize) -> usize {
    let m = m as usize;
    ((f as isize % m as isize) + m as isize) as usize % m
}

mod image;
pub use self::image::*;
use crate::scene::{Camera, Render, Renderer, Scene};
use std::fs;

// shoot scene with camera (render with renderer), save to save_path
#[derive(Deserialize)]
pub struct Task {
    pub scene: Scene,
    pub camera: Camera,
    pub save_path: String,
    pub renderer: Renderer,
}

impl Task {
    pub fn from_json(path: &str) -> Self {
        let data = fs::read_to_string(path).expect(&format!("Unable to read {}", path));
        serde_json::from_str::<Task>(&data).expect("Cannot convert to json")
    }

    pub fn run(&self) {
        let image = self.renderer.render(&self.scene, &self.camera);
        image.dump(&self.save_path, true);
    }
}
