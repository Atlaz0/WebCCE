use axum::{
    extract::{Path, State},
    http::StatusCode,
    response::Json,
    response::{IntoResponse, Response},
};
use serde::{Deserialize, Serialize};
use crate::state::{AppState, Project};
use tracing::info;

// A struct for the incoming save request
#[derive(Deserialize)]
pub struct SaveFileRequest {
    id: i32,
    content: String,
}

// A struct for the file content API response
#[derive(Serialize)]
pub struct FileContentResponse {
    id: i32,
    name: String,
    content: String,
}

// Handler for getting the file tree
pub async fn get_file_tree(
    State(app_state): State<AppState>,
    Path(room_id): Path<String>,
) -> Result<Json<Vec<Project>>, StatusCode> {
    info!("[files] ==> API call to get_file_tree for room: '{}'", room_id);
    let file_system = app_state.file_system.lock().await;
    let keys: Vec<_> = file_system.keys().cloned().collect();
    info!("[files] File system locked. Current rooms are: {:?}", keys);

    match file_system.get(&room_id).cloned() {
        Some(projects) => {
            info!("[files] <== SUCCESS: Found {} projects for room '{}'.", projects.len(), room_id);
            Ok(Json(projects))
        }
        None => {
            info!("[files] <== FAILURE: Room '{}' not found.", room_id);
            Err(StatusCode::NOT_FOUND)
        }
    }
}

// Handler for getting a single file's content
pub async fn get_file_content(
    State(app_state): State<AppState>,
    Path(file_id): Path<i32>,
) -> Response {
    info!("[files] ==> API call to get_file_content for file_id: {}", file_id);
    let file_system = app_state.file_system.lock().await;

    for projects in file_system.values() {
        for project in projects {
            for file in &project.files {
                if file.id == file_id {
                    info!("[files] <== SUCCESS: Found file '{}' with id {}.", file.name, file.id);
                    let response = FileContentResponse {
                        id: file.id,
                        name: file.name.clone(),
                        content: file.content.clone(),
                    };
                    return (StatusCode::OK, Json(response)).into_response();
                }
            }
        }
    }
    info!("[files] <== FAILURE: File with id {} not found.", file_id);
    (StatusCode::NOT_FOUND, "File not found").into_response()
} // <-- Missing brace was added here

// Handler for saving a file's content
pub async fn save_file_content(
    State(app_state): State<AppState>,
    Json(payload): Json<SaveFileRequest>,
) -> StatusCode {
    info!("[files] ==> API call to save_file_content for file_id: {}", payload.id);
    let mut file_system = app_state.file_system.lock().await;

    for projects in file_system.values_mut() {
        for project in projects {
            for file in &mut project.files {
                if file.id == payload.id {
                    file.content = payload.content;
                    info!("[files] <== SUCCESS: Saved content for file '{}'.", file.name);
                    return StatusCode::OK;
                }
            }
        }
    }

    info!("[files] <== FAILURE: Could not find file_id {} to save.", payload.id); // Fixed variable
    StatusCode::NOT_FOUND // Fixed return type
}