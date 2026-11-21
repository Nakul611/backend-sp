use axum::{
    extract::{Path, State},
    http::StatusCode,
    Json,
};

use crate::{
    controllers::project_controller,
    error::AppError,
    models::{CreateProjectRequest, Project, UpdateProjectStatusRequest},
    state::AppState,
};

pub async fn list_projects(state: State<AppState>) -> Result<Json<Vec<Project>>, AppError> {
    project_controller::list_projects(state).await
}

pub async fn get_project(state: State<AppState>, id: Path<String>) -> Result<Json<Project>, AppError> {
    project_controller::get_project(state, id).await
}

pub async fn create_project(
    state: State<AppState>,
    payload: Json<CreateProjectRequest>,
) -> Result<(StatusCode, Json<Project>), AppError> {
    project_controller::create_project(state, payload).await
}

pub async fn update_project_status(
    state: State<AppState>,
    id: Path<String>,
    payload: Json<UpdateProjectStatusRequest>,
) -> Result<Json<Project>, AppError> {
    project_controller::update_project_status(state, id, payload).await
}
