use ::actix::prelude::*;
use ::actix_web::*;
use ::actix_web::middleware::Logger;
use ::log::info;
use ::simplelog::{SimpleLogger, LevelFilter, Config};
use ::std::{thread, time};
use ::std::collections::HashMap;

mod task;
mod queue_actor;

use crate::task::{NewTask};
use crate::queue_actor::{QueueActor};

/// Index route
/// Depending on the
fn webhook(data: web::Data<AppState>, webhook_id: String) -> &'static str {
    let new_task = NewTask {
        id: webhook_id,
        parameters: HashMap::new(),
    };
    data.queue_actor.do_send(new_task);
    info!("New webhook received");

    "Background task started\n"
}

struct AppState {
    queue_actor: Addr<QueueActor>,
}

fn main() {
    let sys = System::new("background-worker-example");
    let queue_actor = SyncArbiter::start(1, move || QueueActor::default());

    let _ = SimpleLogger::init(LevelFilter::Info, Config::default());

    HttpServer::new(move ||
            App::new()
                .data(AppState {
                    queue_actor: queue_actor.clone()
                })
                .wrap(Logger::default())
                .service(web::resource("/webhook/{id}").to(webhook))
        )
        .bind("127.0.0.1:8000")
        .unwrap()
        .start();

    info!("Starting up server on 127.0.0.1:8000");
    sys.run();
}
