use axum::{
    routing::get,
    Router,
    response::Html,
};
use std::net::SocketAddr;
use hyper::Server;

async fn index() -> Html<&'static str> {
    Html("<h1>🚀 WebCCE Rust Backend is Running!</h1>")
}

#[tokio::main]
async fn main() {
    let app = Router::new()
        .route("/", get(index))
        .route("/ping", get(|| async { "pong" }));

    let addr = SocketAddr::from(([0, 0, 0, 0], 8080));
    println!("Server running at http://{}", addr);

    Server::bind(&addr)
        .serve(app.into_make_service())
        .await
        .unwrap();
}
