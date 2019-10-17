use ::actix::prelude::*;
use ::log::info;

use crate::messages::*;
use crate::settings::Settings;
use crate::task_executor::TaskExecutor;

pub struct Scheduler {
    pub task_executor: Addr<TaskExecutor>,
    pub own_addr: Option<Addr<Self>>,
    pub settings: Settings,
    pub current_workers: i32,
}

impl Actor for Scheduler {
    type Context = Context<Self>;

    fn started(&mut self, context: &mut Self::Context) {
        self.own_addr = Some(context.address());
        info!("Queue management actor started up");
    }
}

impl Handler<NewTask> for Scheduler {
    type Result = ();

    fn handle(&mut self, new_task: NewTask, _context: &mut Self::Context) {
        info!("Got new Task: {}", new_task.webhook_name);

        self.dispatch_task(new_task);
    }
}

impl Handler<TaskCompleted> for Scheduler {
    type Result = ();

    fn handle(&mut self, message: TaskCompleted, _context: &mut Self::Context) {
        info!("Finished task: {} - {}", message.webhook_name, message.task_id);
    }
}

impl Scheduler {
    pub fn new(task_executor: Addr<TaskExecutor>, settings: Settings) -> Self {
        Scheduler {
            task_executor: task_executor.clone(),
            own_addr: None,
            settings: settings.clone(),
            current_workers: 0,
        }
    }

    fn dispatch_task(&mut self, new_task: NewTask) {
        let addr = self.own_addr.as_ref().unwrap().clone();

        let start_task = StartTask {
            webhook_name: new_task.webhook_name,
            task_id: 0,
            command: new_task.command,
            cwd: new_task.cwd,
            scheduler: addr,
        };

        self.task_executor.do_send(start_task);
    }
}
