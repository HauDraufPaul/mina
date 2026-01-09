use crate::services::{NewsAggregator, SentimentAnalyzer};
use crate::storage::{Database, StockNewsItem, StockNewsStore, StockTicker};
use std::sync::{Arc, Mutex};
use tauri::{Emitter, State};

#[tauri::command]
pub fn get_stock_tickers(
    index: Option<String>,
    db: State<'_, Mutex<Database>>,
) -> Result<Vec<StockTicker>, String> {
    eprintln!("get_stock_tickers called with index: {:?}", index);
    let db_guard = db.lock().map_err(|e| format!("Database lock error: {}", e))?;
    let store = StockNewsStore::new(db_guard.conn.clone());
    let result = store
        .list_tickers(index.as_deref())
        .map_err(|e| format!("Failed to list tickers: {}", e))?;
    eprintln!("get_stock_tickers returning {} tickers", result.len());
    Ok(result)
}

#[tauri::command]
pub fn get_stock_news(
    tickers: Option<Vec<String>>,
    limit: i32,
    since: Option<i64>,
    db: State<'_, Mutex<Database>>,
) -> Result<Vec<StockNewsItem>, String> {
    eprintln!("get_stock_news called with tickers: {:?}, limit: {}, since: {:?}", tickers, limit, since);
    let db_guard = db.lock().map_err(|e| {
        eprintln!("Database lock error: {}", e);
        format!("Database lock error: {}", e)
    })?;
    let store = StockNewsStore::new(db_guard.conn.clone());
    let result = store
        .get_news(tickers, limit, since)
        .map_err(|e| {
            eprintln!("Failed to get news: {}", e);
            format!("Failed to get news: {}", e)
        })?;
    eprintln!("get_stock_news returning {} items", result.len());
    Ok(result)
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

#[tauri::command]
pub fn get_news_sentiment(
    ticker: String,
    days: i64,
    db: State<'_, Mutex<Database>>,
) -> Result<Vec<(i64, f64)>, String> {
    let db_guard = db.lock().map_err(|e| format!("Database lock error: {}", e))?;
    let store = StockNewsStore::new(db_guard.conn.clone());
    
    let now = chrono::Utc::now().timestamp();
    let since = now - (days * 24 * 3600);
    
    let news = store
        .get_news(Some(vec![ticker]), 1000, Some(since))
        .map_err(|e| format!("Failed to get news: {}", e))?;
    
    // Group by day and calculate average sentiment
    let mut daily_sentiment: std::collections::HashMap<i64, Vec<f64>> = std::collections::HashMap::new();
    
    for item in news {
        if let Some(sentiment) = item.sentiment {
            let day = item.published_at / (24 * 3600);
            daily_sentiment.entry(day).or_insert_with(Vec::new).push(sentiment);
        }
    }
    
    let mut result: Vec<(i64, f64)> = daily_sentiment
        .into_iter()
        .map(|(day, scores)| (day * 24 * 3600, SentimentAnalyzer::aggregate_sentiment(&scores)))
        .collect();
    
    result.sort_by_key(|(ts, _)| *ts);
    
    Ok(result)
}

#[tauri::command]
pub fn get_aggregated_sentiment(
    tickers: Vec<String>,
    db: State<'_, Mutex<Database>>,
) -> Result<std::collections::HashMap<String, f64>, String> {
    let db_guard = db.lock().map_err(|e| format!("Database lock error: {}", e))?;
    let store = StockNewsStore::new(db_guard.conn.clone());
    
    let mut result = std::collections::HashMap::new();
    
    for ticker in tickers {
        let news = store
            .get_news(Some(vec![ticker.clone()]), 100, None)
            .map_err(|e| format!("Failed to get news: {}", e))?;
        
        let sentiments: Vec<f64> = news
            .into_iter()
            .filter_map(|item| item.sentiment)
            .collect();
        
        let avg_sentiment = SentimentAnalyzer::aggregate_sentiment(&sentiments);
        result.insert(ticker, avg_sentiment);
    }
    
    Ok(result)
}

