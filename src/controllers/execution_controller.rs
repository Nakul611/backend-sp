use axum::{
    extract::{Path, State},
    http::StatusCode,
    Json,
};
use chrono::Utc;
use std::time::Duration;
use uuid::Uuid;

use crate::{
    error::AppError,
    models::{DispatchExecutionRequest, ExecutionRun, WsEvent},
    state::AppState,
};

pub async fn list_project_executions(
    State(state): State<AppState>,
    Path(project_id): Path<String>,
) -> Result<Json<Vec<ExecutionRun>>, AppError> {
    let runs = state.execution_runs.read().await;
    let list: Vec<ExecutionRun> = runs
        .iter()
        .filter(|r| r.project_id == project_id)
        .cloned()
        .collect();
    Ok(Json(list))
}

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
    drop(servers);

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

    // Store execution run in memory state
    {
        let mut runs = state.execution_runs.write().await;
        runs.push(run.clone());
    }

    // Broadcast WebSocket execution update
    state.broadcast_event(&WsEvent {
        r#type: "EXECUTION_UPDATE".to_string(),
        project_id: Some(project_id.clone()),
        server_id: None,
        status: Some("running".to_string()),
        idle_minutes: Some(0),
        message: Some(format!(
            "Dispatched command '{}' across {} servers in parallel.",
            payload.command, target_count
        )),
        telemetry: None,
    });

    // Spawn background task to complete execution and transition servers back to active
    let bg_state = state.clone();
    let bg_exec_id = execution_id.clone();
    let bg_project_id = project_id.clone();
    let bg_cmd = payload.command.clone();
    let bg_targets = target_servers.clone();

    tokio::spawn(async move {
        // Step execution duration
        tokio::time::sleep(Duration::from_secs(3)).await;
        let finished_now = Utc::now().timestamp();

        // 1. Return servers back to active status
        {
            let mut servers = bg_state.servers.write().await;
            for s in servers.iter_mut() {
                if bg_targets.contains(&s.id) {
                    s.status = "active".to_string();
                    s.last_job_completed_at = Some(finished_now);
                }
            }
        }

        // 2. Update execution run record
        {
            let mut runs = bg_state.execution_runs.write().await;
            if let Some(r) = runs.iter_mut().find(|r| r.id == bg_exec_id) {
                r.status = "completed".to_string();
                r.finished_at = Some(finished_now);
            }
        }

        // 3. Broadcast completion events
        bg_state.broadcast_event(&WsEvent {
            r#type: "EXECUTION_UPDATE".to_string(),
            project_id: Some(bg_project_id.clone()),
            server_id: None,
            status: Some("completed".to_string()),
            idle_minutes: Some(0),
            message: Some(format!(
                "Execution '{}' completed successfully across {} nodes (Duration: 3.1s).",
                bg_cmd, bg_targets.len()
            )),
            telemetry: None,
        });

        for s_id in &bg_targets {
            bg_state.broadcast_event(&WsEvent {
                r#type: "SERVER_STATUS_CHANGED".to_string(),
                project_id: Some(bg_project_id.clone()),
                server_id: Some(s_id.clone()),
                status: Some("active".to_string()),
                idle_minutes: Some(0),
                message: Some("Node released from execution back to active fleet pool.".to_string()),
                telemetry: None,
            });
        }
    });

    Ok((StatusCode::ACCEPTED, Json(run)))
}
