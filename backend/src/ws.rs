use crate::state::{AppState, Room, UserState};
use axum::{
    extract::{ ws::{Message, WebSocket}, Path, State, WebSocketUpgrade },
    response::IntoResponse,
};
use futures::{stream::StreamExt, SinkExt};
use std::collections::HashMap;
use tokio::sync::mpsc;
use tracing::info; // NEW!

pub async fn ws_handler(
    ws: WebSocketUpgrade,
    State(state): State<AppState>,
    Path((file_id, username)): Path<(i32, String)>,
) -> impl IntoResponse {
    info!("[ws] ==> New WebSocket connection request for file_id: {} from user: '{}'", file_id, username);
    ws.on_upgrade(move |socket| handle_socket(socket, state, file_id, username))
}
// ... rest of the file is the same, but you can change println! to info! if you like ...
// ... handle_socket function here ...
async fn handle_socket(socket: WebSocket, state: AppState, file_id: i32, username: String) {
    let (mut socket_sender, mut socket_receiver) = socket.split();
    let (user_sender, mut user_receiver) = mpsc::unbounded_channel::<Message>();
    tokio::spawn(async move { while let Some(message) = user_receiver.recv().await { if socket_sender.send(message).await.is_err() { break; } } });
    {
        let mut room_manager = state.room_manager.lock().await;
        let room = room_manager.entry(file_id).or_insert_with(|| Room { users: HashMap::new() });
        room.users.insert(username.clone(), UserState { username: username.clone(), sender: user_sender });
        info!("[ws] User '{}' joined room for file {}. Total users: {}", username, file_id, room.users.len());
    }
    while let Some(Ok(msg)) = socket_receiver.next().await {
        if let Message::Text(text) = msg {
            let room_manager = state.room_manager.lock().await;
            if let Some(room) = room_manager.get(&file_id) {
                for (other_username, other_user) in &room.users {
                    if &username != other_username {
                        let _ = other_user.sender.send(Message::Text(text.clone()));
                    }
                }
            }
        }
    }
    {
        let mut room_manager = state.room_manager.lock().await;
        if let Some(room) = room_manager.get_mut(&file_id) {
            room.users.remove(&username);
            info!("[ws] <== User '{}' disconnected from file {}. Users remaining: {}", username, file_id, room.users.len());
        }
    }
}