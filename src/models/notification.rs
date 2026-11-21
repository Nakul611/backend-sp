use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct FleetNotification {
    pub id: String,
    pub project_id: String,
    pub r#type: String,     // "server_idle_alert", "job_completed", "server_error"
    pub severity: String, // "info", "warning", "critical"
    pub title: String,
    pub message: String,
    pub server_id: Option<String>,
    pub host_ip: Option<String>,
    pub idle_minutes: Option<i64>,
    pub read: bool,
    pub created_at: i64,
}

#[derive(Debug, Deserialize, Clone)]
pub struct WhatsAppTestRequest {
    pub recipient_phone: String,
    pub message: Option<String>,
}

#[derive(Debug, Serialize, Clone)]
pub struct WhatsAppTestResponse {
    pub success: bool,
    pub message_sid: String,
    pub recipient: String,
    pub status: String,
    pub timestamp: i64,
}
