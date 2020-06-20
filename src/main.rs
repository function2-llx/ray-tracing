extern crate ray_tracing;

use ray_tracing::graphics::shape::{BezierCurve, BezierRotate, Triangle};
use ray_tracing::graphics::Hittable;
use ray_tracing::math::vector::Vector3f;
use ray_tracing::math::Ray;
use ray_tracing::utils::Task;

fn main() {
    // let triangle = Triangle::new([Vector3f::new([-1.0, -1.0, 0.0]), Vector3f::new([-1.0, 1.0, 0.0]), Vector3f::new([1.0, -1.0, 0.0])]);
    // let ray = Ray::new(Vector3f::new([0.0, 0.0, -3.0]), Vector3f::new([0.0, 0.0, 1.0]));
    // triangle.hit(&ray, 0.0);
    if !std::path::Path::new("output").exists() {
        std::fs::create_dir("output").expect("create output failed");
    }
    println!("{:?}", std::env::args());
    for arg in &std::env::args().collect::<Vec<_>>()[1..] {
        println!("run task {}", arg);
        Task::from_json(&arg).run();
    }
}
