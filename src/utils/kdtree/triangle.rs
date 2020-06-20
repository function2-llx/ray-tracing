use crate::graphics::shape::Triangle;
use crate::graphics::{Bounding, HitTemp, Hittable};
use crate::math::vector::Vector3f;
use crate::math::{sqr, FloatT, Ray, ZERO};
use crate::utils::Positionable;
use std::cmp::Ordering;
use std::collections::BinaryHeap;
use std::mem::swap;

pub struct Node {
    triangles: Vec<Triangle>,
    pub bounding: Bounding,
    pub dim: usize, // 划分维度
    pub l: Option<Box<Node>>,
    pub r: Option<Box<Node>>,
}

impl Node {
    pub fn new(mut triangles: Vec<Triangle>) -> Box<Self> {
        assert!(!triangles.is_empty());
        let mut min = Vector3f::empty();
        let mut max = Vector3f::empty();
        for i in 0..3 {
            min[i] = triangles
                .iter()
                .map(|t| t.bounding.min[i])
                .min_by(|a, b| a.partial_cmp(b).unwrap())
                .unwrap();
            max[i] = triangles
                .iter()
                .map(|t| t.bounding.max[i])
                .max_by(|a, b| a.partial_cmp(b).unwrap())
                .unwrap();
        }
        Self::build(triangles, 0, min, max)
    }

    fn build(mut triangles: Vec<Triangle>, dim: usize, min: Vector3f, max: Vector3f) -> Box<Node> {
        let mid = {
            let mut all = Vec::new();
            for t in triangles.iter() {
                all.push(t.bounding.min[dim]);
                all.push(t.bounding.max[dim]);
            }
            let mid = all.len() / 2;
            all.partition_at_index_by(mid, |a, b| a.partial_cmp(b).unwrap());
            all[mid]
        };
        let mut l = Vec::new();
        let mut r = Vec::new();
        let mut cur = Vec::new();
        for t in triangles.iter() {
            if t.bounding.max[dim] <= mid {
                l.push(t.clone());
            } else if t.bounding.min[dim] >= mid {
                r.push(t.clone());
            } else {
                cur.push(t.clone());
            }
        }
        // println!("({}, {}, {})", l.len(), cur.len(), r.len());
        if l.is_empty() && cur.is_empty() || cur.is_empty() && r.is_empty() {
            return Box::new(Node {
                triangles,
                bounding: Bounding { min, max },
                dim,
                l: None,
                r: None,
            });
        }
        std::mem::drop(triangles);

        Box::new(Node {
            triangles: cur,
            dim,
            l: if !l.is_empty() {
                let mut new_max = max;
                new_max[dim] = mid;
                Some(Self::build(l, (dim + 1) % 3, min, new_max))
            } else {
                None
            },
            r: if !r.is_empty() {
                let mut new_min = min;
                new_min[dim] = mid;
                Some(Self::build(r, (dim + 1) % 3, new_min, max))
            } else {
                None
            },
            bounding: Bounding { min, max },
        })
    }

    pub fn hit(&self, ray: &Ray, t_min: FloatT, mut t_max: FloatT) -> Option<HitTemp> {
        let cur = self
            .triangles
            .iter()
            .filter_map(|t| {
                t.hit(ray, t_min)
                    .map(|hit| if hit.t < t_max { Some(hit) } else { None })
                    .unwrap_or(None)
            })
            .min_by(|a, b| a.t.partial_cmp(&b.t).unwrap());

        if let Some(hit) = &cur {
            t_max = hit.t;
        }

        let l = self
            .l
            .as_ref()
            .map(|x| x.bounding.intersect(ray))
            .unwrap_or(None);
        let r = self
            .r
            .as_ref()
            .map(|x| x.bounding.intersect(ray))
            .unwrap_or(None);
        let sub = match (l, r) {
            (Some(l), Some(r)) => {
                // 由于区域不相交，因此这两个区间也一定不相交
                // assert!(l.1 <= r.0 || r.1 <= l.0);
                if l.1 <= r.0 {
                    let hit = self.l.as_ref().unwrap().hit(ray, t_min, t_max);
                    if hit.is_some() {
                        hit
                    } else {
                        self.r.as_ref().unwrap().hit(ray, t_min, t_max)
                    }
                } else {
                    let hit = self.r.as_ref().unwrap().hit(ray, t_min, t_max);
                    if hit.is_some() {
                        hit
                    } else {
                        self.l.as_ref().unwrap().hit(ray, t_min, t_max)
                    }
                }
            }
            (Some(_), None) => self.l.as_ref().unwrap().hit(ray, t_min, t_max),
            (None, Some(_)) => self.r.as_ref().unwrap().hit(ray, t_min, t_max),
            (None, None) => None,
        };

        if sub.is_some() {
            sub
        } else {
            cur
        }
    }
}
