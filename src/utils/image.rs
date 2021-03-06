use std::fmt::{Debug, Formatter};
use std::fs::File;
use std::io::Write;

use serde::{Deserialize, Deserializer};

use crate::graphics::Color;
use crate::math::vector::Vector3f;
use crate::math::FloatT;
use crate::utils::trans;
use image::{open, GenericImageView};
use std::mem::{swap, MaybeUninit};
use std::path::Path;
use std::sync::Mutex;

#[derive(Clone)]
pub struct Image {
    pub w: usize,
    pub h: usize,
    data: Vec<Color>,
}

impl Debug for Image {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "Image({}, {})", self.w, self.h)
    }
}

#[derive(Debug)]
enum Format {
    JPG,
    PNG,
    PPM,
}

fn infer_format(path: &str) -> Option<Format> {
    let path = path.to_lowercase();
    // let path = path.as_str();
    use Format::*;
    if &path[path.len() - 4..path.len()] == "jpeg" {
        return Some(JPG);
    }
    match &path[path.len() - 3..path.len()] {
        s if s == "jpg" => Some(JPG),
        s if s == "png" => Some(PNG),
        s if s == "ppm" => Some(PPM),
        _ => None,
    }
}

impl Image {
    pub fn index(&self, x: usize, y: usize) -> usize {
        self.w * y + x
    }

    pub fn lr(&mut self) {
        for i in 0..self.w / 2 {
            for j in 0..self.h {
                let a = self.index(i, j);
                let b = self.index(self.w - i - 1, j);
                self.data.swap(a, b);
            }
        }
    }

    pub fn ud(&mut self) {
        for i in 0..self.w {
            for j in 0..self.h / 2 {
                let a = self.index(i, j);
                let b = self.index(i, self.h - j - 1);
                self.data.swap(a, b);
            }
        }
    }

    pub fn empty(w: usize, h: usize) -> Image {
        Image {
            w,
            h,
            data: vec![Color::empty(); w * h],
        }
    }

    pub fn at(&self, x: usize, y: usize) -> Color {
        self.data[y * self.w + x]
    }

    pub fn set(&mut self, x: usize, y: usize, color: Color) {
        self.data[y * self.w + x] = color;
    }

    pub fn add(&mut self, x: usize, y: usize, color: Color) {
        self.data[y * self.w + x] += color;
    }

    // rotate: 是否旋转 180 度
    pub fn dump(&self, name: &str, rotate: bool) {
        let path = format!("output/{}.png", name);
        let back_up = format!("output/{}-bk.png", name);
        if Path::new(&path).exists() {
            std::fs::copy(&path, back_up).unwrap();
        }
        println!("Writing to {}", &path);
        let mut buf = image::ImageBuffer::new(self.w as u32, self.h as u32);
        for x in 0..self.w {
            for y in 0..self.h {
                let t = if rotate {
                    self.at(self.w - x - 1, self.h - y - 1)
                } else {
                    self.at(x, y)
                };
                buf.put_pixel(
                    x as u32,
                    y as u32,
                    image::Rgb([trans(t[0]), trans(t[1]), trans(t[2])]),
                )
            }
        }
        buf.save(&path).expect(&format!("cannot save to {}", path));
        println!("...done");
    }

    pub fn load(path: &str) -> Self {
        use Format::*;
        if let Some(format) = infer_format(path) {
            match format {
                JPG | PNG => {
                    let buf = open(path).unwrap();
                    let mut image = Image::empty(buf.width() as usize, buf.height() as usize);
                    for x in 0..image.w {
                        for y in 0..image.h {
                            let p = buf.get_pixel(x as u32, y as u32);
                            image.set(
                                x,
                                y,
                                Color::new([p[0] as FloatT, p[1] as FloatT, p[2] as FloatT])
                                    / 255.0,
                            );
                        }
                    }
                    image
                }
                PPM => unimplemented!(),
            }
        } else {
            panic!("format not supported: {}", path);
        }
    }
}

impl<'de> Deserialize<'de> for Image {
    fn deserialize<D>(deserializer: D) -> Result<Self, <D as Deserializer<'de>>::Error>
    where
        D: Deserializer<'de>,
    {
        #[derive(Deserialize)]
        struct ImageInfo {
            path: String,
            lr: bool, // 左右翻转
            ud: bool, // 上下翻转
        }

        let info = ImageInfo::deserialize(deserializer)?;
        let mut image = Image::load(&info.path);
        if info.lr {
            image.lr();
        }
        if info.ud {
            image.ud();
        }
        Ok(image)
    }
}
