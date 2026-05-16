use std::collections::HashMap;
use std::fs;
use std::path::PathBuf;

#[derive(Clone, PartialEq, Default)]
pub enum DateRange {
    Last7Days,
    Last30Days,
    #[default]
    AllTime,
}

#[derive(Clone, Debug)]
pub struct MessageData {
    pub timestamp: u64,
    pub model_id: String,
    pub provider_id: String,
    pub cost: f64,
    pub input_tokens: u64,
    pub output_tokens: u64,
    pub reasoning_tokens: u64,
    pub cache_read: u64,
    pub cache_write: u64,
    pub repo_path: String,
}

#[derive(Clone, Debug, Default, PartialEq)]
pub struct DailyStats {
    pub input: u64,
    pub output: u64,
    pub cost: f64,
    pub messages: usize,
}

#[derive(Clone, Debug, Default)]
pub struct RepoStats {
    pub path: String,
    pub name: String,
    pub total_input: u64,
    pub total_output: u64,
    pub total_cost: f64,
    pub message_count: usize,
    pub model_breakdown: HashMap<String, u64>,
    pub daily: HashMap<chrono::NaiveDate, DailyStats>,
}

#[derive(serde::Deserialize)]
struct MessageJson {
    #[serde(rename = "sessionID")]
    session_id: String,
    role: String,
    #[serde(rename = "modelID")]
    model_id: Option<String>,
    #[serde(rename = "providerID")]
    provider_id: Option<String>,
    cost: Option<f64>,
    tokens: Option<TokenJson>,
    path: Option<PathJson>,
    time: Option<TimeJson>,
}

#[derive(serde::Deserialize)]
struct TokenJson {
    input: u64,
    output: u64,
    reasoning: u64,
    cache: Option<CacheJson>,
}

#[derive(serde::Deserialize)]
struct CacheJson {
    read: u64,
    write: u64,
}

#[derive(serde::Deserialize)]
struct PathJson {
    cwd: String,
}

