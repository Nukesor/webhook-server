use ::actix::prelude::*;
use ::log::info;

use crate::task::NewTask;

#[derive(Default)]
pub struct QueueActor;

impl Actor for QueueActor {
    type Context = SyncContext<Self>;

    fn started(&mut self, _: &mut SyncContext<Self>) {
        info!("Background task actor started up")
    }
}

impl Handler<NewTask> for QueueActor {
    type Result = ();

    fn handle(&mut self, task: NewTask, context: &mut SyncContext<Self>) {
        info!("Got new Task: {}", task.id);
    }
}
