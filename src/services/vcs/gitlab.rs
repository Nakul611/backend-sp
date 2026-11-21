pub struct GitLabService;

impl GitLabService {
    pub fn get_ping_latency_ms() -> u32 {
        38
    }

    pub fn sanitize_url(raw: &str) -> String {
        let trimmed = raw.trim();
        if trimmed.starts_with("git@gitlab.com:") || trimmed.starts_with("https://gitlab.com/") {
            trimmed.to_string()
        } else {
            format!("https://gitlab.com/{}", trimmed)
        }
    }
}
