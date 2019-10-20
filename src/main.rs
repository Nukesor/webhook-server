mod authentication;
mod messages;
mod scheduler;
mod settings;
mod task_executor;
mod task_queue;
mod web;

use ::actix::prelude::*;
use ::anyhow::Result;
use ::simplelog::{Config, LevelFilter, SimpleLogger};

use crate::scheduler::Scheduler;
use crate::settings::Settings;
use crate::task_executor::TaskExecutor;
use crate::web::init_web_server;

fn main() -> Result<()> {
    let sys = System::new("webhook-server");
    let _ = SimpleLogger::init(LevelFilter::Info, Config::default());
    let settings = Settings::new()?;

    // Create actix actors and path the reference of the task_executor to the scheduler
    // The scheduler will send it's own address in the StartTask payload for bidirectional communication
    let task_executor =
        SyncArbiter::start(settings.workers, move || TaskExecutor { scheduler: None });
    let scheduler = Scheduler::new(task_executor.clone(), settings.clone());

    init_web_server(scheduler.start(), settings)?;

    let _ = sys.run();

    Ok(())
}
