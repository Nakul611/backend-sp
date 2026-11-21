use tracing::info;

pub struct DatabaseConnection {
    pub uri: String,
    pub is_connected: bool,
}

impl DatabaseConnection {
    pub async fn connect(uri: &str) -> Self {
        info!("Connecting to StackPlane persistence engine at: {}", uri);
        Self {
            uri: uri.to_string(),
            is_connected: true,
        }
    }
}
