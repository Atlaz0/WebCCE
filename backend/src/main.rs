use axum::{
    routing::{get, post},
    Router,
};
use std::collections::HashMap;
use std::sync::Arc;
use tokio::net::TcpListener;
use axum::response::IntoResponse;
use tokio::sync::Mutex;
use tower_http::{
    cors::{Any, CorsLayer},
    trace::TraceLayer,
};
use tracing::Level;
use tracing_subscriber::{fmt, prelude::*, EnvFilter};
use std::env;

mod auth;
mod state;
mod files;
mod ws;
mod chat;

use state::{AppState, create_initial_data};

async fn root() -> impl IntoResponse {
    println!("Backend is working");
    "Backend is working"                   
}

#[tokio::main]
async fn main() {
    let filter = EnvFilter::builder()
        .with_default_directive(Level::INFO.into())
        .from_env_lossy();

    tracing_subscriber::registry()
        .with(fmt::layer())
        .with(filter)
        .init();

    tracing::info!("[main] ==> Application starting up...");

    let openai_key = env::var("OPENAI_API_KEY")
    .expect("OPENAI_API_KEY must be set in environment");

    let app_state = AppState {
        file_system: create_initial_data(),
        room_manager: Arc::new(Mutex::new(HashMap::new())),
        openai_api_key: openai_key,
    };

    let cors = CorsLayer::new()
        .allow_origin(Any)
        .allow_methods(Any)
        .allow_headers(Any);

    let app = Router::new()
        .route("/", get(root))
        .route("/signup", post(auth::signup_user))
        .route("/login", post(auth::login_user))
        .route("/api/file-tree/:room_id", get(files::get_file_tree))
        .route("/api/file/:file_id", get(files::get_file_content))
        .route("/api/file/save", post(files::save_file_content))
        .route("/ws/:file_id/:username", get(ws::ws_handler))
        .route("/api/chat", post(chat::handle_chat))
        .with_state(app_state)
        .layer(cors)
        .layer(TraceLayer::new_for_http());

    let port = std::env::var("PORT").unwrap_or_else(|_| "8080".to_string());
    let addr = format!("0.0.0.0:{}", port);

    let listener = TcpListener::bind(&addr).await.unwrap();
    tracing::info!("[main] <== Server configured. Listening on {}", listener.local_addr().unwrap());

    axum::serve(listener, app).await.unwrap();
}