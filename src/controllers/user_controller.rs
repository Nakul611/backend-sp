use axum::{
    extract::{Path, State},
    http::StatusCode,
    Json,
};
use serde::Deserialize;
use uuid::Uuid;

use crate::{
    auth::AuthUser,
    error::AppError,
    helpers::crypto::hash_password,
    state::AppState,
};

#[derive(Debug, Deserialize)]
pub struct CreateUserRequest {
    pub username: String,
    pub email: String,
    pub password: String,
    pub role: String, // "admin" | "devops" | "member"
    pub assigned_projects: Option<Vec<String>>,
}

pub async fn list_users(
    State(state): State<AppState>,
) -> Result<Json<Vec<AuthUser>>, AppError> {
    let users = state.users.read().await;
    let list: Vec<AuthUser> = users.iter().map(|(u, _)| u.clone()).collect();
    Ok(Json(list))
}

pub async fn create_user(
    State(state): State<AppState>,
    Json(payload): Json<CreateUserRequest>,
) -> Result<(StatusCode, Json<AuthUser>), AppError> {
    let mut users = state.users.write().await;

    // Check duplicate email or username
    if users.iter().any(|(u, _)| {
        u.email.eq_ignore_ascii_case(&payload.email)
            || u.username.eq_ignore_ascii_case(&payload.username)
    }) {
        return Err(AppError::BadRequest(
            "A user with this email or username already exists.".to_string(),
        ));
    }

    let hashed_pw = hash_password(payload.password.trim());

    let new_user = AuthUser {
        id: format!("user-{}", Uuid::new_v4().simple()),
        username: payload.username.trim().to_string(),
        email: payload.email.trim().to_string(),
        role: payload.role.trim().to_string(),
        assigned_projects: payload.assigned_projects.unwrap_or_else(|| vec!["all".to_string()]),
    };

    users.push((new_user.clone(), hashed_pw));

    Ok((StatusCode::CREATED, Json(new_user)))
}

pub async fn delete_user(
    State(state): State<AppState>,
    Path(id): Path<String>,
) -> Result<StatusCode, AppError> {
    let mut users = state.users.write().await;
    let initial_len = users.len();

    // Prevent deleting the primary admin account
    if id == "user-admin-01" {
        return Err(AppError::Forbidden(
            "Primary administrator account cannot be deleted.".to_string(),
        ));
    }

    users.retain(|(u, _)| u.id != id);

    if users.len() == initial_len {
        return Err(AppError::NotFound(format!("User '{}' not found", id)));
    }

    Ok(StatusCode::NO_CONTENT)
}
