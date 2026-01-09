use anyhow::Result;
use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::time::{Duration, Instant};

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

pub struct YahooFinanceProvider {
    client: reqwest::Client,
    rate_limiter: HashMap<String, Instant>,
}

impl YahooFinanceProvider {
    pub fn new() -> Self {
        YahooFinanceProvider {
            client: reqwest::Client::new(),
            rate_limiter: HashMap::new(),
        }
    }

    fn check_rate_limit(&mut self, key: &str, min_interval_ms: u64) -> bool {
        let now = Instant::now();
        if let Some(last_call) = self.rate_limiter.get(key) {
            if now.duration_since(*last_call) < Duration::from_millis(min_interval_ms) {
                return false;
            }
        }
        self.rate_limiter.insert(key.to_string(), now);
        true
    }

    async fn fetch_yahoo_quote(&self, ticker: &str) -> Result<MarketPriceData> {
        // Yahoo Finance API endpoint (using yahoo-finance-api or similar)
        // For now, we'll use a simple approach with yahoo finance quote endpoint
        let url = format!("https://query1.finance.yahoo.com/v8/finance/chart/{}", ticker);
        
        let response = self.client
            .get(&url)
            .header("User-Agent", "Mozilla/5.0")
            .send()
            .await?;

        if !response.status().is_success() {
            anyhow::bail!("Failed to fetch data: {}", response.status());
        }

        let json: serde_json::Value = response.json().await?;
        
        // Parse Yahoo Finance response
        if let Some(chart) = json.get("chart") {
            if let Some(result_array) = chart.get("result").and_then(|r| r.as_array()) {
                if let Some(result) = result_array.first() {
                    if let Some(meta) = result.get("meta") {
                        if let (Some(regular_market_price), Some(previous_close), Some(timestamp)) = (
                            meta.get("regularMarketPrice").and_then(|v| v.as_f64()),
                            meta.get("previousClose").and_then(|v| v.as_f64()),
                            meta.get("regularMarketTime").and_then(|v| v.as_i64()),
                        ) {
                            let change = regular_market_price - previous_close;
                            let change_percent = (change / previous_close) * 100.0;
                            let volume = meta.get("regularMarketVolume")
                                .and_then(|v| v.as_i64())
                                .unwrap_or(0);

                            return Ok(MarketPriceData {
                                ticker: ticker.to_string(),
                                price: regular_market_price,
                                change,
                                change_percent,
                                volume,
                                timestamp,
                            });
                        }
                    }
                }
            }
        }

        anyhow::bail!("Invalid response format")
    }
}

#[async_trait]
impl MarketDataProvider for YahooFinanceProvider {
    async fn get_price(&self, ticker: &str) -> Result<MarketPriceData> {
        self.fetch_yahoo_quote(ticker).await
    }

    async fn get_prices(&self, tickers: &[String]) -> Result<Vec<MarketPriceData>> {
        let mut prices = Vec::new();
        for ticker in tickers {
            match self.get_price(ticker).await {
                Ok(price) => prices.push(price),
                Err(e) => eprintln!("Failed to fetch price for {}: {}", ticker, e),
            }
            // Small delay to avoid rate limiting
            tokio::time::sleep(Duration::from_millis(100)).await;
        }
        Ok(prices)
    }

    async fn get_history(
        &self,
        ticker: &str,
        from_ts: i64,
        to_ts: i64,
        interval: &str,
    ) -> Result<Vec<OHLCVData>> {
        // Map interval to Yahoo Finance interval
        let yahoo_interval = match interval {
            "1m" => "1m",
            "5m" => "5m",
            "15m" => "15m",
            "1h" => "1h",
            "1d" => "1d",
            _ => "1d",
        };

        let url = format!(
            "https://query1.finance.yahoo.com/v8/finance/chart/{}?period1={}&period2={}&interval={}",
            ticker, from_ts, to_ts, yahoo_interval
        );

        let response = self.client
            .get(&url)
            .header("User-Agent", "Mozilla/5.0")
            .send()
            .await?;

        if !response.status().is_success() {
            anyhow::bail!("Failed to fetch history: {}", response.status());
        }

        let json: serde_json::Value = response.json().await?;

        if let Some(chart) = json.get("chart") {
            if let Some(result_array) = chart.get("result").and_then(|r| r.as_array()) {
                if let Some(result) = result_array.first() {
                    if let (Some(timestamps_array), Some(indicators_obj)) = (
                        result.get("timestamp").and_then(|t| t.as_array()),
                        result.get("indicators"),
                    ) {
                        if let Some(quote_array) = indicators_obj.get("quote").and_then(|q| q.as_array()) {
                            if let Some(quote) = quote_array.first() {
                                let opens = quote.get("open").and_then(|o| o.as_array());
                                let highs = quote.get("high").and_then(|h| h.as_array());
                                let lows = quote.get("low").and_then(|l| l.as_array());
                                let closes = quote.get("close").and_then(|c| c.as_array());
                                let volumes = quote.get("volume").and_then(|v| v.as_array());

                                if let (Some(opens), Some(highs), Some(lows), Some(closes), Some(volumes)) = 
                                    (opens, highs, lows, closes, volumes) {
                                    let mut history = Vec::new();
                                    for (i, ts) in timestamps_array.iter().enumerate() {
                                        if let Some(timestamp) = ts.as_i64() {
                                            if let (Some(open), Some(high), Some(low), Some(close)) = (
                                                opens.get(i).and_then(|v| v.as_f64()),
                                                highs.get(i).and_then(|v| v.as_f64()),
                                                lows.get(i).and_then(|v| v.as_f64()),
                                                closes.get(i).and_then(|v| v.as_f64()),
                                            ) {
                                                let volume = volumes.get(i)
                                                    .and_then(|v| v.as_i64())
                                                    .unwrap_or(0);
                                                
                                                history.push(OHLCVData {
                                                    timestamp,
                                                    open,
                                                    high,
                                                    low,
                                                    close,
                                                    volume,
                                                });
                                            }
                                        }
                                    }

                                    return Ok(history);
                                }
                            }
                        }
                    }
                }
            }
        }

        anyhow::bail!("Invalid history response format")
    }

    fn get_name(&self) -> &str {
        "Yahoo Finance"
    }
}

pub struct MarketDataManager {
    providers: Vec<Box<dyn MarketDataProvider>>,
    default_provider: usize,
}

impl MarketDataManager {
    pub fn new() -> Self {
        let mut providers: Vec<Box<dyn MarketDataProvider>> = Vec::new();
        providers.push(Box::new(YahooFinanceProvider::new()));
        
        MarketDataManager {
            providers,
            default_provider: 0,
        }
    }

    pub async fn get_price(&self, ticker: &str) -> Result<MarketPriceData> {
        // Try default provider first
        match self.providers[self.default_provider].get_price(ticker).await {
            Ok(price) => Ok(price),
            Err(e) => {
                eprintln!("Default provider failed: {}, trying fallback", e);
                // Try other providers
                for provider in &self.providers {
                    if let Ok(price) = provider.get_price(ticker).await {
                        return Ok(price);
                    }
                }
                Err(e)
            }
        }
    }

    pub async fn get_prices(&self, tickers: &[String]) -> Result<Vec<MarketPriceData>> {
        self.providers[self.default_provider].get_prices(tickers).await
    }

    pub async fn get_history(
        &self,
        ticker: &str,
        from_ts: i64,
        to_ts: i64,
        interval: &str,
    ) -> Result<Vec<OHLCVData>> {
        self.providers[self.default_provider]
            .get_history(ticker, from_ts, to_ts, interval)
            .await
    }
}
