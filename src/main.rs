mod task;
mod template;
mod queue_actor;
mod task_actor;
mod web;

use ::actix::prelude::*;
use ::simplelog::{Config, LevelFilter, SimpleLogger};

use crate::queue_actor::QueueActor;
use crate::web::init_web_server;


fn main() {
    let sys = System::new("webhook-server");
    let queue_actor = SyncArbiter::start(1, move || QueueActor::default());

    let _ = SimpleLogger::init(LevelFilter::Info, Config::default());

    init_web_server(queue_actor);

    sys.run();
}
