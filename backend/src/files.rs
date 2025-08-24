use axum::{
    extract::{Path, State},
    http::StatusCode,
    response::Json,
    response::{IntoResponse, Response},
};
use serde::Serialize;
use crate::state::{AppState, Project};
use tracing::info; // NEW!

#[derive(Serialize)]
pub struct FileContentResponse {
    id: i32,
    name: String,
    content: String,
}

pub async fn get_file_tree(
    State(app_state): State<AppState>,
    Path(room_id): Path<String>,
) -> Result<Json<Vec<Project>>, StatusCode> {
    info!("[files] ==> API call to get_file_tree for room: '{}'", room_id);
    
    let file_system = app_state.file_system.lock().await;

    // NEW! Log the current state of the file system at the time of the request.
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
}