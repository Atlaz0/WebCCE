use axum::{
    routing::{get, post},
    response::Html,
    Router,
};
use tokio::net::TcpListener;
use tower_http::cors::{Any, CorsLayer};

mod auth;

async fn index() -> Html<&'static str> {
    Html("<h1>WebCCE Rust Backend is Running!</h1>")
}

async fn ping() -> &'static str {
    "pong"
}

#[tokio::main]
async fn main() {
    let cors = CorsLayer::new()
        .allow_origin(Any)
        .allow_methods(Any)
        .allow_headers(Any);

    let app = Router::new()
        .route("/", get(index))
        .route("/ping", get(ping))
        .route("/signup", post(auth::signup_user))
        .route("/login", post(auth::login_user))
        .layer(cors);

    let listener = TcpListener::bind("0.0.0.0:8080").await.unwrap();
    println!("Server running on {}", listener.local_addr().unwrap());

    axum::serve(listener, app).await.unwrap();
}
