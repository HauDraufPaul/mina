use crate::storage::devops::DevOpsStore;
use crate::storage::Database;
use anyhow::{Context, Result};
use std::sync::{Arc, Mutex};
use std::time::Instant;

pub struct HealthChecker;

impl HealthChecker {
    /// Start periodic health check monitoring
    pub fn start_checking(db: Arc<Mutex<Database>>) {
        tauri::async_runtime::spawn(async move {
            let mut interval = tokio::time::interval(tokio::time::Duration::from_secs(30)); // Check every 30 seconds

            loop {
                interval.tick().await;

                if let Err(e) = Self::check_all_health_checks(&db).await {
                    eprintln!("Error checking health checks: {}", e);
                }
            }
        });
    }

    /// Check all health checks and update their status
    async fn check_all_health_checks(db: &Arc<Mutex<Database>>) -> Result<()> {
        // Get all health checks
        let checks = {
            let db_guard = db.lock()
                .map_err(|e| anyhow::anyhow!("Database lock error: {}", e))?;
            let store = DevOpsStore::new(db_guard.conn.clone());
            store.list_health_checks()
                .context("Failed to list health checks")?
        };

        // Check each health check concurrently
        let mut tasks = Vec::new();
        for check in checks {
            let db_clone = db.clone();
            let task = tokio::spawn(async move {
                Self::check_single_health_check(&check.name, &check.url, &db_clone).await
            });
            tasks.push(task);
        }

        // Wait for all checks to complete
        for task in tasks {
            if let Err(e) = task.await {
                eprintln!("Health check task error: {}", e);
            }
        }

        Ok(())
    }

    /// Check a single health check URL and update its status
    async fn check_single_health_check(
        name: &str,
        url: &str,
        db: &Arc<Mutex<Database>>,
    ) -> Result<()> {
        let start_time = Instant::now();
        let client = reqwest::Client::builder()
            .timeout(std::time::Duration::from_secs(10))
            .build()
            .context("Failed to create HTTP client")?;

        let result = client
            .get(url)
            .send()
            .await;

        let response_time = start_time.elapsed().as_millis() as i64;

        let (status, error) = match result {
            Ok(response) => {
                if response.status().is_success() {
                    ("healthy", None)
                } else {
                    (
                        "unhealthy",
                        Some(format!("HTTP {}: {}", response.status(), response.status().canonical_reason().unwrap_or("Unknown")))
                    )
                }
            }
            Err(e) => {
                // Provide more specific error messages and try to auto-start services
                let error_msg = if e.is_connect() {
                    // Try to identify and start the service
                    let service_name = if url.contains(":3000") {
                        "Localhost service (port 3000)"
                    } else if url.contains(":8080") {
                        "API Server (port 8080)"
                    } else if url.contains(":9200") {
                        // Try to start Elasticsearch
                        let _ = Self::try_start_elasticsearch().await;
                        "Elasticsearch"
                    } else {
                        "Service"
                    };
                    format!("Connection refused: {} may not be running. Attempting to start if available...", service_name)
                } else if e.is_timeout() {
                    format!("Request timeout: Service did not respond within 10 seconds")
                } else if e.is_request() {
                    format!("Invalid request: {}", e)
                } else if e.is_decode() {
                    format!("Response decode error: {}", e)
                } else {
                    // Check for common protocol mismatches
                    let error_str = e.to_string();
                    if error_str.contains("error sending request") {
                        if url.contains(":5432") {
                            format!("Cannot connect: PostgreSQL database (port 5432) does not expose HTTP endpoints. Use a database health check API instead.")
                        } else if url.contains(":6379") {
                            format!("Cannot connect: Redis (port 6379) does not expose HTTP endpoints. Use a Redis health check API instead.")
                        } else {
                            format!("Connection failed: {}", error_str)
                        }
                    } else {
                        format!("Request failed: {}", error_str)
                    }
                };
                (
                    "unhealthy",
                    Some(error_msg)
                )
            }
        };

        // Update the health check status
        let db_guard = db.lock()
            .map_err(|e| anyhow::anyhow!("Database lock error: {}", e))?;
        let store = DevOpsStore::new(db_guard.conn.clone());
        store.update_health_check(name, status, Some(response_time), error.as_deref())
            .context(format!("Failed to update health check: {}", name))?;

        Ok(())
    }

    /// Manually trigger a health check for a specific check
    pub async fn check_health_check(
        name: &str,
        db: &Arc<Mutex<Database>>,
    ) -> Result<()> {
        // Get the health check
        let check = {
            let db_guard = db.lock()
                .map_err(|e| anyhow::anyhow!("Database lock error: {}", e))?;
            let store = DevOpsStore::new(db_guard.conn.clone());
            let checks = store.list_health_checks()
                .context("Failed to list health checks")?;
            checks.into_iter()
                .find(|c| c.name == name)
                .ok_or_else(|| anyhow::anyhow!("Health check not found: {}", name))?
        };

        Self::check_single_health_check(&check.name, &check.url, db).await
    }

    /// Try to start Elasticsearch service via Homebrew
    async fn try_start_elasticsearch() -> Result<(), String> {
        use tokio::process::Command;
        
        // Get Homebrew path
        fn get_brew_path() -> String {
            for brew_path in &["/opt/homebrew/bin/brew", "/usr/local/bin/brew", "brew"] {
                if std::process::Command::new(brew_path)
                    .arg("--version")
                    .output()
                    .is_ok() {
                    return brew_path.to_string();
                }
            }
            "brew".to_string()
        }

        let brew_path = get_brew_path();
        
        // Check if Homebrew is available
        if !std::process::Command::new(&brew_path)
            .arg("--version")
            .output()
            .is_ok() {
            return Err("Homebrew not available".to_string());
        }

        let elasticsearch_services = vec!["elasticsearch", "elasticsearch-full"];
        
        for service in elasticsearch_services {
            // Check if installed
            let installed = match Command::new(&brew_path)
                .args(&["list", service])
                .output()
                .await
            {
                Ok(output) => output.status.success(),
                Err(_) => false,
            };

            if installed {
                // Check if running
                let running = match Command::new(&brew_path)
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
                };

                if !running {
                    eprintln!("MINA: Auto-starting Elasticsearch service: {}", service);
                    let output = Command::new(&brew_path)
                        .args(&["services", "start", service])
                        .output()
                        .await
                        .map_err(|e| format!("Failed to execute brew command: {}", e))?;

                    if output.status.success() {
                        return Ok(());
                    }
                } else {
                    return Ok(()); // Already running
                }
            }
        }
        
        Err("Elasticsearch service not found".to_string())
    }
}
