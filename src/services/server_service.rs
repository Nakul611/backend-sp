use crate::models::Server;
use chrono::Utc;

pub struct ServerService;

impl ServerService {
    pub fn is_node_healthy(server: &Server) -> bool {
        server.status == "active" && server.last_error.is_none()
    }

    pub fn calculate_uptime_hours(server: &Server) -> f64 {
        let now = Utc::now().timestamp();
        let diff = (now - server.created_at).max(0);
        ((diff as f64 / 3600.0) * 10.0).round() / 10.0
    }
}
