use crate::ws::WsServer;
use std::sync::Mutex;
use tauri::{State, Manager, Emitter};
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
    app: tauri::AppHandle,
) -> Result<String, String> {
    let connection_id = Uuid::new_v4().to_string();
    let server_guard = server.lock().map_err(|e| format!("Server lock error: {}", e))?;
    let receiver = server_guard.add_connection(connection_id.clone(), topics)
        .map_err(|e| format!("Failed to add connection: {}", e))?;
    
    // Spawn a task to listen to messages from this connection and forward them to the frontend
    let app_handle = app.clone();
    let conn_id = connection_id.clone();
    tauri::async_runtime::spawn(async move {
        let mut rx = receiver;
        loop {
            match rx.recv().await {
                Ok(msg) => {
                    // Convert WsMessage to the format expected by frontend
                    let event_data = match msg {
                        crate::ws::WsMessage::SystemMetrics(metrics) => {
                            serde_json::json!({
                                "type": "system-metrics",
                                "data": metrics,
                                "timestamp": chrono::Utc::now().timestamp_millis(),
                            })
                        }
                        crate::ws::WsMessage::ProcessUpdate(process) => {
                            serde_json::json!({
                                "type": "process-update",
                                "data": process,
                                "timestamp": chrono::Utc::now().timestamp_millis(),
                            })
                        }
                        crate::ws::WsMessage::NetworkUpdate(network) => {
                            serde_json::json!({
                                "type": "network-update",
                                "data": network,
                                "timestamp": chrono::Utc::now().timestamp_millis(),
                            })
                        }
                        crate::ws::WsMessage::Error(error) => {
                            serde_json::json!({
                                "type": "error",
                                "data": error,
                                "timestamp": chrono::Utc::now().timestamp_millis(),
                            })
                        }
                        crate::ws::WsMessage::ConfigUpdate { key, value } => {
                            serde_json::json!({
                                "type": "config-update",
                                "data": { "key": key, "value": value },
                                "timestamp": chrono::Utc::now().timestamp_millis(),
                            })
                        }
                        crate::ws::WsMessage::StockNews(news) => {
                            serde_json::json!({
                                "type": "stock-news",
                                "data": news,
                                "timestamp": chrono::Utc::now().timestamp_millis(),
                            })
                        }
                        crate::ws::WsMessage::StockNewsBatch(news_batch) => {
                            serde_json::json!({
                                "type": "stock-news-batch",
                                "data": news_batch,
                                "timestamp": chrono::Utc::now().timestamp_millis(),
                            })
                        }
                        crate::ws::WsMessage::MarketData(data) => {
                            serde_json::json!({
                                "type": "market-data",
                                "data": data,
                                "timestamp": chrono::Utc::now().timestamp_millis(),
                            })
                        }
                        crate::ws::WsMessage::MarketDataBatch(data_batch) => {
                            serde_json::json!({
                                "type": "market-data-batch",
                                "data": data_batch,
                                "timestamp": chrono::Utc::now().timestamp_millis(),
                            })
                        }
                        crate::ws::WsMessage::Message(msg) => {
                            serde_json::json!({
                                "type": "message",
                                "data": msg,
                                "timestamp": chrono::Utc::now().timestamp_millis(),
                            })
                        }
                        crate::ws::WsMessage::MessageTyping { conversation_id, sender } => {
                            serde_json::json!({
                                "type": "message-typing",
                                "data": { "conversation_id": conversation_id, "sender": sender },
                                "timestamp": chrono::Utc::now().timestamp_millis(),
                            })
                        }
                        crate::ws::WsMessage::Ping => {
                            serde_json::json!({
                                "type": "ping",
                                "data": null,
                                "timestamp": chrono::Utc::now().timestamp_millis(),
                            })
                        }
                        crate::ws::WsMessage::Pong => {
                            serde_json::json!({
                                "type": "pong",
                                "data": null,
                                "timestamp": chrono::Utc::now().timestamp_millis(),
                            })
                        }
                    };
                    
                    // Emit the message to the frontend
                    let _ = app_handle.emit("ws-message", event_data);
                }
                Err(tokio::sync::broadcast::error::RecvError::Closed) => {
                    // Channel closed, connection is being removed
                    break;
                }
                Err(tokio::sync::broadcast::error::RecvError::Lagged(skipped)) => {
                    eprintln!("WebSocket connection {} lagged, skipped {} messages", conn_id, skipped);
                    // Continue receiving
                }
            }
        }
    });
    
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

