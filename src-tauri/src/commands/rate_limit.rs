use crate::storage::rate_limit::RateLimitStore;
use crate::storage::Database;
use std::sync::Mutex;
use tauri::State;

#[tauri::command]
pub fn create_rate_limit_bucket(
    name: String,
    capacity: i64,
    refill_rate: i64,
    refill_interval: i64,
    db: State<'_, Mutex<Database>>,
) -> Result<(), String> {
    let db_guard = db.lock().map_err(|e| format!("Database lock error: {}", e))?;
    let store = RateLimitStore::new(db_guard.conn.clone())
        .map_err(|e| format!("Failed to initialize store: {}", e))?;
    store.create_bucket(&name, capacity, refill_rate, refill_interval)
        .map_err(|e| format!("Failed to create bucket: {}", e))
}

#[tauri::command]
pub fn list_rate_limit_buckets(
    db: State<'_, Mutex<Database>>,
) -> Result<Vec<crate::storage::rate_limit::RateLimitBucket>, String> {
    let db_guard = db.lock().map_err(|e| format!("Database lock error: {}", e))?;
    let store = RateLimitStore::new(db_guard.conn.clone())
        .map_err(|e| format!("Failed to initialize store: {}", e))?;
    store.list_buckets()
        .map_err(|e| format!("Failed to list buckets: {}", e))
}

#[tauri::command]
pub fn get_rate_limit_bucket(
    name: String,
    db: State<'_, Mutex<Database>>,
) -> Result<Option<crate::storage::rate_limit::RateLimitBucket>, String> {
    let db_guard = db.lock().map_err(|e| format!("Database lock error: {}", e))?;
    let store = RateLimitStore::new(db_guard.conn.clone())
        .map_err(|e| format!("Failed to initialize store: {}", e))?;
    store.get_bucket(&name)
        .map_err(|e| format!("Failed to get bucket: {}", e))
}

#[tauri::command]
pub fn consume_rate_limit_token(
    name: String,
    amount: i64,
    db: State<'_, Mutex<Database>>,
) -> Result<bool, String> {
    let db_guard = db.lock().map_err(|e| format!("Database lock error: {}", e))?;
    let store = RateLimitStore::new(db_guard.conn.clone())
        .map_err(|e| format!("Failed to initialize store: {}", e))?;
    store.consume_token(&name, amount)
        .map_err(|e| format!("Failed to consume token: {}", e))
}

#[tauri::command]
pub fn refill_rate_limit_bucket(
    name: String,
    db: State<'_, Mutex<Database>>,
) -> Result<(), String> {
    let db_guard = db.lock().map_err(|e| format!("Database lock error: {}", e))?;
    let store = RateLimitStore::new(db_guard.conn.clone())
        .map_err(|e| format!("Failed to initialize store: {}", e))?;
    store.refill_bucket(&name)
        .map_err(|e| format!("Failed to refill bucket: {}", e))
}

