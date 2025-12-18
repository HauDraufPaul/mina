use crate::storage::Database;
use std::sync::Mutex;
use tauri::State;

#[tauri::command]
pub fn get_config(key: String, db: State<'_, Mutex<Database>>) -> Result<Option<String>, String> {
    let db_guard = db.lock().map_err(|e| format!("Database lock error: {}", e))?;
    db_guard.get_config(&key)
        .map_err(|e| format!("Failed to get config: {}", e))
}

#[tauri::command]
pub fn set_config(key: String, value: String, db: State<'_, Mutex<Database>>) -> Result<(), String> {
    let db_guard = db.lock().map_err(|e| format!("Database lock error: {}", e))?;
    db_guard.set_config(&key, &value)
        .map_err(|e| format!("Failed to set config: {}", e))
}

