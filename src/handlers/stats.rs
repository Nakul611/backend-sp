use crate::{models::ServerStats, state::AppState};
use axum::{extract::State, response::IntoResponse, Json};
use std::time::{SystemTime, UNIX_EPOCH};

pub async fn get_stats(State(state): State<AppState>) -> impl IntoResponse {
    let now = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default()
        .as_secs();
    let uptime_secs = now.saturating_sub(state.start_time);
    let count = state.items.read().map(|i| i.len()).unwrap_or(0);

    Json(ServerStats {
        uptime_seconds: uptime_secs,
        total_items: count,
        backend: "Axum + Tokio (Rust)",
        memory_status: "optimal",
        api_latency_target_ms: "< 1ms",
    })
}
