use crate::providers::market_data::{MarketDataProvider, MarketPriceData, OHLCVData};
use anyhow::Result;
use async_trait::async_trait;
use std::collections::HashMap;
use std::time::{Duration, Instant};

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

        anyhow::bail!("Invalid Yahoo Finance response format")
    }
}

#[async_trait]
impl MarketDataProvider for YahooFinanceProvider {
    async fn get_price(&self, ticker: &str) -> Result<MarketPriceData> {
        self.fetch_yahoo_quote(ticker).await
    }

    async fn get_prices(&self, tickers: &[String]) -> Result<Vec<MarketPriceData>> {
        let mut results = Vec::new();
        for ticker in tickers {
            match self.get_price(ticker).await {
                Ok(price) => results.push(price),
                Err(e) => {
                    eprintln!("Failed to fetch {} from Yahoo: {}", ticker, e);
                }
            }
        }
        Ok(results)
    }

    async fn get_history(
        &self,
        ticker: &str,
        from_ts: i64,
        to_ts: i64,
        interval: &str,
    ) -> Result<Vec<OHLCVData>> {
        let url = format!(
            "https://query1.finance.yahoo.com/v8/finance/chart/{}?interval={}&range=1y",
            ticker, interval
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
                    if let (Some(timestamps), Some(indicators)) = (
                        result.get("timestamp").and_then(|t| t.as_array()),
                        result.get("indicators").and_then(|i| i.as_object()),
                    ) {
                        if let Some(quote_array) = indicators.get("quote").and_then(|q| q.as_array()) {
                            if let Some(quote) = quote_array.first().and_then(|q| q.as_object()) {
                                let empty_vec: Vec<serde_json::Value> = Vec::new();
                                let opens = quote.get("open").and_then(|o| o.as_array()).unwrap_or(&empty_vec);
                                let highs = quote.get("high").and_then(|h| h.as_array()).unwrap_or(&empty_vec);
                                let lows = quote.get("low").and_then(|l| l.as_array()).unwrap_or(&empty_vec);
                                let closes = quote.get("close").and_then(|c| c.as_array()).unwrap_or(&empty_vec);
                                let volumes = quote.get("volume").and_then(|v| v.as_array()).unwrap_or(&empty_vec);

                                let mut data = Vec::new();
                                for (i, ts) in timestamps.iter().enumerate() {
                                    if let Some(timestamp) = ts.as_i64() {
                                        if timestamp >= from_ts && timestamp <= to_ts {
                                            let volume = volumes.get(i).and_then(|v| v.as_i64()).unwrap_or(0i64);
                                            if let (Some(open), Some(high), Some(low), Some(close)) = (
                                                opens.get(i).and_then(|v| v.as_f64()),
                                                highs.get(i).and_then(|v| v.as_f64()),
                                                lows.get(i).and_then(|v| v.as_f64()),
                                                closes.get(i).and_then(|v| v.as_f64()),
                                            ) {
                                                data.push(OHLCVData {
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
                                }
                                return Ok(data);
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

