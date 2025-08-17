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
use rand_core::OsRng; // Correctly imported directly
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

// Helper to create a consistent response format
fn create_response(status: StatusCode, message: &'static str) -> Response {
    (status, Json(message)).into_response()
}

pub async fn signup_user(Json(data): Json<AuthData>) -> Response {
    println!("--- [SIGNUP] New signup request received ---");
    println!("[SIGNUP] Username: {}", data.username);
    println!("[SIGNUP] Password (raw, not recommended in production!): {}", data.password);
    println!("[SIGNUP] Room ID (raw): {}", data.room_id);

    let _lock = FILE_LOCK.lock().unwrap();
    println!("[SIGNUP] Acquired file lock");

    let existing_data = read_to_string(USERS_FILE).unwrap_or_default();
    println!("[SIGNUP] Existing users file read, {} bytes", existing_data.len());

    for (i, line) in existing_data.lines().enumerate() {
        println!("[SIGNUP] Checking line {}: {}", i, line);
        if let Some((stored_username, _, _)) = parse_user_line(line) {
            if data.username == stored_username {
                println!("[SIGNUP] Username '{}' already exists!", data.username);
                return create_response(StatusCode::CONFLICT, "Username already exists");
            }
        }
    }

    let salt = SaltString::generate(&mut OsRng);
    let argon2 = Argon2::default();

    // Hash password
    let hashed_password = match argon2.hash_password(data.password.as_bytes(), &salt) {
        Ok(h) => {
            println!("[SIGNUP] Successfully hashed password");
            h.to_string()
        }
        Err(e) => {
            println!("[SIGNUP] Error hashing password: {:?}", e);
            return create_response(StatusCode::INTERNAL_SERVER_ERROR, "Error hashing password");
        }
    };

    // Hash room ID
    let hashed_room_id = match argon2.hash_password(data.room_id.as_bytes(), &salt) {
        Ok(h) => {
            println!("[SIGNUP] Successfully hashed room ID");
            h.to_string()
        }
        Err(e) => {
            println!("[SIGNUP] Error hashing room ID: {:?}", e);
            return create_response(StatusCode::INTERNAL_SERVER_ERROR, "Error hashing room ID");
        }
    };

    println!("[SIGNUP] Attempting to open file for append: {}", USERS_FILE);
    match OpenOptions::new().append(true).create(true).open(USERS_FILE) {
        Ok(mut file) => {
            let new_line = format!("{},{},{}", data.username, hashed_password, hashed_room_id);
            println!("[SIGNUP] Writing new line to file");
            if let Err(e) = writeln!(file, "{}", new_line) {
                println!("[SIGNUP] Failed to write to file: {}", e);
                return create_response(StatusCode::INTERNAL_SERVER_ERROR, "Error saving user data");
            }
            println!("[SIGNUP] Successfully wrote new user to file");
        }
        Err(e) => {
            println!("[SIGNUP] Failed to open file: {}", e);
            return create_response(StatusCode::INTERNAL_SERVER_ERROR, "Error saving user data");
        }
    }

    println!("[SIGNUP] Signup completed successfully for {}", data.username);
    create_response(StatusCode::CREATED, "User signed up successfully")
}

pub async fn login_user(Json(data): Json<AuthData>) -> Response {
    println!("--- [LOGIN] New login request ---");
    println!("[LOGIN] Username: {}", data.username);
    println!("[LOGIN] Password (raw, not recommended in production!): {}", data.password);
    println!("[LOGIN] Room ID (raw): {}", data.room_id);

    let _lock = FILE_LOCK.lock().unwrap();
    println!("[LOGIN] Acquired file lock");

    let contents = match read_to_string(USERS_FILE) {
        Ok(c) => c,
        Err(_) => {
            println!("[LOGIN] Users file not found or unreadable");
            return create_response(StatusCode::INTERNAL_SERVER_ERROR, "Error reading user data");
        }
    };

    for (i, line) in contents.lines().enumerate() {
        println!("[LOGIN] Checking line {}: {}", i, line);
        if let Some((stored_username, stored_password_hash, stored_room_id_hash)) = parse_user_line(line) {
            if data.username == stored_username {
                println!("[LOGIN] Username match found for {}", stored_username);

                let pass_hash = PasswordHash::new(stored_password_hash).unwrap();
                let pass_ok = Argon2::default().verify_password(data.password.as_bytes(), &pass_hash).is_ok();
                println!("[LOGIN] Password verification: {}", pass_ok);

                let room_hash = PasswordHash::new(stored_room_id_hash).unwrap();
                let room_ok = Argon2::default().verify_password(data.room_id.as_bytes(), &room_hash).is_ok();
                println!("[LOGIN] Room ID verification: {}", room_ok);

                if pass_ok && room_ok {
                    println!("[LOGIN] Login successful for {}", stored_username);
                    return create_response(StatusCode::OK, "Login successful");
                } else {
                    println!("[LOGIN] Failed verification for {}", stored_username);
                }
            }
        }
    }

    println!("[LOGIN] No valid credentials found, login failed");
    create_response(StatusCode::UNAUTHORIZED, "Invalid username, password, or room ID")
}


fn parse_user_line(line: &str) -> Option<(&str, &str, &str)> {
    println!("[PARSER] Parsing line: {}", line);
    let result = line.split_once(',')
        .and_then(|(user, rest)| rest.split_once(',').map(|(pass, room)| (user, pass, room)));

    if result.is_some() {
        println!("[PARSER] Successfully parsed line");
    } else {
        println!("[PARSER] Failed to parse line: {}", line);
    }

    result
}