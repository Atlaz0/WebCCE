// src/state.rs

use axum::extract::ws::Message;
use serde::{Serialize, Deserialize};
use std::collections::HashMap;
use std::sync::{Arc, atomic::{AtomicI32, Ordering}};
use tokio::sync::{Mutex, mpsc};

// --- In-Memory "Database" Structs ---

// A unique ID for each file, generated in memory.
static NEXT_FILE_ID: AtomicI32 = AtomicI32::new(1);

#[derive(Serialize, Clone, Debug)]
pub struct File {
    pub id: i32,
    pub name: String,
    #[serde(skip_serializing)] // Don't send file content in the tree view
    pub content: String,
}

#[derive(Serialize, Clone, Debug)]
pub struct Project {
    pub id: i32,
    pub name: String,
    pub files: Vec<File>,
}

// Our entire in-memory store. For now, we'll have one global "room".
// We can easily change the String key to a room_id later if needed.
pub type FileSystem = Arc<Mutex<HashMap<String, Vec<Project>>>>;


// --- WebSocket State (This stays the same) ---

pub struct UserState {
    pub username: String,
    pub sender: mpsc::UnboundedSender<Message>,
}

pub struct Room {
    pub users: HashMap<String, UserState>,
}

// The key is the file_id being edited.
pub type RoomManager = Arc<Mutex<HashMap<i32, Room>>>;


// --- Global Application State ---

#[derive(Clone)]
pub struct AppState {
    // The in-memory file system
    pub file_system: FileSystem,
    // The manager for live WebSocket rooms
    pub room_manager: RoomManager,
}

// Helper function to create some default data so the editor isn't empty.
pub fn create_initial_data() -> FileSystem {
    let mut fs = HashMap::new();
    
    let html_file = File {
        id: NEXT_FILE_ID.fetch_add(1, Ordering::SeqCst),
        name: "index.html".to_string(),
        content: "<h1>Hello, World!</h1>\n<p>Edit this to see the live preview update.</p>".to_string(),
    };

    let css_file = File {
        id: NEXT_FILE_ID.fetch_add(1, Ordering::SeqCst),
        name: "style.css".to_string(),
        content: "h1 {\n  color: steelblue;\n}".to_string(),
    };
    
    let demo_project = Project {
        id: 1,
        name: "Demo Project".to_string(),
        files: vec![html_file, css_file],
    };

    // We'll put everything under a "public_room" for the "Try Now" button.
    fs.insert("public_room".to_string(), vec![demo_project]);
    
    Arc::new(Mutex::new(fs))
}