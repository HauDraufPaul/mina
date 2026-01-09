use anyhow::{Context, Result};
use serde_json::{json, Value};
use std::collections::HashMap;
use tauri::AppHandle;
use tokio::net::TcpListener;
use tokio::io::{AsyncBufReadExt, AsyncWriteExt, AsyncReadExt, BufReader};

/// HTTP server that bridges Deno scripts to Tauri commands
#[derive(Clone)]
pub struct ScriptBridgeServer {
    app: AppHandle,
    port: u16,
}

impl ScriptBridgeServer {
    pub fn new(app: AppHandle, port: u16) -> Self {
        ScriptBridgeServer {
            app,
            port,
        }
    }

    pub async fn start(&self) -> Result<()> {
        let addr = format!("127.0.0.1:{}", self.port);
        let listener = TcpListener::bind(&addr).await
            .context(format!("Failed to bind script bridge server to {}", addr))?;
        
        let app_clone = self.app.clone();
        
        // Spawn server task
        tauri::async_runtime::spawn(async move {
            eprintln!("Script bridge server listening on {}", addr);
            
            loop {
                match listener.accept().await {
                    Ok((stream, _)) => {
                        let app = app_clone.clone();
                        tauri::async_runtime::spawn(async move {
                            if let Err(e) = Self::handle_connection(stream, app).await {
                                eprintln!("Error handling script bridge connection: {}", e);
                            }
                        });
                    }
                    Err(e) => {
                        eprintln!("Error accepting connection on script bridge: {}", e);
                        break;
                    }
                }
            }
        });
        
        Ok(())
    }

    async fn handle_connection(
        mut stream: tokio::net::TcpStream,
        app: AppHandle,
    ) -> Result<()> {
        let mut reader = BufReader::new(&mut stream);
        let mut request_line = String::new();
        
        // Read request line
        reader.read_line(&mut request_line).await?;
        
        // Parse HTTP request
        let parts: Vec<&str> = request_line.trim().split_whitespace().collect();
        if parts.len() < 2 {
            return Err(anyhow::anyhow!("Invalid HTTP request"));
        }
        
        let method = parts[0];
        let path = parts[1];
        
        // Read headers
        let mut headers = HashMap::new();
        loop {
            let mut line = String::new();
            reader.read_line(&mut line).await?;
            let line = line.trim();
            if line.is_empty() {
                break;
            }
            if let Some((key, value)) = line.split_once(':') {
                headers.insert(key.trim().to_lowercase(), value.trim().to_string());
            }
        }
        
        // Handle POST /invoke
        if method == "POST" && path == "/invoke" {
            // Read body - read remaining bytes from stream directly
            use tokio::io::AsyncReadExt;
            let mut body = Vec::new();
            {
                let mut stream_ref = reader.get_mut();
                stream_ref.read_to_end(&mut body).await?;
            }
            
            // Try to parse as JSON
            let body_str = String::from_utf8_lossy(&body);
            let request: Value = serde_json::from_str(body_str.trim())
                .context("Failed to parse request JSON")?;
            
            let command = request.get("command")
                .and_then(|v| v.as_str())
                .ok_or_else(|| anyhow::anyhow!("Missing 'command' in request"))?;
            let args = request.get("args").cloned().unwrap_or(json!({}));
            
            // Invoke Tauri command via the command dispatcher
            let result = crate::services::CommandDispatcher::invoke_command(&app, command, args).await
                .map_err(|e| anyhow::anyhow!("Command invocation failed: {}", e))?;
            
            // Send response
            let response = json!({
                "success": true,
                "data": result
            });
            let response_json = serde_json::to_string(&response)?;
            let http_response = format!(
                "HTTP/1.1 200 OK\r\nContent-Type: application/json\r\nContent-Length: {}\r\n\r\n{}",
                response_json.len(),
                response_json
            );
            
            stream.write_all(http_response.as_bytes()).await?;
        } else {
            // 404 for other paths
            let response = "HTTP/1.1 404 Not Found\r\n\r\n";
            stream.write_all(response.as_bytes()).await?;
        }
        
        Ok(())
    }

    pub fn get_port(&self) -> u16 {
        self.port
    }
}
