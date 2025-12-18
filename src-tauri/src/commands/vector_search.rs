use crate::storage::vector_store::VectorStore;
use crate::storage::Database;
use std::sync::Mutex;
use tauri::State;

#[tauri::command]
pub fn search_vectors(
    collection: String,
    query_embedding: Vec<f32>,
    limit: i32,
    min_similarity: f64,
    db: State<'_, Mutex<Database>>,
) -> Result<Vec<(crate::storage::vector_store::VectorDocument, f32)>, String> {
    let db_guard = db.lock().map_err(|e| format!("Database lock error: {}", e))?;
    let store = VectorStore::new(db_guard.conn.clone());
    store.search_similar(&collection, &query_embedding, limit, min_similarity as f32)
        .map_err(|e| format!("Failed to search vectors: {}", e))
}

