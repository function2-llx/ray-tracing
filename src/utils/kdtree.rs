use crate::math::vector::Vector3f;
use crate::math::{sqr, FloatT, ZERO};
use crate::utils::Positionable;
use std::cmp::Ordering;
use std::collections::BinaryHeap;

pub struct Node<Item: Positionable + Clone> {
    pub item: Item,
    pub dim: usize, // 划分维度
    pub min: Vector3f,
    pub max: Vector3f, // min, max: 当前维护的范围
    pub l: Option<Box<Node<Item>>>,
    pub r: Option<Box<Node<Item>>>,
}

struct Data<T>(FloatT, T);

impl<T> Data<T> {
    pub fn get(self) -> T {
        self.1
    }
}

impl<T> PartialEq for Data<T> {
    fn eq(&self, other: &Self) -> bool {
        self.0 == other.0
    }
}

impl<T> Eq for Data<T> {
    // ?
}

impl<T> PartialOrd for Data<T> {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        self.0.partial_cmp(&other.0)
    }
}

impl<T> Ord for Data<T> {
    fn cmp(&self, other: &Self) -> Ordering {
        self.0.partial_cmp(&other.0).unwrap()
    }
}

impl<Item: Positionable + Clone> Positionable for Node<Item> {
    fn pos(&self) -> Vector3f {
        self.item.pos()
    }
}

pub fn new<Item: Positionable + Clone>(items: Vec<Item>) -> Box<Node<Item>> {
    Node::<Item>::new(items)
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

    pub fn knn(&self, pos: &Vector3f, k: usize) -> Vec<Item> {
        if k == 0 {
            panic!("k must be greater than 0");
        }
        let mut cur = BinaryHeap::new();
        self.knn_impl(pos, k, &mut cur);
        let mut ret = vec![];
        ret.reserve(cur.len());
        for x in cur {
            ret.push(x.get());
        }
        ret
    }

    // fn contains(&self, pos: &Vector3f) -> bool {
    //     for i in 0..3 {
    //         if self.min[i] > pos[i] || pos[i] > self.max[i] {
    //             return false;
    //         }
    //     }
    //     true
    // }

    // 返回所有 |x - pos| <= r 的点
    pub fn within(&self, pos: &Vector3f, r: FloatT) -> Vec<Item> {
        let mut results = vec![];
        self.within_impl(pos, r * r, &mut results);
        results
    }

    fn within_impl(&self, pos: &Vector3f, r2: FloatT, results: &mut Vec<Item>) {
        if (self.pos() - *pos).length2() <= r2 {
            results.push(self.item.clone());
        }
        if let Some(l) = &self.l {
            if l.lower_bound(pos) <= r2 {
                l.within_impl(pos, r2, results);
            }
        }
        if let Some(r) = &self.r {
            if r.lower_bound(pos) <= r2 {
                r.within_impl(pos, r2, results);
            }
        }
    }

    // 估计 pos 到自身区域内的点的距离平方的下界
    fn lower_bound(&self, pos: &Vector3f) -> FloatT {
        let p = [self.min, self.max];
        let mut lb = ZERO;
        for i in 0..3 {
            if pos[i] < self.min[i] {
                lb += sqr(self.min[i] - pos[i]);
            } else if pos[i] > self.max[i] {
                lb += sqr(self.max[i] - pos[i]);
            }
        }
        lb
    }

    fn knn_impl(&self, pos: &Vector3f, k: usize, cur: &mut BinaryHeap<Data<Item>>) {
        let dis2 = (self.pos() - *pos).length2();
        if cur.len() < k {
            cur.push(Data(dis2, self.item.clone()));
        } else {
            if let Some(data) = cur.peek() {
                if data.0 > dis2 {
                    cur.pop();
                    cur.push(Data(dis2, self.item.clone()));
                }
            }
        }

        match (&self.l, &self.r) {
            (Some(l), Some(r)) => {
                let lower_bound_l = l.lower_bound(pos);
                let lower_bound_r = r.lower_bound(pos);
                if lower_bound_l < lower_bound_r {
                    if k < cur.len() || lower_bound_l < cur.peek().unwrap().0 {
                        l.knn_impl(pos, k, cur);
                        if k < cur.len() || lower_bound_r < cur.peek().unwrap().0 {
                            r.knn_impl(pos, k, cur);
                        }
                    }
                } else {
                    if k < cur.len() || lower_bound_r < cur.peek().unwrap().0 {
                        r.knn_impl(pos, k, cur);
                        if k < cur.len() || lower_bound_l < cur.peek().unwrap().0 {
                            l.knn_impl(pos, k, cur);
                        }
                    }
                }
            }
            (Some(l), None) => {
                if k < cur.len() || l.lower_bound(pos) < cur.peek().unwrap().0 {
                    l.knn_impl(pos, k, cur);
                }
            }
            (None, Some(r)) => {
                if k < cur.len() || r.lower_bound(pos) < cur.peek().unwrap().0 {
                    r.knn_impl(pos, k, cur);
                }
            }
            (None, None) => (),
        }
    }
}
