use std::collections::HashMap;
use std::fs;
use std::path::{Path, PathBuf};

use chrono::{Duration, NaiveDate, Utc};
use serde::de::DeserializeOwned;

#[derive(Clone, PartialEq, Default)]
pub enum DateRange {
    Last7Days,
    Last30Days,
    #[default]
    AllTime,
}

impl DateRange {
    fn cutoff_ms(&self) -> i64 {
        let window = match self {
            DateRange::Last7Days => Some(Duration::days(7)),
            DateRange::Last30Days => Some(Duration::days(30)),
            DateRange::AllTime => None,
        };
        window.map_or(0, |w| Utc::now().timestamp_millis() - w.num_milliseconds())
    }

    fn filter<'a>(&self, messages: &'a [MessageData]) -> impl Iterator<Item = &'a MessageData> {
        let cutoff = self.cutoff_ms();
        messages.iter().filter(move |msg| msg.timestamp >= cutoff)
    }
}

#[derive(Clone, Debug)]
pub struct MessageData {
    pub timestamp: i64,
    pub model_id: String,
    pub cost: f64,
    pub input_tokens: u64,
    pub output_tokens: u64,
    pub repo_path: String,
}

impl MessageData {
    pub fn scan_opencode() -> Vec<Self> {
        let home = std::env::var("HOME").unwrap_or_default();
        let base = PathBuf::from(home).join(".local/share/opencode/storage");
        let project_map = load_project_map(&base.join("project"));
        let sessions = load_session_dirs(&base.join("session"), &project_map);

        message_files_in_subdirs(&base.join("message"))
            .filter_map(|path| read_json::<MessageJson>(&path))
            .filter_map(|msg| msg.into_assistant_data(&sessions))
            .collect()
    }
}

#[derive(Clone, Debug, Default, PartialEq)]
pub struct DailyStats {
    pub input: u64,
    pub output: u64,
    pub cost: f64,
    pub messages: usize,
}

impl DailyStats {
    fn accumulate(&mut self, msg: &MessageData) {
        self.input += msg.input_tokens;
        self.output += msg.output_tokens;
        self.cost += msg.cost;
        self.messages += 1;
    }

    fn merge(&mut self, other: &Self) {
        self.input += other.input;
        self.output += other.output;
        self.cost += other.cost;
        self.messages += other.messages;
    }
}

#[derive(Clone, Debug, Default, PartialEq)]
pub struct RepoStats {
    pub path: String,
    pub name: String,
    pub total_input: u64,
    pub total_output: u64,
    pub total_cost: f64,
    pub message_count: usize,
    pub model_breakdown: HashMap<String, u64>,
    pub daily: HashMap<NaiveDate, DailyStats>,
    pub daily_sorted: Vec<(NaiveDate, DailyStats)>,
    pub models_sorted: Vec<(String, u64)>,
}

impl RepoStats {
    fn for_path(path: &str) -> Self {
        Self {
            path: path.to_string(),
            name: Path::new(path)
                .file_name()
                .map(|name| name.to_string_lossy().into_owned())
                .unwrap_or_default(),
            ..Default::default()
        }
    }

    fn accumulate(&mut self, msg: &MessageData) {
        self.total_input += msg.input_tokens;
        self.total_output += msg.output_tokens;
        self.total_cost += msg.cost;
        self.message_count += 1;
        *self
            .model_breakdown
            .entry(msg.model_id.clone())
            .or_insert(0) += msg.input_tokens + msg.output_tokens;

        let date = chrono::DateTime::from_timestamp_millis(msg.timestamp)
            .map(|dt| dt.date_naive())
            .unwrap_or_else(|| Utc::now().date_naive());
        self.daily.entry(date).or_default().accumulate(msg);
    }

    fn finalize_sorted(&mut self) {
        let mut daily: Vec<_> = self
            .daily
            .iter()
            .map(|(date, stats)| (*date, stats.clone()))
            .collect();
        daily.sort_by_key(|(date, _)| *date);
        self.daily_sorted = daily;

        let mut models: Vec<_> = self
            .model_breakdown
            .iter()
            .map(|(model, tokens)| (model.clone(), *tokens))
            .collect();
        models.sort_by(|left, right| right.1.cmp(&left.1));
        self.models_sorted = models;
    }

    fn merge(&mut self, other: &Self) {
        self.total_input += other.total_input;
        self.total_output += other.total_output;
        self.total_cost += other.total_cost;
        self.message_count += other.message_count;
        for (model, tokens) in &other.model_breakdown {
            *self.model_breakdown.entry(model.clone()).or_insert(0) += *tokens;
        }
        for (date, daily) in &other.daily {
            self.daily.entry(*date).or_default().merge(daily);
        }
    }

