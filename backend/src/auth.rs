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
            println!("[SIGNUP] Parsed username from file: {}", stored_username);
            if data.username == stored_username {
                println!("[SIGNUP] Username '{}' already exists!", data.username);
                return Json("Username already exists");
            }
        } else {
            println!("[SIGNUP] Failed to parse line {}", i);
        }
    }

    // Hash password
    let hashed_password = match hash(&data.password, DEFAULT_COST) {
        Ok(h) => {
            println!("[SIGNUP] Successfully hashed password");
            h
        }
        Err(e) => {
            println!("[SIGNUP] Error hashing password: {:?}", e);
            return Json("Error hashing password");
        }
    };

    // Hash room ID
    let hashed_room_id = match hash(&data.room_id, DEFAULT_COST) {
        Ok(h) => {
            println!("[SIGNUP] Successfully hashed room ID");
            h
        }
        Err(e) => {
            println!("[SIGNUP] Error hashing room ID: {:?}", e);
            return Json("Error hashing room ID");
        }
    };

    println!("[SIGNUP] Attempting to open file for append: {}", USERS_FILE);
    match OpenOptions::new().append(true).create(true).open(USERS_FILE) {
        Ok(mut file) => {
            let new_line = format!("{},{},{}", data.username, hashed_password, hashed_room_id);
            println!("[SIGNUP] Writing new line: {}", new_line);
            if let Err(e) = writeln!(file, "{}", new_line) {
                println!("[SIGNUP] Failed to write to file: {}", e);
                return Json("Error saving user data");
            }
            println!("[SIGNUP] Successfully wrote new user to file");
        }
        Err(e) => {
            println!("[SIGNUP] Failed to open file: {}", e);
            return Json("Error saving user data");
        }
    }

    println!("[SIGNUP] Signup completed successfully for {}", data.username);
    Json("User signed up successfully")
}

pub async fn login_user(Json(data): Json<AuthData>) -> Json<&'static str> {
    println!("--- [LOGIN] New login request ---");
    println!("[LOGIN] Username: {}", data.username);
    println!("[LOGIN] Password (raw, not recommended in production!): {}", data.password);
    println!("[LOGIN] Room ID (raw): {}", data.room_id);

    let _lock = FILE_LOCK.lock().unwrap();
    println!("[LOGIN] Acquired file lock");

    let contents = match read_to_string(USERS_FILE) {
        Ok(c) => {
            println!("[LOGIN] Users file read successfully ({} bytes)", c.len());
            c
        }
        Err(e) => {
            println!("[LOGIN] Error reading user data: {:?}", e);
            return Json("Error reading user data");
        }
    };

    for (i, line) in contents.lines().enumerate() {
        println!("[LOGIN] Checking line {}: {}", i, line);
        if let Some((stored_username, stored_password, stored_room_id)) = parse_user_line(line) {
            println!("[LOGIN] Parsed -> username: {}", stored_username);

            if data.username == stored_username {
                println!("[LOGIN] Username match found for {}", stored_username);

                let pass_ok = verify(&data.password, stored_password).unwrap_or(false);
                println!("[LOGIN] Password verification: {}", pass_ok);

                let room_ok = verify(&data.room_id, stored_room_id).unwrap_or(false);
                println!("[LOGIN] Room ID verification: {}", room_ok);

                if pass_ok && room_ok {
                    println!("[LOGIN] Login successful for {}", stored_username);
                    return Json("Login successful");
                } else {
                    println!("[LOGIN] Failed verification for {}", stored_username);
                }
            }
        } else {
            println!("[LOGIN] Could not parse line {}", i);
        }
    }

    println!("[LOGIN] No valid credentials found, login failed");
    Json("Invalid username, password, or room ID")
}

fn parse_user_line(line: &str) -> Option<(&str, &str, &str)> {
    println!("[PARSER] Parsing line: {}", line);
    let result = line.split_once(',')
        .and_then(|(user, rest)| rest.split_once(',').map(|(pass, room)| (user, pass, room)));

    if let Some((u, _, _)) = result {
        println!("[PARSER] Parsed username: {}", u);
    } else {
        println!("[PARSER] Failed to parse line: {}", line);
    }

    result
}
