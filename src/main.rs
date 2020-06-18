extern crate ray_tracing;

use ray_tracing::utils::Task;

fn main() {
    std::fs::create_dir("output");
    Task::from_json("small").run();
}
