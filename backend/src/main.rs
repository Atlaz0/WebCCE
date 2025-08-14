use axum::{
    routing::{get, post},
    Router,
    response::Html,
};
use std::net::SocketAddr;
use hyper::http::{HeaderValue, Method};
use tower_http::cors::{Any, CorsLayer};

// Import the signup module
mod signup;
use signup::signup_user;

async fn index() -> Html<&'static str> {
    Html("<h1>WebCCE Rust Backend is Running!</h1>")
}

async fn ping() -> &'static str {
    "pong"
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
        .route("/signup", post(signup_user))
        .layer(cors);

    let addr = SocketAddr::from(([0, 0, 0, 0], 8080));
    println!("Server running at http://{}", addr);

    axum::Server::bind(&addr)
        .serve(app.into_make_service())
        .await
        .unwrap();
}
