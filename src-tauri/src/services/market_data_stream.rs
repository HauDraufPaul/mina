use crate::storage::market_data::MarketPrice;
use crate::ws::{WsMessage, WsServer};
use crate::providers::market_data::MarketDataManager;
use crate::storage::market_data::MarketDataStore;
use crate::storage::Database;
use std::collections::HashMap;
use std::sync::{Arc, Mutex};
use tokio::time::{interval, Duration};
use tauri::Emitter;

pub struct MarketDataStreamer {
    ws_server: Arc<WsServer>,
    pending_updates: Arc<Mutex<HashMap<String, MarketPrice>>>,
    subscribers: Arc<Mutex<Vec<String>>>, // List of subscribed tickers
}

impl MarketDataStreamer {
    pub fn new(ws_server: Arc<WsServer>) -> Self {
        MarketDataStreamer {
            ws_server,
            pending_updates: Arc::new(Mutex::new(HashMap::new())),
            subscribers: Arc::new(Mutex::new(Vec::new())),
        }
    }

    pub fn subscribe(&self, tickers: Vec<String>) {
        let mut subs = self.subscribers.lock()
            .map_err(|e| {
                eprintln!("Failed to lock subscribers: {}", e);
                e
            })
            .unwrap_or_else(|_| panic!("Subscribers mutex poisoned"));
        for ticker in tickers {
            if !subs.contains(&ticker) {
                subs.push(ticker);
            }
        }
    }

    pub fn unsubscribe(&self, tickers: Vec<String>) {
        let mut subs = self.subscribers.lock()
            .map_err(|e| {
                eprintln!("Failed to lock subscribers: {}", e);
                e
            })
            .unwrap_or_else(|_| panic!("Subscribers mutex poisoned"));
        subs.retain(|t| !tickers.contains(t));
    }

    pub fn update_price(&self, price: MarketPrice) {
        let mut pending = self.pending_updates.lock()
            .map_err(|e| {
                eprintln!("Failed to lock pending_updates: {}", e);
                e
            })
            .unwrap_or_else(|_| panic!("Pending updates mutex poisoned"));
        pending.insert(price.ticker.clone(), price);
    }

    pub fn start_batching(&self, app: tauri::AppHandle) {
        let pending_updates = self.pending_updates.clone();
        let ws_server = self.ws_server.clone();
        let subscribers = self.subscribers.clone();

        tauri::async_runtime::spawn(async move {
            let mut interval = interval(Duration::from_millis(100)); // Batch every 100ms

            loop {
                interval.tick().await;

                // Get pending updates
                let updates: Vec<MarketPrice> = {
                    let mut pending = pending_updates.lock()
                        .map_err(|e| {
                            eprintln!("Failed to lock pending_updates: {}", e);
                            e
                        })
                        .unwrap_or_else(|_| panic!("Pending updates mutex poisoned"));
                    let subs = subscribers.lock()
                        .map_err(|e| {
                            eprintln!("Failed to lock subscribers: {}", e);
                            e
                        })
                        .unwrap_or_else(|_| panic!("Subscribers mutex poisoned"));
                    
                    // Only send updates for subscribed tickers
                    let filtered: Vec<MarketPrice> = pending
                        .values()
                        .filter(|p| subs.contains(&p.ticker))
                        .cloned()
                        .collect();
                    
                    pending.clear();
                    filtered
                };

                if !updates.is_empty() {
                    // Send batch update
                    let msg = WsMessage::MarketDataBatch(updates.clone());
                    let _ = ws_server.publish("market-data", msg.clone());

                    // Also emit Tauri event for frontend
                    let _ = app.emit("ws-message", serde_json::json!({
                        "type": "market-data-batch",
                        "data": updates,
                        "timestamp": chrono::Utc::now().timestamp_millis(),
                    }));

                    // Send individual updates for single-price subscriptions
                    for price in &updates {
                        let msg = WsMessage::MarketData(price.clone());
                        let _ = ws_server.publish("market-data", msg);
                    }
                }
            }
        });
    }

