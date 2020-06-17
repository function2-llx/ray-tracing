use crate::math::vector::Vector3f;
use crate::math::{FloatT, INF, ZERO};
use crate::utils::Positionable;
use serde::de::Unexpected::Float;
use std::cmp::Ordering;
use std::collections::{BTreeSet, BinaryHeap};
use std::mem::swap;

struct Node<Item: Positionable + Clone> {
    pub item: Item,
    pub dim: usize, // 划分维度
    pub min: Vector3f,
    pub max: Vector3f, // min, max: 当前维护的范围
    pub l: Option<Box<Node<Item>>>,
    pub r: Option<Box<Node<Item>>>,
}

struct Data(FloatT, Vector3f);

impl PartialEq for Data {
    fn eq(&self, other: &Self) -> bool {
        self.0 == other.0
    }
}

impl Eq for Data {
    // ?
}

impl PartialOrd for Data {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        self.0.partial_cmp(&other.0)
    }
}

impl Ord for Data {
    fn cmp(&self, other: &Self) -> Ordering {
        self.0.partial_cmp(&other.0).unwrap()
    }
}

impl<Item: Positionable + Clone> Positionable for Node<Item> {
    fn pos(&self) -> Vector3f {
        self.item.pos()
    }
}

impl<'a, Item: Positionable + Clone> Node<Item> {
    pub fn new(mut items: Vec<Item>) -> Box<Self> {
        assert!(!items.is_empty());
        let mut min = Vector3f::empty();
        let mut max = Vector3f::empty();
        for i in 0..3 {
            min[i] = items
                .iter()
                .map(|x| x.pos()[i])
                .min_by(|a, b| a.partial_cmp(b).unwrap())
                .unwrap();
            max[i] = items
                .iter()
                .map(|x| x.pos()[i])
                .max_by(|a, b| a.partial_cmp(b).unwrap())
                .unwrap();
        }
        Self::build(items.as_mut_slice(), 0, min, max)
    }

    fn build(items: &mut [Item], dim: usize, min: Vector3f, max: Vector3f) -> Box<Node<Item>> {
        if items.len() == 1 {
            let item = items[0].clone();
            let pos = item.pos();
            Box::new(Node {
                item,
                dim,
                min: pos,
                max: pos,
                l: None,
                r: None,
            })
        } else {
            let len = items.len();
            let mid = len / 2;
            items.partition_at_index_by(mid, |x, y| {
                x.pos()[dim].partial_cmp(&y.pos()[dim]).unwrap()
            });
            let item = items[mid].clone();
            let pos = item.pos()[dim];
            let mut ret = Box::new(Node {
                item,
                dim,
                min,
                max,
                l: None,
                r: None,
            });
            if mid > 0 {
                let mut new_max = max;
                new_max[dim] = pos;
                ret.l = Some(Self::build(&mut items[0..mid], (dim + 1) % 3, min, new_max));
            }
            if mid + 1 < len {
                let mut new_min = min;
                new_min[dim] = pos;
                ret.r = Some(Self::build(
                    &mut items[mid + 1..len],
                    (dim + 1) % 3,
                    new_min,
                    max,
                ));
            }
            ret
        }
    }

    pub fn knn(&self, pos: &Vector3f, k: usize) -> Vec<Vector3f> {
        if k == 0 {
            panic!("k must be greater than 0");
        }
        let mut cur = BinaryHeap::new();
        self.knn_impl(pos, k, &mut cur);
        cur.iter().map(|x| x.1).collect()
    }

    // 估计 pos 到自身的最远距离的平方
    fn estimate_max(&self, pos: &Vector3f) -> FloatT {
        let p = [self.min, self.max];
        let mut max = ZERO;
        for i in 0..2 {
            for j in 0..2 {
                for k in 0..2 {
                    max = max.max((*pos - Vector3f::new([p[i][0], p[j][1], p[k][2]])).length2());
                }
            }
        }
        max
    }

    fn knn_impl(&self, pos: &Vector3f, k: usize, cur: &mut BinaryHeap<Data>) {
        let dis2 = (self.pos() - *pos).length2();
        if cur.len() < k {
            cur.push(Data(dis2, self.pos()));
        } else {
            if let Some(data) = cur.peek() {
                if data.0 > dis2 {
                    cur.pop();
                    cur.push(Data(dis2, self.pos()));
                }
            }
        }

        match (&self.l, &self.r) {
            (Some(l), Some(r)) => {
                let maxl = l.estimate_max(pos);
                let maxr = r.estimate_max(pos);
                if maxl < maxr {
                    if k < cur.len() || maxl < cur.peek().unwrap().0 {
                        l.knn_impl(pos, k, cur);
                        if k < cur.len() || maxr < cur.peek().unwrap().0 {
                            r.knn_impl(pos, k, cur);
                        }
                    }
                } else {
                    if k < cur.len() || maxr < cur.peek().unwrap().0 {
                        r.knn_impl(pos, k, cur);
                        if k < cur.len() || maxl < cur.peek().unwrap().0 {
                            l.knn_impl(pos, k, cur);
                        }
                    }
                }
            }
            (Some(l), None) => {
                if k < cur.len() || l.estimate_max(pos) < cur.peek().unwrap().0 {
                    l.knn_impl(pos, k, cur);
                }
            }
            (None, Some(r)) => {
                if k < cur.len() || r.estimate_max(pos) < cur.peek().unwrap().0 {
                    r.knn_impl(pos, k, cur);
                }
            }
            (None, None) => (),
        }
    }
}
