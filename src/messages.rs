use ::chrono::prelude::*;
use ::actix::prelude::*;
use ::std::collections::HashMap;

use crate::queue_actor::QueueActor;

#[derive(Message)]
pub struct NewTask {
    pub webhook_name: String,
    pub parameters: HashMap<String, String>,
    pub cwd: String,
    pub command: String,
    pub added_at: DateTime<Local>,
}

#[derive(Message)]
pub struct StartTask {
    pub webhook_name: String,
    pub task_id: usize,
    pub command: String,
    pub cwd: String,
    pub queue_actor: Addr<QueueActor>,
}

#[derive(Message)]
pub struct TaskCompleted {
    pub webhook_name: String,
    pub task_id: usize,
    pub exit_code: i32,
    pub stdout: String,
    pub stderr: String,
}
