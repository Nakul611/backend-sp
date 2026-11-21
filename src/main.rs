use tokio::net::TcpListener;
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};

mod auth;
mod config;
mod constants;
mod controllers;
mod core;
mod db;
mod error;
mod handlers;
mod helpers;
mod idle_watcher;
mod models;
mod routes;
mod services;
mod state;
mod telemetry_ticker;

use config::Config;
use idle_watcher::start_idle_watcher;
use routes::create_router;
use state::AppState;
use telemetry_ticker::start_telemetry_ticker;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    tracing_subscriber::registry()
        .with(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| "backend=info,tower_http=info".into()),
        )
        .with(tracing_subscriber::fmt::layer())
        .init();

    let config = Config::from_env();
    tracing::info!("Initializing StackPlane Control Plane Backend...");

    let state = AppState::new(&config.mongodb_uri).await;

    // Start background fleet idle watcher & live telemetry stream ticker
    start_idle_watcher(state.clone());
    start_telemetry_ticker(state.clone());

    let app = create_router(state);

    let listener = TcpListener::bind(config.addr()).await?;
    tracing::info!("StackPlane Axum REST & WebSocket server listening on http://{}", config.addr());

    axum::serve(listener, app).await?;

    Ok(())
}
