use crate::storage::Database;
use std::sync::Mutex;
use tauri::AppHandle;

#[tauri::command]
pub fn get_config(key: String, app: AppHandle) -> Result<Option<String>, String> {
    let db = app.try_state::<Mutex<Database>>()
        .ok_or("Database not found")?;
    let db = db.lock().map_err(|e| format!("Database lock error: {}", e))?;
    db.get_config(&key)
        .map_err(|e| format!("Failed to get config: {}", e))
}

#[tauri::command]
pub fn set_config(key: String, value: String, app: AppHandle) -> Result<(), String> {
    let db = app.try_state::<Mutex<Database>>()
        .ok_or("Database not found")?;
    let db = db.lock().map_err(|e| format!("Database lock error: {}", e))?;
    db.set_config(&key, &value)
        .map_err(|e| format!("Failed to set config: {}", e))
}

