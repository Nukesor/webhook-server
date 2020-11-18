use ::serde::Serialize;
use chrono::prelude::*;

use crate::messages::NewTask;

/// The Task is a simple struct to store all information about the state of a task.
#[derive(Serialize, Debug, Clone)]
pub struct Task {
    pub webhook_name: String,
    pub task_id: i32,
    pub command: String,
    pub cwd: String,
    pub exit_code: Option<u32>,
    pub stdout: Option<String>,
    pub stderr: Option<String>,
    pub added_at: DateTime<Local>,
}

impl Task {
    pub fn new(new_task: NewTask, id: i32) -> Task {
        Task {
            webhook_name: new_task.webhook_name,
            task_id: id,
            command: new_task.command,
            cwd: new_task.cwd,
            exit_code: None,
            stdout: None,
            stderr: None,
            added_at: new_task.added_at,
        }
    }
}
