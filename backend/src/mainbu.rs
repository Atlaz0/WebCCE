use axum::{
    routing::{get, post},
    Router,
};
use std::collections::HashMap;
use std::sync::Arc;
use tokio::net::TcpListener;
use tokio::sync::Mutex;
use tower_http::cors::{Any, CorsLayer};

mod auth;
mod state;
mod files;
mod ws;

use state::{AppState, create_initial_data};

#[tokio::main]
async fn main() {
    let app_state = AppState {
        file_system: create_initial_data(),
        room_manager: Arc::new(Mutex::new(HashMap::new())),
    };

    println!("In-memory state created with demo project.");

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
        .layer(cors);

    let listener = TcpListener::bind("0.0.0.0:8080").await.unwrap();
    println!("Server running on {}", listener.local_addr().unwrap());

    axum::serve(listener, app).await.unwrap();
}