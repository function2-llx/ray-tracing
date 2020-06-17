extern crate ray_tracing;

use ray_tracing::utils::Task;

fn main() {
    Task::from_json("task/example.json").run();
}
