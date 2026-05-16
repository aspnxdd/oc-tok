use freya::radio::RadioChannel;

use crate::data::{DateRange, MessageData, RepoStats};

#[derive(Default, Clone)]
pub struct AppState {
    pub messages: Vec<MessageData>,
    pub selected_repo: Option<String>,
    pub date_range: DateRange,
    pub is_loading: bool,
    pub last_updated: Option<String>,
    pub repo_stats: Vec<RepoStats>,
    pub dashboard_stats: RepoStats,
}

impl AppState {
    pub fn refresh_all(&mut self) {
        self.repo_stats = RepoStats::aggregate(&self.messages, &self.date_range);
        self.refresh_dashboard();
    }

    pub fn refresh_dashboard(&mut self) {
        self.dashboard_stats =
            RepoStats::for_dashboard(&self.repo_stats, self.selected_repo.as_deref());
    }
}

#[derive(PartialEq, Eq, Clone, Copy, Hash)]
pub enum AppChannel {
    All,
}

impl RadioChannel<AppState> for AppChannel {}
