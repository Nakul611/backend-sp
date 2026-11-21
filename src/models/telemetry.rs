use serde::{Deserialize, Serialize};

#[derive(Debug, Deserialize, Clone)]
pub struct TelemetryHeartbeatRequest {
    pub server_id: String,
    pub project_id: String,
    pub cpu_usage_percent: f64,
    pub ram_used_gb: f64,
    pub ram_total_gb: f64,
    pub storage_used_gb: f64,
    pub storage_total_gb: f64,
    pub network_rx_mb: Option<f64>,
    pub network_tx_mb: Option<f64>,
    pub active_job: Option<String>,
    pub uptime_seconds: Option<u64>,
}

#[derive(Debug, Serialize, Clone)]
pub struct TelemetryHeartbeatResponse {
    pub status: String,
    pub server_id: String,
    pub is_live: bool,
    pub message: String,
    pub timestamp: i64,
}

#[derive(Debug, Serialize, Clone)]
pub struct BootstrapScriptResponse {
    pub script: String,
    pub one_liner: String,
    pub project_id: String,
    pub server_id: String,
    pub vcs_provider: String,
}
