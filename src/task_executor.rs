use ::actix::prelude::*;
use ::log::{info, warn};
use ::subprocess::{CaptureData, Exec, ExitStatus, Redirection};

use crate::messages::*;
use crate::scheduler::Scheduler;

/// The Actor that's actually responsible for executing tasks
pub struct TaskExecutor {
    pub scheduler: Option<Addr<Scheduler>>,
}

impl Actor for TaskExecutor {
    type Context = SyncContext<Self>;

    fn started(&mut self, _context: &mut Self::Context) {}
}

impl Handler<StartTask> for TaskExecutor {
    type Result = ();

    fn handle(&mut self, task: StartTask, _context: &mut Self::Context) {
        info!("Starting Task: {}", task.command);

        let result = Exec::shell(task.command)
            .cwd(task.cwd)
            .stdout(Redirection::Pipe)
            .stderr(Redirection::Pipe)
            .capture();

        let captured_data: CaptureData;

        match result {
            Ok(data) => {
                captured_data = data;
            }
            Err(error) => {
                warn!("Error during task execution: {}", error);
                return;
            }
        }

        let mut exit_code = 1;
        match captured_data.exit_status {
            ExitStatus::Exited(_exit_code) => {
                exit_code = _exit_code;
            }
            ExitStatus::Signaled(_signal) => {}
            ExitStatus::Other(_other) => {}
            ExitStatus::Undetermined => {}
        }

        let stdout = captured_data.stdout_str();
        let stderr = captured_data.stderr_str();

        let message = TaskCompleted {
            webhook_name: task.webhook_name,
            task_id: task.task_id,
            exit_code: exit_code,
            stdout: stdout,
            stderr: stderr,
        };

        task.scheduler.do_send(message);
    }
}
