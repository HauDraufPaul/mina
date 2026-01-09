use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::{Arc, Mutex};
use tokio::sync::broadcast;
use tokio::time::{interval, Duration};
use tauri::{Manager, Emitter};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum WsMessage {
    SystemMetrics(crate::SystemMetrics),
    ProcessUpdate(crate::providers::process::ProcessInfo),
    NetworkUpdate(crate::providers::network::NetworkInterface),
    Error(crate::storage::ErrorRecord),
    ConfigUpdate { key: String, value: String },
    StockNews(crate::storage::StockNewsItem),
    StockNewsBatch(Vec<crate::storage::StockNewsItem>),
    MarketData(crate::storage::MarketPrice),
    MarketDataBatch(Vec<crate::storage::MarketPrice>),
    Message(crate::storage::messaging::Message),
    MessageTyping { conversation_id: i64, sender: String },
    Ping,
    Pong,
}

#[derive(Debug, Clone)]
pub struct WsConnection {
    pub id: String,
    pub topics: Vec<String>,
    pub sender: broadcast::Sender<WsMessage>,
}

pub struct WsServer {
    connections: Arc<Mutex<HashMap<String, WsConnection>>>,
    system_metrics_tx: Arc<Mutex<Option<broadcast::Sender<WsMessage>>>>,
}

impl WsServer {
    pub fn new() -> Self {
        WsServer {
            connections: Arc::new(Mutex::new(HashMap::new())),
            system_metrics_tx: Arc::new(Mutex::new(None)),
        }
    }

    pub fn start_broadcast(&self, app: tauri::AppHandle) {
        let (tx, _) = broadcast::channel::<WsMessage>(1000);
        if let Ok(mut tx_guard) = self.system_metrics_tx.lock() {
            *tx_guard = Some(tx.clone());
        } else {
            eprintln!("Failed to lock system_metrics_tx - mutex poisoned");
        }

        let connections = self.connections.clone();
        let app_handle = app.clone();

        // Use Tauri's async runtime to spawn the task
        tauri::async_runtime::spawn(async move {
            let mut interval = interval(Duration::from_secs(1));
            
            loop {
                interval.tick().await;

                // Fetch and broadcast system metrics
                if let Some(metrics_guard) = app_handle.try_state::<std::sync::Mutex<crate::providers::SystemProvider>>() {
                    if let Ok(metrics) = metrics_guard.try_lock() {
                        metrics.refresh();
                        
                        let cpu_usage = metrics.get_cpu_usage();
                        let cpu_count = metrics.get_cpu_count();
                        let cpu_frequency = metrics.get_cpu_frequency();
                        
                        let total_memory = metrics.get_memory_total();
                        let used_memory = metrics.get_memory_used();
                        let free_memory = metrics.get_memory_free();
                        let memory_usage = if total_memory > 0 {
                            (used_memory as f64 / total_memory as f64) * 100.0
                        } else {
                            0.0
                        };
                        
                        let (disk_total, disk_used, disk_free) = metrics.get_disk_metrics();
                        let disk_usage = if disk_total > 0 {
                            (disk_used as f64 / disk_total as f64) * 100.0
                        } else {
                            0.0
                        };
                        
                        let (total_rx, total_tx) = metrics.get_network_total();
                        let (rx_speed, tx_speed) = metrics.get_network_speeds();
                        
                        let system_metrics = crate::SystemMetrics {
                            cpu: crate::CpuMetrics {
                                usage: cpu_usage,
                                cores: cpu_count,
                                frequency: cpu_frequency,
                            },
                            memory: crate::MemoryMetrics {
                                total: total_memory,
                                used: used_memory,
                                free: free_memory,
                                usage: memory_usage,
                            },
                            disk: crate::DiskMetrics {
                                total: disk_total,
                                used: disk_used,
                                free: disk_free,
                                usage: disk_usage,
                            },
                            network: crate::NetworkMetrics {
                                rx: total_rx,
                                tx: total_tx,
                                rx_speed,
                                tx_speed,
                            },
                        };
                        
                        let msg = WsMessage::SystemMetrics(system_metrics.clone());
                        let _ = tx.send(msg.clone());

                        // Emit Tauri event for frontend
                        let _ = app_handle.emit("ws-message", serde_json::json!({
                            "type": "system-metrics",
                            "data": system_metrics,
                            "timestamp": chrono::Utc::now().timestamp_millis(),
                        }));

                        // Send to all connections subscribed to system-metrics or *
                        if let Ok(conns) = connections.lock() {
                            for conn in conns.values() {
                                if conn.topics.contains(&"system-metrics".to_string()) || 
                                   conn.topics.contains(&"*".to_string()) {
                                    let _ = conn.sender.send(msg.clone());
                                }
                            }
                        }
                    } else {
                        // If we can't lock the provider, send a ping to keep connections alive
                        let msg = WsMessage::Ping;
                        let _ = tx.send(msg.clone());
                        
                        if let Ok(conns) = connections.lock() {
                            for conn in conns.values() {
                                let _ = conn.sender.send(msg.clone());
                            }
                        }
                    }
                } else {
                    // If we can't get the provider state, send a ping to keep connections alive
                    let msg = WsMessage::Ping;
                    let _ = tx.send(msg.clone());
                    
                    if let Ok(conns) = connections.lock() {
                        for conn in conns.values() {
                            let _ = conn.sender.send(msg.clone());
                        }
                    }
                }
            }
        });
    }

