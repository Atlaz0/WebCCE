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
    use std::mem::drop; // Import drop to explicitly release the lock

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
        println!("--- [SIGNUP] New signup request ---");
        println!("[SIGNUP] Received data for user: '{}', room: '{}'", data.username, data.room_id);
        println!("[SIGNUP] Password (raw, not for production logs!): '{}'", data.password);

        let _lock = FILE_LOCK.lock().unwrap();
        println!("[SIGNUP] Acquired file lock.");

        let existing_data = read_to_string(USERS_FILE).unwrap_or_default();
        println!("[SIGNUP] Read {} bytes from users file.", existing_data.len());

        for line in existing_data.lines() {
            if let Some((stored_username, _, _)) = parse_user_line(line) {
                if data.username == stored_username {
                    println!("[SIGNUP] Abort: Username '{}' already exists.", data.username);
                    drop(_lock);
                    return create_response(StatusCode::CONFLICT, "Username already exists");
                }
            }
        }
        println!("[SIGNUP] Username '{}' is available.", data.username);

        // --- SOLUTION: Create a fresh Argon2 instance for each hashing operation ---
        
        // Hash password
        let salt_pass = SaltString::generate(&mut OsRng);
        let hashed_password = match Argon2::default().hash_password(data.password.as_bytes(), &salt_pass) {
            Ok(h) => h.to_string(),
            Err(e) => {
                println!("[SIGNUP] CRITICAL: Error hashing password: {:?}", e);
                drop(_lock);
                return create_response(StatusCode::INTERNAL_SERVER_ERROR, "Error hashing password");
            }
        };
        println!("[SIGNUP] Successfully hashed password.");

        // Hash room ID
        let salt_room = SaltString::generate(&mut OsRng);
        let hashed_room_id = match Argon2::default().hash_password(data.room_id.as_bytes(), &salt_room) {
            Ok(h) => h.to_string(),
            Err(e) => {
                println!("[SIGNUP] CRITICAL: Error hashing room ID: {:?}", e);
                drop(_lock);
                return create_response(StatusCode::INTERNAL_SERVER_ERROR, "Error hashing room ID");
            }
        };
        println!("[SIGNUP] Successfully hashed room ID.");

        println!("[SIGNUP] Attempting to open '{}' for appending.", USERS_FILE);
        match OpenOptions::new().append(true).create(true).open(USERS_FILE) {
            Ok(mut file) => {
                let new_line = format!("{},{},{}", data.username, hashed_password, hashed_room_id);
                if writeln!(file, "{}", new_line).is_err() {
                    println!("[SIGNUP] CRITICAL: Failed to write new user to file.");
                    drop(_lock);
                    return create_response(StatusCode::INTERNAL_SERVER_ERROR, "Error saving user data");
                }
                println!("[SIGNUP] Successfully wrote new user to file.");
            }
            Err(e) => {
                println!("[SIGNUP] CRITICAL: Failed to open file for writing: {}", e);
                drop(_lock);
                return create_response(StatusCode::INTERNAL_SERVER_ERROR, "Error saving user data");
            }
        }
        
        drop(_lock);
        println!("[SIGNUP] SUCCESS: Signup completed for user '{}'.", data.username);
        create_response(StatusCode::CREATED, "User signed up successfully")
    }

    pub async fn login_user(Json(data): Json<AuthData>) -> Response {
        println!("--- [LOGIN] New login request ---");
        println!("[LOGIN] Attempting login for user: '{}', room: '{}'", data.username, data.room_id);
        println!("[LOGIN] Password (raw, not for production logs!): '{}'", data.password);

        let _lock = FILE_LOCK.lock().unwrap();
        println!("[LOGIN] Acquired file lock.");

        let contents = match read_to_string(USERS_FILE) {
            Ok(c) => c,
            Err(_) => {
                println!("[LOGIN] Abort: Could not read users file '{}'.", USERS_FILE);
                drop(_lock);
                return create_response(StatusCode::INTERNAL_SERVER_ERROR, "Error reading user data");
            }
        };

        for (i, line) in contents.lines().enumerate() {
            if let Some((stored_username, stored_password_hash, stored_room_id_hash)) = parse_user_line(line) {
                if data.username == stored_username {
                    println!("[LOGIN] Found matching username: '{}'. Proceeding to verify credentials.", stored_username);

                    let is_valid_login = || -> Option<bool> {
                        let argon2 = Argon2::default(); // Use a fresh instance for verification

                        let pass_hash = PasswordHash::new(stored_password_hash).ok()?;
                        let pass_ok = argon2.verify_password(data.password.as_bytes(), &pass_hash).is_ok();
                        println!("[LOGIN]   -> Password verification result: {}", pass_ok);

                        let room_hash = PasswordHash::new(stored_room_id_hash).ok()?;
                        let room_ok = argon2.verify_password(data.room_id.as_bytes(), &room_hash).is_ok();
                        println!("[LOGIN]   -> Room ID verification result: {}", room_ok);
                        
                        Some(pass_ok && room_ok)
                    }();

                    if let Some(true) = is_valid_login {
                        println!("[LOGIN] SUCCESS: Credentials verified for user '{}'.", stored_username);
                        drop(_lock);
                        return create_response(StatusCode::OK, "Login successful");
                    } else {
                        println!("[LOGIN] FAILED: Invalid credentials or malformed hash for user '{}'. Breaking loop.", stored_username);
                        drop(_lock);
                        return create_response(StatusCode::UNAUTHORIZED, "Invalid username, password, or room ID");
                    }
                }
            } else {
                println!("[LOGIN] Warning: Could not parse line #{}. Skipping.", i);
            }
        }

        println!("[LOGIN] FAILED: Reached end of file. User '{}' not found.", data.username);
        drop(_lock);
        create_response(StatusCode::UNAUTHORIZED, "Invalid username, password, or room ID")
    }

    fn parse_user_line(line: &str) -> Option<(&str, &str, &str)> {
        let result = line.split_once(',')
            .and_then(|(user, rest)| rest.split_once(',').map(|(pass, room)| (user, pass, room)));
        if result.is_none() {
            println!("[PARSER] Warning: Failed to parse line: '{}'", line);
        }
        result
    }