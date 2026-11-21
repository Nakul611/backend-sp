use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct ExecutionRun {
    pub id: String,
    pub project_id: String,
    pub triggered_by: String,
    pub command: String,
    pub status: String, // "queued", "running", "completed", "failed"
    pub total_servers: usize,
    pub success_count: usize,
    pub failed_count: usize,
    pub started_at: i64,
    pub finished_at: Option<i64>,
}

#[derive(Debug, Deserialize, Clone)]
pub struct DispatchExecutionRequest {
    pub command: String,
    pub server_ids: Option<Vec<String>>,
}
