use crate::ws::WsServer;
use std::sync::Mutex;
use tauri::AppHandle;

#[tauri::command]
pub fn get_ws_connection_count(app: AppHandle) -> Result<usize, String> {
    let server = app.try_state::<Mutex<WsServer>>()
        .ok_or("WsServer not found")?;
    let server = server.lock().map_err(|e| format!("Server lock error: {}", e))?;
    Ok(server.get_connection_count())
}

#[tauri::command]
pub fn get_ws_topics(app: AppHandle) -> Result<Vec<String>, String> {
    let server = app.try_state::<Mutex<WsServer>>()
        .ok_or("WsServer not found")?;
    let server = server.lock().map_err(|e| format!("Server lock error: {}", e))?;
    Ok(server.get_topics())
}

#[tauri::command]
pub fn publish_ws_message(
    topic: String,
    message_type: String,
    app: AppHandle,
) -> Result<(), String> {
    let server = app.try_state::<Mutex<WsServer>>()
        .ok_or("WsServer not found")?;
    let server = server.lock().map_err(|e| format!("Server lock error: {}", e))?;
    
    // Create appropriate message based on type
    let msg = match message_type.as_str() {
        "ping" => crate::ws::WsMessage::Ping,
        _ => return Err("Unknown message type".to_string()),
    };
    
    server.publish(&topic, msg)
}

