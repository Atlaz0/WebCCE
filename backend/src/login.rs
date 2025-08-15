use axum::Json;
use bcrypt::verify;
use once_cell::sync::Lazy;
use serde::Deserialize;
use std::fs::read_to_string;
use std::sync::Mutex;

static FILE_LOCK: Lazy<Mutex<()>> = Lazy::new(|| Mutex::new(()));

#[derive(Deserialize)]
pub struct LoginData {
    pub username: String,
    pub password: String,
    pub room_id: String,
}

pub async fn login_user(Json(data): Json<LoginData>) -> Json<&'static str> {
    let _lock = FILE_LOCK.lock().unwrap();

    let contents = match read_to_string("users.txt") {
        Ok(c) => c,
        Err(_) => return Json("Error reading user data"),
    };

    for line in contents.lines() {
        if let Some((stored_username, stored_password, stored_room_id)) = line
            .split_once(',')
            .and_then(|(u, rest)| rest.split_once(',').map(|(p, r)| (u, p, r)))
        {
            if verify(&data.username, stored_username).unwrap_or(false)
                && verify(&data.password, stored_password).unwrap_or(false)
                && verify(&data.room_id, stored_room_id).unwrap_or(false)
            {
                return Json("Login successful");
            }
        }
    }

    Json("Invalid username, password, or room ID")
}
