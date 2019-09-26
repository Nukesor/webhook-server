use ::actix::prelude::*;
use ::std::collections::HashMap;

use crate::queue_actor::QueueActor;

#[derive(Message)]
pub struct NewTask {
    pub id: String,
    pub parameters: HashMap<String, String>,
    pub command: String,
}

#[derive(Message)]
pub struct StartTask {
    pub command: String,
    pub cwd: String,
    pub queue_actor: Addr<QueueActor>,
}
