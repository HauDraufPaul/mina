use crate::providers::market_data::MarketDataManager;
use crate::storage::market_data::{MarketDataStore, MarketPrice, PriceHistory};
use crate::storage::Database;
use crate::services::market_data_stream::MarketDataStreamer;
use crate::services::market_cache::MarketDataCache;
use crate::services::rate_limiter::RateLimiter;
use crate::services::api_key_manager::APIKeyManager;
use serde::{Deserialize, Serialize};
use std::sync::{Arc, Mutex};
use tauri::State;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChartDataPoint {
    pub time: i64,
    pub open: f64,
    pub high: f64,
    pub low: f64,
    pub close: f64,
    pub volume: i64,
}

#[tauri::command]
pub async fn get_market_price(
    ticker: String,
    db: State<'_, Mutex<Database>>,
    cache: State<'_, Mutex<MarketDataCache>>,
) -> Result<Option<MarketPrice>, String> {
    // Try in-memory cache first
    if let Ok(cache_guard) = cache.lock() {
        if let Some(price) = cache_guard.get_price(&ticker) {
            return Ok(Some(price));
        }
    }
    
    let conn = {
        let db_guard = db.lock().map_err(|e| format!("Database lock error: {}", e))?;
        db_guard.conn.clone()
    };
    let store = MarketDataStore::new(conn);
    
    // Try database cache
    if let Ok(Some(price)) = store.get_price(&ticker) {
        let now = chrono::Utc::now().timestamp();
        if now - price.timestamp < 60 {
            // Update in-memory cache
            if let Ok(cache_guard) = cache.lock() {
                cache_guard.set_price(ticker.clone(), price.clone());
            }
            return Ok(Some(price));
        }
    }

    // Fetch from provider
    let api_key_manager = app.try_state::<Arc<APIKeyManager>>();
    let manager = MarketDataManager::new(api_key_manager.map(|m| m.as_ref()));
    // Rate limiter is optional - pass None to avoid holding lock across await
    match manager.get_price(&ticker, None).await {
        Ok(price_data) => {
            let price = MarketPrice {
                ticker: price_data.ticker.clone(),
                price: price_data.price,
                change: price_data.change,
                change_percent: price_data.change_percent,
                volume: price_data.volume,
                timestamp: price_data.timestamp,
            };
            
            // Cache in memory
            if let Ok(cache_guard) = cache.lock() {
                cache_guard.set_price(ticker.clone(), price.clone());
            }
            
            // Cache in database
            let conn = {
                let db_guard = db.lock().map_err(|e| format!("Database lock error: {}", e))?;
                db_guard.conn.clone()
            };
            let store = MarketDataStore::new(conn);
            if let Err(e) = store.upsert_price(&price) {
                eprintln!("Failed to cache price in database: {}", e);
            }
            
            Ok(Some(price))
        }
        Err(e) => Err(format!("Failed to fetch price: {}", e)),
    }
}

#[tauri::command]
pub async fn get_market_prices(
    tickers: Vec<String>,
    db: State<'_, Mutex<Database>>,
    streamer: State<'_, Mutex<MarketDataStreamer>>,
    cache: State<'_, Mutex<MarketDataCache>>,
    app: tauri::AppHandle,
) -> Result<Vec<MarketPrice>, String> {
    let mut result_map: std::collections::HashMap<String, MarketPrice> = std::collections::HashMap::new();
    let mut to_fetch: Vec<String> = Vec::new();
    
    // Check in-memory cache first
    if let Ok(cache_guard) = cache.lock() {
        for ticker in &tickers {
            if let Some(price) = cache_guard.get_price(ticker) {
                result_map.insert(ticker.clone(), price);
            } else {
                to_fetch.push(ticker.clone());
            }
        }
    } else {
        to_fetch = tickers.clone();
    }
    
    // Check database cache for remaining tickers
    if !to_fetch.is_empty() {
        let conn = {
            let db_guard = db.lock().map_err(|e| format!("Database lock error: {}", e))?;
            db_guard.conn.clone()
        };
        let store = MarketDataStore::new(conn);
        
        let cached = store.get_prices(&to_fetch).unwrap_or_default();
        let now = chrono::Utc::now().timestamp();
        
        let mut still_to_fetch = Vec::new();
        for price in cached {
            if now - price.timestamp < 60 {
                // Fresh cache
                result_map.insert(price.ticker.clone(), price.clone());
                // Update in-memory cache
                if let Ok(cache_guard) = cache.lock() {
                    cache_guard.set_price(price.ticker.clone(), price);
                }
            } else {
                still_to_fetch.push(price.ticker.clone());
            }
        }
        
        // Add tickers not in database cache
        for ticker in &to_fetch {
            if !result_map.contains_key(ticker) {
                still_to_fetch.push(ticker.clone());
            }
        }
        to_fetch = still_to_fetch;
    }

    // Fetch missing/expired prices
    if !to_fetch.is_empty() {
        let api_key_manager = app.try_state::<Arc<APIKeyManager>>();
        let manager = MarketDataManager::new(api_key_manager.map(|m| m.as_ref()));
        // Rate limiter is optional - pass None to avoid holding lock across await
        if let Ok(prices) = manager.get_prices(&to_fetch, None).await {
            for price_data in prices {
                let price = MarketPrice {
                    ticker: price_data.ticker.clone(),
                    price: price_data.price,
                    change: price_data.change,
                    change_percent: price_data.change_percent,
                    volume: price_data.volume,
                    timestamp: price_data.timestamp,
                };
                
                // Cache in memory
                if let Ok(cache_guard) = cache.lock() {
                    cache_guard.set_price(price.ticker.clone(), price.clone());
                }
                
                // Cache in database
                let conn = {
                    let db_guard = db.lock().map_err(|e| format!("Database lock error: {}", e))?;
                    db_guard.conn.clone()
                };
                let store = MarketDataStore::new(conn);
                if let Err(e) = store.upsert_price(&price) {
                    eprintln!("Failed to cache price: {}", e);
                }
                
                // Push to streamer and subscribe ticker
                if let Ok(streamer_guard) = streamer.lock() {
                    streamer_guard.update_price(price.clone());
                    streamer_guard.subscribe(vec![price.ticker.clone()]);
                }
                
                result_map.insert(price.ticker.clone(), price);
            }
        }
    }

    // Return prices in order requested
    let result: Vec<MarketPrice> = tickers
        .iter()
        .filter_map(|t| result_map.get(t).cloned())
        .collect();

    Ok(result)
}

