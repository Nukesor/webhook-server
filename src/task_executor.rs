use ::actix::prelude::*;
use ::log::{info, warn};
use ::subprocess::{CaptureData, Exec, ExitStatus, Redirection};

use crate::messages::*;
use crate::scheduler::Scheduler;

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

        match captured_data.exit_status {
            ExitStatus::Exited(_exit_code) => {}
            ExitStatus::Signaled(_signal) => {}
            ExitStatus::Other(_other) => {}
            ExitStatus::Undetermined => {}
        }

        let stdout = captured_data.stdout_str();
        let stderr = captured_data.stderr_str();

        info!("{}", stdout);

        let message = TaskCompleted {
            webhook_name: task.webhook_name,
            task_id: task.task_id,
            exit_code: 0,
            stdout: stdout,
            stderr: stderr,
        };

        task.scheduler.do_send(message);
    }
}
