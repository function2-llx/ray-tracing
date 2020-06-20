use std::io::{BufRead, Read};

use crate::graphics::shape::Triangle;
use crate::graphics::{Bounding, HitTemp, Hittable};
use crate::math::vector::Vector3f;
use crate::math::{FloatT, Ray, INF};
use crate::utils::kdtree::triangle::Node;
use serde::export::Formatter;
use serde::{Deserialize, Deserializer};
use std::fmt::Debug;
use crate::math::matrix::Matrix3;

pub struct Mesh {
    points: Vec<Vector3f>,
    triangles: Vec<Triangle>,
    bounding: Bounding,
    kdtree: Box<Node>,
}

impl Debug for Mesh {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Mesh")
            .field("points", &self.points)
            .finish()
    }
}

impl<'de> Deserialize<'de> for Mesh {
    fn deserialize<D>(deserializer: D) -> Result<Self, <D as Deserializer<'de>>::Error>
    where
        D: Deserializer<'de>,
    {
        #[derive(Deserialize)]
        struct Rotate {
            dim: usize,
            degree: FloatT,
        }
        #[derive(Deserialize)]
        struct MeshInfo {
            path: String,
            shift: Vector3f,
            scale: Vector3f,
            rotates: Vec<Rotate>,
        }

        let info = MeshInfo::deserialize(deserializer)?;
        let rotates = info.rotates.iter().map(|r| {
            let t = r.degree.to_radians();
            let cos = t.cos();
            let sin = t.sin();

            match r.dim {
                0 => Matrix3([[1.0, 0.0, 0.0], [0.0, cos, -sin], [0.0, sin, cos]]),
                1 => Matrix3([[cos, 0.0, sin], [0.0, 1.0, 0.0], [-sin, 0.0, cos]]),
                2 => Matrix3([[cos, -sin, 0.0], [sin, cos, 0.0], [0.0, 0.0, 1.0]]),
                _ => panic!("bad dim"),
            }
        }).collect::<Vec<_>>();
        Ok(Mesh::from_obj(&info.path, info.shift, info.scale, rotates))
    }
}

impl Mesh {
    pub fn from_obj(path: &str, shift: Vector3f, scale: Vector3f, rotates: Vec<Matrix3>) -> Self {
        let data = std::fs::read_to_string(path).expect(&format!("cannot read from {}", path));
        let mut object = wavefront_obj::obj::parse(data)
            .expect(&format!("load from {} failed", path))
            .objects
            .pop()
            .expect("no object");
        let mut points = object
            .vertices
            .iter()
            .map(|v| Vector3f::new([v.x, v.y, v.z]))
            .collect::<Vec<_>>();
        let mid = {
            let bounding = Bounding::build(&points);
            (bounding.min + bounding.max) / 2.0
        };
        points
            .iter_mut()
            .for_each(|p| {
                rotates.iter().for_each(|r| *p = *r * *p);
                *p = mid + scale * (*p - mid) + shift;
            });
        let bounding = Bounding::build(&points);
        let normals = object
            .normals
            .iter()
            .map(|n| Vector3f::new([n.x, n.y, n.z]).normalized())
            .collect::<Vec<_>>();
        let triangles = object
            .geometry
            .pop()
            .expect("no geometry found")
            .shapes
            .iter()
            .map(|s| {
                use wavefront_obj::obj::Primitive;
                match s.primitive {
                    Primitive::Triangle((a, _, na), (b, _, nb), (c, _, nc)) => Triangle::new(
                        [points[a], points[b], points[c]],
                        if na.is_some() {
                            Some([
                                normals[na.unwrap()],
                                normals[nb.unwrap()],
                                normals[nc.unwrap()],
                            ])
                        } else {
                            None
                        },
                    ),
                    _ => panic!("unsupported"),
                }
            })
            .collect::<Vec<_>>();

        Self {
            points,
            bounding,
            triangles: triangles.clone(),
            kdtree: Node::new(triangles),
        }
    }
}

impl Hittable for Mesh {
    fn hit(&self, ray: &Ray, t_min: f64) -> Option<HitTemp> {
        if let Some((l, r)) = self.bounding.intersect(ray) {
            if let Some(ret) = self.kdtree.hit(ray, t_min, INF) {
                Some(ret)
            } else {
                None
            }
        } else {
            None
        }
    }
}
