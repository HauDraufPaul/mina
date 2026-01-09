use crate::storage::api_keys::APIKeyStore;
use crate::storage::Database;
use std::sync::Mutex;
use tauri::State;

#[tauri::command]
pub fn store_api_key(
    provider: String,
    key: String,
    db: State<'_, Mutex<Database>>,
) -> Result<(), String> {
    let db_guard = db.lock().map_err(|e| format!("Database lock error: {}", e))?;
    let store = APIKeyStore::new(db_guard.conn.clone())
        .map_err(|e| format!("Failed to initialize API key store: {}", e))?;
    store.store_key(&provider, &key)
        .map_err(|e| format!("Failed to store API key: {}", e))
}

#[tauri::command]
pub fn get_api_key(
    provider: String,
    db: State<'_, Mutex<Database>>,
) -> Result<Option<String>, String> {
    let db_guard = db.lock().map_err(|e| format!("Database lock error: {}", e))?;
    let store = APIKeyStore::new(db_guard.conn.clone())
        .map_err(|e| format!("Failed to initialize API key store: {}", e))?;
    store.get_key(&provider)
        .map_err(|e| format!("Failed to get API key: {}", e))
}

#[tauri::command]
pub fn delete_api_key(
    provider: String,
    db: State<'_, Mutex<Database>>,
) -> Result<(), String> {
    let db_guard = db.lock().map_err(|e| format!("Database lock error: {}", e))?;
    let store = APIKeyStore::new(db_guard.conn.clone())
        .map_err(|e| format!("Failed to initialize API key store: {}", e))?;
    store.delete_key(&provider)
        .map_err(|e| format!("Failed to delete API key: {}", e))
}

#[tauri::command]
pub fn list_api_key_providers(
    db: State<'_, Mutex<Database>>,
) -> Result<Vec<String>, String> {
    let db_guard = db.lock().map_err(|e| format!("Database lock error: {}", e))?;
    let store = APIKeyStore::new(db_guard.conn.clone())
        .map_err(|e| format!("Failed to initialize API key store: {}", e))?;
    store.list_providers()
        .map_err(|e| format!("Failed to list providers: {}", e))
}

#[tauri::command]
pub fn has_api_key(
    provider: String,
    db: State<'_, Mutex<Database>>,
) -> Result<bool, String> {
    let db_guard = db.lock().map_err(|e| format!("Database lock error: {}", e))?;
    let store = APIKeyStore::new(db_guard.conn.clone())
        .map_err(|e| format!("Failed to initialize API key store: {}", e))?;
    store.has_key(&provider)
        .map_err(|e| format!("Failed to check API key: {}", e))
}

