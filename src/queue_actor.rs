use ::actix::prelude::*;
use ::log::info;
use ::std::{thread, time};

use crate::task::{NewTask};

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

    fn handle(&mut self, _: NewTask, _: &mut SyncContext<Self>) {
        info!("Got new Task");
    }
}
