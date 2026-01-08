use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::{Arc, Mutex};
use tokio::sync::broadcast;
use tokio::time::{interval, Duration};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum WsMessage {
    SystemMetrics(crate::SystemMetrics),
    ProcessUpdate(crate::providers::process::ProcessInfo),
    NetworkUpdate(crate::providers::network::NetworkInterface),
    Error(crate::storage::ErrorRecord),
    ConfigUpdate { key: String, value: String },
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
        *self.system_metrics_tx.lock().unwrap() = Some(tx.clone());

        let connections = self.connections.clone();
        let app_handle = app.clone();

        // Use Tauri's async runtime to spawn the task
        tauri::async_runtime::spawn(async move {
            let mut interval = interval(Duration::from_secs(1));
            
            loop {
                interval.tick().await;

                // Fetch and broadcast system metrics
                if let Ok(metrics) = app_handle.state::<std::sync::Mutex<crate::providers::SystemProvider>>().try_lock() {
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
                    let conns = connections.lock().unwrap();
                    for conn in conns.values() {
                        if conn.topics.contains(&"system-metrics".to_string()) || 
                           conn.topics.contains(&"*".to_string()) {
                            let _ = conn.sender.send(msg.clone());
                        }
                    }
                } else {
                    // If we can't get the provider, send a ping to keep connections alive
                    let msg = WsMessage::Ping;
                    let _ = tx.send(msg.clone());
                    
                    let conns = connections.lock().unwrap();
                    for conn in conns.values() {
                        let _ = conn.sender.send(msg.clone());
                    }
                }
            }
        });
    }

    pub fn subscribe(&self, connection_id: String, topics: Vec<String>) -> Result<(), String> {
        let mut conns = self.connections.lock().unwrap();
        if let Some(conn) = conns.get_mut(&connection_id) {
            conn.topics = topics;
            Ok(())
        } else {
            Err("Connection not found".to_string())
        }
    }

    pub fn publish(&self, topic: &str, message: WsMessage) -> Result<(), String> {
        let conns = self.connections.lock().unwrap();
        for conn in conns.values() {
            if conn.topics.contains(&topic.to_string()) || conn.topics.contains(&"*".to_string()) {
                let _ = conn.sender.send(message.clone());
            }
        }
        Ok(())
    }

    pub fn get_connection_count(&self) -> usize {
        self.connections.lock().unwrap().len()
    }

    pub fn get_topics(&self) -> Vec<String> {
        let conns = self.connections.lock().unwrap();
        let mut topics = std::collections::HashSet::new();
        for conn in conns.values() {
            for topic in &conn.topics {
                topics.insert(topic.clone());
            }
        }
        topics.into_iter().collect()
    }
}

impl Default for WsServer {
    fn default() -> Self {
        Self::new()
    }
}

