use crate::storage::price_alerts::{PriceAlertStore, PriceAlert};
use crate::storage::Database;
use std::sync::Mutex;
use tauri::State;

#[tauri::command]
pub fn create_price_alert(
    ticker: String,
    condition: String,
    target_price: f64,
    db: State<'_, Mutex<Database>>,
) -> Result<i64, String> {
    let db_guard = db.lock().map_err(|e| format!("Database lock error: {}", e))?;
    let store = PriceAlertStore::new(db_guard.conn.clone());
    
    // Validate condition
    let valid_conditions = ["above", "below", "cross_above", "cross_below"];
    if !valid_conditions.contains(&condition.as_str()) {
        return Err(format!("Invalid condition. Must be one of: {}", valid_conditions.join(", ")));
    }
    
    store.create_alert(&ticker.to_uppercase(), &condition, target_price)
        .map_err(|e| format!("Failed to create alert: {}", e))
}

#[tauri::command]
pub fn list_price_alerts(
    ticker: Option<String>,
    enabled_only: bool,
    db: State<'_, Mutex<Database>>,
) -> Result<Vec<PriceAlert>, String> {
    let db_guard = db.lock().map_err(|e| format!("Database lock error: {}", e))?;
    let store = PriceAlertStore::new(db_guard.conn.clone());
    
    store.list_alerts(ticker.as_deref(), enabled_only)
        .map_err(|e| format!("Failed to list alerts: {}", e))
}

#[tauri::command]
pub fn get_price_alert(
    id: i64,
    db: State<'_, Mutex<Database>>,
) -> Result<Option<PriceAlert>, String> {
    let db_guard = db.lock().map_err(|e| format!("Database lock error: {}", e))?;
    let store = PriceAlertStore::new(db_guard.conn.clone());
    
    store.get_alert(id)
        .map_err(|e| format!("Failed to get alert: {}", e))
}

#[tauri::command]
pub fn update_price_alert(
    id: i64,
    condition: Option<String>,
    target_price: Option<f64>,
    enabled: Option<bool>,
    db: State<'_, Mutex<Database>>,
) -> Result<(), String> {
    let db_guard = db.lock().map_err(|e| format!("Database lock error: {}", e))?;
    let store = PriceAlertStore::new(db_guard.conn.clone());
    
    // Validate condition if provided
    if let Some(ref c) = condition {
        let valid_conditions = ["above", "below", "cross_above", "cross_below"];
        if !valid_conditions.contains(&c.as_str()) {
            return Err(format!("Invalid condition. Must be one of: {}", valid_conditions.join(", ")));
        }
    }
    
    store.update_alert(id, condition.as_deref(), target_price, enabled)
        .map_err(|e| format!("Failed to update alert: {}", e))
}

#[tauri::command]
pub fn delete_price_alert(
    id: i64,
    db: State<'_, Mutex<Database>>,
) -> Result<(), String> {
    let db_guard = db.lock().map_err(|e| format!("Database lock error: {}", e))?;
    let store = PriceAlertStore::new(db_guard.conn.clone());
    
    store.delete_alert(id)
        .map_err(|e| format!("Failed to delete alert: {}", e))
}

