use ::chrono::prelude::*;
use ::serde::Serialize;
use ::std::collections::{BTreeMap, HashMap};

use crate::messages::NewTask;
use crate::messages::TaskCompleted;
use crate::settings::Settings;

/// The Task is a simple struct to store all information about the state of a task.
#[derive(Serialize, Debug, Clone)]
pub struct Task {
    pub webhook_name: String,
    pub task_id: i32,
    pub command: String,
    pub cwd: String,
    pub exit_code: Option<u32>,
    pub stdout: Option<String>,
    pub stderr: Option<String>,
    pub added_at: DateTime<Local>,
}

impl Task {
    pub fn new(new_task: NewTask, id: i32) -> Task {
        Task {
            webhook_name: new_task.webhook_name,
            task_id: id,
            command: new_task.command,
            cwd: new_task.cwd,
            exit_code: None,
            stdout: None,
            stderr: None,
            added_at: new_task.added_at,
        }
    }
}

/// The TaskQueue represents the current state of all tasks and is also
/// responsible for the management of these.
/// In here it's decided whether a new task should be added to the queue
/// and a task is ready to be dispatched.
#[derive(Serialize, Debug)]
pub struct TaskQueue {
    max_id: i32,
    #[serde(skip_serializing)]
    settings: Settings,

    running_count: HashMap<String, i32>,
    queued_count: HashMap<String, i32>,

    queued: BTreeMap<i32, Task>,
    running: BTreeMap<i32, Task>,
    finished: BTreeMap<i32, Task>,
}

impl TaskQueue {
    pub fn new(settings: Settings) -> TaskQueue {
        let mut queued_count = HashMap::new();
        let mut running_count = HashMap::new();

        // Initialize the counter for each
        for webhook in &settings.webhooks {
            queued_count.insert(webhook.name.clone(), 0);
            running_count.insert(webhook.name.clone(), 0);
        }

        TaskQueue {
            max_id: 0,
            settings: settings.clone(),

            queued_count: queued_count,
            running_count: running_count,

            queued: BTreeMap::new(),
            running: BTreeMap::new(),
            finished: BTreeMap::new(),
        }
    }

    /// Decide whether a new task should be added to the queue
    pub fn add_task(&mut self, incoming: NewTask) {
        let name = incoming.webhook_name.clone();
        let settings = self.settings.get_webhook_by_name(&name).unwrap();

        // Check whether the task should be added
        // single only allows a single task either running or queued
        // deploy allows a single running AND a single queued
        // parallel always adds the task
        match settings.mode.as_str() {
            "single" => {
                if *self.queued_count.get(&name).unwrap() > 0 {
                    return;
                } else if *self.running_count.get(&name).unwrap() > 0 {
                    return;
                }
            }
            "deploy" => {
                if *self.queued_count.get(&name).unwrap() > 0 {
                    return;
                }
            }
            "parallel" => {}
            _ => return,
        }

        self.max_id += 1;
        let task = Task::new(incoming, self.max_id);
        self.queued.insert(self.max_id, task);
        self.increment_queued_count(&name);
    }

    /// Get all tasks that can be dispatched right now.
    /// This also updates the current state of these tasks.
    /// Only call, if these tasks really will be dispatched!
    pub fn get_tasks_for_dispatch(&mut self) -> Vec<Task> {
        let global_open_slots = self.settings.workers - self.running.len();
        // The pool is already fully saturated
        if global_open_slots == 0 {
            return Vec::new();
        }

        // Get id's of suitable tasks
        let mut tasks = Vec::new();
        let ids: Vec<i32> = self.queued.keys().cloned().collect();
        for id in ids {
            // We already got enough new tasks
            if tasks.len() == global_open_slots {
                break;
            }

            // Get the next task
            let task = self.queued.remove(&id).unwrap();
            let name = task.webhook_name.clone();
            let settings = self.settings.get_webhook_by_name(&name).unwrap();

            match settings.mode.as_str() {
                // Always schedule single, since there cannot be more than one in queued/running anyway
                "single" => {
                    self.schedule_task(task, &mut tasks);
                    continue;
                }
                // Deploy can always be scheduled, if there isn't already one running
                "deploy" => {
                    if *self.running_count.get(&name).unwrap() == 0 {
                        self.schedule_task(task, &mut tasks);
                        continue;
                    }
                }
                // Parallel can be scheduled, if there are less than the max
                // specified parallel parallel_processes running
                "parallel" => {
                    if *self.running_count.get(&name).unwrap() < settings.parallel_processes {
                        self.schedule_task(task, &mut tasks);
                        continue;
                    }
                }
                _ => {}
            }

            // Tasks couldn't be scheduled yet. Put it back into the queue
            self.queued.insert(task.task_id, task);
        }

        tasks
    }

    /// A task has finished. Remove it from running and insert new data from the finished process
    pub fn finish_task(&mut self, completed: TaskCompleted) {
        let mut task = self.running.remove(&completed.task_id).unwrap();
        task.exit_code = Some(completed.exit_code);
        task.stdout = Some(completed.stdout);
        task.stderr = Some(completed.stderr);

        self.decrement_running_count(&task.webhook_name);
        self.finished.insert(task.task_id, task);
    }

    /// Helper to easily change the state of a task to running
    fn schedule_task(&mut self, task: Task, tasks: &mut Vec<Task>) {
        let name = task.webhook_name.clone();
        // Decrement queued count
        let queued = self.queued_count.get(&name).unwrap().clone();
        self.queued_count.insert(name.clone(), queued - 1);

        // Increment running count
        let running = self.running_count.get(&name).unwrap().clone();
        self.running_count.insert(name, running + 1);

        // Push task into running
        self.running.insert(task.task_id, task.clone());
        tasks.push(task)
    }

    fn increment_queued_count(&mut self, name: &String) {
        let value = self.queued_count.get(name).unwrap().clone();
        self.queued_count.insert(name.clone(), value + 1);
    }

    fn decrement_running_count(&mut self, name: &String) {
        let value = self.running_count.get(name).unwrap().clone();
        self.running_count.insert(name.clone(), value - 1);
    }
}