#[tauri::command]
pub async fn get_chart_data(
    ticker: String,
    from_ts: i64,
    to_ts: i64,
    interval: String,
    db: State<'_, Mutex<Database>>,
    cache: State<'_, Mutex<MarketDataCache>>,
    app: tauri::AppHandle,
) -> Result<Vec<ChartDataPoint>, String> {
    use crate::providers::market_data::OHLCVData;
    
    // Try in-memory cache first
    if let Ok(cache_guard) = cache.lock() {
        if let Some(history) = cache_guard.get_history(&ticker, from_ts, to_ts) {
            return Ok(history
                .into_iter()
                .map(|d| ChartDataPoint {
                    time: d.timestamp,
                    open: d.open,
                    high: d.high,
                    low: d.low,
                    close: d.close,
                    volume: d.volume,
                })
                .collect());
        }
    }
    
    let conn = {
        let db_guard = db.lock().map_err(|e| format!("Database lock error: {}", e))?;
        db_guard.conn.clone()
    };
    let store = MarketDataStore::new(conn);
    
    // Try database cache
    if let Ok(history) = store.get_price_history(&ticker, from_ts, to_ts, Some(10000)) {
        if !history.is_empty() {
            let ohlcv_data: Vec<OHLCVData> = history
                .into_iter()
                .map(|h| OHLCVData {
                    timestamp: h.timestamp,
                    open: h.open,
                    high: h.high,
                    low: h.low,
                    close: h.close,
                    volume: h.volume,
                })
                .collect();
            
            // Cache in memory
            if let Ok(cache_guard) = cache.lock() {
                cache_guard.set_history(ticker.clone(), from_ts, to_ts, ohlcv_data.clone());
            }
            
            return Ok(ohlcv_data
                .into_iter()
                .map(|d| ChartDataPoint {
                    time: d.timestamp,
                    open: d.open,
                    high: d.high,
                    low: d.low,
                    close: d.close,
                    volume: d.volume,
                })
                .collect());
        }
    }

    // Fetch from provider
    let api_key_manager = app.try_state::<Arc<APIKeyManager>>();
    let manager = MarketDataManager::new(api_key_manager.map(|m| m.as_ref()));
    // Rate limiter is optional - pass None to avoid holding lock across await
    match manager.get_history(&ticker, from_ts, to_ts, &interval, None).await {
        Ok(ohlcv_data) => {
            // Cache in memory
            if let Ok(cache_guard) = cache.lock() {
                cache_guard.set_history(ticker.clone(), from_ts, to_ts, ohlcv_data.clone());
            }
            
            // Cache in database
            let conn = {
                let db_guard = db.lock().map_err(|e| format!("Database lock error: {}", e))?;
                db_guard.conn.clone()
            };
            let store = MarketDataStore::new(conn);
            for data in &ohlcv_data {
                let history = PriceHistory {
                    id: 0,
                    ticker: ticker.clone(),
                    timestamp: data.timestamp,
                    open: data.open,
                    high: data.high,
                    low: data.low,
                    close: data.close,
                    volume: data.volume,
                };
                
                if let Err(e) = store.insert_price_history(&history) {
                    eprintln!("Failed to cache history: {}", e);
                }
            }

            Ok(ohlcv_data
                .into_iter()
                .map(|d| ChartDataPoint {
                    time: d.timestamp,
                    open: d.open,
                    high: d.high,
                    low: d.low,
                    close: d.close,
                    volume: d.volume,
                })
                .collect())
        }
        Err(e) => Err(format!("Failed to fetch chart data: {}", e)),
    }
}

#[tauri::command]
pub fn get_events_for_chart(
    ticker: String,
    from_ts: i64,
    to_ts: i64,
    db: State<'_, Mutex<Database>>,
) -> Result<Vec<crate::storage::temporal::TemporalEvent>, String> {
    use crate::storage::temporal::TemporalStore;
    
    let db_guard = db.lock().map_err(|e| format!("Database lock error: {}", e))?;
    let store = TemporalStore::new(db_guard.conn.clone());
    
    // Get events in time range
    // For now, filter by ticker in title/summary (will be enhanced later)
    let events = store
        .list_events(1000, Some(from_ts), Some(to_ts))
        .map_err(|e| format!("Failed to list events: {}", e))?;
    
    // Filter events that mention the ticker
    let filtered: Vec<_> = events
        .into_iter()
        .filter(|e| {
            e.title.to_uppercase().contains(&ticker.to_uppercase())
                || e.summary.to_uppercase().contains(&ticker.to_uppercase())
        })
        .collect();
    
    Ok(filtered)
}
