use axum::{
    extract::{Request, State},
    http::header::AUTHORIZATION,
    middleware::Next,
    response::Response,
};

use crate::{
    auth::{verify_token, AuthUser},
    error::AppError,
    state::AppState,
};

/// Middleware ensuring the request contains a valid Bearer JWT access token.
/// Attaches the authenticated `AuthUser` to request extensions for downstream handlers.
pub async fn require_auth(
    State(state): State<AppState>,
    mut req: Request,
    next: Next,
) -> Result<Response, AppError> {
    let auth_header = req
        .headers()
        .get(AUTHORIZATION)
        .and_then(|h| h.to_str().ok())
        .ok_or_else(|| {
            AppError::Unauthorized("Authorization header missing. Expected 'Bearer <token>'.".to_string())
        })?;

    let token = auth_header
        .strip_prefix("Bearer ")
        .ok_or_else(|| {
            AppError::Unauthorized("Invalid Authorization header format. Expected 'Bearer <token>'.".to_string())
        })?;

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

    req.extensions_mut().insert(user);

    Ok(next.run(req).await)
}

/// Middleware ensuring the authenticated user has 'admin' privileges.
pub async fn require_admin(
    req: Request,
    next: Next,
) -> Result<Response, AppError> {
    let user = req
        .extensions()
        .get::<AuthUser>()
        .ok_or_else(|| AppError::Unauthorized("Authentication required prior to privilege check.".to_string()))?;

    if user.role != "admin" {
        return Err(AppError::Forbidden(
            "Access denied: Administrator privileges required for this resource.".to_string(),
        ));
    }

    Ok(next.run(req).await)
}