#[derive(serde::Deserialize)]
struct TimeJson {
    created: u64,
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

impl Default for TokenJson {
    fn default() -> Self {
        TokenJson {
            input: 0,
            output: 0,
            reasoning: 0,
            cache: None,
        }
    }
}

pub fn scan_opencode_data() -> Vec<MessageData> {
    let home = std::env::var("HOME").unwrap_or_default();
    println!("Scanning data in home directory: {}", home);
    let base = PathBuf::from(&home).join(".local/share/opencode/storage");
    let message_dir = base.join("message");
    let session_dir = base.join("session");
    let project_dir = base.join("project");

    let mut project_map: HashMap<String, String> = HashMap::new();

    if let Ok(entries) = fs::read_dir(&project_dir) {
        for entry in entries.flatten() {
            let path = entry.path();
            if path.extension().and_then(|e| e.to_str()) == Some("json") {
                if let Ok(content) = fs::read_to_string(&path) {
                    if let Ok(project) = serde_json::from_str::<ProjectJson>(&content) {
                        if let Some(worktree) = project.worktree {
                            project_map.insert(project.id, worktree);
                        }
                    }
                }
            }
        }
    }

    let mut session_dirs: HashMap<String, String> = HashMap::new();

    if let Ok(entries) = fs::read_dir(&session_dir) {
        for entry in entries.flatten() {
            let path = entry.path();
            if path.is_dir() {
                if let Ok(files) = fs::read_dir(&path) {
                    for file in files.flatten() {
                        let file_path = file.path();
                        if file_path.extension().and_then(|e| e.to_str()) == Some("json") {
                            if let Ok(content) = fs::read_to_string(&file_path) {
                                if let Ok(session) = serde_json::from_str::<SessionJson>(&content)
                                {
                                    let dir = if session.directory.is_empty() {
                                        session
                                            .project_id
                                            .as_ref()
                                            .and_then(|pid| project_map.get(pid))
                                            .cloned()
                                            .unwrap_or_default()
                                    } else {
                                        session.directory
                                    };
                                    session_dirs.insert(session.id, dir);
                                }
                            }
                        }
                    }
                }
            }
        }
    }

    let mut messages = Vec::new();

    if let Ok(entries) = fs::read_dir(&message_dir) {
        for entry in entries.flatten() {
            let path = entry.path();
            if path.is_dir() {
                if let Ok(files) = fs::read_dir(&path) {
                    for file in files.flatten() {
                        let file_path = file.path();
                        if file_path
                            .file_name()
                            .and_then(|n| n.to_str())
                            .map(|n| n.starts_with("msg_") && n.ends_with(".json"))
                            .unwrap_or(false)
                        {
                            if let Ok(content) = fs::read_to_string(&file_path) {
                                if let Ok(msg) = serde_json::from_str::<MessageJson>(&content) {
                                    if msg.role == "assistant" {
                                        let repo_path = msg
                                            .path
                                            .as_ref()
                                            .map(|p| p.cwd.clone())
                                            .or_else(|| {
                                                session_dirs.get(&msg.session_id).cloned()
                                            })
                                            .unwrap_or_default();

                                        let tokens = msg.tokens.unwrap_or_default();
                                        let cache = tokens.cache.unwrap_or(CacheJson {
                                            read: 0,
                                            write: 0,
                                        });

                                        messages.push(MessageData {
                                            timestamp: msg.time.map(|t| t.created).unwrap_or(0),
                                            model_id: msg.model_id.unwrap_or_default(),
                                            provider_id: msg.provider_id.unwrap_or_default(),
                                            cost: msg.cost.unwrap_or(0.0),
                                            input_tokens: tokens.input,
                                            output_tokens: tokens.output,
                                            reasoning_tokens: tokens.reasoning,
                                            cache_read: cache.read,
                                            cache_write: cache.write,
                                            repo_path,
                                        });
                                    }
                                }
                            }
                        }
                    }
                }
            }
        }
    }

    messages
}

pub fn aggregate_repos(messages: &[MessageData], date_range: &DateRange) -> Vec<RepoStats> {
    let filtered = filter_by_date_range(messages, date_range);

    let mut repo_map: HashMap<String, RepoStats> = HashMap::new();
    for msg in filtered {
        let stats = repo_map
            .entry(msg.repo_path.clone())
            .or_insert_with(|| RepoStats {
                path: msg.repo_path.clone(),
                name: std::path::Path::new(&msg.repo_path)
                    .file_name()
                    .map(|n| n.to_string_lossy().to_string())
                    .unwrap_or_default(),
                ..Default::default()
            });
        stats.total_input += msg.input_tokens;
        stats.total_output += msg.output_tokens;
        stats.total_cost += msg.cost;
        stats.message_count += 1;
        let total_tokens = msg.input_tokens + msg.output_tokens;
        *stats
            .model_breakdown
            .entry(msg.model_id.clone())
            .or_insert(0) += total_tokens;

        let date = chrono::DateTime::from_timestamp_millis(msg.timestamp as i64)
            .map(|dt| dt.date_naive())
            .unwrap_or_else(|| chrono::Utc::now().date_naive());
        let daily = stats.daily.entry(date).or_default();
        daily.input += msg.input_tokens;
        daily.output += msg.output_tokens;
        daily.cost += msg.cost;
        daily.messages += 1;
    }

    let mut repos: Vec<RepoStats> = repo_map.into_values().collect();
    repos.sort_by(|a, b| a.path.cmp(&b.path));
    println!("Aggregated repositories: {:?}", repos);
    repos
}

pub fn get_dashboard_stats(repos: &[RepoStats], selected_repo: Option<&str>) -> RepoStats {
    if let Some(repo) = selected_repo {
        repos
            .iter()
            .find(|r| r.path == repo)
            .cloned()
            .unwrap_or_default()
    } else {
        let mut combined = RepoStats::default();
        combined.path = "All Repositories".to_string();
        combined.name = "All Repositories".to_string();
        for repo in repos {
            combined.total_input += repo.total_input;
            combined.total_output += repo.total_output;
            combined.total_cost += repo.total_cost;
            combined.message_count += repo.message_count;

            for (model, tokens) in &repo.model_breakdown {
                *combined.model_breakdown.entry(model.clone()).or_insert(0) += *tokens;
            }

            for (date, daily) in &repo.daily {
                let d = combined.daily.entry(*date).or_default();
                d.input += daily.input;
                d.output += daily.output;
                d.cost += daily.cost;
                d.messages += daily.messages;
            }
        }
        combined
    }
}

fn filter_by_date_range<'a>(
    messages: &'a [MessageData],
    date_range: &DateRange,
) -> Vec<&'a MessageData> {
    let now = chrono::Utc::now().timestamp_millis() as u64;
    let cutoff = match date_range {
        DateRange::Last7Days => now - 7 * 24 * 60 * 60 * 1000,
        DateRange::Last30Days => now - 30 * 24 * 60 * 60 * 1000,
        DateRange::AllTime => 0,
    };

    messages.iter().filter(|m| m.timestamp >= cutoff).collect()
}

pub fn get_all_repo_paths(messages: &[MessageData]) -> Vec<String> {
    let mut repos: Vec<String> = messages
        .iter()
        .map(|m| m.repo_path.clone())
        .collect::<std::collections::HashSet<_>>()
        .into_iter()
        .collect();
    repos.sort();
    repos
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_scan_data() {
        let messages = scan_opencode_data();
        println!("Found {} messages", messages.len());
        assert!(!messages.is_empty(), "Should find some messages");
    }
}


