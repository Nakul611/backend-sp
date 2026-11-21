pub struct GitHubService;

impl GitHubService {
    pub fn get_ping_latency_ms() -> u32 {
        24
    }

    pub fn sanitize_url(raw: &str) -> String {
        let trimmed = raw.trim();
        if trimmed.starts_with("git@github.com:") || trimmed.starts_with("https://github.com/") {
            trimmed.to_string()
        } else {
            format!("https://github.com/{}", trimmed)
        }
    }
}
