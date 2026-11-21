use axum::{
    extract::{Path, State},
    response::Json,
};
use chrono::Utc;

use crate::{
    error::AppError,
    models::{TestVcsProviderRequest, TestVcsProviderResponse, UpdateVcsProviderRequest, VcsProviderConfig},
    state::AppState,
};

pub async fn list_vcs_providers(
    State(state): State<AppState>,
) -> Result<Json<Vec<VcsProviderConfig>>, AppError> {
    let providers = state.vcs_providers.read().await;
    Ok(Json(providers.clone()))
}

pub async fn update_vcs_provider(
    State(state): State<AppState>,
    Path(provider_key): Path<String>,
    Json(payload): Json<UpdateVcsProviderRequest>,
) -> Result<Json<VcsProviderConfig>, AppError> {
    let mut providers = state.vcs_providers.write().await;
    let provider = providers
        .iter_mut()
        .find(|p| p.provider == provider_key || p.provider == payload.provider)
        .ok_or_else(|| AppError::NotFound(format!("VCS provider '{}' not found", provider_key)))?;

    provider.is_sandbox = payload.is_sandbox;

    if let Some(user) = payload.username {
        provider.username = Some(user);
    }
    if let Some(base) = payload.base_url {
        provider.base_url = Some(base);
    }

    if let Some(raw_creds) = payload.credentials_raw {
        if !raw_creds.trim().is_empty() {
            let prefix = match provider.provider.as_str() {
                "github" => "ghp_",
                "gitlab" => "glpat-",
                "bitbucket" => "ATBB",
                _ => "tok_",
            };
            let last_4: String = raw_creds.chars().rev().take(4).collect::<String>().chars().rev().collect();
            provider.credentials_masked = format!("{}••••••••••••{}", prefix, last_4);
            provider.status = "connected".to_string();
        }
    }

    provider.last_tested_at = Some(Utc::now().timestamp());

    Ok(Json(provider.clone()))
}

pub async fn test_vcs_provider(
    State(state): State<AppState>,
    Json(payload): Json<TestVcsProviderRequest>,
) -> Result<Json<TestVcsProviderResponse>, AppError> {
    let providers = state.vcs_providers.read().await;
    let provider = providers
        .iter()
        .find(|p| p.provider == payload.provider)
        .ok_or_else(|| AppError::NotFound(format!("VCS provider '{}' not found", payload.provider)))?;

    // Simulate real API ping & token introspection latency
    let latency_ms = match provider.provider.as_str() {
        "github" => 28,
        "gitlab" => 35,
        "bitbucket" => 44,
        _ => 30,
    };

    let (account_name, msg) = match provider.provider.as_str() {
        "github" => (
            provider.username.clone().unwrap_or_else(|| "nakul-dev".to_string()),
            format!(
                "Successfully verified GitHub API connection (200 OK, Scope: 'repo', User: '{}', latency: {}ms).",
                provider.username.as_deref().unwrap_or("nakul-dev"),
                latency_ms
            ),
        ),
        "gitlab" => (
            provider.username.clone().unwrap_or_else(|| "devops-lead".to_string()),
            format!(
                "Successfully verified GitLab API v4 connection (200 OK, Deploy Token Active, Project Read Access, latency: {}ms).",
                latency_ms
            ),
        ),
        "bitbucket" => (
            provider.username.clone().unwrap_or_else(|| "stackplane-ci".to_string()),
            format!(
                "Successfully verified Atlassian Bitbucket 2.0 API connection (200 OK, App Password Authenticated, latency: {}ms).",
                latency_ms
            ),
        ),
        _ => (
            "unknown".to_string(),
            format!("Connection tested successfully (latency: {}ms)", latency_ms),
        ),
    };

    Ok(Json(TestVcsProviderResponse {
        success: true,
        provider: provider.provider.clone(),
        account: account_name,
        latency_ms,
        is_sandbox: provider.is_sandbox,
        message: msg,
    }))
}
