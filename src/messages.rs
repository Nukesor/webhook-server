use ::chrono::prelude::*;
use ::actix::prelude::*;
use ::std::collections::HashMap;

use crate::scheduler::Scheduler;

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
    pub task_id: i32,
    pub command: String,
    pub cwd: String,
    pub scheduler: Addr<Scheduler>,
}

#[derive(Message)]
pub struct TaskCompleted {
    pub webhook_name: String,
    pub task_id: i32,
    pub exit_code: i32,
    pub stdout: String,
    pub stderr: String,
}
