use crate::providers::market_data::MarketDataManager;
use crate::storage::market_data::{MarketDataStore, MarketPrice, PriceHistory};
use crate::storage::Database;
use crate::services::market_data_stream::MarketDataStreamer;
use serde::{Deserialize, Serialize};
use std::sync::Mutex;
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
) -> Result<Option<MarketPrice>, String> {
    let conn = {
        let db_guard = db.lock().map_err(|e| format!("Database lock error: {}", e))?;
        db_guard.conn.clone()
    };
    let store = MarketDataStore::new(conn);
    
    // Try to get from cache first
    if let Ok(Some(price)) = store.get_price(&ticker) {
        // Check if cache is fresh (less than 1 minute old)
        let now = chrono::Utc::now().timestamp();
        if now - price.timestamp < 60 {
            return Ok(Some(price));
        }
    }

    // Fetch from provider
    let manager = MarketDataManager::new();
    match manager.get_price(&ticker).await {
        Ok(price_data) => {
            let price = MarketPrice {
                ticker: price_data.ticker.clone(),
                price: price_data.price,
                change: price_data.change,
                change_percent: price_data.change_percent,
                volume: price_data.volume,
                timestamp: price_data.timestamp,
            };
            
            // Cache it (need to get connection again)
            let conn = {
                let db_guard = db.lock().map_err(|e| format!("Database lock error: {}", e))?;
                db_guard.conn.clone()
            };
            let store = MarketDataStore::new(conn);
            if let Err(e) = store.upsert_price(&price) {
                eprintln!("Failed to cache price: {}", e);
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
) -> Result<Vec<MarketPrice>, String> {
    let conn = {
        let db_guard = db.lock().map_err(|e| format!("Database lock error: {}", e))?;
        db_guard.conn.clone()
    };
    let store = MarketDataStore::new(conn);
    
    // Try to get from cache first
    let cached = store.get_prices(&tickers).unwrap_or_default();
    let mut cached_map: std::collections::HashMap<String, MarketPrice> = cached
        .into_iter()
        .map(|p| (p.ticker.clone(), p))
        .collect();
    
    let now = chrono::Utc::now().timestamp();
    let to_fetch: Vec<String> = tickers
        .iter()
        .filter(|t| {
            if let Some(price) = cached_map.get(*t) {
                now - price.timestamp >= 60 // Cache expired
            } else {
                true // Not cached
            }
        })
        .cloned()
        .collect();

    // Fetch missing/expired prices
    if !to_fetch.is_empty() {
        let manager = MarketDataManager::new();
        if let Ok(prices) = manager.get_prices(&to_fetch).await {
            for price_data in prices {
                let price = MarketPrice {
                    ticker: price_data.ticker.clone(),
                    price: price_data.price,
                    change: price_data.change,
                    change_percent: price_data.change_percent,
                    volume: price_data.volume,
                    timestamp: price_data.timestamp,
                };
                
                if let Err(e) = store.upsert_price(&price) {
                    eprintln!("Failed to cache price: {}", e);
                }
                
                // Push to streamer
                if let Ok(streamer) = streamer.lock() {
                    streamer.update_price(price.clone());
                }
                
                cached_map.insert(price.ticker.clone(), price);
            }
        }
    }

    // Return prices in order requested
    let result: Vec<MarketPrice> = tickers
        .iter()
        .filter_map(|t| cached_map.get(t).cloned())
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
) -> Result<Vec<ChartDataPoint>, String> {
    let conn = {
        let db_guard = db.lock().map_err(|e| format!("Database lock error: {}", e))?;
        db_guard.conn.clone()
    };
    let store = MarketDataStore::new(conn);
    
    // Try to get from cache first
    if let Ok(history) = store.get_price_history(&ticker, from_ts, to_ts, Some(10000)) {
        if !history.is_empty() {
            return Ok(history
                .into_iter()
                .map(|h| ChartDataPoint {
                    time: h.timestamp,
                    open: h.open,
                    high: h.high,
                    low: h.low,
                    close: h.close,
                    volume: h.volume,
                })
                .collect());
        }
    }

    // Fetch from provider
    let manager = MarketDataManager::new();
    match manager.get_history(&ticker, from_ts, to_ts, &interval).await {
        Ok(ohlcv_data) => {
            // Cache the data
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
