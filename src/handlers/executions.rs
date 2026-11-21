use axum::{
    extract::{Path, State},
    http::StatusCode,
    Json,
};
use chrono::Utc;
use uuid::Uuid;

use crate::{
    error::AppError,
    models::{DispatchExecutionRequest, ExecutionRun, WsEvent},
    state::AppState,
};

pub async fn dispatch_execution(
    State(state): State<AppState>,
    Path(project_id): Path<String>,
    Json(payload): Json<DispatchExecutionRequest>,
) -> Result<(StatusCode, Json<ExecutionRun>), AppError> {
    let now = Utc::now().timestamp();
    let mut servers = state.servers.write().await;

    let target_servers: Vec<String> = servers
        .iter()
        .filter(|s| s.project_id == project_id)
        .filter(|s| match &payload.server_ids {
            Some(ids) => ids.contains(&s.id),
            None => true,
        })
        .map(|s| s.id.clone())
        .collect();

    let target_count = target_servers.len();

    // Mark target servers as executing & reset idle
    for server in servers.iter_mut() {
        if target_servers.contains(&server.id) {
            server.status = "executing".to_string();
            server.last_job_started_at = Some(now);
            server.idle_since = None;
            server.idle_minutes = 0;
        }
    }

    let execution_id = format!("exec-{}", Uuid::new_v4().simple());

    let run = ExecutionRun {
        id: execution_id.clone(),
        project_id: project_id.clone(),
        triggered_by: "member-admin".to_string(),
        command: payload.command.clone(),
        status: "running".to_string(),
        total_servers: target_count,
        success_count: target_count,
        failed_count: 0,
        started_at: now,
        finished_at: None,
    };

    // Broadcast WebSocket execution update
    state.broadcast_event(&WsEvent {
        r#type: "EXECUTION_UPDATE".to_string(),
        project_id: Some(project_id),
        server_id: None,
        status: Some("running".to_string()),
        idle_minutes: Some(0),
        message: Some(format!(
            "Dispatched command '{}' across {} servers in parallel.",
            payload.command, target_count
        )),
        telemetry: None,
    });

    Ok((StatusCode::ACCEPTED, Json(run)))
}
