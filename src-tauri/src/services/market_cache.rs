use crate::storage::market_data::MarketPrice;
use crate::providers::market_data::OHLCVData;
use std::collections::HashMap;
use std::sync::{Arc, Mutex};
use std::time::{Duration, Instant};

pub struct MarketDataCache {
    prices: Arc<Mutex<HashMap<String, (MarketPrice, Instant)>>>,
    history: Arc<Mutex<HashMap<String, (Vec<OHLCVData>, Instant)>>>,
    price_ttl: Duration,
    history_ttl: Duration,
}

impl MarketDataCache {
    pub fn new() -> Self {
        MarketDataCache {
            prices: Arc::new(Mutex::new(HashMap::new())),
            history: Arc::new(Mutex::new(HashMap::new())),
            price_ttl: Duration::from_secs(60), // 1 minute for prices
            history_ttl: Duration::from_secs(3600), // 1 hour for history
        }
    }

    /// Get cached price if available and fresh
    pub fn get_price(&self, ticker: &str) -> Option<MarketPrice> {
        let cache = self.prices.lock().ok()?;
        if let Some((price, timestamp)) = cache.get(ticker) {
            if timestamp.elapsed() < self.price_ttl {
                return Some(price.clone());
            }
        }
        None
    }

    /// Set price in cache
    pub fn set_price(&self, ticker: String, price: MarketPrice) {
        if let Ok(mut cache) = self.prices.lock() {
            cache.insert(ticker, (price, Instant::now()));
        }
    }

    /// Get cached history if available and fresh
    pub fn get_history(&self, ticker: &str, from_ts: i64, to_ts: i64) -> Option<Vec<OHLCVData>> {
        let cache = self.history.lock().ok()?;
        let cache_key = format!("{}:{}:{}", ticker, from_ts, to_ts);
        if let Some((history, timestamp)) = cache.get(&cache_key) {
            if timestamp.elapsed() < self.history_ttl {
                // Filter history to requested time range
                let filtered: Vec<OHLCVData> = history
                    .iter()
                    .filter(|d| d.timestamp >= from_ts && d.timestamp <= to_ts)
                    .cloned()
                    .collect();
                if !filtered.is_empty() {
                    return Some(filtered);
                }
            }
        }
        None
    }

    /// Set history in cache
    pub fn set_history(&self, ticker: String, from_ts: i64, to_ts: i64, history: Vec<OHLCVData>) {
        if let Ok(mut cache) = self.history.lock() {
            let cache_key = format!("{}:{}:{}", ticker, from_ts, to_ts);
            cache.insert(cache_key, (history, Instant::now()));
        }
    }

    /// Clear expired entries
    pub fn cleanup(&self) {
        let now = Instant::now();
        
        // Cleanup prices
        if let Ok(mut cache) = self.prices.lock() {
            cache.retain(|_, (_, timestamp)| now.duration_since(*timestamp) < self.price_ttl);
        }
        
        // Cleanup history
        if let Ok(mut cache) = self.history.lock() {
            cache.retain(|_, (_, timestamp)| now.duration_since(*timestamp) < self.history_ttl);
        }
    }

    /// Clear all cache
    pub fn clear(&self) {
        if let Ok(mut cache) = self.prices.lock() {
            cache.clear();
        }
        if let Ok(mut cache) = self.history.lock() {
            cache.clear();
        }
    }

    /// Get cache statistics
    pub fn stats(&self) -> (usize, usize) {
        let price_count = self.prices.lock().map(|c| c.len()).unwrap_or(0);
        let history_count = self.history.lock().map(|c| c.len()).unwrap_or(0);
        (price_count, history_count)
    }
}

impl Default for MarketDataCache {
    fn default() -> Self {
        Self::new()
    }
}

impl Clone for MarketDataCache {
    fn clone(&self) -> Self {
        MarketDataCache {
            prices: self.prices.clone(),
            history: self.history.clone(),
            price_ttl: self.price_ttl,
            history_ttl: self.history_ttl,
        }
    }
}

