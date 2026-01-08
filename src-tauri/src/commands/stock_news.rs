use crate::services::NewsAggregator;
use crate::storage::{Database, StockNewsItem, StockNewsStore, StockTicker};
use std::sync::{Arc, Mutex};
use tauri::{Emitter, State};

#[tauri::command]
pub fn get_stock_tickers(
    index: Option<String>,
    db: State<'_, Mutex<Database>>,
) -> Result<Vec<StockTicker>, String> {
    let db_guard = db.lock().map_err(|e| format!("Database lock error: {}", e))?;
    let store = StockNewsStore::new(db_guard.conn.clone());
    store
        .list_tickers(index.as_deref())
        .map_err(|e| format!("Failed to list tickers: {}", e))
}

#[tauri::command]
pub fn get_stock_news(
    tickers: Option<Vec<String>>,
    limit: i32,
    since: Option<i64>,
    db: State<'_, Mutex<Database>>,
) -> Result<Vec<StockNewsItem>, String> {
    let db_guard = db.lock().map_err(|e| format!("Database lock error: {}", e))?;
    let store = StockNewsStore::new(db_guard.conn.clone());
    store
        .get_news(tickers, limit, since)
        .map_err(|e| format!("Failed to get news: {}", e))
}

#[tauri::command]
pub fn search_stock_news(
    query: String,
    tickers: Option<Vec<String>>,
    limit: i32,
    db: State<'_, Mutex<Database>>,
) -> Result<Vec<StockNewsItem>, String> {
    let db_guard = db.lock().map_err(|e| format!("Database lock error: {}", e))?;
    let store = StockNewsStore::new(db_guard.conn.clone());
    store
        .search_news(&query, tickers, limit)
        .map_err(|e| format!("Failed to search news: {}", e))
}

#[tauri::command]
pub fn get_news_for_ticker(
    ticker: String,
    limit: i32,
    db: State<'_, Mutex<Database>>,
) -> Result<Vec<StockNewsItem>, String> {
    let db_guard = db.lock().map_err(|e| format!("Database lock error: {}", e))?;
    let store = StockNewsStore::new(db_guard.conn.clone());
    store
        .get_news(Some(vec![ticker]), limit, None)
        .map_err(|e| format!("Failed to get news for ticker: {}", e))
}

#[tauri::command]
pub async fn refresh_stock_news(
    app: tauri::AppHandle,
    tickers: Option<Vec<String>>,
    db: State<'_, Mutex<Database>>,
) -> Result<usize, String> {
    // Get store
    let store = {
        let db_guard = db.lock().map_err(|e| format!("Database lock error: {}", e))?;
        Arc::new(Mutex::new(StockNewsStore::new(db_guard.conn.clone())))
    };

    // Create aggregator
    let aggregator = NewsAggregator::new(store)
        .map_err(|e| format!("Failed to create news aggregator: {}", e))?;

    // Fetch news
    let symbols = tickers.unwrap_or_default();
    let items = aggregator
        .fetch_all_news(&symbols, None)
        .await
        .map_err(|e| format!("Failed to fetch news: {}", e))?;

    let count = items.len();

    // Emit events for new items
    if count > 0 {
        for item in &items {
            let _ = app.emit(
                "ws-message",
                serde_json::json!({
                    "type": "stock-news",
                    "data": item,
                    "timestamp": chrono::Utc::now().timestamp_millis()
                }),
            );
        }
    }

    Ok(count)
}

#[tauri::command]
pub async fn start_news_stream(
    app: tauri::AppHandle,
    db: State<'_, Mutex<Database>>,
) -> Result<(), String> {
    // Get store
    let store = {
        let db_guard = db.lock().map_err(|e| format!("Database lock error: {}", e))?;
        Arc::new(Mutex::new(StockNewsStore::new(db_guard.conn.clone())))
    };

    // Create aggregator
    let aggregator = NewsAggregator::new(store)
        .map_err(|e| format!("Failed to create news aggregator: {}", e))?;

    // Start real-time stream
    aggregator
        .start_realtime_stream(app)
        .await
        .map_err(|e| format!("Failed to start news stream: {}", e))?;

    Ok(())
}

#[tauri::command]
pub fn cleanup_old_stock_news(
    days: i64,
    db: State<'_, Mutex<Database>>,
) -> Result<usize, String> {
    let db_guard = db.lock().map_err(|e| format!("Database lock error: {}", e))?;
    let store = StockNewsStore::new(db_guard.conn.clone());
    store
        .cleanup_old_news(days)
        .map_err(|e| format!("Failed to cleanup old news: {}", e))
}

