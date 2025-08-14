use axum::{Json};
use serde::Deserialize;
use std::fs::OpenOptions;
use std::io::Write;
use std::sync::Mutex;
use once_cell::sync::Lazy;

// Simple in-memory lock for file writes
static FILE_LOCK: Lazy<Mutex<()>> = Lazy::new(|| Mutex::new(()));

#[derive(Deserialize)]
pub struct SignUpData {
    pub username: String,
    pub password: String,
    pub room_id: String,
}

pub async fn signup_user(Json(data): Json<SignUpData>) -> Json<&'static str> {
    println!("Received SignUp:");
    println!("  Username: {}", data.username);
    println!("  Password: {}", data.password);
    println!("  Room ID: {}", data.room_id);

    // Save to a file safely
    let _lock = FILE_LOCK.lock().unwrap();
    let mut file = OpenOptions::new()
        .append(true)
        .create(true)
        .open("users.txt")
        .unwrap();

    writeln!(file, "{},{},{}", data.username, data.password, data.room_id).unwrap();

    Json("User signed up successfully")
}
