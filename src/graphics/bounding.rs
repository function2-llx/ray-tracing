use crate::math::vector::Vector3f;
use crate::math::{FloatT, Ray};
use std::cmp::min;
use std::mem::swap;

// 包围盒
#[derive(Debug)]
pub struct Bounding {
    pub min: Vector3f,
    pub max: Vector3f,
}

impl Bounding {
    // 返回包围盒
    pub fn build(points: &Vec<Vector3f>) -> Self {
        assert!(!points.is_empty());
        let mut min = Vector3f::empty();
        let mut max = Vector3f::empty();
        for i in 0..3 {
            min[i] = points.iter().map(|x| x[i]).min_by(|a, b| a.partial_cmp(b).unwrap()).unwrap();
            max[i] = points.iter().map(|x| x[i]).max_by(|a, b| a.partial_cmp(b).unwrap()).unwrap();
        }
        Bounding {
            min,
            max
        }
    }

    // 求直线与包围盒的相交区间
    pub fn intersect(&self, ray: &Ray) -> Option<(FloatT, FloatT)> {
        let mut l = (self.min - ray.origin) / ray.direction;
        let mut r = (self.max - ray.origin) / ray.direction;
        for i in 0..3 {
            if l[i] > r[i] {
                swap(&mut l[i], &mut r[i]);
            }
            if l[i] < 0.0 {
                l[i] = 0.0;
            }
        }
        let l = l[0].max(l[1]).max(l[2]);
        let r = r[0].min(r[1]).min(r[2]);
        if l <= r {
            Some((l, r))
        } else {
            None
        }
    }
}
