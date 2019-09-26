mod messages;
mod queue_actor;
mod task_actor;
mod template;
mod web;

use ::actix::prelude::*;
use ::simplelog::{Config, LevelFilter, SimpleLogger};

use crate::queue_actor::QueueActor;
use crate::task_actor::TaskActor;
use crate::web::init_web_server;

fn main() {
    let sys = System::new("webhook-server");
    let task_actor = SyncArbiter::start(8, move || TaskActor { queue_actor: None });
    let queue_actor = QueueActor {
        task_actor: task_actor.clone(),
        own_addr: None,
    };

    let _ = SimpleLogger::init(LevelFilter::Info, Config::default());

    init_web_server(queue_actor.start());

    sys.run();
}
