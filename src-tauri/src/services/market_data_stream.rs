use crate::storage::market_data::MarketPrice;
use crate::ws::{WsMessage, WsServer};
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
        let mut subs = self.subscribers.lock().unwrap();
        for ticker in tickers {
            if !subs.contains(&ticker) {
                subs.push(ticker);
            }
        }
    }

    pub fn unsubscribe(&self, tickers: Vec<String>) {
        let mut subs = self.subscribers.lock().unwrap();
        subs.retain(|t| !tickers.contains(t));
    }

    pub fn update_price(&self, price: MarketPrice) {
        let mut pending = self.pending_updates.lock().unwrap();
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
                    let mut pending = pending_updates.lock().unwrap();
                    let subs = subscribers.lock().unwrap();
                    
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
}

