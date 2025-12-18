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

