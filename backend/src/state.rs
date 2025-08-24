use axum::extract::ws::Message;
use serde::Serialize;
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

// Our entire in-memory store. The key is a room_id (e.g., "public_room").
pub type FileSystem = Arc<Mutex<HashMap<String, Vec<Project>>>>;


// --- WebSocket State ---

#[allow(dead_code)] // Suppress warning until all fields are used
pub struct UserState {
    pub username: String,
    pub sender: mpsc::UnboundedSender<Message>,
}

#[allow(dead_code)] // Suppress warning until all fields are used
pub struct Room {
    pub users: HashMap<String, UserState>,
}

// The key is the file_id being edited.
pub type RoomManager = Arc<Mutex<HashMap<i32, Room>>>;


// --- Global Application State ---

#[derive(Clone)]
#[allow(dead_code)] // Suppress warning until all fields are used
pub struct AppState {
    pub file_system: FileSystem,
    pub room_manager: RoomManager,
}

// Helper function to create some default data so the editor isn't empty.
pub fn create_initial_data() -> FileSystem {
    let mut fs = HashMap::new();
    
    // --- Project 1: Demo Project ---
    let html_file = File {
        id: NEXT_FILE_ID.fetch_add(1, Ordering::SeqCst),
        name: "index.html".to_string(),
        content: "<h1>Hello, World!</h1>\n<p>Edit this to see the live preview update.</p>\n<script src=\"script.js\"></script>".to_string(),
    };

    let css_file = File {
        id: NEXT_FILE_ID.fetch_add(1, Ordering::SeqCst),
        name: "style.css".to_string(),
        content: "h1 {\n  color: steelblue;\n  font-family: sans-serif;\n}".to_string(),
    };

    let js_file = File {
        id: NEXT_FILE_ID.fetch_add(1, Ordering::SeqCst),
        name: "script.js".to_string(),
        content: "console.log('Hello from the script!');\nalert('This is a test.');".to_string(),
    };
    
    let demo_project = Project {
        id: 1,
        name: "Demo Website".to_string(), // Renamed for clarity
        files: vec![html_file, css_file, js_file],
    };

    // --- Project 2: Another Project ---
    let readme_file = File {
        id: NEXT_FILE_ID.fetch_add(1, Ordering::SeqCst),
        name: "README.md".to_string(),
        content: "# Another Project\n\nThis is a simple markdown file.".to_string(),
    };

    let data_file = File {
        id: NEXT_FILE_ID.fetch_add(1, Ordering::SeqCst),
        name: "data.json".to_string(),
        content: "{\n  \"key\": \"value\",\n  \"is_test\": true\n}".to_string(),
    };
    
    let another_project = Project {
        id: 2,
        name: "Another Project".to_string(),
        files: vec![readme_file, data_file],
    };


    // We'll put everything under a "public_room" for the "Try Now" button.
    fs.insert("public_room".to_string(), vec![demo_project, another_project]);
    
    Arc::new(Mutex::new(fs))
}