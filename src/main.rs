mod messages;
mod scheduler;
mod settings;
mod task;
mod web;

use ::actix::prelude::*;
use ::anyhow::Result;
use ::log::info;
use ::simplelog::{Config, LevelFilter, SimpleLogger};

use crate::scheduler::Scheduler;
use crate::settings::Settings;
use crate::task::executor::TaskExecutor;
use crate::web::init_web_server;

fn main() -> Result<()> {
    // Beautify panics for better debug output.
    better_panic::install();

    let system = System::new();
    let _ = SimpleLogger::init(LevelFilter::Info, Config::default());
    let settings = Settings::new()?;

    // Create actix actors and path the reference of the task_executor to the scheduler
    // The scheduler will send it's own address in the StartTask payload for bidirectional communication
    info!("Starting task executor with {} workers", settings.workers);
    let task_executor =
        SyncArbiter::start(settings.workers, move || TaskExecutor { scheduler: None });

    info!("Initializing scheduler");
    let scheduler = Scheduler::new(task_executor.clone(), settings.clone());

    info!("Starting scheduler");
    let scheduler = scheduler.start();

    info!("Starting web server");
    init_web_server(scheduler, settings)?;

    info!("Done");
    let _ = system.run();

    Ok(())
}
