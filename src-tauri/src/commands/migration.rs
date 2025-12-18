use crate::storage::migration_tracking::MigrationTracker;
use crate::storage::Database;
use std::sync::Mutex;
use tauri::State;

#[tauri::command]
pub fn list_migrations(
    db: State<'_, Mutex<Database>>,
) -> Result<Vec<crate::storage::migration_tracking::MigrationRecord>, String> {
    let db_guard = db.lock().map_err(|e| format!("Database lock error: {}", e))?;
    let tracker = MigrationTracker::new(db_guard.conn.clone());
    tracker.list_migrations()
        .map_err(|e| format!("Failed to list migrations: {}", e))
}

#[tauri::command]
pub fn get_latest_migration_version(
    db: State<'_, Mutex<Database>>,
) -> Result<i32, String> {
    let db_guard = db.lock().map_err(|e| format!("Database lock error: {}", e))?;
    let tracker = MigrationTracker::new(db_guard.conn.clone());
    tracker.get_latest_version()
        .map_err(|e| format!("Failed to get latest version: {}", e))
}

