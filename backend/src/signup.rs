use axum::Json;
use bcrypt::{hash, DEFAULT_COST};
use once_cell::sync::Lazy;
use serde::Deserialize;
use std::fs::{OpenOptions, read_to_string};
use std::io::Write;
use std::sync::Mutex;

static FILE_LOCK: Lazy<Mutex<()>> = Lazy::new(|| Mutex::new(()));

#[derive(Deserialize)]
pub struct SignUpData {
    pub username: String,
    pub password: String,
    pub room_id: String,
}

pub async fn signup_user(Json(data): Json<SignUpData>) -> Json<&'static str> {
    println!("--- New signup request received ---");

    let _lock = FILE_LOCK.lock().unwrap();

    // Read existing users
    let existing_data = read_to_string("users.txt").unwrap_or_default();
    for line in existing_data.lines() {
        if let Some((stored_username, _, _)) = line.split_once(',').and_then(|(u, rest)| {
            rest.split_once(',').map(|(p, r)| (u, p, r))
        }) {
            // Compare hashed username
            if bcrypt::verify(&data.username, stored_username).unwrap_or(false) {
                return Json("Username already exists");
            }
        }
    }

    // Hash username, password, and room_id
    let hashed_username = match hash(&data.username, DEFAULT_COST) {
        Ok(h) => h,
        Err(_) => return Json("Error hashing username"),
    };
    let hashed_password = match hash(&data.password, DEFAULT_COST) {
        Ok(h) => h,
        Err(_) => return Json("Error hashing password"),
    };
    let hashed_room_id = match hash(&data.room_id, DEFAULT_COST) {
        Ok(h) => h,
        Err(_) => return Json("Error hashing room ID"),
    };

    // Append to file
    match OpenOptions::new().append(true).create(true).open("users.txt") {
        Ok(mut file) => {
            if let Err(e) = writeln!(file, "{},{},{}", hashed_username, hashed_password, hashed_room_id) {
                eprintln!("Failed to write to file: {}", e);
                return Json("Error saving user");
            }
        }
        Err(e) => {
            eprintln!("Failed to open file: {}", e);
            return Json("Error saving user");
        }
    }

    Json("User signed up successfully")
}
