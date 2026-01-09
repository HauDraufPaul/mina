use crate::storage::messaging::{MessagingStore, MessagingConversation, Message, MessageAttachment};
use crate::storage::Database;
use crate::ws::WsServer;
use serde_json::Value;
use std::sync::Mutex;
use tauri::{State, Emitter};

#[tauri::command]
pub fn messaging_create_conversation(
    name: String,
    db: State<'_, Mutex<Database>>,
) -> Result<i64, String> {
    let db_guard = db.lock().map_err(|e| format!("Database lock error: {}", e))?;
    let store = MessagingStore::new(db_guard.conn.clone());
    store
        .create_conversation(&name)
        .map_err(|e| format!("Failed to create conversation: {}", e))
}

#[tauri::command]
pub fn messaging_list_conversations(
    db: State<'_, Mutex<Database>>,
) -> Result<Vec<MessagingConversation>, String> {
    let db_guard = db.lock().map_err(|e| format!("Database lock error: {}", e))?;
    let store = MessagingStore::new(db_guard.conn.clone());
    store
        .list_conversations()
        .map_err(|e| format!("Failed to list conversations: {}", e))
}

#[tauri::command]
pub fn send_message(
    conversation_id: i64,
    sender: String,
    content: String,
    db: State<'_, Mutex<Database>>,
    ws_server: State<'_, Mutex<WsServer>>,
    app: tauri::AppHandle,
) -> Result<i64, String> {
    let db_guard = db.lock().map_err(|e| format!("Database lock error: {}", e))?;
    let store = MessagingStore::new(db_guard.conn.clone());
    
    let message_id = store
        .send_message(conversation_id, &sender, &content)
        .map_err(|e| format!("Failed to send message: {}", e))?;
    
    // Get the created message
    if let Ok(Some(message)) = store.get_message(message_id) {
        // Emit WebSocket event
        let ws_guard = ws_server.lock().map_err(|e| format!("WebSocket lock error: {}", e))?;
        let msg = crate::ws::WsMessage::Message(message.clone());
        let _ = ws_guard.publish("messaging", msg.clone());
        
        // Also emit Tauri event for frontend
        let _ = app.emit("ws-message", serde_json::json!({
            "type": "message",
            "data": message,
            "timestamp": chrono::Utc::now().timestamp_millis(),
        }));
    }
    
    Ok(message_id)
}

#[tauri::command]
pub fn get_conversation_messages(
    conversation_id: i64,
    limit: Option<i64>,
    db: State<'_, Mutex<Database>>,
) -> Result<Vec<Message>, String> {
    let db_guard = db.lock().map_err(|e| format!("Database lock error: {}", e))?;
    let store = MessagingStore::new(db_guard.conn.clone());
    store
        .get_conversation_messages(conversation_id, limit.unwrap_or(100))
        .map_err(|e| format!("Failed to get messages: {}", e))
}

#[tauri::command]
pub fn attach_market_context(
    message_id: i64,
    attachment_type: String,
    data_json: Value,
    db: State<'_, Mutex<Database>>,
) -> Result<i64, String> {
    let db_guard = db.lock().map_err(|e| format!("Database lock error: {}", e))?;
    let store = MessagingStore::new(db_guard.conn.clone());
    store
        .attach_to_message(message_id, &attachment_type, &data_json)
        .map_err(|e| format!("Failed to attach context: {}", e))
}

#[tauri::command]
pub fn get_message_attachments(
    message_id: i64,
    db: State<'_, Mutex<Database>>,
) -> Result<Vec<MessageAttachment>, String> {
    let db_guard = db.lock().map_err(|e| format!("Database lock error: {}", e))?;
    let store = MessagingStore::new(db_guard.conn.clone());
    store
        .get_message_attachments(message_id)
        .map_err(|e| format!("Failed to get attachments: {}", e))
}
