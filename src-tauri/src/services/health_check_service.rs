use anyhow::{Context, Result};
use serde_json::json;
use std::time::Instant;
use tokio::net::TcpListener;
use tokio::io::{AsyncBufReadExt, AsyncWriteExt, BufReader};
use tokio::process::Command;

/// HTTP server that provides health check endpoints for Database and Redis
/// This wraps native protocol checks (PostgreSQL, Redis) into HTTP endpoints
#[derive(Clone)]
pub struct HealthCheckService {
    port: u16,
    database_url: String,
    redis_url: String,
}

impl HealthCheckService {
    pub fn new(port: u16) -> Self {
        // Default connection strings - can be configured via environment variables
        let database_url = std::env::var("DATABASE_URL")
            .unwrap_or_else(|_| "postgresql://localhost:5432/postgres".to_string());
        let redis_url = std::env::var("REDIS_URL")
            .unwrap_or_else(|_| "redis://localhost:6379".to_string());
        
        HealthCheckService {
            port,
            database_url,
            redis_url,
        }
    }

    pub async fn start(&self) -> Result<()> {
        // Try to ensure required services are running
        eprintln!("MINA: Checking and starting required services...");
        Self::ensure_services_running().await;
        
        let addr = format!("127.0.0.1:{}", self.port);
        let listener = TcpListener::bind(&addr).await
            .context(format!("Failed to bind health check service to {}", addr))?;
        
        let service = self.clone();
        
        // Spawn server task
        tauri::async_runtime::spawn(async move {
            eprintln!("MINA: Health check service listening on {}", addr);
            
            loop {
                match listener.accept().await {
                    Ok((stream, _)) => {
                        let service_clone = service.clone();
                        tauri::async_runtime::spawn(async move {
                            if let Err(e) = Self::handle_connection(stream, service_clone).await {
                                eprintln!("Error handling health check connection: {}", e);
                            }
                        });
                    }
                    Err(e) => {
                        eprintln!("Error accepting connection on health check service: {}", e);
                        break;
                    }
                }
            }
        });
        
        Ok(())
    }

    /// Ensure PostgreSQL and Redis services are running
    /// Tries to start them via Homebrew if available
    async fn ensure_services_running() {
        // Check if Homebrew is available
        if !Self::is_homebrew_available() {
            eprintln!("MINA: Homebrew not available - skipping automatic service startup");
            eprintln!("MINA: Please ensure PostgreSQL and Redis are running manually");
            return;
        }

        // List of services to check and start
        let services = vec![
            ("postgresql@14", "postgresql"),
            ("postgresql@15", "postgresql"),
            ("postgresql@16", "postgresql"),
            ("postgresql", "postgresql"),
            ("redis", "redis"),
            ("elasticsearch", "elasticsearch"),
            ("elasticsearch-full", "elasticsearch"),
        ];

        for (service_name, service_type) in services {
            if Self::is_service_installed(service_name).await {
                if !Self::is_service_running(service_name).await {
                    eprintln!("MINA: {} service '{}' is not running, attempting to start...", service_type, service_name);
                    if let Err(e) = Self::start_homebrew_service(service_name).await {
                        eprintln!("MINA: Failed to start {} service '{}': {}", service_type, service_name, e);
                } else {
                    eprintln!("MINA: Successfully started {} service '{}'", service_type, service_name);
                    // Wait a bit for the service to fully start
                    tokio::time::sleep(tokio::time::Duration::from_secs(2)).await;
                }
            } else {
                eprintln!("MINA: {} service '{}' is already running", service_type, service_name);
            }
            // Found the service, break to avoid checking other variants
            if service_type == "postgresql" || service_type == "elasticsearch" {
                break;
            }
        }
        }
    }

    fn get_brew_path() -> String {
        // Try common Homebrew paths
        for brew_path in &["/opt/homebrew/bin/brew", "/usr/local/bin/brew", "brew"] {
            if std::process::Command::new(brew_path)
                .arg("--version")
                .output()
                .is_ok() {
                return brew_path.to_string();
            }
        }
        "brew".to_string() // Fallback
    }

    fn is_homebrew_available() -> bool {
        let brew_path = Self::get_brew_path();
        std::process::Command::new(&brew_path)
            .arg("--version")
            .output()
            .is_ok()
    }

    async fn is_service_installed(service: &str) -> bool {
        let brew_path = Self::get_brew_path();
        match Command::new(&brew_path)
            .args(&["list", service])
            .output()
            .await
        {
            Ok(output) => output.status.success(),
            Err(_) => false,
        }
    }

