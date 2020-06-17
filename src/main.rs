extern crate ray_tracing;

use ray_tracing::scene::Task;
use ray_tracing::*;

fn main() {
    let task = Task::from_json("task/example.json");
    let image = task.camera.shoot(&task.scene, task.max_depth);
    image.dump_png(&task.save_path);
}
