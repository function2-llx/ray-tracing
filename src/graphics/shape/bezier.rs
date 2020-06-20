use serde::{Deserialize, Deserializer};

use crate::graphics::{Bounding, HitTemp, Hittable, TextureMap};
use crate::math::matrix::Matrix3;
use crate::math::vector::Vector3f;
use crate::math::{sqr, FloatT, Ray, EPS, PI};
use image::math::utils::clamp;
use rand::{thread_rng, Rng};

#[derive(Debug)]
pub struct BezierCurve {
    pub n: usize,
    // x(t), y(t) 的各项系数
    pub a: Box<[(FloatT, FloatT)]>,
}

impl BezierCurve {
    pub fn new(points: Vec<(FloatT, FloatT)>) -> Self {
        assert!(!points.is_empty());
        let n = points.len() - 1;
        let mut x = points.iter().map(|p| p.0).collect::<Vec<_>>();
        let mut y = points.iter().map(|p| p.1).collect::<Vec<_>>();
        let mut t = 1.0;
        let mut a = vec![];
        for i in 0..=n {
            a.push((x[0] * t, y[0] * t));
            t = t * (n - i) as FloatT / (i + 1) as FloatT;
            for j in 0..n - i {
                x[j] = x[j + 1] - x[j];
                y[j] = y[j + 1] - y[j];
            }
        }

        BezierCurve {
            n,
            a: a.into_boxed_slice(),
        }
    }

    pub fn eval(&self, t: FloatT) -> (FloatT, FloatT) {
        let (mut x, mut y) = (0.0, 0.0);
        for i in (0..=self.n).rev() {
            x = self.a[i].0 + x * t;
            y = self.a[i].1 + y * t;
        }
        (x, y)
    }

    pub fn x(&self, t: FloatT) -> FloatT {
        let mut x = 0.0;
        for i in (0..=self.n).rev() {
            x = self.a[i].0 + x * t;
        }
        x
    }

    pub fn y(&self, t: FloatT) -> FloatT {
        let mut y = 0.0;
        for i in (0..=self.n).rev() {
            y = self.a[i].1 + y * t;
        }
        y
    }

    pub fn derivative(&self, t: FloatT) -> (FloatT, FloatT) {
        let (mut x, mut y) = (0.0, 0.0);
        for i in (1..=self.n).rev() {
            x = self.a[i].0 * i as FloatT + x * t;
            y = self.a[i].1 * i as FloatT + y * t;
        }
        (x, y)
    }

    // 注意没有归一化
    pub fn normal(&self, t: FloatT) -> (FloatT, FloatT) {
        let (x, y) = self.derivative(t);
        // 前进方向的右侧
        (y, -x)
    }
}

// 旋转曲面生成：在 z = 0 平面上作 curve，绕 (0, 1, 0) 旋转，接着平移 shift
#[derive(Debug)]
pub struct BezierRotate {
    curve: BezierCurve,
    shift: Vector3f,
    bounding: Bounding,
}

impl BezierRotate {
    pub fn new(points: Vec<(FloatT, FloatT)>, shift: Vector3f) -> Self {
        let (mut max_x, mut max_y, mut min_y) = (points[0].0.abs(), points[0].1, points[0].1);
        for (x, y) in &points[1..] {
            max_x = max_x.max(x.abs());
            min_y = min_y.min(*y);
            max_y = max_y.max(*y);
        }
        BezierRotate {
            curve: BezierCurve::new(points),
            shift,
            bounding: Bounding {
                min: Vector3f::new([-max_x, min_y, -max_x]) + shift,
                max: Vector3f::new([max_x, max_y, max_x]) + shift,
            },
        }
    }
}

impl TextureMap for BezierRotate {
    fn texture_map(
        &self,
        pos: Vector3f,
        uv: Option<(FloatT, FloatT)>,
        w: usize,
        h: usize,
    ) -> (usize, usize) {
        let (u, v) = uv.unwrap();
        assert!(0.0 <= u && u <= 1.0);
        assert!(0.0 <= v && v <= 1.0);
        ((w as FloatT * u) as usize % w, (h as FloatT * v) as usize % h)
    }
}

