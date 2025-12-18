use crate::storage::auth::AuthManager;
use crate::storage::Database;
use std::sync::Mutex;
use tauri::State;

#[tauri::command]
pub fn set_pin(user_id: String, pin: String, db: State<'_, Mutex<Database>>) -> Result<(), String> {
    let db_guard = db.lock().map_err(|e| format!("Database lock error: {}", e))?;
    let auth = AuthManager::new(db_guard.conn.clone());
    auth.set_pin(&user_id, &pin)
        .map_err(|e| format!("Failed to set PIN: {}", e))
}

#[tauri::command]
pub fn verify_pin(user_id: String, pin: String, db: State<'_, Mutex<Database>>) -> Result<bool, String> {
    let db_guard = db.lock().map_err(|e| format!("Database lock error: {}", e))?;
    let auth = AuthManager::new(db_guard.conn.clone());
    let valid = auth.verify_pin(&user_id, &pin)
        .map_err(|e| format!("Failed to verify PIN: {}", e))?;
    
    // Log attempt
    let _ = auth.log_auth_attempt(&user_id, valid, None);
    
    Ok(valid)
}

#[tauri::command]
pub fn create_session(user_id: String, db: State<'_, Mutex<Database>>) -> Result<String, String> {
    let db_guard = db.lock().map_err(|e| format!("Database lock error: {}", e))?;
    let auth = AuthManager::new(db_guard.conn.clone());
    auth.create_session(&user_id)
        .map_err(|e| format!("Failed to create session: {}", e))
}

#[tauri::command]
pub fn validate_session(session_id: String, db: State<'_, Mutex<Database>>) -> Result<bool, String> {
    let db_guard = db.lock().map_err(|e| format!("Database lock error: {}", e))?;
    let auth = AuthManager::new(db_guard.conn.clone());
    auth.validate_session(&session_id)
        .map_err(|e| format!("Failed to validate session: {}", e))
}

#[tauri::command]
pub fn get_auth_attempts(limit: i32, db: State<'_, Mutex<Database>>) -> Result<Vec<crate::storage::auth::AuthAttempt>, String> {
    let db_guard = db.lock().map_err(|e| format!("Database lock error: {}", e))?;
    let auth = AuthManager::new(db_guard.conn.clone());
    auth.get_recent_attempts(limit)
        .map_err(|e| format!("Failed to get auth attempts: {}", e))
}

#[tauri::command]
pub fn check_permission(
    user_id: String,
    resource: String,
    action: String,
    db: State<'_, Mutex<Database>>,
) -> Result<bool, String> {
    let db_guard = db.lock().map_err(|e| format!("Database lock error: {}", e))?;
    let auth = AuthManager::new(db_guard.conn.clone());
    auth.check_permission(&user_id, &resource, &action)
        .map_err(|e| format!("Failed to check permission: {}", e))
}