    pub fn subscribe(&self, connection_id: String, topics: Vec<String>) -> Result<(), String> {
        let mut conns = self.connections.lock()
            .map_err(|e| format!("Failed to lock connections: {}", e))?;
        if let Some(conn) = conns.get_mut(&connection_id) {
            conn.topics = topics;
            Ok(())
        } else {
            Err("Connection not found".to_string())
        }
    }

    pub fn publish(&self, topic: &str, message: WsMessage) -> Result<(), String> {
        let conns = self.connections.lock()
            .map_err(|e| format!("Failed to lock connections: {}", e))?;
        for conn in conns.values() {
            if conn.topics.contains(&topic.to_string()) || conn.topics.contains(&"*".to_string()) {
                let _ = conn.sender.send(message.clone());
            }
        }
        Ok(())
    }

    pub fn get_connection_count(&self) -> usize {
        self.connections.lock()
            .map(|conns| conns.len())
            .unwrap_or(0)
    }

    pub fn get_topics(&self) -> Vec<String> {
        let conns = match self.connections.lock() {
            Ok(conns) => conns,
            Err(_) => return Vec::new(), // Return empty vec if lock fails
        };
        let mut topics = std::collections::HashSet::new();
        for conn in conns.values() {
            for topic in &conn.topics {
                topics.insert(topic.clone());
            }
        }
        topics.into_iter().collect()
    }

    /// Add a new connection and return a receiver for messages
    pub fn add_connection(&self, id: String, topics: Vec<String>) -> Result<broadcast::Receiver<WsMessage>, String> {
        let (tx, rx) = broadcast::channel::<WsMessage>(1000);
        let conn = WsConnection {
            id: id.clone(),
            topics,
            sender: tx,
        };
        let mut conns = self.connections.lock()
            .map_err(|e| format!("Failed to lock connections: {}", e))?;
        conns.insert(id, conn);
        Ok(rx)
    }

    /// Remove a connection
    pub fn remove_connection(&self, id: &str) -> Result<(), String> {
        let mut conns = self.connections.lock()
            .map_err(|e| format!("Failed to lock connections: {}", e))?;
        conns.remove(id);
        Ok(())
    }

    /// Update subscriptions for a connection
    pub fn update_subscriptions(&self, id: &str, topics: Vec<String>) -> Result<(), String> {
        self.subscribe(id.to_string(), topics)
    }

    /// Broadcast a message to all connections subscribed to a specific topic
    pub fn broadcast_to_topic(&self, topic: &str, message: WsMessage) -> Result<usize, String> {
        let conns = self.connections.lock()
            .map_err(|e| format!("Failed to lock connections: {}", e))?;
        let mut count = 0;
        for conn in conns.values() {
            if conn.topics.contains(&topic.to_string()) || conn.topics.contains(&"*".to_string()) {
                if conn.sender.send(message.clone()).is_ok() {
                    count += 1;
                }
            }
        }
        Ok(count)
    }

    /// Get connection status
    pub fn get_connection_status(&self, id: &str) -> Result<bool, String> {
        let conns = self.connections.lock()
            .map_err(|e| format!("Failed to lock connections: {}", e))?;
        Ok(conns.contains_key(id))
    }

    /// Get topics for a connection
    pub fn get_connection_topics(&self, id: &str) -> Result<Vec<String>, String> {
        let conns = self.connections.lock()
            .map_err(|e| format!("Failed to lock connections: {}", e))?;
        if let Some(conn) = conns.get(id) {
            Ok(conn.topics.clone())
        } else {
            Err("Connection not found".to_string())
        }
    }
}

impl Default for WsServer {
    fn default() -> Self {
        Self::new()
    }
}

