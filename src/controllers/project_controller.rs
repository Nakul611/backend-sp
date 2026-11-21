use axum::{
    extract::{Path, State},
    http::StatusCode,
    Json,
};
use chrono::Utc;
use uuid::Uuid;

use crate::{
    error::AppError,
    models::{CreateProjectRequest, Project, ProjectRepository, UpdateProjectStatusRequest},
    state::AppState,
};

pub async fn list_projects(State(state): State<AppState>) -> Result<Json<Vec<Project>>, AppError> {
    let mut projects = state.projects.read().await.clone();
    let servers = state.servers.read().await;

    // Recalculate server metrics per project
    for project in projects.iter_mut() {
        let proj_servers: Vec<_> = servers.iter().filter(|s| s.project_id == project.id).collect();
        project.server_count = proj_servers.len() as i64;
        project.active_count = proj_servers
            .iter()
            .filter(|s| s.status == "active" || s.status == "executing")
            .count() as i64;
        project.idle_count = proj_servers.iter().filter(|s| s.status == "idle").count() as i64;
    }

    Ok(Json(projects))
}

pub async fn get_project(
    State(state): State<AppState>,
    Path(id): Path<String>,
) -> Result<Json<Project>, AppError> {
    let projects = state.projects.read().await.clone();
    let servers = state.servers.read().await;

    let mut project = projects
        .into_iter()
        .find(|p| p.id == id)
        .ok_or_else(|| AppError::NotFound(format!("Project '{}' not found", id)))?;

    let proj_servers: Vec<_> = servers.iter().filter(|s| s.project_id == project.id).collect();
    project.server_count = proj_servers.len() as i64;
    project.active_count = proj_servers
        .iter()
        .filter(|s| s.status == "active" || s.status == "executing")
        .count() as i64;
    project.idle_count = proj_servers.iter().filter(|s| s.status == "idle").count() as i64;

    Ok(Json(project))
}

pub async fn create_project(
    State(state): State<AppState>,
    Json(payload): Json<CreateProjectRequest>,
) -> Result<(StatusCode, Json<Project>), AppError> {
    let now = Utc::now().timestamp();
    let project_id = format!("proj-{}", Uuid::new_v4().simple());
    let slug = payload
        .name
        .to_lowercase()
        .replace(|c: char| !c.is_alphanumeric(), "-");

    let repo = payload.git_url.map(|url| ProjectRepository {
        git_url: url,
        branch: payload.branch.unwrap_or_else(|| "main".to_string()),
        build_command: None,
        run_command: payload.run_command.unwrap_or_else(|| "./run_job.sh".to_string()),
    });

    let new_project = Project {
        id: project_id,
        name: payload.name,
        slug,
        description: payload.description.unwrap_or_default(),
        status: "open".to_string(),
        repository: repo,
        default_credential_id: payload.credential_id,
        idle_alert_threshold_minutes: payload.idle_alert_threshold_minutes.unwrap_or(15),
        auto_stop_idle: false,
        budget_cap_usd: Some(10000.0),
        member_ids: vec!["member-admin".to_string()],
        server_count: 0,
        active_count: 0,
        idle_count: 0,
        created_at: now,
        updated_at: now,
    };

    let mut projects = state.projects.write().await;
    projects.push(new_project.clone());

    Ok((StatusCode::CREATED, Json(new_project)))
}

pub async fn update_project_status(
    State(state): State<AppState>,
    Path(id): Path<String>,
    Json(payload): Json<UpdateProjectStatusRequest>,
) -> Result<Json<Project>, AppError> {
    let now = Utc::now().timestamp();
    let mut projects = state.projects.write().await;

    let project = projects
        .iter_mut()
        .find(|p| p.id == id)
        .ok_or_else(|| AppError::NotFound(format!("Project '{}' not found", id)))?;

    project.status = payload.status;
    project.updated_at = now;

    let updated = project.clone();
    Ok(Json(updated))
}
