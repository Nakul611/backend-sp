use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct ProjectRepository {
    pub git_url: String,
    pub branch: String,
    pub build_command: Option<String>,
    pub run_command: String,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Project {
    pub id: String,
    pub name: String,
    pub slug: String,
    pub description: String,
    pub status: String, // "open" or "closed"
    pub repository: Option<ProjectRepository>,
    pub default_credential_id: Option<String>,
    pub idle_alert_threshold_minutes: i64,
    pub auto_stop_idle: bool,
    pub budget_cap_usd: Option<f64>,
    pub member_ids: Vec<String>,
    pub server_count: i64,
    pub active_count: i64,
    pub idle_count: i64,
    pub created_at: i64,
    pub updated_at: i64,
}

#[derive(Debug, Deserialize, Clone)]
pub struct UpdateProjectStatusRequest {
    pub status: String,
}

#[derive(Debug, Deserialize, Clone)]
pub struct CreateProjectRequest {
    pub name: String,
    pub description: Option<String>,
    pub git_url: Option<String>,
    pub branch: Option<String>,
    pub run_command: Option<String>,
    pub credential_id: Option<String>,
    pub idle_alert_threshold_minutes: Option<i64>,
}
