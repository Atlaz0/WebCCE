use axum::{
    http::StatusCode,
    response::{IntoResponse, Response},
    Json,
};
use argon2::{
    self,
    password_hash::{PasswordHash, PasswordHasher, PasswordVerifier, SaltString},
    Argon2,
};
use once_cell::sync::Lazy;
use rand_core::OsRng;
use serde::Deserialize;
use std::fs::{read_to_string, OpenOptions};
use std::io::Write;
use std::sync::Mutex;
use std::mem::drop;

static FILE_LOCK: Lazy<Mutex<()>> = Lazy::new(|| Mutex::new(()));
const USERS_FILE: &str = "users.txt";

#[derive(Deserialize)]
pub struct AuthData {
    pub username: String,
    pub password: String,
    pub room_id: String,
}

fn create_response(status: StatusCode, message: &'static str) -> Response {
    (status, Json(message)).into_response()
}

pub async fn signup_user(Json(data): Json<AuthData>) -> Response {
    println!("--- [SIGNUP] New signup request ---");
    let _lock = FILE_LOCK.lock().unwrap();

    let existing_data = read_to_string(USERS_FILE).unwrap_or_default();
    for line in existing_data.lines() {
        if let Some((stored_username, _, _)) = parse_user_line(line) {
            if data.username == stored_username {
                println!("[SIGNUP] Abort: Username '{}' already exists.", data.username);
                drop(_lock);
                return create_response(StatusCode::CONFLICT, "Username already exists");
            }
        }
    }
    
    let salt_pass = SaltString::generate(&mut OsRng);
    let hashed_password = match Argon2::default().hash_password(data.password.as_bytes(), &salt_pass) {
        Ok(h) => h.to_string(),
        Err(_) => return create_response(StatusCode::INTERNAL_SERVER_ERROR, "Error hashing password"),
    };

    let salt_room = SaltString::generate(&mut OsRng);
    let hashed_room_id = match Argon2::default().hash_password(data.room_id.as_bytes(), &salt_room) {
        Ok(h) => h.to_string(),
        Err(_) => return create_response(StatusCode::INTERNAL_SERVER_ERROR, "Error hashing room ID"),
    };

    match OpenOptions::new().append(true).create(true).open(USERS_FILE) {
        Ok(mut file) => {
            let new_line = format!("{},{},{}", data.username, hashed_password, hashed_room_id);
            if writeln!(file, "{}", new_line).is_err() {
                return create_response(StatusCode::INTERNAL_SERVER_ERROR, "Error saving user data");
            }
        }
        Err(_) => return create_response(StatusCode::INTERNAL_SERVER_ERROR, "Error saving user data"),
    }
    
    drop(_lock);
    println!("[SIGNUP] SUCCESS: Signup completed for user '{}'.", data.username);
    create_response(StatusCode::CREATED, "User signed up successfully")
}

pub async fn login_user(Json(data): Json<AuthData>) -> Response {
    println!("--- [LOGIN] New login request for user: '{}' ---", data.username);
    let _lock = FILE_LOCK.lock().unwrap();

    let contents = match read_to_string(USERS_FILE) {
        Ok(c) => c,
        Err(_) => return create_response(StatusCode::INTERNAL_SERVER_ERROR, "Error reading user data"),
    };

    for line in contents.lines() {
        if let Some((stored_username, stored_password_hash, stored_room_id_hash)) = parse_user_line(line) {
            if data.username == stored_username {
                let is_valid_login = || -> Option<bool> {
                    let argon2 = Argon2::default();
                    let pass_hash = PasswordHash::new(stored_password_hash).ok()?;
                    let pass_ok = argon2.verify_password(data.password.as_bytes(), &pass_hash).is_ok();
                    let room_hash = PasswordHash::new(stored_room_id_hash).ok()?;
                    let room_ok = argon2.verify_password(data.room_id.as_bytes(), &room_hash).is_ok();
                    Some(pass_ok && room_ok)
                }();

                if let Some(true) = is_valid_login {
                    println!("[LOGIN] SUCCESS: Credentials verified for user '{}'.", stored_username);
                    drop(_lock);
                    return create_response(StatusCode::OK, "Login successful");
                } else {
                    println!("[LOGIN] FAILED: Invalid credentials for user '{}'.", stored_username);
                    drop(_lock);
                    return create_response(StatusCode::UNAUTHORIZED, "Invalid username, password, or room ID");
                }
            }
        }
    }

    println!("[LOGIN] FAILED: User '{}' not found.", data.username);
    drop(_lock);
    create_response(StatusCode::UNAUTHORIZED, "Invalid username, password, or room ID")
}

// --- THIS IS THE CORRECTED FUNCTION ---
fn parse_user_line(line: &str) -> Option<(&str, &str, &str)> {
    line.split_once(',')
        .and_then(|(username, rest)| rest.rsplit_once(',').map(|(pass_hash, room_hash)| (username, pass_hash, room_hash)))
}