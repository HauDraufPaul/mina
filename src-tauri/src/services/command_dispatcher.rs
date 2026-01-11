use anyhow::{Context, Result};
use serde_json::Value;
use std::sync::{Arc, Mutex};
use tauri::{AppHandle, Manager};

use crate::services::command_registry::CommandRegistry;

/// Dispatcher for invoking Tauri commands from workflow steps
pub struct CommandDispatcher {
    registry: Arc<Mutex<CommandRegistry>>,
}

impl CommandDispatcher {
    /// Create a new command dispatcher with a command registry
    pub fn new() -> Self {
        CommandDispatcher {
            registry: Arc::new(Mutex::new(CommandRegistry::new())),
        }
    }
    
    /// Get or create the global command registry
    fn get_registry(app: &AppHandle) -> Result<Arc<Mutex<CommandRegistry>>> {
        // Try to get existing registry from app state, or create new one
        if let Some(registry_guard) = app.try_state::<Arc<Mutex<CommandRegistry>>>() {
            Ok(registry_guard.inner().clone())
        } else {
            // Create new registry and store in app state
            let registry = Arc::new(Mutex::new(CommandRegistry::new()));
            app.manage(registry.clone());
            Ok(registry)
        }
    }
    
    /// Invoke a Tauri command by name with arguments
    pub async fn invoke_command(
        app: &AppHandle,
        command: &str,
        args: Value,
    ) -> Result<Value> {
        let registry = Self::get_registry(app)?;
        let registry_guard = registry.lock()
            .map_err(|e| anyhow::anyhow!("Registry lock error: {}", e))?;
        
        // Check if command is registered
        if !registry_guard.has_command(command) {
            // Return a helpful error message
            let available = registry_guard.list_commands();
            return Err(anyhow::anyhow!(
                "Command '{}' not found in registry. Available commands: {}",
                command,
                available.join(", ")
            ));
        }
        
        // Invoke the command
        registry_guard.invoke(command, args, app)
            .map_err(|e| anyhow::anyhow!("Failed to invoke command {}: {}", command, e))
    }
    
    /// List all available commands (for documentation/autocomplete)
    pub fn list_commands(app: &AppHandle) -> Result<Vec<String>> {
        let registry = Self::get_registry(app)?;
        let registry_guard = registry.lock()
            .map_err(|e| anyhow::anyhow!("Registry lock error: {}", e))?;
        Ok(registry_guard.list_commands())
    }
}

impl Default for CommandDispatcher {
    fn default() -> Self {
        Self::new()
    }
}
