use crate::state::{AppState, Room, UserState};
use axum::{
    extract::{
        ws::{Message, WebSocket},
        Path, State, WebSocketUpgrade,
    },
    response::IntoResponse,
};
use futures::{stream::StreamExt, SinkExt};
use std::collections::HashMap;
use tokio::sync::mpsc;

// This is our main WebSocket handler. It's called when a user connects.
pub async fn ws_handler(
    ws: WebSocketUpgrade,
    State(state): State<AppState>,
    Path((file_id, username)): Path<(i32, String)>,
) -> impl IntoResponse {
    println!("[WS] New connection request for file {} from user {}", file_id, username);
    ws.on_upgrade(move |socket| handle_socket(socket, state, file_id, username))
}

// This function handles the lifecycle of a single WebSocket connection.
async fn handle_socket(socket: WebSocket, state: AppState, file_id: i32, username: String) {
    let (mut socket_sender, mut socket_receiver) = socket.split();

    // Create a channel for this user to receive messages on.
    let (user_sender, mut user_receiver) = mpsc::unbounded_channel::<Message>();

    // Spawn a task to forward messages from the channel to the user's WebSocket.
    tokio::spawn(async move {
        while let Some(message) = user_receiver.recv().await {
            if socket_sender.send(message).await.is_err() {
                break; // Client disconnected.
            }
        }
    });

    // Add the user to the room.
    {
        let mut room_manager = state.room_manager.lock().await;
        let room = room_manager.entry(file_id).or_insert_with(|| Room {
            users: HashMap::new(),
        });
        room.users.insert(
            username.clone(),
            UserState {
                username: username.clone(),
                sender: user_sender,
            },
        );
        println!("[WS] User '{}' joined room for file {}. Total users: {}", username, file_id, room.users.len());
    }

    // Listen for incoming messages from this user.
    while let Some(Ok(msg)) = socket_receiver.next().await {
        if let Message::Text(text) = msg {
            println!("[WS] Received message from {}: {}", username, text);

            // Broadcast the message to all other users in the same room.
            let room_manager = state.room_manager.lock().await;
            if let Some(room) = room_manager.get(&file_id) {
                for (other_username, other_user) in &room.users {
                    if &username != other_username {
                        println!("[WS] Broadcasting to {}", other_username);
                        let _ = other_user.sender.send(Message::Text(text.clone()));
                    }
                }
            }
        }
    }

    // Cleanup: Remove the user from the room when they disconnect.
    {
        let mut room_manager = state.room_manager.lock().await;
        if let Some(room) = room_manager.get_mut(&file_id) {
            room.users.remove(&username);
            println!("[WS] User '{}' disconnected from file {}. Users remaining: {}", username, file_id, room.users.len());
        }
    }
}