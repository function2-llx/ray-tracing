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
