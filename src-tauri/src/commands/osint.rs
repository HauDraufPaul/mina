use crate::storage::osint::OSINTStore;
use crate::storage::Database;
use std::sync::Mutex;
use tauri::State;

#[tauri::command]
pub fn create_rss_feed(
    url: String,
    name: String,
    db: State<'_, Mutex<Database>>,
) -> Result<i64, String> {
    let db_guard = db.lock().map_err(|e| format!("Database lock error: {}", e))?;
    let store = OSINTStore::new(db_guard.conn.clone());
    store.create_feed(&url, &name)
        .map_err(|e| format!("Failed to create feed: {}", e))
}

#[tauri::command]
pub fn list_rss_feeds(
    db: State<'_, Mutex<Database>>,
) -> Result<Vec<crate::storage::osint::RSSFeed>, String> {
    let db_guard = db.lock().map_err(|e| format!("Database lock error: {}", e))?;
    let store = OSINTStore::new(db_guard.conn.clone());
    store.list_feeds()
        .map_err(|e| format!("Failed to list feeds: {}", e))
}

#[tauri::command]
pub fn save_rss_item(
    feed_id: i64,
    title: String,
    content: String,
    url: String,
    published_at: i64,
    db: State<'_, Mutex<Database>>,
) -> Result<i64, String> {
    let db_guard = db.lock().map_err(|e| format!("Database lock error: {}", e))?;
    let store = OSINTStore::new(db_guard.conn.clone());
    store.save_rss_item(feed_id, &title, &content, &url, published_at)
        .map_err(|e| format!("Failed to save RSS item: {}", e))
}

#[tauri::command]
pub fn get_recent_rss_items(
    limit: i32,
    db: State<'_, Mutex<Database>>,
) -> Result<Vec<crate::storage::osint::RSSItem>, String> {
    let db_guard = db.lock().map_err(|e| format!("Database lock error: {}", e))?;
    let store = OSINTStore::new(db_guard.conn.clone());
    store.get_recent_items(limit)
        .map_err(|e| format!("Failed to get RSS items: {}", e))
}

#[tauri::command]
pub fn create_entity(
    entity_type: String,
    name: String,
    metadata: String,
    db: State<'_, Mutex<Database>>,
) -> Result<i64, String> {
    let db_guard = db.lock().map_err(|e| format!("Database lock error: {}", e))?;
    let store = OSINTStore::new(db_guard.conn.clone());
    store.create_entity(&entity_type, &name, &metadata)
        .map_err(|e| format!("Failed to create entity: {}", e))
}

#[tauri::command]
pub fn list_entities(
    entity_type: Option<String>,
    db: State<'_, Mutex<Database>>,
) -> Result<Vec<crate::storage::osint::Entity>, String> {
    let db_guard = db.lock().map_err(|e| format!("Database lock error: {}", e))?;
    let store = OSINTStore::new(db_guard.conn.clone());
    store.list_entities(entity_type.as_deref())
        .map_err(|e| format!("Failed to list entities: {}", e))
}

#[tauri::command]
pub fn create_entity_relationship(
    source_id: i64,
    target_id: i64,
    relationship_type: String,
    strength: f64,
    db: State<'_, Mutex<Database>>,
) -> Result<i64, String> {
    let db_guard = db.lock().map_err(|e| format!("Database lock error: {}", e))?;
    let store = OSINTStore::new(db_guard.conn.clone());
    store.create_relationship(source_id, target_id, &relationship_type, strength)
        .map_err(|e| format!("Failed to create relationship: {}", e))
}

