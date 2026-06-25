use serde::{Deserialize, Serialize};

mod runner;

pub use runner::{resume_automation_on_startup, SchedulerRunnerHandle};

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
#[serde(rename_all = "camelCase")]
pub struct SchedulerStatus {
    pub running: bool,
    pub current_task: Option<String>,
    pub completed_tasks: Vec<String>,
    pub last_error: Option<String>,
}

pub struct SchedulerState {
    pub status: SchedulerStatus,
}

impl Default for SchedulerState {
    fn default() -> Self {
        Self {
            status: SchedulerStatus::default(),
        }
    }
}

impl SchedulerState {
    pub fn start(&mut self, tasks: &[String]) {
        self.status.running = true;
        self.status.completed_tasks.clear();
        self.status.last_error = None;
        self.status.current_task = tasks.first().cloned();
    }

    pub fn advance(&mut self, tasks: &[String]) {
        if let Some(current) = self.status.current_task.take() {
            self.status.completed_tasks.push(current);
        }
        let done = self.status.completed_tasks.len();
        self.status.current_task = tasks.get(done).cloned();
        if self.status.current_task.is_none() {
            self.status.running = false;
        }
    }

    pub fn stop(&mut self) {
        self.status.running = false;
        self.status.current_task = None;
    }
}
