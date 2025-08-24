use axum::{
    extract::Path,
    routing::get,
    Router,
};
use tokio::net::TcpListener;

async fn get_file_tree_test(Path(room_id): Path<String>) -> String {
    format!("SUCCESS! You reached the file tree for room: {}", room_id)
}

#[tokio::main]
async fn main() {
    let app = Router::new()
        .route("/api/file-tree/:room_id", get(get_file_tree_test));

    let listener = TcpListener::bind("0.0.0.0:8080").await.unwrap();
    println!("--- CLEAN DEBUG SERVER RUNNING ---");
    println!("--- Test with: curl -v http://127.0.0.1:8080/api/file-tree/public_room ---");

    axum::serve(listener, app).await.unwrap();
}