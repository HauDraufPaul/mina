use crate::storage::projects::ProjectStore;
use crate::storage::Database;
use std::sync::Mutex;
use tauri::State;

#[tauri::command]
pub fn create_project(
    name: String,
    project_type: String,
    content: String,
    db: State<'_, Mutex<Database>>,
) -> Result<i64, String> {
    let db_guard = db.lock().map_err(|e| format!("Database lock error: {}", e))?;
    let store = ProjectStore::new(db_guard.conn.clone());
    store.create_project(&name, &project_type, &content)
        .map_err(|e| format!("Failed to create project: {}", e))
}

#[tauri::command]
pub fn update_project(
    id: i64,
    name: Option<String>,
    content: Option<String>,
    db: State<'_, Mutex<Database>>,
) -> Result<(), String> {
    let db_guard = db.lock().map_err(|e| format!("Database lock error: {}", e))?;
    let store = ProjectStore::new(db_guard.conn.clone());
    store.update_project(id, name.as_deref(), content.as_deref())
        .map_err(|e| format!("Failed to update project: {}", e))
}

#[tauri::command]
pub fn list_projects(
    project_type: Option<String>,
    db: State<'_, Mutex<Database>>,
) -> Result<Vec<crate::storage::projects::Project>, String> {
    let db_guard = db.lock().map_err(|e| format!("Database lock error: {}", e))?;
    let store = ProjectStore::new(db_guard.conn.clone());
    store.list_projects(project_type.as_deref())
        .map_err(|e| format!("Failed to list projects: {}", e))
}

#[tauri::command]
pub fn get_project(
    id: i64,
    db: State<'_, Mutex<Database>>,
) -> Result<Option<crate::storage::projects::Project>, String> {
    let db_guard = db.lock().map_err(|e| format!("Database lock error: {}", e))?;
    let store = ProjectStore::new(db_guard.conn.clone());
    store.get_project(id)
        .map_err(|e| format!("Failed to get project: {}", e))
}

#[tauri::command]
pub fn delete_project(
    id: i64,
    db: State<'_, Mutex<Database>>,
) -> Result<(), String> {
    let db_guard = db.lock().map_err(|e| format!("Database lock error: {}", e))?;
    let store = ProjectStore::new(db_guard.conn.clone());
    store.delete_project(id)
        .map_err(|e| format!("Failed to delete project: {}", e))
}

