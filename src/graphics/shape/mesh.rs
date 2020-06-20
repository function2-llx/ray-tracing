use std::io::BufRead;

use tobj;
use crate::math::vector::Vector3f;
use crate::math::{FloatT, Ray};
use crate::graphics::{Bounding, Hittable, HitTemp};
use crate::graphics::shape::Triangle;
use serde::{Deserialize, Deserializer};

#[derive(Debug)]
pub struct Mesh {
    points: Vec<Vector3f>,
    triangles: Vec<Triangle>,
    bounding: Bounding,
}

impl<'de> Deserialize<'de> for Mesh {
    fn deserialize<D>(deserializer: D) -> Result<Self, <D as Deserializer<'de>>::Error> where
        D: Deserializer<'de>
    {
        #[derive(Deserialize)]
        struct MeshInfo {
            path: String,
            shift: Vector3f,
            scale: Vector3f,
        }

        let info = MeshInfo::deserialize(deserializer)?;
        Ok(Mesh::from_obj(&info.path, info.shift, info.scale))
    }
}

impl Mesh {
    pub fn from_obj(path: &str, shift: Vector3f, scale: Vector3f) -> Self {
        let (mut models, _) = tobj::load_obj(path, true).expect(&format!("load from {} failed", path));
        let mesh = models.pop().expect("no model found").mesh;

        let mut points = (0..mesh.positions.len()).step_by(3).map(|i| {
            Vector3f::new([mesh.positions[i] as FloatT, mesh.positions[i + 1] as FloatT, mesh.positions[i + 2] as FloatT]) + shift
        }).collect::<Vec<_>>();
        let bounding = Bounding::build(&points);
        let mid = (bounding.min + bounding.max) / 2.0;
        points.iter_mut().for_each(|p| *p = mid + scale * (*p - mid));

        let triangles = (0..mesh.indices.len()).step_by(3).map(|i| {
            Triangle::new([points[mesh.indices[i] as usize], points[mesh.indices[i + 1] as usize], points[mesh.indices[i + 2] as usize]])
        }).collect::<Vec<_>>();

        Self {
            points,
            bounding,
            triangles,
        }
    }
}

impl Hittable for Mesh {
    fn hit(&self, ray: &Ray, t_min: f64) -> Option<HitTemp> {
        if let Some((l, r)) = self.bounding.intersect(ray) {
            self.triangles.iter().filter_map(|triangle| {
                triangle.hit(ray, t_min)
            }).min_by(|x, y| {
                x.t.partial_cmp(&y.t).unwrap()
            })
        } else {
            None
        }
    }
}