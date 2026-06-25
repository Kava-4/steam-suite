use crate::modules::giveaways::bot::GiveawayBotHandle;
use crate::modules::scheduler::{SchedulerRunnerHandle, SchedulerState};
use crate::modules::settings::AppSettings;
use std::sync::Mutex;

pub struct AppState {
    pub settings: Mutex<AppSettings>,
    pub giveaway_bot: GiveawayBotHandle,
    pub scheduler: Mutex<SchedulerState>,
    pub scheduler_runner: SchedulerRunnerHandle,
}

impl AppState {
    pub fn new() -> Self {
        Self {
            settings: Mutex::new(crate::modules::settings::load_settings()),
            giveaway_bot: GiveawayBotHandle::default(),
            scheduler: Mutex::new(SchedulerState::default()),
            scheduler_runner: SchedulerRunnerHandle::default(),
        }
    }
}
