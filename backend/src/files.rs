use axum::{
    extract::{Path, State},
    http::StatusCode,
    response::Json,
    response::{IntoResponse, Response},
};
use serde::Serialize;
use crate::state::{AppState, Project};

pub async fn get_file_tree(
    State(app_state): State<AppState>,
    Path(room_id): Path<String>,
) -> Result<Json<Vec<Project>>, StatusCode> {
    println!("[API] Request for file tree in room: {}", room_id);
    let file_system = app_state.file_system.lock().await;

    match file_system.get(&room_id).cloned() {
        Some(projects) => {
            println!("[API] Found {} projects for room '{}'.", projects.len(), room_id);
            Ok(Json(projects))
        }
        None => {
            println!("[API] No projects found for room '{}'.", room_id);
            Err(StatusCode::NOT_FOUND)
        }
    }
}

#[derive(Serialize)]
pub struct FileContentResponse {
    id: i32,
    name: String,
    content: String,
}

pub async fn get_file_content(
    State(app_state): State<AppState>,
    Path(file_id): Path<i32>,
) -> Response {
    println!("[API] Request for content of file_id: {}", file_id);
    let file_system = app_state.file_system.lock().await;

    for projects in file_system.values() {
        for project in projects {
            for file in &project.files {
                if file.id == file_id {
                    println!("[API] Found file '{}' with id {}.", file.name, file.id);
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

    println!("[API] File with id {} not found.", file_id);
    (StatusCode::NOT_FOUND, "File not found").into_response()
}