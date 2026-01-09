use crate::ws::WsServer;
use std::sync::Mutex;
use tauri::State;
use uuid::Uuid;

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

/// Connect to WebSocket server and return connection ID
#[tauri::command]
pub fn ws_connect(
    topics: Vec<String>,
    server: State<'_, Mutex<WsServer>>,
) -> Result<String, String> {
    let connection_id = Uuid::new_v4().to_string();
    let server_guard = server.lock().map_err(|e| format!("Server lock error: {}", e))?;
    server_guard.add_connection(connection_id.clone(), topics)
        .map_err(|e| format!("Failed to add connection: {}", e))?;
    Ok(connection_id)
}

/// Subscribe to topics for an existing connection
#[tauri::command]
pub fn ws_subscribe(
    connection_id: String,
    topics: Vec<String>,
    server: State<'_, Mutex<WsServer>>,
) -> Result<(), String> {
    let server_guard = server.lock().map_err(|e| format!("Server lock error: {}", e))?;
    server_guard.update_subscriptions(&connection_id, topics)
        .map_err(|e| format!("Failed to subscribe: {}", e))?;
    Ok(())
}

/// Unsubscribe from topics (or remove connection if empty)
#[tauri::command]
pub fn ws_unsubscribe(
    connection_id: String,
    topics: Vec<String>,
    server: State<'_, Mutex<WsServer>>,
) -> Result<(), String> {
    let server_guard = server.lock().map_err(|e| format!("Server lock error: {}", e))?;
    // Get current topics
    let current_topics = server_guard.get_connection_topics(&connection_id)
        .map_err(|e| format!("Failed to get connection topics: {}", e))?;
    
    // Remove specified topics
    let new_topics: Vec<String> = current_topics
        .into_iter()
        .filter(|t| !topics.contains(t))
        .collect();
    
    // Update subscriptions
    server_guard.update_subscriptions(&connection_id, new_topics)
        .map_err(|e| format!("Failed to unsubscribe: {}", e))?;
    Ok(())
}

/// Get connection status
#[tauri::command]
pub fn ws_get_connection_status(
    connection_id: String,
    server: State<'_, Mutex<WsServer>>,
) -> Result<bool, String> {
    let server_guard = server.lock().map_err(|e| format!("Server lock error: {}", e))?;
    server_guard.get_connection_status(&connection_id)
        .map_err(|e| format!("Failed to get connection status: {}", e))
}

/// Disconnect from WebSocket server
#[tauri::command]
pub fn ws_disconnect(
    connection_id: String,
    server: State<'_, Mutex<WsServer>>,
) -> Result<(), String> {
    let server_guard = server.lock().map_err(|e| format!("Server lock error: {}", e))?;
    server_guard.remove_connection(&connection_id)
        .map_err(|e| format!("Failed to disconnect: {}", e))?;
    Ok(())
}

