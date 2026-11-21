use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct FinancialBurden {
    pub acquired_at: i64,
    pub lifetime_hours: f64,
    pub active_hours: f64,
    pub idle_hours: f64,
    pub active_cost: f64,
    pub idle_cost: f64,
    pub idle_waste_percent: f64,
    pub data_uploaded_gb: f64,
    pub data_downloaded_gb: f64,
    pub upload_cost: f64,
    pub download_cost: f64,
    pub storage_capacity_cost: f64,
    pub total_transfer_cost: f64,
    pub total_accumulated_burden: f64,
    pub efficiency_score: f64,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct ResourceTelemetry {
    pub ram_total_gb: f64,
    pub ram_used_gb: f64,
    pub ram_usage_percent: f64,
    pub storage_total_gb: f64,
    pub storage_used_gb: f64,
    pub storage_usage_percent: f64,
    pub hourly_cost: f64,
    pub instance_type: String,
    pub cpu_cores: u32,
    pub cpu_usage_percent: f64,
    pub fetched_at: i64,
    pub is_live: bool,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub burden: Option<FinancialBurden>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Server {
    pub id: String,
    pub project_id: String,
    pub server_id: String,
    pub name: String,
    pub provider: String,      // "aws", "azure", "gcp", "custom"
    pub resource_type: String, // "ec2", "vm", "s3", "storage"
    pub host_ip: String,
    pub ssh_port: u16,
    pub ssh_user: String,
    pub region: String,
    pub tags: Vec<String>,
    pub status: String, // "active", "idle", "executing", "error", "terminated"
    pub active_since: i64,
    pub last_job_started_at: Option<i64>,
    pub last_job_completed_at: Option<i64>,
    pub idle_since: Option<i64>,
    pub idle_minutes: i64,
    pub last_error: Option<String>,
    pub created_at: i64,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub service_id: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub role: Option<String>, // "frontend", "backend", "database", "scraper", "curator", "cleaner", "storage"
    #[serde(skip_serializing_if = "Option::is_none")]
    pub service_name: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub database_engine: Option<String>, // "PostgreSQL 16", "Redis 7.2", "MongoDB 7.0", "MySQL 8.0"
    #[serde(skip_serializing_if = "Option::is_none")]
    pub port: Option<u16>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub telemetry: Option<ResourceTelemetry>,
}

#[derive(Debug, Deserialize, Clone)]
pub struct CreateServerRequest {
    pub server_id: String,
    pub name: String,
    pub provider: String,
    pub resource_type: String,
    pub host_ip: String,
    pub ssh_port: Option<u16>,
    pub ssh_user: Option<String>,
    pub region: String,
    pub tags: Option<Vec<String>>,
    pub service_id: Option<String>,
    pub role: Option<String>,
    pub service_name: Option<String>,
    pub database_engine: Option<String>,
    pub port: Option<u16>,
}

#[derive(Debug, Deserialize, Clone)]
pub struct BatchCreateServersRequest {
    pub servers: Vec<CreateServerRequest>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct BatchCreateServersResponse {
    pub imported: usize,
    pub servers: Vec<Server>,
}

#[derive(Debug, Deserialize, Clone)]
pub struct ServerTerminalExecRequest {
    pub command: String,
    pub mode: Option<String>, // "bash" or "ssm"
}

#[derive(Debug, Serialize, Clone)]
pub struct ServerTerminalExecResponse {
    pub session_id: String,
    pub server_id: String,
    pub host_ip: String,
    pub command: String,
    pub output: String,
    pub exit_code: i32,
    pub duration_ms: u64,
    pub ssm_tunnel_active: bool,
    pub timestamp: i64,
}
