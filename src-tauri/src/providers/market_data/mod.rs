pub mod yahoo;
pub mod alpha_vantage;
pub mod polygon;

pub use yahoo::YahooFinanceProvider;
pub use alpha_vantage::AlphaVantageProvider;
pub use polygon::PolygonProvider;

use anyhow::Result;
use async_trait::async_trait;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OHLCVData {
    pub timestamp: i64,
    pub open: f64,
    pub high: f64,
    pub low: f64,
    pub close: f64,
    pub volume: i64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MarketPriceData {
    pub ticker: String,
    pub price: f64,
    pub change: f64,
    pub change_percent: f64,
    pub volume: i64,
    pub timestamp: i64,
}

#[async_trait]
pub trait MarketDataProvider: Send + Sync {
    async fn get_price(&self, ticker: &str) -> Result<MarketPriceData>;
    async fn get_prices(&self, tickers: &[String]) -> Result<Vec<MarketPriceData>>;
    async fn get_history(
        &self,
        ticker: &str,
        from_ts: i64,
        to_ts: i64,
        interval: &str,
    ) -> Result<Vec<OHLCVData>>;
    fn get_name(&self) -> &str;
}

pub struct MarketDataManager {
    providers: Vec<Box<dyn MarketDataProvider>>,
    default_provider: usize,
}

impl MarketDataManager {
    pub fn new(api_key_manager: Option<&crate::services::api_key_manager::APIKeyManager>) -> Self {
        let mut providers: Vec<Box<dyn MarketDataProvider>> = Vec::new();
        
        // Always add Yahoo Finance (no key required)
        providers.push(Box::new(YahooFinanceProvider::new()));
        
        // Add Alpha Vantage if key available
        if let Some(key_mgr) = api_key_manager {
            if let Ok(Some(api_key)) = key_mgr.get_key_optional("alpha_vantage") {
                providers.push(Box::new(AlphaVantageProvider::new(api_key)));
            }
        }
        
        // Add Polygon if key available
        if let Some(key_mgr) = api_key_manager {
            if let Ok(Some(api_key)) = key_mgr.get_key_optional("polygon") {
                providers.push(Box::new(PolygonProvider::new(api_key)));
            }
        }
        
        MarketDataManager {
            providers,
            default_provider: 0, // Yahoo Finance is default
        }
    }

    pub async fn get_price(&self, ticker: &str, rate_limiter: Option<&crate::services::rate_limiter::RateLimiter>) -> Result<MarketPriceData> {
        // Try default provider first
        let provider_name = self.providers[self.default_provider].get_name();
        if let Some(limiter) = rate_limiter {
            limiter.wait_if_needed(provider_name).await;
        }
        match self.providers[self.default_provider].get_price(ticker).await {
            Ok(price) => {
                if let Some(limiter) = rate_limiter {
                    limiter.record_request(provider_name);
                }
                Ok(price)
            },
            Err(e) => {
                eprintln!("Default provider failed: {}, trying fallback", e);
                // Try other providers
                for provider in &self.providers {
                    let provider_name = provider.get_name();
                    if let Some(limiter) = rate_limiter {
                        limiter.wait_if_needed(provider_name).await;
                    }
                    if let Ok(price) = provider.get_price(ticker).await {
                        if let Some(limiter) = rate_limiter {
                            limiter.record_request(provider_name);
                        }
                        return Ok(price);
                    }
                }
                Err(e)
            }
        }
    }

    pub async fn get_prices(&self, tickers: &[String], rate_limiter: Option<&crate::services::rate_limiter::RateLimiter>) -> Result<Vec<MarketPriceData>> {
        // Try default provider first
        let provider_name = self.providers[self.default_provider].get_name();
        if let Some(limiter) = rate_limiter {
            limiter.wait_if_needed(provider_name).await;
        }
        match self.providers[self.default_provider].get_prices(tickers).await {
            Ok(prices) => {
                if let Some(limiter) = rate_limiter {
                    limiter.record_request(provider_name);
                }
                Ok(prices)
            },
            Err(e) => {
                eprintln!("Default provider failed: {}, trying fallback", e);
                // Try other providers
                for provider in &self.providers {
                    let provider_name = provider.get_name();
                    if let Some(limiter) = rate_limiter {
                        limiter.wait_if_needed(provider_name).await;
                    }
                    if let Ok(prices) = provider.get_prices(tickers).await {
                        if let Some(limiter) = rate_limiter {
                            limiter.record_request(provider_name);
                        }
                        return Ok(prices);
                    }
                }
                Err(e)
            }
        }
    }

    pub async fn get_history(
        &self,
        ticker: &str,
        from_ts: i64,
        to_ts: i64,
        interval: &str,
        rate_limiter: Option<&crate::services::rate_limiter::RateLimiter>,
    ) -> Result<Vec<OHLCVData>> {
        // Try default provider first
        let provider_name = self.providers[self.default_provider].get_name();
        if let Some(limiter) = rate_limiter {
            limiter.wait_if_needed(provider_name).await;
        }
        match self.providers[self.default_provider]
            .get_history(ticker, from_ts, to_ts, interval)
            .await
        {
            Ok(history) => {
                if let Some(limiter) = rate_limiter {
                    limiter.record_request(provider_name);
                }
                Ok(history)
            },
            Err(e) => {
                eprintln!("Default provider failed: {}, trying fallback", e);
                // Try other providers
                for provider in &self.providers {
                    let provider_name = provider.get_name();
                    if let Some(limiter) = rate_limiter {
                        limiter.wait_if_needed(provider_name).await;
                    }
                    if let Ok(history) = provider.get_history(ticker, from_ts, to_ts, interval).await {
                        if let Some(limiter) = rate_limiter {
                            limiter.record_request(provider_name);
                        }
                        return Ok(history);
                    }
                }
                Err(e)
            }
        }
    }
}