    async fn is_service_running(service: &str) -> bool {
        let brew_path = Self::get_brew_path();
        match Command::new(&brew_path)
            .args(&["services", "list"])
            .output()
            .await
        {
            Ok(output) => {
                if let Ok(stdout) = std::str::from_utf8(&output.stdout) {
                    stdout.lines().any(|line| {
                        let parts: Vec<&str> = line.split_whitespace().collect();
                        parts.len() >= 2 
                            && parts[0] == service 
                            && (parts[1] == "started" || parts[1] == "running")
                    })
                } else {
                    false
                }
            }
            Err(_) => false,
        }
    }

    async fn start_homebrew_service(service: &str) -> Result<(), String> {
        let brew_path = Self::get_brew_path();
        let output = Command::new(&brew_path)
            .args(&["services", "start", service])
            .output()
            .await
            .map_err(|e| format!("Failed to execute brew command: {}", e))?;

        if output.status.success() {
            Ok(())
        } else {
            let stderr = std::str::from_utf8(&output.stderr)
                .unwrap_or("Unknown error");
            Err(format!("brew services start failed: {}", stderr))
        }
    }

    async fn handle_connection(
        mut stream: tokio::net::TcpStream,
        service: HealthCheckService,
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
        
        // Read headers (skip for simplicity, but read until empty line)
        loop {
            let mut line = String::new();
            reader.read_line(&mut line).await?;
            if line.trim().is_empty() {
                break;
            }
        }
        
        // Handle GET requests
        if method == "GET" {
            let (status_code, body) = if path == "/health" || path == "/health/" {
                // Overall health endpoint
                (200, json!({
                    "status": "healthy",
                    "service": "MINA Health Check Service",
                    "endpoints": {
                        "database": format!("http://127.0.0.1:{}/health/database", service.port),
                        "redis": format!("http://127.0.0.1:{}/health/redis", service.port),
                    }
                }))
            } else if path == "/health/database" {
                // Database health check
                match Self::check_database(&service.database_url).await {
                    Ok((healthy, message, response_time)) => {
                        let status_code = if healthy { 200 } else { 503 };
                        (status_code, json!({
                            "status": if healthy { "healthy" } else { "unhealthy" },
                            "service": "PostgreSQL Database",
                            "message": message,
                            "response_time_ms": response_time,
                            "timestamp": chrono::Utc::now().to_rfc3339(),
                        }))
                    }
                    Err(e) => {
                        (503, json!({
                            "status": "unhealthy",
                            "service": "PostgreSQL Database",
                            "message": format!("Health check failed: {}", e),
                            "error": e.to_string(),
                            "timestamp": chrono::Utc::now().to_rfc3339(),
                        }))
                    }
                }
            } else if path == "/health/redis" {
                // Redis health check
                match Self::check_redis(&service.redis_url).await {
                    Ok((healthy, message, response_time)) => {
                        let status_code = if healthy { 200 } else { 503 };
                        (status_code, json!({
                            "status": if healthy { "healthy" } else { "unhealthy" },
                            "service": "Redis",
                            "message": message,
                            "response_time_ms": response_time,
                            "timestamp": chrono::Utc::now().to_rfc3339(),
                        }))
                    }
                    Err(e) => {
                        (503, json!({
                            "status": "unhealthy",
                            "service": "Redis",
                            "message": format!("Health check failed: {}", e),
                            "error": e.to_string(),
                            "timestamp": chrono::Utc::now().to_rfc3339(),
                        }))
                    }
                }
            } else {
                // 404 for unknown paths
                (404, json!({
                    "error": "Not Found",
                    "message": format!("Unknown endpoint: {}", path),
                    "available_endpoints": [
                        "/health",
                        "/health/database",
                        "/health/redis",
                    ]
                }))
            };
            
            let body_json = serde_json::to_string(&body)?;
            let http_response = format!(
                "HTTP/1.1 {} {}\r\nContent-Type: application/json\r\nAccess-Control-Allow-Origin: *\r\nContent-Length: {}\r\n\r\n{}",
                status_code,
                if status_code == 200 { "OK" } else if status_code == 503 { "Service Unavailable" } else { "Not Found" },
                body_json.len(),
                body_json
            );
            
            stream.write_all(http_response.as_bytes()).await?;
        } else {
            // Method not allowed
            let response = "HTTP/1.1 405 Method Not Allowed\r\n\r\n";
            stream.write_all(response.as_bytes()).await?;
        }
        
        Ok(())
    }

