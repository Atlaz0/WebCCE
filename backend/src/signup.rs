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
    println!("--- New signup request received ---");

    println!("Step 1: Parsed JSON data");
    println!("  Username: {}", data.username);
    println!("  Password: {}", data.password);
    println!("  Room ID: {}", data.room_id);

    // Try saving to file
    println!("Step 2: Attempting to save user to users.txt");
    let _lock = FILE_LOCK.lock().unwrap();
    match OpenOptions::new()
        .append(true)
        .create(true)
        .open("users.txt")
    {
        Ok(mut file) => {
            if let Err(e) = writeln!(file, "{},{},{}", data.username, data.password, data.room_id) {
                eprintln!("Failed to write to file: {}", e);
                return Json("Error saving user");
            }
            println!("Successfully saved user to users.txt");
        }
        Err(e) => {
            eprintln!("Failed to open file: {}", e);
            return Json("Error saving user");
        }
    }

    println!("Step 3: Sending success response to client");
    Json("User signed up successfully")
}