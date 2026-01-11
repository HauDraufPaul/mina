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
                (
                    "unhealthy",
                    Some(format!("Request failed: {}", e))
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
}
