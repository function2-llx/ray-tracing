use serde::Deserialize;

use crate::scene::{Camera, Scene};
use crate::utils::Image;

mod pt;

pub use pt::*;

pub trait Render {
    fn render(&self, scene: &Scene, camera: &Camera) -> Image;
}

#[derive(Deserialize)]
pub enum Renderer {
    PT(PT),
}

impl Render for Renderer {
    fn render(&self, scene: &Scene, camera: &Camera) -> Image {
        use Renderer::*;
        match self {
            PT(pt) => pt.render(scene, camera),
        }
    }
}
