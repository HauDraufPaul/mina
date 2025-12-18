use crate::storage::vector_store::VectorStore;
use crate::storage::Database;
use std::sync::Mutex;
use tauri::State;

#[tauri::command]
pub fn create_collection(name: String, dimension: i32, db: State<'_, Mutex<Database>>) -> Result<(), String> {
    let db_guard = db.lock().map_err(|e| format!("Database lock error: {}", e))?;
    let store = VectorStore::new(db_guard.conn.clone());
    store.create_collection(&name, dimension)
        .map_err(|e| format!("Failed to create collection: {}", e))
}

#[tauri::command]
pub fn list_collections(db: State<'_, Mutex<Database>>) -> Result<Vec<String>, String> {
    let db_guard = db.lock().map_err(|e| format!("Database lock error: {}", e))?;
    let store = VectorStore::new(db_guard.conn.clone());
    store.list_collections()
        .map_err(|e| format!("Failed to list collections: {}", e))
}

#[tauri::command]
pub fn get_collection_stats(collection: String, db: State<'_, Mutex<Database>>) -> Result<crate::storage::vector_store::CollectionStats, String> {
    let db_guard = db.lock().map_err(|e| format!("Database lock error: {}", e))?;
    let store = VectorStore::new(db_guard.conn.clone());
    store.get_collection_stats(&collection)
        .map_err(|e| format!("Failed to get collection stats: {}", e))
}

#[tauri::command]
pub fn cleanup_expired_vectors(db: State<'_, Mutex<Database>>) -> Result<usize, String> {
    let db_guard = db.lock().map_err(|e| format!("Database lock error: {}", e))?;
    let store = VectorStore::new(db_guard.conn.clone());
    store.delete_expired()
        .map_err(|e| format!("Failed to cleanup expired vectors: {}", e))
}
