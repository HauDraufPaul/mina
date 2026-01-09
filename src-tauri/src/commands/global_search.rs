use crate::services::global_search::GlobalSearchService;
use crate::storage::Database;
use std::sync::Mutex;
use tauri::State;

#[tauri::command]
pub fn global_search(
    query: String,
    limit: Option<i32>,
    db: State<'_, Mutex<Database>>,
) -> Result<Vec<crate::services::global_search::SearchResult>, String> {
    if query.trim().is_empty() {
        return Ok(Vec::new());
    }

    GlobalSearchService::search(&query, limit, &db)
        .map_err(|e| format!("Global search failed: {}", e))
}

