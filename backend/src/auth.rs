use axum::Json;
use bcrypt::{hash, verify, DEFAULT_COST};
use once_cell::sync::Lazy;
use serde::Deserialize;
use std::fs::{read_to_string, OpenOptions};
use std::io::Write;
use std::sync::Mutex;

static FILE_LOCK: Lazy<Mutex<()>> = Lazy::new(|| Mutex::new(()));
const USERS_FILE: &str = "users.txt";

#[derive(Deserialize)]
pub struct AuthData {
    pub username: String,
    pub password: String,
    pub room_id: String,
}

pub async fn signup_user(Json(data): Json<AuthData>) -> Json<&'static str> {
    println!("--- New signup request received ---");

    let _lock = FILE_LOCK.lock().unwrap();

    let existing_data = read_to_string(USERS_FILE).unwrap_or_default();
    for line in existing_data.lines() {
        if let Some((stored_username, _, _)) = parse_user_line(line) {
            // ⚠️ bcrypt hashes are non-deterministic; use plaintext for username instead
            if data.username == stored_username {
                return Json("Username already exists");
            }
        }
    }

    // Store username in plaintext for uniqueness check
    let hashed_password = match hash(&data.password, DEFAULT_COST) {
        Ok(h) => h,
        Err(_) => return Json("Error hashing password"),
    };
    let hashed_room_id = match hash(&data.room_id, DEFAULT_COST) {
        Ok(h) => h,
        Err(_) => return Json("Error hashing room ID"),
    };

    match OpenOptions::new().append(true).create(true).open(USERS_FILE) {
        Ok(mut file) => {
            let new_line = format!("{},{},{}", data.username, hashed_password, hashed_room_id);
            if let Err(e) = writeln!(file, "{}", new_line) {
                eprintln!("Failed to write to file: {}", e);
                return Json("Error saving user data");
            }
        }
        Err(e) => {
            eprintln!("Failed to open file: {}", e);
            return Json("Error saving user data");
        }
    }

    Json("User signed up successfully")
}

pub async fn login_user(Json(data): Json<AuthData>) -> Json<&'static str> {
    let _lock = FILE_LOCK.lock().unwrap();

    let contents = match read_to_string(USERS_FILE) {
        Ok(c) => c,
        Err(_) => return Json("Error reading user data"),
    };

    for line in contents.lines() {
        if let Some((stored_username, stored_password, stored_room_id)) = parse_user_line(line) {
            if data.username == stored_username
                && verify(&data.password, stored_password).unwrap_or(false)
                && verify(&data.room_id, stored_room_id).unwrap_or(false)
            {
                return Json("Login successful");
            }
        }
    }

    Json("Invalid username, password, or room ID")
}

fn parse_user_line(line: &str) -> Option<(&str, &str, &str)> {
    line.split_once(',')
        .and_then(|(user, rest)| rest.split_once(',').map(|(pass, room)| (user, pass, room)))
}
