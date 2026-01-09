use crate::storage::grid_layouts::{GridLayoutStore, GridLayoutData};
use crate::storage::Database;
use serde_json::Value;
use std::sync::Mutex;
use tauri::State;

#[tauri::command]
pub fn create_grid_layout(
    id: String,
    name: String,
    layout_json: String,
    columns: i32,
    rows: i32,
    is_template: bool,
    db: State<'_, Mutex<Database>>,
) -> Result<(), String> {
    let db_guard = db.lock().map_err(|e| format!("Database lock error: {}", e))?;
    let store = GridLayoutStore::new(db_guard.conn.clone());
    
    store.create_layout(&id, &name, &layout_json, columns, rows, is_template)
        .map_err(|e| format!("Failed to create layout: {}", e))?;
    
    Ok(())
}

#[tauri::command]
pub fn update_grid_layout(
    id: String,
    name: Option<String>,
    layout_json: Option<String>,
    columns: Option<i32>,
    rows: Option<i32>,
    db: State<'_, Mutex<Database>>,
) -> Result<(), String> {
    let db_guard = db.lock().map_err(|e| format!("Database lock error: {}", e))?;
    let store = GridLayoutStore::new(db_guard.conn.clone());
    
    store.update_layout(
        &id,
        name.as_deref(),
        layout_json.as_deref(),
        columns,
        rows,
    )
    .map_err(|e| format!("Failed to update layout: {}", e))?;
    
    Ok(())
}

#[tauri::command]
pub fn get_grid_layout(
    id: String,
    db: State<'_, Mutex<Database>>,
) -> Result<Option<GridLayoutData>, String> {
    let db_guard = db.lock().map_err(|e| format!("Database lock error: {}", e))?;
    let store = GridLayoutStore::new(db_guard.conn.clone());
    
    store.get_layout(&id)
        .map_err(|e| format!("Failed to get layout: {}", e))
}

#[tauri::command]
pub fn list_grid_layouts(
    include_templates: bool,
    db: State<'_, Mutex<Database>>,
) -> Result<Vec<GridLayoutData>, String> {
    let db_guard = db.lock().map_err(|e| format!("Database lock error: {}", e))?;
    let store = GridLayoutStore::new(db_guard.conn.clone());
    
    store.list_layouts(include_templates)
        .map_err(|e| format!("Failed to list layouts: {}", e))
}

#[tauri::command]
pub fn delete_grid_layout(
    id: String,
    db: State<'_, Mutex<Database>>,
) -> Result<(), String> {
    let db_guard = db.lock().map_err(|e| format!("Database lock error: {}", e))?;
    let store = GridLayoutStore::new(db_guard.conn.clone());
    
    store.delete_layout(&id)
        .map_err(|e| format!("Failed to delete layout: {}", e))?;
    
    Ok(())
}

#[tauri::command]
pub fn list_grid_layout_templates(
    db: State<'_, Mutex<Database>>,
) -> Result<Vec<GridLayoutData>, String> {
    let db_guard = db.lock().map_err(|e| format!("Database lock error: {}", e))?;
    let store = GridLayoutStore::new(db_guard.conn.clone());
    
    store.list_templates()
        .map_err(|e| format!("Failed to list templates: {}", e))
}

