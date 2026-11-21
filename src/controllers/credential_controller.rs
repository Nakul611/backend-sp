use axum::{
    extract::State,
    http::{header::AUTHORIZATION, HeaderMap, StatusCode},
    Json,
};
use chrono::Utc;
use uuid::Uuid;

use crate::{
    auth::verify_token,
    error::AppError,
    models::{CreateCredentialRequest, Credential},
    state::AppState,
};

fn require_admin_role(headers: &HeaderMap) -> Result<(), AppError> {
    let auth_header = headers
        .get(AUTHORIZATION)
        .and_then(|h| h.to_str().ok())
        .ok_or_else(|| AppError::Unauthorized("Missing Authorization header".to_string()))?;

    let token = auth_header
        .strip_prefix("Bearer ")
        .ok_or_else(|| AppError::Unauthorized("Invalid Authorization format".to_string()))?;

    let claims = verify_token(token, "access")?;

    if claims.role != "admin" {
        return Err(AppError::Forbidden(
            "Access denied: Only administrators can view or upload PEM credentials.".to_string(),
        ));
    }

    Ok(())
}

pub async fn list_credentials(
    State(state): State<AppState>,
    headers: HeaderMap,
) -> Result<Json<Vec<Credential>>, AppError> {
    require_admin_role(&headers)?;

    let creds = state.credentials.read().await;
    Ok(Json(creds.clone()))
}

pub async fn create_credential(
    State(state): State<AppState>,
    headers: HeaderMap,
    Json(payload): Json<CreateCredentialRequest>,
) -> Result<(StatusCode, Json<Credential>), AppError> {
    require_admin_role(&headers)?;

    let now = Utc::now().timestamp();
    let cred_id = format!("cred-{}", Uuid::new_v4().simple());

    // Compute public key fingerprint and encrypt private key material for backend vault storage
    let fingerprint = crate::helpers::crypto::compute_key_fingerprint(&payload.pem_content);
    let _encrypted_vault_entry = crate::helpers::crypto::encrypt_pem_vault(&payload.pem_content);

    let new_cred = Credential {
        id: cred_id,
        name: payload.name,
        provider: payload.provider,
        credential_type: "ssh_pem".to_string(),
        ssh_user: payload.ssh_user,
        fingerprint: Some(fingerprint),
        is_encrypted: true,
        created_at: now,
    };

    let mut creds = state.credentials.write().await;
    creds.push(new_cred.clone());

    Ok((StatusCode::CREATED, Json(new_cred)))
}
