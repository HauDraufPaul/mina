use crate::storage::devops::DevOpsStore;
use crate::storage::Database;
use std::sync::Mutex;
use tauri::State;

#[tauri::command]
pub fn create_health_check(
    name: String,
    url: String,
    db: State<'_, Mutex<Database>>,
) -> Result<i64, String> {
    let db_guard = db.lock().map_err(|e| format!("Database lock error: {}", e))?;
    let store = DevOpsStore::new(db_guard.conn.clone());
    store.create_health_check(&name, &url)
        .map_err(|e| format!("Failed to create health check: {}", e))
}

#[tauri::command]
pub fn update_health_check(
    name: String,
    status: String,
    response_time: Option<i64>,
    error: Option<String>,
    db: State<'_, Mutex<Database>>,
) -> Result<(), String> {
    let db_guard = db.lock().map_err(|e| format!("Database lock error: {}", e))?;
    let store = DevOpsStore::new(db_guard.conn.clone());
    store.update_health_check(&name, &status, response_time, error.as_deref())
        .map_err(|e| format!("Failed to update health check: {}", e))
}

#[tauri::command]
pub fn list_health_checks(
    db: State<'_, Mutex<Database>>,
) -> Result<Vec<crate::storage::devops::HealthCheck>, String> {
    let db_guard = db.lock().map_err(|e| format!("Database lock error: {}", e))?;
    let store = DevOpsStore::new(db_guard.conn.clone());
    store.list_health_checks()
        .map_err(|e| format!("Failed to list health checks: {}", e))
}

#[tauri::command]
pub fn create_alert(
    name: String,
    severity: String,
    message: String,
    source: String,
    db: State<'_, Mutex<Database>>,
) -> Result<i64, String> {
    let db_guard = db.lock().map_err(|e| format!("Database lock error: {}", e))?;
    let store = DevOpsStore::new(db_guard.conn.clone());
    store.create_alert(&name, &severity, &message, &source)
        .map_err(|e| format!("Failed to create alert: {}", e))
}

#[tauri::command]
pub fn list_alerts(
    limit: i32,
    unresolved_only: bool,
    db: State<'_, Mutex<Database>>,
) -> Result<Vec<crate::storage::devops::Alert>, String> {
    let db_guard = db.lock().map_err(|e| format!("Database lock error: {}", e))?;
    let store = DevOpsStore::new(db_guard.conn.clone());
    store.list_alerts(limit, unresolved_only)
        .map_err(|e| format!("Failed to list alerts: {}", e))
}

#[tauri::command]
pub fn resolve_alert(
    id: i64,
    db: State<'_, Mutex<Database>>,
) -> Result<(), String> {
    let db_guard = db.lock().map_err(|e| format!("Database lock error: {}", e))?;
    let store = DevOpsStore::new(db_guard.conn.clone());
    store.resolve_alert(id)
        .map_err(|e| format!("Failed to resolve alert: {}", e))
}

#[tauri::command]
pub fn save_prometheus_metric(
    name: String,
    value: f64,
    labels: String,
    db: State<'_, Mutex<Database>>,
) -> Result<(), String> {
    let db_guard = db.lock().map_err(|e| format!("Database lock error: {}", e))?;
    let store = DevOpsStore::new(db_guard.conn.clone());
    store.save_prometheus_metric(&name, value, &labels)
        .map_err(|e| format!("Failed to save metric: {}", e))
}

#[tauri::command]
pub fn get_prometheus_metrics(
    name: String,
    start_time: i64,
    end_time: i64,
    db: State<'_, Mutex<Database>>,
) -> Result<Vec<crate::storage::devops::PrometheusMetric>, String> {
    let db_guard = db.lock().map_err(|e| format!("Database lock error: {}", e))?;
    let store = DevOpsStore::new(db_guard.conn.clone());
    store.get_prometheus_metrics(&name, start_time, end_time)
        .map_err(|e| format!("Failed to get metrics: {}", e))
}

#[tauri::command]
pub fn init_default_health_checks(
    db: State<'_, Mutex<Database>>,
) -> Result<usize, String> {
    let db_guard = db.lock().map_err(|e| format!("Database lock error: {}", e))?;
    let store = DevOpsStore::new(db_guard.conn.clone());
    
    // Get current count
    let before = store.list_health_checks()
        .map_err(|e| format!("Failed to list health checks: {}", e))?
        .len();
    
    // Initialize defaults (will only add if none exist)
    store.init_default_health_checks()
        .map_err(|e| format!("Failed to initialize default health checks: {}", e))?;
    
    // Get new count
    let after = store.list_health_checks()
        .map_err(|e| format!("Failed to list health checks: {}", e))?
        .len();
    
    Ok(after - before)
}

#[tauri::command]
pub async fn check_health_check(
    name: String,
    db: State<'_, Mutex<Database>>,
) -> Result<(), String> {
    let db_arc = Arc::new(Mutex::new(Database {
        conn: {
            let db_guard = db.lock().map_err(|e| format!("Database lock error: {}", e))?;
            db_guard.conn.clone()
        },
    }));
    
    crate::services::health_checker::HealthChecker::check_health_check(&name, &db_arc)
        .await
        .map_err(|e| format!("Failed to check health check: {}", e))
}
