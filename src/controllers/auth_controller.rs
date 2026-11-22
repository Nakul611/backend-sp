use axum::{
    extract::State,
    http::{header::AUTHORIZATION, HeaderMap, StatusCode},
    Json,
};

use crate::{
    auth::{
        generate_tokens, verify_token, AuthResponse, AuthUser, LoginRequest, RefreshRequest,
        RefreshResponse, ACCESS_TOKEN_EXPIRY_SECONDS,
    },
    error::AppError,
    state::AppState,
};

pub async fn login(
    State(state): State<AppState>,
    Json(payload): Json<LoginRequest>,
) -> Result<Json<AuthResponse>, AppError> {
    let users = state.users.read().await;

    let matched = users.iter().find(|(u, pass)| {
        (u.email.eq_ignore_ascii_case(&payload.email) || u.username.eq_ignore_ascii_case(&payload.email))
            && crate::helpers::crypto::verify_password(&payload.password, pass)
    });

    let (user, _) = matched.ok_or_else(|| {
        AppError::Unauthorized("Invalid username/email or password.".to_string())
    })?;

    let (access_token, refresh_token) = generate_tokens(user)?;

    Ok(Json(AuthResponse {
        access_token,
        refresh_token,
        token_type: "Bearer".to_string(),
        expires_in: ACCESS_TOKEN_EXPIRY_SECONDS,
        user: user.clone(),
    }))
}

pub async fn refresh_token_handler(
    State(state): State<AppState>,
    Json(payload): Json<RefreshRequest>,
) -> Result<Json<RefreshResponse>, AppError> {
    let claims = verify_token(&payload.refresh_token, "refresh")?;
    let users = state.users.read().await;

    let user = users
        .iter()
        .find(|(u, _)| u.id == claims.sub)
        .map(|(u, _)| u.clone())
        .unwrap_or_else(|| AuthUser {
            id: claims.sub,
            username: claims.username,
            email: claims.email,
            role: claims.role,
            assigned_projects: vec!["all".to_string()],
        });

    let (new_access_token, _) = generate_tokens(&user)?;

    Ok(Json(RefreshResponse {
        access_token: new_access_token,
        token_type: "Bearer".to_string(),
        expires_in: ACCESS_TOKEN_EXPIRY_SECONDS,
    }))
}

pub async fn get_current_user(
    State(state): State<AppState>,
    headers: HeaderMap,
) -> Result<Json<AuthUser>, AppError> {
    let auth_header = headers
        .get(AUTHORIZATION)
        .and_then(|h| h.to_str().ok())
        .ok_or_else(|| AppError::Unauthorized("Missing Authorization header".to_string()))?;

    let token = auth_header
        .strip_prefix("Bearer ")
        .ok_or_else(|| AppError::Unauthorized("Invalid Authorization format, expected 'Bearer <token>'".to_string()))?;

    let claims = verify_token(token, "access")?;
    let users = state.users.read().await;

    let user = users
        .iter()
        .find(|(u, _)| u.id == claims.sub)
        .map(|(u, _)| u.clone())
        .unwrap_or_else(|| AuthUser {
            id: claims.sub,
            username: claims.username,
            email: claims.email,
            role: claims.role,
            assigned_projects: vec!["all".to_string()],
        });

    Ok(Json(user))
}

pub async fn logout() -> StatusCode {
    StatusCode::OK
}
