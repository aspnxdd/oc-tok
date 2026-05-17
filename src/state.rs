use freya::radio::RadioChannel;

use crate::data::{DateRange, MessageData, RepoStats};

#[derive(Default, Clone)]
pub struct AppState {
    pub messages: Vec<MessageData>,
    pub date_range: DateRange,
    pub is_loading: bool,
    pub last_updated: Option<String>,
    pub repo_stats: Vec<RepoStats>,
}

impl AppState {
    pub fn refresh_all(&mut self) {
        self.repo_stats = RepoStats::aggregate(&self.messages, &self.date_range);
    }
}

#[derive(PartialEq, Eq, Clone, Copy, Hash)]
pub enum AppChannel {
    All,
}

impl RadioChannel<AppState> for AppChannel {}
