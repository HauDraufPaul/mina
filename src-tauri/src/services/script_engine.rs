use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};
use std::sync::{Arc, Mutex};
use std::time::Duration;
use tauri::AppHandle;
use tokio::process::Command;
use tokio::time::timeout;

use crate::storage::Database;
use crate::storage::automation::AutomationStore;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ScriptExecutionResult {
    pub success: bool,
    pub data: Value,
    pub stdout: String,
    pub stderr: String,
    pub execution_time_ms: u64,
    pub error: Option<String>,
}

pub struct ScriptEngine {
    db: Arc<Mutex<Database>>,
    app: AppHandle,
}

impl ScriptEngine {
    pub fn new(db: Arc<Mutex<Database>>, app: AppHandle) -> Self {
        ScriptEngine { db, app }
    }

    pub async fn execute_script(
        &self,
        script_id: i64,
        inputs: Option<Value>,
    ) -> Result<ScriptExecutionResult> {
        let start_time = std::time::Instant::now();

        // Load script from database
        let script = {
            let db_guard = self.db.lock()
                .map_err(|e| anyhow::anyhow!("Database lock poisoned: {}", e))?;
            let store = AutomationStore::new(db_guard.conn.clone())
                .map_err(|e| anyhow::anyhow!("Failed to initialize AutomationStore: {}", e))?;
            store.get_script(script_id)
                .context("Failed to load script")?
                .ok_or_else(|| anyhow::anyhow!("Script not found: {}", script_id))?
        };

        if !script.enabled {
            return Ok(ScriptExecutionResult {
                success: false,
                data: Value::Null,
                stdout: String::new(),
                stderr: String::new(),
                execution_time_ms: 0,
                error: Some("Script is disabled".to_string()),
            });
        }

        // Prepare script code with Tauri bridge and get temp file path
        let script_path = self.prepare_script_code(&script.content, &script.language, inputs)?;

        // Execute script with timeout using Deno subprocess
        let execution_result = timeout(
            Duration::from_secs(30),
            self.execute_with_deno(&script_path, &script.language),
        )
        .await;

        let execution_time_ms = start_time.elapsed().as_millis() as u64;

        match execution_result {
            Ok(Ok((data, stdout, stderr))) => Ok(ScriptExecutionResult {
                success: true,
                data,
                stdout,
                stderr,
                execution_time_ms,
                error: None,
            }),
            Ok(Err(e)) => Ok(ScriptExecutionResult {
                success: false,
                data: Value::Null,
                stdout: String::new(),
                stderr: e.to_string(),
                execution_time_ms,
                error: Some(e.to_string()),
            }),
            Err(_) => Ok(ScriptExecutionResult {
                success: false,
                data: Value::Null,
                stdout: String::new(),
                stderr: String::new(),
                execution_time_ms,
                error: Some("Script execution timeout (30s)".to_string()),
            }),
        }
    }

    fn prepare_script_code(
        &self,
        content: &str,
        language: &str,
        inputs: Option<Value>,
    ) -> Result<String> {
        // Inject Tauri bridge and inputs
        let inputs_json = inputs
            .map(|v| serde_json::to_string(&v).unwrap_or_else(|_| "{}".to_string()))
            .unwrap_or_else(|| "{}".to_string());

        // Create a bridge server URL (will be set up separately)
        // For now, scripts will use HTTP to communicate with Tauri
        let bridge_code = format!(
            r#"
// MINA Script Bridge for Deno
const __MINA_INPUTS__ = {};
const __MINA_BRIDGE_URL__ = Deno.env.get("MINA_BRIDGE_URL") || "http://localhost:1421";

// Tauri command bridge
async function invoke(command, args = {{}}) {{
    try {{
        const response = await fetch(`${{__MINA_BRIDGE_URL__}}/invoke`, {{
            method: "POST",
            headers: {{ "Content-Type": "application/json" }},
            body: JSON.stringify({{ command, args }})
        }});
        
        if (!response.ok) {{
            throw new Error(`HTTP error: ${{response.status}}`);
        }}
        
        return await response.json();
    }} catch (error) {{
        throw new Error(`Failed to invoke command ${{command}}: ${{error.message}}`);
    }}
}}

// Make invoke available globally
globalThis.invoke = invoke;
globalThis.__MINA_INVOKE__ = invoke;
"#,
            inputs_json
        );

        // Wrap user script
        let user_code = content;
        let file_extension = if language == "typescript" { "ts" } else { "js" };

        // Create a complete script that can be executed by Deno
        let final_script = format!(
            r#"
{}

// User script
{}
"#,
            bridge_code, user_code
        );

        // Write to temp file for Deno to execute
        let temp_dir = std::env::temp_dir();
        let script_path = temp_dir.join(format!("mina_script_{}.{}", uuid::Uuid::new_v4(), file_extension));
        std::fs::write(&script_path, &final_script)
            .context("Failed to write script to temp file")?;

        // Store path for execution
        Ok(script_path.to_string_lossy().to_string())
    }

    async fn execute_with_deno(&self, script_path: &str, language: &str) -> Result<(Value, String, String)> {
        // Execute with Deno binary
        // Deno provides secure sandboxing by default
        let mut deno_cmd = Command::new("deno");
        
        // Set up Deno with minimal permissions (no file system, network only for localhost bridge)
        deno_cmd
            .arg("run")
            .arg("--allow-net=127.0.0.1:1421")
            .arg("--no-remote")
            .arg("--unstable") // For some APIs we might need
            .env("MINA_BRIDGE_URL", "http://127.0.0.1:1421")
            .arg(script_path);

        // For TypeScript, Deno handles it natively
        let output = deno_cmd
            .output()
            .await
            .context("Failed to execute script with Deno. Make sure Deno is installed (https://deno.land)")?;

        // Clean up temp file
        let _ = std::fs::remove_file(script_path);

        let stdout = String::from_utf8_lossy(&output.stdout).to_string();
        let stderr = String::from_utf8_lossy(&output.stderr).to_string();

        // Try to parse stdout as JSON (script result)
        let data = if output.status.success() {
            // Look for JSON in stdout (scripts should output JSON as last line)
            let lines: Vec<&str> = stdout.lines().collect();
            let last_line = lines.last().unwrap_or(&"");
            
            serde_json::from_str(last_line).unwrap_or_else(|_| {
                // If not JSON, wrap in object
                json!({"output": stdout.trim(), "success": true})
            })
        } else {
            json!({"success": false, "error": stderr.trim()})
        };

        Ok((data, stdout, stderr))
    }
}
