use anyhow::{Context, Result};
use serde_json::Value;
use std::collections::HashMap;
use std::sync::{Arc, Mutex};
use tauri::{AppHandle, Manager};

use crate::storage::Database;
use crate::services::api_key_manager::APIKeyManager;

/// Command handler function type
type CommandHandler = Box<dyn Fn(&AppHandle, Value) -> Result<Value> + Send + Sync>;

/// Registry for Tauri commands that can be invoked from Rust
pub struct CommandRegistry {
    handlers: HashMap<String, CommandHandler>,
}

impl CommandRegistry {
    pub fn new() -> Self {
        let mut registry = CommandRegistry {
            handlers: HashMap::new(),
        };
        
        // Register all available commands
        registry.register_commands();
        
        registry
    }
    
    fn register_commands(&mut self) {
        // Register system commands
        self.register("get_system_metrics", |app, _args| {
            let provider = app.try_state::<Mutex<crate::providers::SystemProvider>>()
                .ok_or_else(|| anyhow::anyhow!("SystemProvider not found in app state"))?;
            let provider_guard = provider.lock()
                .map_err(|e| anyhow::anyhow!("Provider lock error: {}", e))?;
            provider_guard.refresh();
            
            // Manually implement get_system_metrics logic
            let cpu_usage = provider_guard.get_cpu_usage();
            let cpu_count = provider_guard.get_cpu_count();
            let cpu_frequency = provider_guard.get_cpu_frequency();
            let total_memory = provider_guard.get_memory_total();
            let used_memory = provider_guard.get_memory_used();
            let free_memory = provider_guard.get_memory_free();
            let memory_usage = if total_memory > 0 {
                (used_memory as f64 / total_memory as f64) * 100.0
            } else {
                0.0
            };
            let (disk_total, disk_used, disk_free) = provider_guard.get_disk_metrics();
            let disk_usage = if disk_total > 0 {
                (disk_used as f64 / disk_total as f64) * 100.0
            } else {
                0.0
            };
            let (total_rx, total_tx) = provider_guard.get_network_total();
            let (rx_speed, tx_speed) = provider_guard.get_network_speeds();
            
            let result = crate::SystemMetrics {
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
            Ok(serde_json::to_value(result)?)
        });
        
        // Register network commands - simplified implementation
        self.register("get_network_interfaces", |app, _args| {
            let provider = app.try_state::<Mutex<crate::providers::NetworkProvider>>()
                .ok_or_else(|| anyhow::anyhow!("NetworkProvider not found in app state"))?;
            let mut provider_guard = provider.lock()
                .map_err(|e| anyhow::anyhow!("Provider lock error: {}", e))?;
            provider_guard.refresh();
            let interfaces = provider_guard.get_interfaces();
            Ok(serde_json::to_value(interfaces)?)
        });
        
        self.register("get_network_connections", |app, _args| {
            let provider = app.try_state::<Mutex<crate::providers::NetworkProvider>>()
                .ok_or_else(|| anyhow::anyhow!("NetworkProvider not found in app state"))?;
            let provider_guard = provider.lock()
                .map_err(|e| anyhow::anyhow!("Provider lock error: {}", e))?;
            let connections = provider_guard.get_connections();
            Ok(serde_json::to_value(connections)?)
        });
        
        // Register database-backed commands
        self.register("global_search", |app, args| {
            let query = args.get("query")
                .and_then(|v| v.as_str())
                .ok_or_else(|| anyhow::anyhow!("Missing 'query' parameter"))?
                .to_string();
            let limit = args.get("limit").and_then(|v| v.as_i64()).map(|l| l as i32);
            let db = app.try_state::<Mutex<Database>>()
                .ok_or_else(|| anyhow::anyhow!("Database not found in app state"))?;
            // Call the service directly with the database mutex
            let result = crate::services::GlobalSearchService::search(&query, limit, db.inner())
                .map_err(|e| anyhow::anyhow!("{}", e))?;
            Ok(serde_json::to_value(result)?)
        });
        
        // Register temporal commands - call service directly
        self.register("temporal_list_events", |app, args| {
            let limit = args.get("limit").and_then(|v| v.as_i64());
            let from_ts = args.get("from_ts").and_then(|v| v.as_i64());
            let to_ts = args.get("to_ts").and_then(|v| v.as_i64());
            let db = app.try_state::<Mutex<Database>>()
                .ok_or_else(|| anyhow::anyhow!("Database not found in app state"))?;
            let db_guard = db.lock()
                .map_err(|e| anyhow::anyhow!("Database lock error: {}", e))?;
            let store = crate::storage::TemporalStore::new(db_guard.conn.clone());
            let result = store.list_events(limit.unwrap_or(200), from_ts, to_ts)
                .map_err(|e| anyhow::anyhow!("{}", e))?;
            Ok(serde_json::to_value(result)?)
        });
        
        self.register("temporal_search", |app, args| {
            let query = args.get("query")
                .and_then(|v| v.as_str())
                .ok_or_else(|| anyhow::anyhow!("Missing 'query' parameter"))?
                .to_string();
            let limit = args.get("limit").and_then(|v| v.as_i64());
            let db = app.try_state::<Mutex<Database>>()
                .ok_or_else(|| anyhow::anyhow!("Database not found in app state"))?;
            let db_guard = db.lock()
                .map_err(|e| anyhow::anyhow!("Database lock error: {}", e))?;
            let store = crate::storage::TemporalStore::new(db_guard.conn.clone());
            let result = store.search(&query, limit.unwrap_or(20))
                .map_err(|e| anyhow::anyhow!("{}", e))?;
            Ok(serde_json::to_value(result)?)
        });
        
        // Register portfolio commands
        self.register("list_portfolios", |app, _args| {
            let db = app.try_state::<Mutex<Database>>()
                .ok_or_else(|| anyhow::anyhow!("Database not found in app state"))?;
            let db_guard = db.lock()
                .map_err(|e| anyhow::anyhow!("Database lock error: {}", e))?;
            let store = crate::storage::portfolio::PortfolioStore::new(db_guard.conn.clone());
            let result = store.list_portfolios()
                .map_err(|e| anyhow::anyhow!("{}", e))?;
            Ok(serde_json::to_value(result)?)
        });
        
        self.register("get_portfolio", |app, args| {
            let id = args.get("id")
                .and_then(|v| v.as_i64())
                .ok_or_else(|| anyhow::anyhow!("Missing 'id' parameter"))?;
            let db = app.try_state::<Mutex<Database>>()
                .ok_or_else(|| anyhow::anyhow!("Database not found in app state"))?;
            let db_guard = db.lock()
                .map_err(|e| anyhow::anyhow!("Database lock error: {}", e))?;
            let store = crate::storage::portfolio::PortfolioStore::new(db_guard.conn.clone());
            let result = store.get_portfolio(id)
                .map_err(|e| anyhow::anyhow!("{}", e))?;
            Ok(serde_json::to_value(result)?)
        });
        
        // Register vector store commands
        self.register("create_collection", |app, args| {
            let name = args.get("name")
                .and_then(|v| v.as_str())
                .ok_or_else(|| anyhow::anyhow!("Missing 'name' parameter"))?
                .to_string();
            let dimension = args.get("dimension")
                .and_then(|v| v.as_i64())
                .map(|d| d as i32)
                .unwrap_or(1536);
            let db = app.try_state::<Mutex<Database>>()
                .ok_or_else(|| anyhow::anyhow!("Database not found in app state"))?;
            let db_guard = db.lock()
                .map_err(|e| anyhow::anyhow!("Database lock error: {}", e))?;
            let store = crate::storage::VectorStore::new(db_guard.conn.clone());
            store.create_collection(&name, dimension)
                .map_err(|e| anyhow::anyhow!("{}", e))?;
            Ok(serde_json::json!({"success": true}))
        });
        
        self.register("list_collections", |app, _args| {
            let db = app.try_state::<Mutex<Database>>()
                .ok_or_else(|| anyhow::anyhow!("Database not found in app state"))?;
            let db_guard = db.lock()
                .map_err(|e| anyhow::anyhow!("Database lock error: {}", e))?;
            let store = crate::storage::VectorStore::new(db_guard.conn.clone());
            let result = store.list_collections()
                .map_err(|e| anyhow::anyhow!("{}", e))?;
            Ok(serde_json::to_value(result)?)
        });
        
        // Add more commands as needed...
        // This is a representative sample. In a full implementation,
        // you would register all 197+ commands here.
    }
    
    fn register<F>(&mut self, name: &str, handler: F)
    where
        F: Fn(&AppHandle, Value) -> Result<Value> + Send + Sync + 'static,
    {
        self.handlers.insert(name.to_string(), Box::new(handler));
    }
    
    /// Invoke a command by name with arguments
    pub fn invoke(&self, command: &str, args: Value, app: &AppHandle) -> Result<Value> {
        let handler = self.handlers.get(command)
            .ok_or_else(|| anyhow::anyhow!("Command not found: {}", command))?;
        
        handler(app, args)
            .with_context(|| format!("Failed to invoke command: {}", command))
    }
    
    /// List all registered commands
    pub fn list_commands(&self) -> Vec<String> {
        self.handlers.keys().cloned().collect()
    }
    
    /// Check if a command is registered
    pub fn has_command(&self, command: &str) -> bool {
        self.handlers.contains_key(command)
    }
}

impl Default for CommandRegistry {
    fn default() -> Self {
        Self::new()
    }
}
