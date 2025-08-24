use axum::{
    routing::{get, post},
    Router,
};
use std::collections::HashMap;
use std::sync::Arc;
use tokio::net::TcpListener;
use tokio::sync::Mutex;
use tower_http::cors::{Any, CorsLayer};

// Import all our application modules
mod auth;
mod state;
mod files;
mod ws;

use state::{AppState, create_initial_data};

#[tokio::main]
async fn main() {
    // Create our shared, in-memory application state with demo data
    let app_state = AppState {
        file_system: create_initial_data(),
        room_manager: Arc::new(Mutex::new(HashMap::new())),
    };

    println!("Full application state created with demo project.");

    // Configure CORS to allow requests from any origin
    let cors = CorsLayer::new()
        .allow_origin(Any)
        .allow_methods(Any)
        .allow_headers(Any);

    // Build the main application router
    let app = Router::new()
        // Authentication routes (placeholders for now)
        .route("/signup", post(auth::signup_user))
        .route("/login", post(auth::login_user))
        
        // API routes for the file manager
        .route("/api/file-tree/:room_id", get(files::get_file_tree))
        .route("/api/file/:file_id", get(files::get_file_content))

        // The WebSocket route for real-time collaboration
        .route("/ws/:file_id/:username", get(ws::ws_handler))
        
        // Add the shared state and CORS layer to the router
        .with_state(app_state)
        .layer(cors);

    // Bind the server to port 8080
    let listener = TcpListener::bind("0.0.0.0:8080").await.unwrap();
    println!("Server running on {}", listener.local_addr().unwrap());
    println!("Test API at: http://127.0.0.1:8080/api/file-tree/public_room");

    // Start the server
    axum::serve(listener, app).await.unwrap();
}