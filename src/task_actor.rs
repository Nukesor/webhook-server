use ::actix::prelude::*;
use ::log::info;

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
    }
}
