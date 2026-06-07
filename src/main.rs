use std::net::SocketAddr;

use routes::build_router;
use state::AppState;

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

    let state = match db::connect_from_env().await {
        Ok(pool) => {
            println!("Connected to PostgreSQL");
            AppState::postgres(pool)
        }
        Err(e) => {
            println!(
                "PostgreSQL not available ({:?}), using in-memory storage",
                e
            );
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

    println!("Atoms Demo API listening on http://{addr}");

    axum::serve(listener, app)
        .await
        .expect("failed to start server");
}
