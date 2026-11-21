use serde::{Deserialize, Serialize};
use super::server::ResourceTelemetry;

#[derive(Debug, Serialize, Deserialize, Clone, Default)]
pub struct WsEvent {
    pub r#type: String,
    pub project_id: Option<String>,
    pub server_id: Option<String>,
    pub status: Option<String>,
    pub idle_minutes: Option<i64>,
    pub message: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub telemetry: Option<ResourceTelemetry>,
}
