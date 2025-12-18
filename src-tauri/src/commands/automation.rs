use crate::storage::automation::AutomationStore;
use crate::storage::Database;
use std::sync::Mutex;
use tauri::State;

#[tauri::command]
pub fn create_script(
    name: String,
    content: String,
    language: String,
    db: State<'_, Mutex<Database>>,
) -> Result<i64, String> {
    let db_guard = db.lock().map_err(|e| format!("Database lock error: {}", e))?;
    let store = AutomationStore::new(db_guard.conn.clone())
        .map_err(|e| format!("Failed to initialize AutomationStore: {}", e))?;
    store.create_script(&name, &content, &language)
        .map_err(|e| format!("Failed to create script: {}", e))
}

#[tauri::command]
pub fn list_scripts(
    db: State<'_, Mutex<Database>>,
) -> Result<Vec<crate::storage::automation::Script>, String> {
    let db_guard = db.lock().map_err(|e| format!("Database lock error: {}", e))?;
    let store = AutomationStore::new(db_guard.conn.clone())
        .map_err(|e| format!("Failed to initialize AutomationStore: {}", e))?;
    store.list_scripts()
        .map_err(|e| format!("Failed to list scripts: {}", e))
}

#[tauri::command]
pub fn get_script(
    id: i64,
    db: State<'_, Mutex<Database>>,
) -> Result<Option<crate::storage::automation::Script>, String> {
    let db_guard = db.lock().map_err(|e| format!("Database lock error: {}", e))?;
    let store = AutomationStore::new(db_guard.conn.clone())
        .map_err(|e| format!("Failed to initialize AutomationStore: {}", e))?;
    store.get_script(id)
        .map_err(|e| format!("Failed to get script: {}", e))
}

#[tauri::command]
pub fn create_workflow(
    name: String,
    description: Option<String>,
    trigger_type: String,
    trigger_config: String,
    steps: String,
    db: State<'_, Mutex<Database>>,
) -> Result<i64, String> {
    let db_guard = db.lock().map_err(|e| format!("Database lock error: {}", e))?;
    let store = AutomationStore::new(db_guard.conn.clone())
        .map_err(|e| format!("Failed to initialize AutomationStore: {}", e))?;
    store.create_workflow(&name, description.as_deref(), &trigger_type, &trigger_config, &steps)
        .map_err(|e| format!("Failed to create workflow: {}", e))
}

#[tauri::command]
pub fn list_workflows(
    db: State<'_, Mutex<Database>>,
) -> Result<Vec<crate::storage::automation::Workflow>, String> {
    let db_guard = db.lock().map_err(|e| format!("Database lock error: {}", e))?;
    let store = AutomationStore::new(db_guard.conn.clone())
        .map_err(|e| format!("Failed to initialize AutomationStore: {}", e))?;
    store.list_workflows()
        .map_err(|e| format!("Failed to list workflows: {}", e))
}

#[tauri::command]
pub fn record_workflow_execution(
    workflow_id: i64,
    status: String,
    error: Option<String>,
    db: State<'_, Mutex<Database>>,
) -> Result<i64, String> {
    let db_guard = db.lock().map_err(|e| format!("Database lock error: {}", e))?;
    let store = AutomationStore::new(db_guard.conn.clone())
        .map_err(|e| format!("Failed to initialize AutomationStore: {}", e))?;
    store.record_execution(workflow_id, &status, error.as_deref())
        .map_err(|e| format!("Failed to record execution: {}", e))
}

#[tauri::command]
pub fn get_workflow_executions(
    workflow_id: Option<i64>,
    limit: i32,
    db: State<'_, Mutex<Database>>,
) -> Result<Vec<crate::storage::automation::WorkflowExecution>, String> {
    let db_guard = db.lock().map_err(|e| format!("Database lock error: {}", e))?;
    let store = AutomationStore::new(db_guard.conn.clone())
        .map_err(|e| format!("Failed to initialize AutomationStore: {}", e))?;
    store.get_executions(workflow_id, limit)
        .map_err(|e| format!("Failed to get executions: {}", e))
}