    /// Start active fetching loop for subscribed tickers
    pub fn start_fetching_loop(&self, api_key_manager: Option<std::sync::Arc<crate::services::api_key_manager::APIKeyManager>>, db: Arc<Mutex<Database>>) {
        let streamer = Arc::new(self.clone());
        let subscribers = self.subscribers.clone();
        let pending_updates = self.pending_updates.clone();

        tauri::async_runtime::spawn(async move {
            let mut interval = interval(Duration::from_secs(5)); // Fetch every 5 seconds
            let manager = MarketDataManager::new(api_key_manager.as_ref().map(|m| m.as_ref()));
            let last_fetch_time = Arc::new(Mutex::new(std::collections::HashMap::<String, i64>::new()));

            loop {
                interval.tick().await;

                // Get subscribed tickers
                let tickers_to_fetch: Vec<String> = {
                    let subs = subscribers.lock()
                        .map_err(|e| {
                            eprintln!("Failed to lock subscribers: {}", e);
                            e
                        })
                        .unwrap_or_else(|_| panic!("Subscribers mutex poisoned"));
                    subs.clone()
                };

                if !tickers_to_fetch.is_empty() {
                    // Rate limit: only fetch if last fetch was > 1 second ago
                    let now = chrono::Utc::now().timestamp();
                    let tickers_to_fetch_now: Vec<String> = {
                        let lft = last_fetch_time.lock()
                            .map_err(|e| {
                                eprintln!("Failed to lock last_fetch_time: {}", e);
                                e
                            })
                            .unwrap_or_else(|_| panic!("Last fetch time mutex poisoned"));
                        tickers_to_fetch
                            .iter()
                            .filter(|ticker| {
                                let last = lft.get(*ticker).copied().unwrap_or(0);
                                now - last >= 1 // At least 1 second between fetches per ticker
                            })
                            .cloned()
                            .collect()
                    };

                    if !tickers_to_fetch_now.is_empty() {
                        // Fetch prices from provider
                        if let Ok(prices) = manager.get_prices(&tickers_to_fetch_now, None).await {
                            // Update last fetch time
                            {
                                let mut lft = last_fetch_time.lock()
                                    .map_err(|e| {
                                        eprintln!("Failed to lock last_fetch_time: {}", e);
                                        e
                                    })
                                    .unwrap_or_else(|_| panic!("Last fetch time mutex poisoned"));
                                for ticker in &tickers_to_fetch_now {
                                    lft.insert(ticker.clone(), now);
                                }
                            }

                            // Store in database and push to streamer
                            let conn = {
                                let db_guard = db.lock()
                                    .map_err(|e| {
                                        eprintln!("Failed to lock database: {}", e);
                                        e
                                    })
                                    .unwrap_or_else(|_| panic!("Database mutex poisoned"));
                                db_guard.conn.clone()
                            };
                            let store = MarketDataStore::new(conn);

                            for price_data in prices {
                                let price = MarketPrice {
                                    ticker: price_data.ticker.clone(),
                                    price: price_data.price,
                                    change: price_data.change,
                                    change_percent: price_data.change_percent,
                                    volume: price_data.volume,
                                    timestamp: price_data.timestamp,
                                };

                                // Cache in database
                                if let Err(e) = store.upsert_price(&price) {
                                    eprintln!("Failed to cache price: {}", e);
                                }

                                // Push to streamer for batching
                                {
                                    let mut pending = pending_updates.lock()
                                        .map_err(|e| {
                                            eprintln!("Failed to lock pending_updates: {}", e);
                                            e
                                        })
                                        .unwrap_or_else(|_| panic!("Pending updates mutex poisoned"));
                                    pending.insert(price.ticker.clone(), price);
                                }
                            }
                        }
                    }
                }
            }
        });
    }
}

impl Clone for MarketDataStreamer {
    fn clone(&self) -> Self {
        MarketDataStreamer {
            ws_server: self.ws_server.clone(),
            pending_updates: Arc::new(Mutex::new(HashMap::new())),
            subscribers: Arc::new(Mutex::new({
                let subs = self.subscribers.lock()
                    .map_err(|e| {
                        eprintln!("Failed to lock subscribers: {}", e);
                        e
                    })
                    .unwrap_or_else(|_| panic!("Subscribers mutex poisoned"));
                subs.clone()
            })),
        }
    }
}