    pub fn aggregate(messages: &[MessageData], date_range: &DateRange) -> Vec<Self> {
        let mut by_repo: HashMap<String, RepoStats> = HashMap::new();
        for msg in date_range.filter(messages) {
            by_repo
                .entry(msg.repo_path.clone())
                .or_insert_with(|| Self::for_path(&msg.repo_path))
                .accumulate(msg);
        }
        let mut repos: Vec<Self> = by_repo.into_values().collect();
        repos.sort_by(|left, right| left.path.cmp(&right.path));
        for repo in &mut repos {
            repo.finalize_sorted();
        }
        repos
    }

    pub fn for_dashboard(repos: &[Self], selected_repo: Option<&str>) -> Self {
        if let Some(repo) = selected_repo {
            return repos
                .iter()
                .find(|stats| stats.path == repo)
                .cloned()
                .unwrap_or_default();
        }
        let mut combined = repos.iter().fold(
            Self {
                path: "All Repositories".to_string(),
                name: "All Repositories".to_string(),
                ..Default::default()
            },
            |mut acc, repo| {
                acc.merge(repo);
                acc
            },
        );
        combined.finalize_sorted();
        combined
    }
}

#[derive(serde::Deserialize)]
struct MessageJson {
    #[serde(rename = "sessionID")]
    session_id: String,
    role: String,
    #[serde(rename = "modelID")]
    model_id: Option<String>,
    cost: Option<f64>,
    tokens: Option<TokenJson>,
    path: Option<PathJson>,
    time: Option<TimeJson>,
}

impl MessageJson {
    fn into_assistant_data(self, sessions: &HashMap<String, String>) -> Option<MessageData> {
        if self.role != "assistant" {
            return None;
        }
        let repo_path = self
            .path
            .map(|path| path.cwd)
            .or_else(|| sessions.get(&self.session_id).cloned())
            .unwrap_or_default();
        let tokens = self.tokens.unwrap_or_default();
        Some(MessageData {
            timestamp: self.time.map(|time| time.created).unwrap_or(0),
            model_id: self.model_id.unwrap_or_default(),
            cost: self.cost.unwrap_or(0.0),
            input_tokens: tokens.input,
            output_tokens: tokens.output,
            repo_path,
        })
    }
}

#[derive(serde::Deserialize, Default)]
struct TokenJson {
    input: u64,
    output: u64,
}

#[derive(serde::Deserialize)]
struct PathJson {
    cwd: String,
}

#[derive(serde::Deserialize)]
struct TimeJson {
    created: i64,
}

#[derive(serde::Deserialize)]
struct SessionJson {
    id: String,
    directory: String,
    #[serde(rename = "projectID")]
    project_id: Option<String>,
}

#[derive(serde::Deserialize)]
struct ProjectJson {
    id: String,
    worktree: Option<String>,
}

fn read_json<T: DeserializeOwned>(path: &Path) -> Option<T> {
    let content = fs::read_to_string(path).ok()?;
    serde_json::from_str(&content).ok()
}

fn json_files_in(dir: &Path) -> impl Iterator<Item = PathBuf> {
    fs::read_dir(dir)
        .into_iter()
        .flatten()
        .flatten()
        .map(|entry| entry.path())
        .filter(|path| path.extension().and_then(|ext| ext.to_str()) == Some("json"))
}

fn json_files_in_subdirs(dir: &Path) -> impl Iterator<Item = PathBuf> {
    fs::read_dir(dir)
        .into_iter()
        .flatten()
        .flatten()
        .map(|entry| entry.path())
        .filter(|path| path.is_dir())
        .flat_map(|sub| json_files_in(&sub).collect::<Vec<_>>())
}

fn message_files_in_subdirs(dir: &Path) -> impl Iterator<Item = PathBuf> {
    json_files_in_subdirs(dir).filter(|path| {
        path.file_name()
            .and_then(|name| name.to_str())
            .is_some_and(|name| name.starts_with("msg_"))
    })
}

fn load_project_map(dir: &Path) -> HashMap<String, String> {
    json_files_in(dir)
        .filter_map(|path| read_json::<ProjectJson>(&path))
        .filter_map(|project| project.worktree.map(|worktree| (project.id, worktree)))
        .collect()
}

fn load_session_dirs(dir: &Path, project_map: &HashMap<String, String>) -> HashMap<String, String> {
    json_files_in_subdirs(dir)
        .filter_map(|path| read_json::<SessionJson>(&path))
        .map(|session| {
            let resolved = if session.directory.is_empty() {
                session
                    .project_id
                    .as_ref()
                    .and_then(|pid| project_map.get(pid))
                    .cloned()
                    .unwrap_or_default()
            } else {
                session.directory
            };
            (session.id, resolved)
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_scan_data() {
        let messages = MessageData::scan_opencode();
        println!("Found {} messages", messages.len());
        assert!(!messages.is_empty(), "Should find some messages");
    }
}
