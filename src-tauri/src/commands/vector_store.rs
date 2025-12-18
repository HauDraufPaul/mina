use crate::storage::vector_store::VectorStore;
use crate::storage::Database;
use std::sync::Mutex;
use tauri::AppHandle;

#[tauri::command]
pub fn create_collection(name: String, dimension: i32, app: AppHandle) -> Result<(), String> {
    let db = app.try_state::<Mutex<Database>>()
        .ok_or("Database not found")?;
    let db = db.lock().map_err(|e| format!("Database lock error: {}", e))?;
    
    let store = VectorStore::new(db.conn.clone());
    store.create_collection(&name, dimension)
        .map_err(|e| format!("Failed to create collection: {}", e))
}

#[tauri::command]
pub fn list_collections(app: AppHandle) -> Result<Vec<String>, String> {
    let db = app.try_state::<Mutex<Database>>()
        .ok_or("Database not found")?;
    let db = db.lock().map_err(|e| format!("Database lock error: {}", e))?;
    
    let store = VectorStore::new(db.conn.clone());
    store.list_collections()
        .map_err(|e| format!("Failed to list collections: {}", e))
}

#[tauri::command]
pub fn get_collection_stats(collection: String, app: AppHandle) -> Result<crate::storage::vector_store::CollectionStats, String> {
    let db = app.try_state::<Mutex<Database>>()
        .ok_or("Database not found")?;
    let db = db.lock().map_err(|e| format!("Database lock error: {}", e))?;
    
    let store = VectorStore::new(db.conn.clone());
    store.get_collection_stats(&collection)
        .map_err(|e| format!("Failed to get collection stats: {}", e))
}

#[tauri::command]
pub fn cleanup_expired_vectors(app: AppHandle) -> Result<usize, String> {
    let db = app.try_state::<Mutex<Database>>()
        .ok_or("Database not found")?;
    let db = db.lock().map_err(|e| format!("Database lock error: {}", e))?;
    
    let store = VectorStore::new(db.conn.clone());
    store.delete_expired()
        .map_err(|e| format!("Failed to cleanup expired vectors: {}", e))
}

