use crate::storage::automation::AutomationStore;
use crate::storage::Database;
use crate::services::script_engine::ScriptEngine;
use crate::services::workflow_engine::WorkflowEngine;
use serde_json::Value;
use std::sync::{Arc, Mutex};
use tauri::{AppHandle, Manager, State};

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
pub fn update_script(
    id: i64,
    name: String,
    content: String,
    language: String,
    db: State<'_, Mutex<Database>>,
) -> Result<(), String> {
    let db_guard = db.lock().map_err(|e| format!("Database lock error: {}", e))?;
    let store = AutomationStore::new(db_guard.conn.clone())
        .map_err(|e| format!("Failed to initialize AutomationStore: {}", e))?;
    store.update_script(id, &name, &content, &language)
        .map_err(|e| format!("Failed to update script: {}", e))
}

#[tauri::command]
pub fn delete_script(
    id: i64,
    db: State<'_, Mutex<Database>>,
) -> Result<(), String> {
    let db_guard = db.lock().map_err(|e| format!("Database lock error: {}", e))?;
    let store = AutomationStore::new(db_guard.conn.clone())
        .map_err(|e| format!("Failed to initialize AutomationStore: {}", e))?;
    store.delete_script(id)
        .map_err(|e| format!("Failed to delete script: {}", e))
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
pub fn get_workflow(
    id: i64,
    db: State<'_, Mutex<Database>>,
) -> Result<Option<crate::storage::automation::Workflow>, String> {
    let db_guard = db.lock().map_err(|e| format!("Database lock error: {}", e))?;
    let store = AutomationStore::new(db_guard.conn.clone())
        .map_err(|e| format!("Failed to initialize AutomationStore: {}", e))?;
    store.get_workflow(id)
        .map_err(|e| format!("Failed to get workflow: {}", e))
}

#[tauri::command]
pub fn update_workflow(
    id: i64,
    name: Option<String>,
    description: Option<String>,
    trigger_type: Option<String>,
    trigger_config: Option<String>,
    steps: Option<String>,
    enabled: Option<bool>,
    db: State<'_, Mutex<Database>>,
) -> Result<(), String> {
    let db_guard = db.lock().map_err(|e| format!("Database lock error: {}", e))?;
    let store = AutomationStore::new(db_guard.conn.clone())
        .map_err(|e| format!("Failed to initialize AutomationStore: {}", e))?;
    store.update_workflow(
        id,
        name.as_deref(),
        description.as_deref(),
        trigger_type.as_deref(),
        trigger_config.as_deref(),
        steps.as_deref(),
        enabled,
    )
    .map_err(|e| format!("Failed to update workflow: {}", e))
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

#[tauri::command]
pub async fn execute_script(
    script_id: i64,
    inputs: Option<Value>,
    db: State<'_, Mutex<Database>>,
    app: AppHandle,
) -> Result<crate::services::script_engine::ScriptExecutionResult, String> {
    let conn_clone = {
        let db_guard = db.lock().map_err(|e| format!("Database lock error: {}", e))?;
        db_guard.conn.clone()
    }; // Lock released here
    
    let db_arc = Arc::new(Mutex::new(Database {
        conn: conn_clone,
    }));
    
    let script_engine = ScriptEngine::new(db_arc, app);
    script_engine.execute_script(script_id, inputs)
        .await
        .map_err(|e| format!("Failed to execute script: {}", e))
}

#[tauri::command]
pub async fn execute_workflow(
    workflow_id: i64,
    trigger_data: Option<Value>,
    db: State<'_, Mutex<Database>>,
    app: AppHandle,
) -> Result<i64, String> {
    let conn_clone = {
        let db_guard = db.lock().map_err(|e| format!("Database lock error: {}", e))?;
        db_guard.conn.clone()
    }; // Lock released here
    
    let db_arc = Arc::new(Mutex::new(Database {
        conn: conn_clone,
    }));
    
    let workflow_engine = WorkflowEngine::new(db_arc, app);
    workflow_engine.execute_workflow(workflow_id, trigger_data)
        .await
        .map_err(|e| format!("Failed to execute workflow: {}", e))
}

