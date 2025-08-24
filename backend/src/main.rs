use axum::{
    routing::{get, post},
    Router,
};
use std::collections::HashMap;
use std::sync::Arc;
use tokio::net::TcpListener;
use tokio::sync::Mutex;
use tower_http::{
    cors::{Any, CorsLayer},
    trace::TraceLayer, // NEW! Import the TraceLayer for request logging
};
use tracing_subscriber::{fmt, prelude::*, EnvFilter}; // NEW! Imports for logging setup

mod auth;
mod state;
mod files;
mod ws;

use state::{AppState, create_initial_data};

#[tokio::main]
async fn main() {
    // NEW! Setup structured logging
    tracing_subscriber::registry()
        .with(fmt::layer())
        .with(EnvFilter::from_default_env().add_directive("backend=info,tower_http=info".parse().unwrap()))
        .init();

    tracing::info!("[main] ==> Application starting up...");

    let app_state = AppState {
        file_system: create_initial_data(),
        room_manager: Arc::new(Mutex::new(HashMap::new())),
    };

    let cors = CorsLayer::new()
        .allow_origin(Any)
        .allow_methods(Any)
        .allow_headers(Any);

    let app = Router::new()
        .route("/signup", post(auth::signup_user))
        .route("/login", post(auth::login_user))
        .route("/api/file-tree/:room_id", get(files::get_file_tree))
        .route("/api/file/:file_id", get(files::get_file_content))
        .route("/ws/:file_id/:username", get(ws::ws_handler))
        .with_state(app_state)
        .layer(cors)
        // NEW! Add the request tracing middleware. This is a crucial debugging tool.
        .layer(TraceLayer::new_for_http());

    let port = std::env::var("PORT").unwrap_or_else(|_| "8080".to_string());
    let addr = format!("0.0.0.0:{}", port);

    let listener = TcpListener::bind(&addr).await.unwrap();
    tracing::info!("[main] <== Server configured. Listening on {}", listener.local_addr().unwrap());

    axum::serve(listener, app).await.unwrap();
}