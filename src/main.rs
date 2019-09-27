mod settings;
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
use crate::settings::Settings;

fn main() {
    let sys = System::new("webhook-server");
    let settings = Settings::new().unwrap();

    // Create actix actors and path the reference of the task_actor to the queue_actor
    // The queue_actor will send it's own address in the StartTask payload for bidirectional communication
    let task_actor = SyncArbiter::start(settings.workers, move || TaskActor { queue_actor: None });
    let queue_actor = QueueActor {
        task_actor: task_actor.clone(),
        own_addr: None,
        settings: settings.clone(),
    };

    let _ = SimpleLogger::init(LevelFilter::Info, Config::default());

    init_web_server(queue_actor.start(), settings);

    let _ = sys.run();
}
