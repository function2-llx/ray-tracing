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
    // 实现思路：维护射线的各维坐标出现在 [min, max] 之间的时间，看是否相交
    pub fn intersect(&self, ray: &Ray) -> bool {
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
        l[0].max(l[1]).max(l[2]) <= r[0].min(r[1]).min(r[2])
    }
}
