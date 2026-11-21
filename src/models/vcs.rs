use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct VcsProviderConfig {
    pub id: String,
    pub provider: String,          // "github", "gitlab", "bitbucket"
    pub name: String,              // "GitHub", "GitLab", "Atlassian Bitbucket"
    pub status: String,            // "connected", "unconfigured", "error"
    pub is_sandbox: bool,          // true if in sandbox test mode
    pub username: Option<String>,
    pub base_url: Option<String>,  // e.g. "https://github.com" or self-hosted GitLab
    pub credentials_masked: String,
    pub auth_type: String,         // "token", "deploy_key", "oauth"
    pub last_ping_ms: Option<u32>,
    pub last_tested_at: Option<i64>,
}

#[derive(Debug, Deserialize, Clone)]
pub struct UpdateVcsProviderRequest {
    pub provider: String,
    pub is_sandbox: bool,
    pub username: Option<String>,
    pub base_url: Option<String>,
    pub credentials_raw: Option<String>,
}

#[derive(Debug, Deserialize, Clone)]
pub struct TestVcsProviderRequest {
    pub provider: String,
    pub base_url: Option<String>,
}

#[derive(Debug, Serialize, Clone)]
pub struct TestVcsProviderResponse {
    pub success: bool,
    pub provider: String,
    pub account: String,
    pub latency_ms: u32,
    pub is_sandbox: bool,
    pub message: String,
}

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct GitWebhookPayload {
    pub repository_url: Option<String>,
    pub git_url: Option<String>,
    pub branch: Option<String>,
    pub r#ref: Option<String>,
    pub commit_sha: Option<String>,
    pub commit_message: Option<String>,
    pub author_name: Option<String>,
    pub sender: Option<String>,
}

#[derive(Debug, Serialize, Clone)]
pub struct GitWebhookResponse {
    pub success: bool,
    pub matched_project_id: Option<String>,
    pub matched_project_name: Option<String>,
    pub target_branch: String,
    pub commit_sha: String,
    pub servers_dispatched: usize,
    pub execution_command: String,
    pub message: String,
}
