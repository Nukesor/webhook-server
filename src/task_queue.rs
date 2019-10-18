use ::std::collections::{HashMap, BTreeMap};

use ::serde::Serialize;
use ::chrono::prelude::*;
use ::serde;

use crate::messages::NewTask;
use crate::messages::TaskCompleted;
use crate::settings::Settings;

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


#[derive(Serialize, Debug, Clone)]
pub struct Task {
    pub webhook_name: String,
    pub task_id: i32,
    pub command: String,
    pub cwd: String,
    pub exit_code: Option<i32>,
    pub stdout: Option<String>,
    pub stderr: Option<String>,
    pub added_at: DateTime<Local>,
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

    pub fn add_task(&mut self, incoming: NewTask) {
        let name = incoming.webhook_name.clone();
        let settings = self.settings.get_webhook_by_name(&name).unwrap();

        // Check whether the task should be added
        // single only allows a single task either running or queued
        // deploy allows a single running AND a single queued
        // parallel always adds the task
        println!("Wut");
        match settings.mode.as_str() {
            "single" => {
                if *self.queued_count.get(&name).unwrap() > 0 {
                    println!("Wot");
                    return
                } else if *self.running_count.get(&name).unwrap() > 0 {
                    println!("Wit");
                    return
                }
            },
            "deploy" => {
                if *self.queued_count.get(&name).unwrap() > 0 {
                    println!("Wöt");
                    return
                }
            },
            "parallel" => {},
            _ => return,

        }

        self.max_id += 1;
        let task = Task::new(incoming, self.max_id);
        self.queued.insert(self.max_id, task);
        self.increment_queued_count(&name);

        println!("TaskQueue: {:?}", serde_json::to_string(&self).unwrap());
    }

    /// Dispatch new tasks to the task executor pool
    pub fn get_tasks_for_dispatch(&mut self) -> Vec<Task> {
        let global_open_slots = self.settings.workers - self.running.len();
        // The pool is already fully saturated
        if global_open_slots == 0 {
            return Vec::new();
        }

        // Get id's of suitable tasks
        let mut task_ids: Vec<i32> = Vec::new();
        for (id, task) in self.queued.iter() {
            // We already got enough new tasks
            if task_ids.len() == global_open_slots {
                break;
            }
            task_ids.push(*id);
        }

        // Move the selected tasks from queued to running state
        let mut tasks = Vec::new();
        for task_id in task_ids {
            let task = self.queued.remove(&task_id).unwrap();
            self.decrement_queued_count(&task.webhook_name);
            self.increment_running_count(&task.webhook_name);

            self.running.insert(task_id, task.clone());
            tasks.push(task);
        }

        println!("TaskQueue: {:?}", serde_json::to_string(&self).unwrap());

        tasks
    }

    pub fn finish_task(&mut self, completed: TaskCompleted) {
        let mut task = self.running.remove(&completed.task_id).unwrap();
        task.exit_code = Some(completed.exit_code);
        task.stdout = Some(completed.stdout);
        task.stderr = Some(completed.stderr);

        self.decrement_running_count(&task.webhook_name);
        self.finished.insert(task.task_id, task);
    }

    fn increment_queued_count(&mut self, name: &String) {
        let value = self.queued_count.get(name).unwrap().clone();
        self.queued_count.insert(name.clone(), value + 1);
    }

    fn decrement_queued_count(&mut self, name: &String) {
        let value = self.queued_count.get(name).unwrap().clone();
        self.queued_count.insert(name.clone(), value - 1);
    }

    fn increment_running_count(&mut self, name: &String) {
        let value = self.running_count.get(name).unwrap().clone();
        self.running_count.insert(name.clone(), value + 1);
    }

    fn decrement_running_count(&mut self, name: &String) {
        let value = self.running_count.get(name).unwrap().clone();
        self.running_count.insert(name.clone(), value - 1);
    }
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