    async fn check_database(url: &str) -> Result<(bool, String, u64)> {
        let start = Instant::now();
        
        // Try to connect to PostgreSQL with a short timeout
        match tokio::time::timeout(
            std::time::Duration::from_secs(5),
            tokio_postgres::connect(url, tokio_postgres::NoTls)
        ).await {
            Ok(Ok((client, connection))) => {
                // Spawn connection task to handle the connection
                tauri::async_runtime::spawn(async move {
                    if let Err(e) = connection.await {
                        eprintln!("Database connection error: {}", e);
                    }
                });
                
                // Try a simple query
                match tokio::time::timeout(
                    std::time::Duration::from_secs(2),
                    client.query_one("SELECT 1", &[])
                ).await {
                    Ok(Ok(_)) => {
                        let response_time = start.elapsed().as_millis() as u64;
                        Ok((true, "Database is healthy and responding".to_string(), response_time))
                    }
                    Ok(Err(e)) => {
                        let response_time = start.elapsed().as_millis() as u64;
                        Ok((false, format!("Database query failed: {}", e), response_time))
                    }
                    Err(_) => {
                        let response_time = start.elapsed().as_millis() as u64;
                        Ok((false, "Database query timeout".to_string(), response_time))
                    }
                }
            }
            Ok(Err(e)) => {
                let response_time = start.elapsed().as_millis() as u64;
                let error_msg = if e.to_string().contains("Connection refused") {
                    // Try to start PostgreSQL automatically
                    let _ = Self::try_start_postgres().await;
                    "Database connection refused: Attempted to start PostgreSQL service. Please wait a moment and check again."
                } else if e.to_string().contains("timeout") {
                    "Database connection timeout: PostgreSQL is not responding."
                } else {
                    &format!("Database connection failed: {}", e)
                };
                Ok((false, error_msg.to_string(), response_time))
            }
            Err(_) => {
                let response_time = start.elapsed().as_millis() as u64;
                Ok((false, "Database connection timeout: PostgreSQL did not respond within 5 seconds".to_string(), response_time))
            }
        }
    }

    async fn check_redis(url: &str) -> Result<(bool, String, u64)> {
        let start = Instant::now();
        
        // Parse Redis URL and create client (synchronous operation)
        let client = match redis::Client::open(url) {
            Ok(client) => client,
            Err(e) => {
                let response_time = start.elapsed().as_millis() as u64;
                return Ok((false, format!("Failed to create Redis client: {}", e), response_time));
            }
        };
        
        // Try to get a connection and ping
        match tokio::time::timeout(
            std::time::Duration::from_secs(5),
            client.get_async_connection()
        ).await {
            Ok(Ok(mut conn)) => {
                // Try PING command
                match tokio::time::timeout(
                    std::time::Duration::from_secs(2),
                    redis::cmd("PING").query_async::<_, String>(&mut conn)
                ).await {
                    Ok(Ok(_)) => {
                        let response_time = start.elapsed().as_millis() as u64;
                        Ok((true, "Redis is healthy and responding".to_string(), response_time))
                    }
                    Ok(Err(e)) => {
                        let response_time = start.elapsed().as_millis() as u64;
                        Ok((false, format!("Redis PING failed: {}", e), response_time))
                    }
                    Err(_) => {
                        let response_time = start.elapsed().as_millis() as u64;
                        Ok((false, "Redis PING timeout".to_string(), response_time))
                    }
                }
            }
            Ok(Err(e)) => {
                let response_time = start.elapsed().as_millis() as u64;
                let error_msg = if e.to_string().contains("Connection refused") {
                    // Try to start Redis automatically
                    let _ = Self::try_start_redis().await;
                    "Redis connection refused: Attempted to start Redis service. Please wait a moment and check again."
                } else if e.to_string().contains("timeout") {
                    "Redis connection timeout: Redis is not responding."
                } else {
                    &format!("Redis connection failed: {}", e)
                };
                Ok((false, error_msg.to_string(), response_time))
            }
            Err(_) => {
                let response_time = start.elapsed().as_millis() as u64;
                Ok((false, "Redis connection timeout: Redis did not respond within 5 seconds".to_string(), response_time))
            }
        }
    }

    pub fn get_port(&self) -> u16 {
        self.port
    }

    /// Try to start PostgreSQL service
    async fn try_start_postgres() -> Result<(), String> {
        if !Self::is_homebrew_available() {
            return Err("Homebrew not available".to_string());
        }

        let postgres_services = vec!["postgresql@14", "postgresql@15", "postgresql@16", "postgresql"];
        
        for service in postgres_services {
            if Self::is_service_installed(service).await {
                if !Self::is_service_running(service).await {
                    eprintln!("MINA: Auto-starting PostgreSQL service: {}", service);
                    return Self::start_homebrew_service(service).await;
                }
            }
        }
        
        Err("PostgreSQL service not found or already running".to_string())
    }

    /// Try to start Redis service
    async fn try_start_redis() -> Result<(), String> {
        if !Self::is_homebrew_available() {
            return Err("Homebrew not available".to_string());
        }

        if Self::is_service_installed("redis").await {
            if !Self::is_service_running("redis").await {
                eprintln!("MINA: Auto-starting Redis service");
                return Self::start_homebrew_service("redis").await;
            }
        }
        
        Err("Redis service not found or already running".to_string())
    }
}
