pub struct BitbucketService;

impl BitbucketService {
    pub fn get_ping_latency_ms() -> u32 {
        42
    }

    pub fn sanitize_url(raw: &str) -> String {
        let trimmed = raw.trim();
        if trimmed.starts_with("git@bitbucket.org:") || trimmed.starts_with("https://bitbucket.org/") {
            trimmed.to_string()
        } else {
            format!("https://bitbucket.org/{}", trimmed)
        }
    }
}
