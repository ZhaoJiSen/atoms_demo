use std::net::SocketAddr;

use routes::build_router;
use state::AppState;
use tracing_subscriber::{EnvFilter, fmt};

mod ai;
mod auth;
mod db;
mod errors;
mod init;
mod mock;
mod models;
mod routes;
mod servers;
mod state;
mod time;

#[tokio::main]
async fn main() {
    // Load .env file if it exists
    let _ = dotenvy::dotenv();

    // Initialize tracing. Override verbosity with RUST_LOG, e.g.
    // `RUST_LOG=atoms_demo=debug,tower_http=debug`.
    fmt()
        .with_env_filter(EnvFilter::try_from_default_env().unwrap_or_else(|_| {
            EnvFilter::new("info,atoms_demo=debug,tower_http=info")
        }))
        .init();

    let state = match db::connect_from_env().await {
        Ok(pool) => {
            tracing::info!("Connected to PostgreSQL");
            AppState::postgres(pool)
        }
        Err(e) => {
            tracing::warn!(error = ?e, "PostgreSQL not available, using in-memory storage");
            AppState::memory()
        }
    };

    let app = build_router(state);
    let port = std::env::var("ATOMS_DEMO_API_PORT")
        .ok()
        .and_then(|value| value.parse::<u16>().ok())
        .unwrap_or(3001);
    let addr = SocketAddr::from(([0, 0, 0, 0], port));
    let listener = tokio::net::TcpListener::bind(addr)
        .await
        .expect("failed to bind server port");

    tracing::info!("Atoms Demo API listening on http://{addr}");

    axum::serve(listener, app)
        .await
        .expect("failed to start server");
}
