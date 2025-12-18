use crate::storage::auth::AuthManager;
use std::sync::Mutex;
use tauri::AppHandle;

#[tauri::command]
pub fn set_pin(user_id: String, pin: String, app: AppHandle) -> Result<(), String> {
    let db = app.try_state::<Mutex<crate::storage::Database>>()
        .ok_or("Database not found")?;
    let db = db.lock().map_err(|e| format!("Database lock error: {}", e))?;
    
    // Get connection from database
    // Note: This is a simplified approach - in production, you'd want a better way to share the connection
    let auth = AuthManager::new(db.conn.clone());
    auth.set_pin(&user_id, &pin)
        .map_err(|e| format!("Failed to set PIN: {}", e))
}

#[tauri::command]
pub fn verify_pin(user_id: String, pin: String, app: AppHandle) -> Result<bool, String> {
    let db = app.try_state::<Mutex<crate::storage::Database>>()
        .ok_or("Database not found")?;
    let db = db.lock().map_err(|e| format!("Database lock error: {}", e))?;
    
    let auth = AuthManager::new(db.conn.clone());
    let valid = auth.verify_pin(&user_id, &pin)
        .map_err(|e| format!("Failed to verify PIN: {}", e))?;
    
    // Log attempt
    let _ = auth.log_auth_attempt(&user_id, valid, None);
    
    Ok(valid)
}

#[tauri::command]
pub fn create_session(user_id: String, app: AppHandle) -> Result<String, String> {
    let db = app.try_state::<Mutex<crate::storage::Database>>()
        .ok_or("Database not found")?;
    let db = db.lock().map_err(|e| format!("Database lock error: {}", e))?;
    
    let auth = AuthManager::new(db.conn.clone());
    auth.create_session(&user_id)
        .map_err(|e| format!("Failed to create session: {}", e))
}

#[tauri::command]
pub fn validate_session(session_id: String, app: AppHandle) -> Result<bool, String> {
    let db = app.try_state::<Mutex<crate::storage::Database>>()
        .ok_or("Database not found")?;
    let db = db.lock().map_err(|e| format!("Database lock error: {}", e))?;
    
    let auth = AuthManager::new(db.conn.clone());
    auth.validate_session(&session_id)
        .map_err(|e| format!("Failed to validate session: {}", e))
}

#[tauri::command]
pub fn get_auth_attempts(limit: i32, app: AppHandle) -> Result<Vec<crate::storage::auth::AuthAttempt>, String> {
    let db = app.try_state::<Mutex<crate::storage::Database>>()
        .ok_or("Database not found")?;
    let db = db.lock().map_err(|e| format!("Database lock error: {}", e))?;
    
    let auth = AuthManager::new(db.conn.clone());
    auth.get_recent_attempts(limit)
        .map_err(|e| format!("Failed to get auth attempts: {}", e))
}

#[tauri::command]
pub fn check_permission(
    user_id: String,
    resource: String,
    action: String,
    app: AppHandle,
) -> Result<bool, String> {
    let db = app.try_state::<Mutex<crate::storage::Database>>()
        .ok_or("Database not found")?;
    let db = db.lock().map_err(|e| format!("Database lock error: {}", e))?;
    
    let auth = AuthManager::new(db.conn.clone());
    auth.check_permission(&user_id, &resource, &action)
        .map_err(|e| format!("Failed to check permission: {}", e))
}

