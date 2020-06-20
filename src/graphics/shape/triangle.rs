use crate::math::vector::Vector3f;
use crate::graphics::{Hittable, HitTemp};
use crate::math::Ray;
use crate::math::matrix::Matrix3;

#[derive(Debug)]
pub struct Triangle {
    vertices: [Vector3f; 3],
    normal: Vector3f,
    e1: Vector3f,
    e2: Vector3f
}

impl Triangle {
    pub fn new(vertices: [Vector3f; 3]) -> Self {
        let e1 = vertices[0] - vertices[1];
        let e2 = vertices[0] - vertices[2];

        Self {
            vertices,
            normal: Vector3f::cross(&(vertices[1] - vertices[0]), &(vertices[2] - vertices[0])).normalized(),
            e1,
            e2
        }
    }
}

impl Hittable for Triangle {
    fn hit(&self, ray: &Ray, t_min: f64) -> Option<HitTemp> {
        let s = self.vertices[0] - ray.origin;
        let d = Matrix3::from_vectors([ray.direction, self.e1, self.e2], true).determinant();
        let t = Matrix3::from_vectors([s, self.e1, self.e2], true).determinant() / d;
        let beta = Matrix3::from_vectors([ray.direction, s, self.e2], true).determinant() / d;
        let gamma = Matrix3::from_vectors([ray.direction, self.e1, s], true).determinant() / d;
        if t > t_min && beta >= 0.0 && gamma >= 0.0 && beta + gamma <= 1.0 {
            Some(HitTemp {
                t,
                normal: self.normal,
                uv: None
            })
        } else {
            None
        }
    }
}