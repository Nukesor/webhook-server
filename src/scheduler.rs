use ::actix::prelude::*;
use ::log::info;

use crate::messages::*;
use crate::settings::Settings;
use crate::task_executor::TaskExecutor;
use crate::task_queue::{TaskQueue, Task};

pub struct Scheduler {
    pub task_executor: Addr<TaskExecutor>,
    pub own_addr: Option<Addr<Self>>,
    pub settings: Settings,
    pub current_workers: i32,
    task_queue: TaskQueue,
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

        self.task_queue.add_task(new_task);
        let tasks = self.task_queue.get_tasks_for_dispatch();

        println!("Got tasks: {:?}", tasks);
        for task in tasks {
            self.dispatch_task(task);
        }
    }
}

impl Handler<TaskCompleted> for Scheduler {
    type Result = ();

    fn handle(&mut self, message: TaskCompleted, _context: &mut Self::Context) {
        info!("Finished task: {} - {}", message.webhook_name, message.task_id);
        self.task_queue.finish_task(message);
    }
}

impl Scheduler {
    pub fn new(task_executor: Addr<TaskExecutor>, settings: Settings) -> Self {
        Scheduler {
            task_executor: task_executor.clone(),
            own_addr: None,
            settings: settings.clone(),
            current_workers: 0,
            task_queue: TaskQueue::new(settings),
        }
    }

    fn dispatch_task(&mut self, task: Task) {
        let addr = self.own_addr.as_ref().unwrap().clone();

        let message = StartTask {
            webhook_name: task.webhook_name,
            task_id: task.task_id,
            command: task.command,
            cwd: task.cwd,
            scheduler: addr,
        };

        self.task_executor.do_send(message );
    }
}
