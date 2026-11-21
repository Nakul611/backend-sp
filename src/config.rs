use std::env;

#[derive(Debug, Clone)]
pub struct Config {
    pub host: String,
    pub port: u16,
    pub mongodb_uri: String,
    pub database_name: String,
}

impl Config {
    pub fn from_env() -> Self {
        dotenvy::dotenv().ok();

        let host = env::var("HOST").unwrap_or_else(|_| "0.0.0.0".to_string());
        let port = env::var("PORT")
            .ok()
            .and_then(|p| p.parse().ok())
            .unwrap_or(8000);

        let mongodb_uri = env::var("MONGODB_URI").unwrap_or_else(|_| {
            "mongodb://127.0.0.1:27017".to_string()
        });

        let database_name = env::var("DATABASE_NAME").unwrap_or_else(|_| {
            "stackplane_db".to_string()
        });

        Self {
            host,
            port,
            mongodb_uri,
            database_name,
        }
    }

    pub fn addr(&self) -> String {
        format!("{}:{}", self.host, self.port)
    }
}
