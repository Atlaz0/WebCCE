use axum::Json;
use serde::Deserialize;
use std::fs::{OpenOptions, read_to_string};
use std::io::Write;
use std::sync::Mutex;
use once_cell::sync::Lazy;
use bcrypt::{hash, DEFAULT_COST};

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

    let _lock = FILE_LOCK.lock().unwrap();

    // Step 2: Read existing users
    let existing_data = read_to_string("users.txt").unwrap_or_default();
    for line in existing_data.lines() {
        let parts: Vec<&str> = line.split(',').collect();
        if parts.len() >= 3 {
            if parts[0] == data.username {
                return Json("Error: Username already exists");
            }
            if parts[2] == data.room_id {
                return Json("Error: Room ID already exists");
            }
        }
    }

    // Step 3: Hash password
    let hashed_password = match hash(&data.password, DEFAULT_COST) {
        Ok(h) => h,
        Err(e) => {
            eprintln!("Failed to hash password: {}", e);
            return Json("Error hashing password");
        }
    };

    // Step 4: Save to file
    match OpenOptions::new()
        .append(true)
        .create(true)
        .open("users.txt")
    {
        Ok(mut file) => {
            if let Err(e) = writeln!(file, "{},{},{}", data.username, hashed_password, data.room_id) {
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

    println!("Step 5: Sending success response to client");
    Json("User signed up successfully")
}
