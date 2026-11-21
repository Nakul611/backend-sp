use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct ProjectRepository {
    pub git_url: String,
    pub branch: String,
    pub build_command: Option<String>,
    pub run_command: String,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct ProjectService {
    pub id: String,
    pub name: String,                    // e.g. "Frontend Web", "API Backend", "PostgreSQL Database", "Scraping Fleet"
    pub service_type: String,            // "frontend" | "backend" | "database" | "scraper" | "curator" | "cleaner" | "worker" | "storage"
    #[serde(skip_serializing_if = "Option::is_none")]
    pub git_url: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub branch: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub root_directory: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub build_command: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub run_command: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub port: Option<u16>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub database_engine: Option<String>, // e.g. "PostgreSQL 16", "Redis 7.2", "MongoDB 7.0", "MySQL 8.0"
    pub node_count: u32,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Project {
    pub id: String,
    pub name: String,
    pub slug: String,
    pub description: String,
    pub status: String, // "open" or "closed"
    #[serde(skip_serializing_if = "Option::is_none")]
    pub project_type: Option<String>, // "fullstack" or "mass_pipeline"
    #[serde(skip_serializing_if = "Option::is_none")]
    pub repo_strategy: Option<String>, // "monorepo" or "polyrepo"
    #[serde(skip_serializing_if = "Option::is_none")]
    pub services: Option<Vec<ProjectService>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub domain_guidance: Option<String>,
    pub repository: Option<ProjectRepository>,
    pub default_credential_id: Option<String>,
    pub idle_alert_threshold_minutes: i64,
    pub auto_stop_idle: bool,
    pub budget_cap_usd: Option<f64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub budget_cadence: Option<String>, // "daily" | "weekly" | "monthly" | "yearly"
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
pub struct CreateProjectServiceRequest {
    pub name: String,
    pub service_type: String,
    pub git_url: Option<String>,
    pub branch: Option<String>,
    pub root_directory: Option<String>,
    pub build_command: Option<String>,
    pub run_command: Option<String>,
    pub port: Option<u16>,
    pub database_engine: Option<String>,
    pub node_count: Option<u32>,
}

#[derive(Debug, Deserialize, Clone)]
pub struct CreateProjectRequest {
    pub name: String,
    pub description: Option<String>,
    pub project_type: Option<String>, // "fullstack" or "mass_pipeline"
    pub repo_strategy: Option<String>, // "monorepo" or "polyrepo"
    pub services: Option<Vec<CreateProjectServiceRequest>>,
    pub pipeline_stages: Option<Vec<String>>, // ["scraping", "curating", "cleaning"]
    pub git_url: Option<String>,
    pub branch: Option<String>,
    pub run_command: Option<String>,
    pub credential_id: Option<String>,
    pub idle_alert_threshold_minutes: Option<i64>,
    pub budget_cap_usd: Option<f64>,
    pub budget_cadence: Option<String>, // "daily" | "weekly" | "monthly" | "yearly"
}
