use crate::storage::testing::TestingStore;
use crate::storage::Database;
use std::sync::Mutex;
use tauri::State;

#[tauri::command]
pub fn create_test_suite(
    name: String,
    test_type: String,
    db: State<'_, Mutex<Database>>,
) -> Result<i64, String> {
    let db_guard = db.lock().map_err(|e| format!("Database lock error: {}", e))?;
    let store = TestingStore::new(db_guard.conn.clone());
    store.create_suite(&name, &test_type)
        .map_err(|e| format!("Failed to create test suite: {}", e))
}

#[tauri::command]
pub fn list_test_suites(
    db: State<'_, Mutex<Database>>,
) -> Result<Vec<crate::storage::testing::TestSuite>, String> {
    let db_guard = db.lock().map_err(|e| format!("Database lock error: {}", e))?;
    let store = TestingStore::new(db_guard.conn.clone());
    store.list_suites()
        .map_err(|e| format!("Failed to list test suites: {}", e))
}

#[tauri::command]
pub fn save_test_result(
    suite_id: i64,
    name: String,
    status: String,
    duration: Option<f64>,
    error: Option<String>,
    db: State<'_, Mutex<Database>>,
) -> Result<i64, String> {
    let db_guard = db.lock().map_err(|e| format!("Database lock error: {}", e))?;
    let store = TestingStore::new(db_guard.conn.clone());
    store.save_test_result(suite_id, &name, &status, duration, error.as_deref())
        .map_err(|e| format!("Failed to save test result: {}", e))
}

#[tauri::command]
pub fn get_suite_results(
    suite_id: i64,
    db: State<'_, Mutex<Database>>,
) -> Result<Vec<crate::storage::testing::TestResult>, String> {
    let db_guard = db.lock().map_err(|e| format!("Database lock error: {}", e))?;
    let store = TestingStore::new(db_guard.conn.clone());
    store.get_suite_results(suite_id)
        .map_err(|e| format!("Failed to get suite results: {}", e))
}

#[tauri::command]
pub fn get_suite_stats(
    suite_id: i64,
    db: State<'_, Mutex<Database>>,
) -> Result<crate::storage::testing::TestSuiteStats, String> {
    let db_guard = db.lock().map_err(|e| format!("Database lock error: {}", e))?;
    let store = TestingStore::new(db_guard.conn.clone());
    store.get_suite_stats(suite_id)
        .map_err(|e| format!("Failed to get suite stats: {}", e))
}

