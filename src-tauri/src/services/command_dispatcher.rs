use anyhow::{Context, Result};
use serde_json::{json, Value};
use tauri::AppHandle;

/// Dispatcher for invoking Tauri commands from workflow steps
/// Note: Direct command invocation from Rust is limited. For full command support,
/// scripts should use the Tauri invoke bridge (when implemented).
pub struct CommandDispatcher;

impl CommandDispatcher {
    /// Invoke a Tauri command by name with arguments
    /// Currently supports a limited set of commands that can be called directly
    pub async fn invoke_command(
        _app: &AppHandle,
        command: &str,
        args: Value,
    ) -> Result<Value> {
        // For now, CallCommand step will return a structured response
        // indicating the command that would be called
        // Full implementation would require a command bridge server
        let result = json!({
            "command": command,
            "args": args,
            "status": "queued",
            "message": "CallCommand executed. Note: Direct Tauri command invocation from workflows requires a command bridge server (not yet implemented). For now, use ExecuteScript step with Deno to call Tauri commands via invoke()."
        });
        
        Ok(result)
    }
    
    /// List all available commands (for documentation/autocomplete)
    pub fn list_commands() -> Vec<&'static str> {
        vec![
            "get_system_metrics",
            "get_market_prices",
            "create_health_check",
            "list_health_checks",
            "create_alert",
        ]
    }
}
