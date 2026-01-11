use crate::storage::Database;
use crate::storage::AnalyticsStore;
use crate::providers::SystemProvider;
use std::sync::{Arc, Mutex};
use anyhow::Result;

pub struct AnalyticsCollector;

impl AnalyticsCollector {
    /// Start periodic metric collection and saving
    pub fn start_collecting(
        db: Arc<Mutex<Database>>,
        system_provider: Arc<Mutex<SystemProvider>>,
    ) {
        tauri::async_runtime::spawn(async move {
            // Initial delay to let system settle
            tokio::time::sleep(tokio::time::Duration::from_secs(2)).await;
            
            let mut interval = tokio::time::interval(tokio::time::Duration::from_secs(5)); // Collect every 5 seconds

            loop {
                interval.tick().await;

                if let Err(e) = Self::collect_and_save_metrics(&db, &system_provider).await {
                    eprintln!("Error collecting analytics metrics: {}", e);
                }
            }
        });
    }

    async fn collect_and_save_metrics(
        db: &Arc<Mutex<Database>>,
        system_provider: &Arc<Mutex<SystemProvider>>,
    ) -> Result<()> {
        // Get system provider and refresh
        let provider = system_provider.lock()
            .map_err(|e| anyhow::anyhow!("Failed to lock SystemProvider: {}", e))?;
        
        // Refresh system state
        provider.refresh();

        // Collect metrics
        let cpu_usage = provider.get_cpu_usage();
        let total_memory = provider.get_memory_total();
        let used_memory = provider.get_memory_used();
        let memory_usage = if total_memory > 0 {
            (used_memory as f64 / total_memory as f64) * 100.0
        } else {
            0.0
        };

        let (disk_total, disk_used, _disk_free) = provider.get_disk_metrics();
        let disk_usage = if disk_total > 0 {
            (disk_used as f64 / disk_total as f64) * 100.0
        } else {
            0.0
        };

        let (_total_rx, _total_tx) = provider.get_network_total();
        let (rx_speed, tx_speed) = provider.get_network_speeds();
        let network_total = rx_speed + tx_speed;

        // Log collected metrics for debugging (only every 10th collection to avoid spam)
        static mut COUNTER: u64 = 0;
        unsafe {
            COUNTER += 1;
            if COUNTER % 10 == 0 {
                eprintln!("MINA Analytics: CPU={:.2}%, Memory={:.2}%, Disk={:.2}%, Network={:.2} bytes/s", 
                    cpu_usage, memory_usage, disk_usage, network_total);
            }
        }

        drop(provider); // Release lock before database operations

        // Save metrics to analytics store
        // Clone connection before locking to avoid holding lock across operations
        let conn_clone = {
            match db.lock() {
                Ok(guard) => guard.conn.clone(),
                Err(e) => {
                    eprintln!("Warning: Failed to lock database for analytics: {}", e);
                    // Try to recover from poisoned lock
                    e.into_inner().conn.clone()
                }
            }
        };
        let analytics_store = AnalyticsStore::new(conn_clone);

        // Save each metric (only if values are valid)
        if cpu_usage >= 0.0 && cpu_usage <= 100.0 {
            if let Err(e) = analytics_store.save_metric("cpu", cpu_usage, None) {
                eprintln!("Warning: Failed to save CPU metric: {}", e);
            }
        }
        
        if memory_usage >= 0.0 && memory_usage <= 100.0 {
            if let Err(e) = analytics_store.save_metric("memory", memory_usage, None) {
                eprintln!("Warning: Failed to save memory metric: {}", e);
            }
        }
        
        if disk_usage >= 0.0 && disk_usage <= 100.0 {
            if let Err(e) = analytics_store.save_metric("disk", disk_usage, None) {
                eprintln!("Warning: Failed to save disk metric: {}", e);
            }
        }
        
        if network_total >= 0.0 {
            if let Err(e) = analytics_store.save_metric("network", network_total, None) {
                eprintln!("Warning: Failed to save network metric: {}", e);
            }
        }

        Ok(())
    }
}
