use ::actix::prelude::*;
use ::log::info;
use ::subprocess::{CaptureData, Exec, ExitStatus, Redirection};

use crate::messages::*;
use crate::queue_actor::QueueActor;

pub struct TaskActor {
    pub queue_actor: Option<Addr<QueueActor>>,
}

impl Actor for TaskActor {
    type Context = SyncContext<Self>;

    fn started(&mut self, _context: &mut Self::Context) {}
}

impl Handler<StartTask> for TaskActor {
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
                info!("Error during task execution: {}", error);
                return;
            }
        }

        match captured_data.exit_status {
            ExitStatus::Exited(_exit_code) => {}
            ExitStatus::Signaled(_signal) => {}
            ExitStatus::Other(_other) => {}
            ExitStatus::Undetermined => {}
        }

        let output = captured_data.stdout_str();

        info!("{}", output);
    }
}
