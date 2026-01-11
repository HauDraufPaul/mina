use crate::storage::analytics::AnalyticsStore;
use crate::storage::Database;
use std::sync::Mutex;
use tauri::State;

#[tauri::command]
pub fn save_metric(
    metric_type: String,
    value: f64,
    metadata: Option<String>,
    db: State<'_, Mutex<Database>>,
) -> Result<(), String> {
    let db_guard = db.lock().map_err(|e| format!("Database lock error: {}", e))?;
    let store = AnalyticsStore::new(db_guard.conn.clone());
    store.save_metric(&metric_type, value, metadata.as_deref())
        .map_err(|e| format!("Failed to save metric: {}", e))
}

#[tauri::command]
pub fn get_metrics(
    metric_type: String,
    start_time: Option<i64>,
    end_time: Option<i64>,
    limit: Option<i32>,
    db: State<'_, Mutex<Database>>,
) -> Result<Vec<crate::storage::analytics::AnalyticsMetrics>, String> {
    let db_guard = db.lock().map_err(|e| format!("Database lock error: {}", e))?;
    let store = AnalyticsStore::new(db_guard.conn.clone());
    store.get_metrics(&metric_type, start_time, end_time, limit)
        .map_err(|e| format!("Failed to get metrics: {}", e))
}

#[tauri::command]
pub fn get_statistics(
    metric_type: String,
    start_time: Option<i64>,
    end_time: Option<i64>,
    db: State<'_, Mutex<Database>>,
) -> Result<crate::storage::analytics::Statistics, String> {
    let db_guard = db.lock().map_err(|e| format!("Database lock error: {}", e))?;
    let store = AnalyticsStore::new(db_guard.conn.clone());
    store.get_statistics(&metric_type, start_time, end_time)
        .map_err(|e| format!("Failed to get statistics: {}", e))
}

#[tauri::command]
pub fn test_analytics_collection(
    db: State<'_, Mutex<Database>>,
    system_provider: State<'_, Mutex<crate::providers::SystemProvider>>,
) -> Result<String, String> {
    let provider_guard = system_provider.lock().map_err(|e| format!("Provider lock error: {}", e))?;
    provider_guard.refresh();
    
    let cpu_usage = provider_guard.get_cpu_usage();
    let total_memory = provider_guard.get_memory_total();
    let used_memory = provider_guard.get_memory_used();
    let memory_usage = if total_memory > 0 {
        (used_memory as f64 / total_memory as f64) * 100.0
    } else {
        0.0
    };
    let (disk_total, disk_used, _) = provider_guard.get_disk_metrics();
    let disk_usage = if disk_total > 0 {
        (disk_used as f64 / disk_total as f64) * 100.0
    } else {
        0.0
    };
    
    // Try to save a test metric
    let db_guard = db.lock().map_err(|e| format!("Database lock error: {}", e))?;
    let store = crate::storage::AnalyticsStore::new(db_guard.conn.clone());
    drop(db_guard);
    
    if let Err(e) = store.save_metric("cpu", cpu_usage, Some("test")) {
        return Err(format!("Failed to save test metric: {}", e));
    }
    
    Ok(format!("Test successful! CPU: {:.2}%, Memory: {:.2}%, Disk: {:.2}%", cpu_usage, memory_usage, disk_usage))
}
