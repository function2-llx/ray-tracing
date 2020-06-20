use crate::graphics::{Bounding, HitTemp, Hittable};
use crate::math::matrix::Matrix3;
use crate::math::vector::Vector3f;
use crate::math::Ray;

#[derive(Debug, Clone)]
pub struct Triangle {
    vertices: [Vector3f; 3],
    normals: [Vector3f; 3],
    e1: Vector3f,
    e2: Vector3f,
    pub bounding: Bounding,
}

impl Triangle {
    pub fn new(vertices: [Vector3f; 3], normals: Option<[Vector3f; 3]>) -> Self {
        let e1 = vertices[0] - vertices[1];
        let e2 = vertices[0] - vertices[2];
        let normals = normals.unwrap_or([Vector3f::cross(&e1, &e2); 3]);
        // let normals = normals.unwrap();
        Self {
            vertices,
            normals,
            e1,
            e2,
            bounding: Bounding::build(&vertices),
        }
    }
}

impl Hittable for Triangle {
    fn hit(&self, ray: &Ray, t_min: f64) -> Option<HitTemp> {
        if self.bounding.intersect(ray).is_none() {
            return None
        }
        let s = self.vertices[0] - ray.origin;
        let d = Matrix3::from_vectors([ray.direction, self.e1, self.e2], true).determinant();
        let t = Matrix3::from_vectors([s, self.e1, self.e2], true).determinant() / d;
        let beta = Matrix3::from_vectors([ray.direction, s, self.e2], true).determinant() / d;
        let gamma = Matrix3::from_vectors([ray.direction, self.e1, s], true).determinant() / d;
        let alpha = 1.0 - beta - gamma;
        if t > t_min && alpha >= 0.0 && beta >= 0.0 && gamma >= 0.0 {
            let normal = (alpha * self.normals[0] + beta * self.normals[1] + gamma * self.normals[2]).normalized();
            Some(HitTemp {
                t,
                normal,
                uv: None,
            })
        } else {
            None
        }
    }
}
