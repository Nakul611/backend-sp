pub mod bitbucket;
pub mod github;
pub mod gitlab;

pub use bitbucket::BitbucketService;
pub use github::GitHubService;
pub use gitlab::GitLabService;

pub struct VcsService;

impl VcsService {
    pub fn get_provider_ping(provider: &str) -> u32 {
        match provider.to_lowercase().as_str() {
            "github" => GitHubService::get_ping_latency_ms(),
            "gitlab" => GitLabService::get_ping_latency_ms(),
            "bitbucket" => BitbucketService::get_ping_latency_ms(),
            _ => 30,
        }
    }

    pub fn sanitize_repository_url(provider: &str, url: &str) -> String {
        match provider.to_lowercase().as_str() {
            "gitlab" => GitLabService::sanitize_url(url),
            "bitbucket" => BitbucketService::sanitize_url(url),
            _ => GitHubService::sanitize_url(url),
        }
    }
}
