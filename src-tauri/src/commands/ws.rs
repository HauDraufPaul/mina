use crate::ws::WsServer;
use std::sync::Mutex;
use tauri::State;

#[tauri::command]
pub fn get_ws_connection_count(server: State<'_, Mutex<WsServer>>) -> Result<usize, String> {
    let server_guard = server.lock().map_err(|e| format!("Server lock error: {}", e))?;
    Ok(server_guard.get_connection_count())
}

#[tauri::command]
pub fn get_ws_topics(server: State<'_, Mutex<WsServer>>) -> Result<Vec<String>, String> {
    let server_guard = server.lock().map_err(|e| format!("Server lock error: {}", e))?;
    Ok(server_guard.get_topics())
}

#[tauri::command]
pub fn publish_ws_message(
    topic: String,
    message_type: String,
    server: State<'_, Mutex<WsServer>>,
) -> Result<(), String> {
    let server_guard = server.lock().map_err(|e| format!("Server lock error: {}", e))?;
    
    // Create appropriate message based on type
    let msg = match message_type.as_str() {
        "ping" => crate::ws::WsMessage::Ping,
        _ => return Err("Unknown message type".to_string()),
    };
    
    server_guard.publish(&topic, msg)
}

