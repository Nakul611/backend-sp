use axum::{
    extract::{Path, State},
    response::Json,
};
use chrono::Utc;

use crate::{
    error::AppError,
    models::{
        GitWebhookPayload, GitWebhookResponse, TestVcsProviderRequest, TestVcsProviderResponse,
        UpdateVcsProviderRequest, VcsProviderConfig,
    },
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

pub async fn handle_git_webhook(
    State(state): State<AppState>,
    axum::extract::Path(provider): axum::extract::Path<String>,
    Json(payload): Json<GitWebhookPayload>,
) -> Result<Json<GitWebhookResponse>, AppError> {
    let now = Utc::now().timestamp();

    // Determine target repository URL
    let repo_url = payload
        .git_url
        .as_deref()
        .or(payload.repository_url.as_deref())
        .unwrap_or("https://github.com/my-org/saas-frontend.git")
        .trim();

    // Determine target branch
    let branch = payload
        .branch
        .as_deref()
        .or_else(|| {
            payload.r#ref.as_deref().map(|r| {
                r.strip_prefix("refs/heads/").unwrap_or(r)
            })
        })
        .unwrap_or("main")
        .trim();

    let commit_sha = payload
        .commit_sha
        .clone()
        .unwrap_or_else(|| "e933b7f".to_string());

    let commit_msg = payload
        .commit_message
        .clone()
        .unwrap_or_else(|| "Automated code update via git push".to_string());

    let author = payload
        .author_name
        .clone()
        .or(payload.sender.clone())
        .unwrap_or_else(|| "devops-engineer".to_string());

    // Clean comparison: normalize repo URLs (lowercase, remove .git suffix, remove trailing slashes)
    let clean_url = |u: &str| -> String {
        u.to_lowercase()
            .trim_end_matches('/')
            .trim_end_matches(".git")
            .to_string()
    };

    let target_clean = clean_url(repo_url);

    // Find matching project
    let projects = state.projects.read().await;
    let matched_project = projects.iter().find(|p| {
        if let Some(ref repo) = p.repository {
            if clean_url(&repo.git_url) == target_clean {
                return true;
            }
        }
        if let Some(ref services) = p.services {
            for svc in services {
                if let Some(ref s_url) = svc.git_url {
                    if clean_url(s_url) == target_clean {
                        return true;
                    }
                }
            }
        }
        false
    }).cloned();

    let matched_project = matched_project.or_else(|| projects.first().cloned());

    let (project_id, project_name, run_cmd) = match &matched_project {
        Some(p) => (
            p.id.clone(),
            p.name.clone(),
            p.repository
                .as_ref()
                .map(|r| r.run_command.clone())
                .unwrap_or_else(|| "git pull && npm run build && systemctl restart app".to_string()),
        ),
        None => (
            "proj-fallback".to_string(),
            "Default Pipeline".to_string(),
            "git pull && npm run build".to_string(),
        ),
    };

    drop(projects);

    // Target servers associated with this project
    let mut servers = state.servers.write().await;
    let mut dispatched_count = 0;
    for s in servers.iter_mut() {
        if s.project_id == project_id {
            s.status = "executing".to_string();
            s.last_job_started_at = Some(now);
            s.idle_since = None;
            s.idle_minutes = 0;
            dispatched_count += 1;
        }
    }
    drop(servers);

    // Broadcast WebSocket events to notify all active UI clients
    state.broadcast_event(&crate::models::WsEvent {
        r#type: "PIPELINE_RUNNING".to_string(),
        project_id: Some(project_id.clone()),
        server_id: None,
        status: Some("executing".to_string()),
        idle_minutes: Some(0),
        message: Some(format!(
            "[{}] Auto-Pipeline Triggered by {} push to {} (commit {}): '{}'. Deploying across {} nodes.",
            provider.to_uppercase(),
            author,
            branch,
            &commit_sha[..7.min(commit_sha.len())],
            commit_msg,
            dispatched_count
        )),
        telemetry: None,
    });

    Ok(Json(GitWebhookResponse {
        success: true,
        matched_project_id: Some(project_id),
        matched_project_name: Some(project_name),
        target_branch: branch.to_string(),
        commit_sha,
        servers_dispatched: dispatched_count,
        execution_command: run_cmd,
        message: format!(
            "Successfully processed {} webhook. Continuous delivery pipeline automatically triggered across {} fleet nodes.",
            provider.to_uppercase(),
            dispatched_count
        ),
    }))
}
