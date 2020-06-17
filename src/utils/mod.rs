use crate::graphics::Color;
use crate::math::vector::{Vector2f, Vector3f};
use crate::math::{clamp, FloatT};
use serde::export::fmt::Debug;
use serde::export::Formatter;
use serde::{Deserialize, Deserializer};
use std::fs::File;
use std::io::Write;

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

fn trans(x: FloatT) -> u8 {
    (clamp(x).powf(1.0 / 2.2) * 255.0 + 0.5) as u8
}

impl Image {
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

    pub fn dump_ppm(&self, path: &str) {
        println!("Writing to {}", path);
        let errmsg = &format!("cannot save PPM to {}", path);
        let mut file = File::create(path).expect(errmsg);
        let mut data = String::new();
        data.push_str(&format!("P3\n{} {}\n255\n", self.w, self.h));
        self.data.iter().for_each(|t| {
            data.push_str(&format!("{} {} {} ", trans(t[0]), trans(t[1]), trans(t[2])));
        });
        file.write_all(data.as_bytes()).expect(errmsg);
        file.flush().expect(errmsg);
        println!("...done");
    }

    pub fn dump_png(&self, path: &str) {
        println!("Writing to {}", path);
        let mut imgbuf = image::ImageBuffer::new(self.w as u32, self.h as u32);

        {
            let mut it = self.data.iter();
            for p in imgbuf.pixels_mut() {
                let t = it.next().unwrap();
                *p = image::Rgb([trans(t[0]), trans(t[1]), trans(t[2])]);
            }
        }
        imgbuf
            .save(&path)
            .expect(&format!("cannot save PNG to {}", path));
        println!("...done");
    }
}

impl<'de> Deserialize<'de> for Image {
    fn deserialize<D>(deserializer: D) -> Result<Self, <D as Deserializer<'de>>::Error>
    where
        D: Deserializer<'de>,
    {
        unimplemented!()
    }
}
