use freya::radio::*;

use crate::data::{DateRange, MessageData, RepoStats};

#[derive(Default, Clone)]
pub struct AppState {
    pub messages: Vec<MessageData>,
    pub repos: Vec<String>,
    pub selected_repo: Option<String>,
    pub date_range: DateRange,
    pub is_loading: bool,
    pub last_updated: Option<String>,
    pub repo_stats: Vec<RepoStats>,
    pub dashboard_stats: RepoStats,
}

#[derive(PartialEq, Eq, Clone, Debug, Copy, Hash)]
pub enum AppChannel {
    Data,
    Selection,
}

impl RadioChannel<AppState> for AppChannel {
    fn derive_channel(self, _state: &AppState) -> Vec<Self> {
        match self {
            AppChannel::Data => vec![AppChannel::Data, AppChannel::Selection],
            AppChannel::Selection => vec![AppChannel::Data, AppChannel::Selection],
        }
    }
}

pub fn update_all_stats(state: &mut AppState) {
    state.repos = crate::data::get_all_repo_paths(&state.messages);
    state.repo_stats = crate::data::aggregate_repos(&state.messages, &state.date_range);
    state.dashboard_stats =
        crate::data::get_dashboard_stats(&state.repo_stats, state.selected_repo.as_deref());
}

pub fn update_dashboard_stats(state: &mut AppState) {
    state.dashboard_stats =
        crate::data::get_dashboard_stats(&state.repo_stats, state.selected_repo.as_deref());
}
