use ::actix::prelude::*;
use ::log::info;

use crate::task::NewTask;


#[derive(Default)]
pub struct TaskActor;

impl Actor for TaskActor {
    type Context = SyncContext<Self>;
}

impl Handler<NewTask> for TaskActor {
    type Result = ();

    fn handle(&mut self, task: NewTask, context: &mut SyncContext<Self>) {
        info!("Got new Task: {}", task.id);
    }
}
