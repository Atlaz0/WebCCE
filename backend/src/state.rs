use axum::extract::ws::Message;
use serde::Serialize;
use std::collections::HashMap;
use std::sync::{Arc, atomic::{AtomicI32, Ordering}};
use tokio::sync::{Mutex, mpsc};
use tracing::info;

// --- In-Memory "Database" Structs ---

// A unique ID for each file, generated in memory.
static NEXT_FILE_ID: AtomicI32 = AtomicI32::new(1);

#[derive(Serialize, Clone, Debug)]
pub struct File {
    pub id: i32,
    pub name: String,
    #[serde(skip_serializing)]
    pub content: String,
}

#[derive(Serialize, Clone, Debug)]
pub struct Project {
    pub id: i32,
    pub name: String,
    pub files: Vec<File>,
}

pub type FileSystem = Arc<Mutex<HashMap<String, Vec<Project>>>>;
static NEXT_FILE_ID: AtomicI32 = AtomicI32::new(1);


#[allow(dead_code)]
pub struct UserState {
    pub username: String,
    pub sender: mpsc::UnboundedSender<Message>,
}

#[allow(dead_code)]
pub struct Room {
    pub users: HashMap<String, UserState>,
}

pub type RoomManager = Arc<Mutex<HashMap<i32, Room>>>;

#[derive(Clone)]
#[allow(dead_code)]
pub struct AppState {
    pub file_system: FileSystem,
    pub room_manager: RoomManager,
}


pub fn create_initial_data() -> FileSystem {
    info!("[state] ==> create_initial_data() called. Initializing in-memory file system.");

    let mut fs = HashMap::new();
    
    let html_file = File { id: NEXT_FILE_ID.fetch_add(1, Ordering::SeqCst), name: "index.html".to_string(), content: "<h1>Hello</h1>".to_string() };
    let css_file = File { id: NEXT_FILE_ID.fetch_add(1, Ordering::SeqCst), name: "style.css".to_string(), content: "h1 { color: blue; }".to_string() };
    let js_file = File { id: NEXT_FILE_ID.fetch_add(1, Ordering::SeqCst), name: "script.js".to_string(), content: "console.log('hello')".to_string() };
    let demo_project = Project { id: 1, name: "Demo Website".to_string(), files: vec![html_file, css_file, js_file] };
    
    let readme_file = File { id: NEXT_FILE_ID.fetch_add(1, Ordering::SeqCst), name: "README.md".to_string(), content: "# README".to_string() };
    let another_project = Project { id: 2, name: "Another Project".to_string(), files: vec![readme_file] };

    fs.insert("public_room".to_string(), vec![demo_project, another_project]);
    
    // NEW! Extremely important log to confirm the data is there.
    let keys: Vec<_> = fs.keys().cloned().collect();
    info!("[state] Inserted data. In-memory file system now contains rooms: {:?}", keys);
    
    info!("[state] <== Initial data creation complete.");
    Arc::new(Mutex::new(fs))
}