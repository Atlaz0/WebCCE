use axum::{
    routing::{get, post},
    Router,
    response::Html,
    Json,
};
use serde::Deserialize;
use std::net::SocketAddr;
use hyper::{Server, http::{HeaderValue, Method}};
use tower_http::cors::{Any, CorsLayer};

async fn index() -> Html<&'static str> {
    Html("<h1>WebCCE Rust Backend is Running!</h1>")
}

async fn ping() -> &'static str {
    "pong"
}

#[derive(Deserialize)]
struct RegisterData {
    username: String,
    password: String,
    roomid: String,
}

async fn register_user(Json(data): Json<RegisterData>) -> Json<&'static str> {
    println!("Received registration:");
    println!("  Username: {}", data.username);
    println!("  Password: {}", data.password);
    println!("  Room ID: {}", data.roomid);

    Json("User registered successfully")
}

#[tokio::main]
async fn main() {
    let cors = CorsLayer::new()
        .allow_origin(HeaderValue::from_static("https://mp2upnhs.my"))
        .allow_methods([Method::POST, Method::GET])
        .allow_headers(Any);

    let app = Router::new()
        .route("/", get(index))
        .route("/ping", get(ping))
        .route("/register", post(register_user))
        .layer(cors);

    let addr = SocketAddr::from(([0, 0, 0, 0], 8080));
    println!("Server running at http://{}", addr);

    Server::bind(&addr)
        .serve(app.into_make_service())
        .await
        .unwrap();
}
