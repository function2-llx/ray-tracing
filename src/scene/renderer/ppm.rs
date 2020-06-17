use serde::Deserialize;

use crate::scene::{Render, Camera, Scene};
use crate::utils::Image;

#[derive(Deserialize)]
pub struct PPM {

}

impl Render for PPM {
    fn render(&self, scene: &Scene, camera: &Camera) -> Image {
        let image = Image::empty(camera.w, camera.h);
        image
    }
}