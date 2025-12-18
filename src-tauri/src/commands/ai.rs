use crate::storage::ai::AIStore;
use crate::storage::Database;
use std::sync::Mutex;
use tauri::State;

#[tauri::command]
pub fn create_conversation(
    id: String,
    title: String,
    model: Option<String>,
    db: State<'_, Mutex<Database>>,
) -> Result<(), String> {
    let db_guard = db.lock().map_err(|e| format!("Database lock error: {}", e))?;
    let store = AIStore::new(db_guard.conn.clone())
        .map_err(|e| format!("Failed to initialize AIStore: {}", e))?;
    store.create_conversation(&id, &title, model.as_deref())
        .map_err(|e| format!("Failed to create conversation: {}", e))
}

#[tauri::command]
pub fn list_conversations(
    db: State<'_, Mutex<Database>>,
) -> Result<Vec<crate::storage::ai::Conversation>, String> {
    let db_guard = db.lock().map_err(|e| format!("Database lock error: {}", e))?;
    let store = AIStore::new(db_guard.conn.clone())
        .map_err(|e| format!("Failed to initialize AIStore: {}", e))?;
    store.list_conversations()
        .map_err(|e| format!("Failed to list conversations: {}", e))
}

#[tauri::command]
pub fn add_chat_message(
    conversation_id: String,
    role: String,
    content: String,
    model: Option<String>,
    tokens: Option<i32>,
    db: State<'_, Mutex<Database>>,
) -> Result<i64, String> {
    let db_guard = db.lock().map_err(|e| format!("Database lock error: {}", e))?;
    let store = AIStore::new(db_guard.conn.clone())
        .map_err(|e| format!("Failed to initialize AIStore: {}", e))?;
    store.add_message(&conversation_id, &role, &content, model.as_deref(), tokens)
        .map_err(|e| format!("Failed to add message: {}", e))
}

#[tauri::command]
pub fn get_chat_messages(
    conversation_id: String,
    db: State<'_, Mutex<Database>>,
) -> Result<Vec<crate::storage::ai::ChatMessage>, String> {
    let db_guard = db.lock().map_err(|e| format!("Database lock error: {}", e))?;
    let store = AIStore::new(db_guard.conn.clone())
        .map_err(|e| format!("Failed to initialize AIStore: {}", e))?;
    store.get_messages(&conversation_id)
        .map_err(|e| format!("Failed to get messages: {}", e))
}

#[tauri::command]
pub fn create_prompt_template(
    name: String,
    template: String,
    description: Option<String>,
    db: State<'_, Mutex<Database>>,
) -> Result<i64, String> {
    let db_guard = db.lock().map_err(|e| format!("Database lock error: {}", e))?;
    let store = AIStore::new(db_guard.conn.clone())
        .map_err(|e| format!("Failed to initialize AIStore: {}", e))?;
    store.create_template(&name, &template, description.as_deref())
        .map_err(|e| format!("Failed to create template: {}", e))
}

#[tauri::command]
pub fn list_prompt_templates(
    db: State<'_, Mutex<Database>>,
) -> Result<Vec<crate::storage::ai::PromptTemplate>, String> {
    let db_guard = db.lock().map_err(|e| format!("Database lock error: {}", e))?;
    let store = AIStore::new(db_guard.conn.clone())
        .map_err(|e| format!("Failed to initialize AIStore: {}", e))?;
    store.list_templates()
        .map_err(|e| format!("Failed to list templates: {}", e))
}

#[tauri::command]
pub fn get_prompt_template(
    name: String,
    db: State<'_, Mutex<Database>>,
) -> Result<Option<crate::storage::ai::PromptTemplate>, String> {
    let db_guard = db.lock().map_err(|e| format!("Database lock error: {}", e))?;
    let store = AIStore::new(db_guard.conn.clone())
        .map_err(|e| format!("Failed to initialize AIStore: {}", e))?;
    store.get_template(&name)
        .map_err(|e| format!("Failed to get template: {}", e))
}