impl<'de> Deserialize<'de> for BezierRotate {
    fn deserialize<D>(deserializer: D) -> Result<Self, <D as Deserializer<'de>>::Error>
    where
        D: Deserializer<'de>,
    {
        #[derive(Deserialize)]
        struct BezierRotateInfo {
            pub points: Vec<(FloatT, FloatT)>,
            pub shift: Vector3f,
        };
        let info = BezierRotateInfo::deserialize(deserializer).unwrap();
        Ok(BezierRotate::new(info.points, info.shift))
    }
}

impl Hittable for BezierRotate {
    //                          [cos theta  0  -sin theta]   [x(t)]
    //  r.o + r.d * k = shift + [    0      1       0    ] * [y(t)]
    //                          [sin theta  0   cos theta]   [  0 ]
    //
    // 1. 注意到两侧 y 值与 theta 无关，可建立 t, k 的关系；
    // 2. 对于 x, z 可平方相加消去 theta，然后代入上面的关系变为关于 t 一元的非线性方程。
    fn hit(&self, ray: &Ray, k_min: FloatT) -> Option<HitTemp> {
        if self.bounding.intersect(ray).is_none() {
            return None;
        }
        let Ray {
            origin: mut o,
            direction: d,
        } = &ray;
        o -= self.shift;
        let t1 = o.x() * d.y() - o.y() * d.x();
        let t2 = o.z() * d.y() - o.y() * d.z();
        let a = sqr(d.x()) + sqr(d.z());
        let b = 2.0 * (t1 * d.x() + t2 * d.z());
        let c = sqr(t1) + sqr(t2);
        let w = -sqr(d.y());
        let solve = |mut t| {
            let (mut x, mut y) = self.curve.eval(t);
            let mut f = a * sqr(y) + b * y + c + w * sqr(x);
            for _ in 0..20 {
                let (dx, dy) = self.curve.derivative(t);
                let df = 2.0 * a * y * dy + b * dy + 2.0 * w * x * dx;
                let s = f / df;
                let mut lambda = 1.0;
                let weight = if t < 0.1 || t > 0.9 { 0.9 } else { 0.5 };
                while lambda > 1e-5 {
                    let nt = t - lambda * s;
                    if nt < 0.0 || nt > 1.0 {
                        lambda *= weight;
                        continue;
                    }
                    x = self.curve.x(nt);
                    y = self.curve.y(nt);

                    let nf = a * sqr(y) + b * y + c + w * sqr(x);
                    if nf.abs() < f.abs() {
                        t = nt;
                        f = nf;
                        break;
                    }
                    lambda *= weight;
                }
                if f.abs() < 1e-10 {
                    return Some(t);
                }
            }
            None
        };

        let samples = self.curve.n;
        let mut ans: Option<(FloatT, FloatT)> = None;
        for i in 0..=samples {
            if let Some(t) = solve(i as FloatT / samples as FloatT) {
                let k = if d.y().abs() > EPS {
                    (self.curve.y(t) - o.y()) / d.y()
                } else {
                    let a = sqr(d.x()) + sqr(d.z());
                    let b = 2.0 * (o.x() * d.x() + o.z() * d.z());
                    let c = sqr(o.x()) + sqr(d.x()) - sqr(self.curve.x(t));
                    let delta = (sqr(b) - 4.0 * a * c).sqrt();
                    let k = (-b - delta) / (2.0 * a);
                    if k >= k_min {
                        k
                    } else {
                        (-b + delta) / (2.0 * a)
                    }
                };
                if k > k_min && (ans.is_none() || k < ans.unwrap().1) {
                    ans = Some((t, k));
                }
            }
        }
        if let Some((t, k)) = ans {
            let x = self.curve.x(t);
            if x.abs() > EPS {
                let cos = (o.x() + k * d.x()) / x;
                let sin = (o.z() + k * d.z()) / x;
                let (x, y) = self.curve.normal(t);
                let normal = Vector3f::new([x * cos, y, x * sin]).normalized();
                let mut u = clamp(cos, -1.0, 1.0).acos();
                if sin < 0.0 {
                    u += PI;
                }
                Some(HitTemp {
                    t: k,
                    normal,
                    uv: Some((u / (2.0 * PI), t)),
                })
            } else {
                Some(HitTemp {
                    t: k,
                    // 面向数据 233
                    normal: if t > 0.5 {
                        Vector3f::new([0.0, 1.0, 0.0])
                    } else {
                        Vector3f::new([0.0, -1.0, 0.0])
                    },
                    uv: Some((0.0, 0.0)),
                })
            }
        } else {
            None
        }
    }
}
