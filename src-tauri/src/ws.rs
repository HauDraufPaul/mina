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
        let mut interval = interval(Duration::from_secs(1));

        tokio::spawn(async move {
            loop {
                interval.tick().await;

                // Broadcast system metrics
                // Note: In a real implementation, this would fetch metrics from the provider
                // For now, we'll just send ping messages to keep connections alive
                let msg = WsMessage::Ping;
                let _ = tx.send(msg.clone());

                // Send to all connections
                let conns = connections.lock().unwrap();
                for conn in conns.values() {
                    let _ = conn.sender.send(msg.clone());
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

