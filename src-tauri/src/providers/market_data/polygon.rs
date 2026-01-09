use crate::providers::market_data::{MarketDataProvider, MarketPriceData, OHLCVData};
use anyhow::{Context, Result};
use async_trait::async_trait;

pub struct PolygonProvider {
    api_key: String,
    client: reqwest::Client,
}

impl PolygonProvider {
    pub fn new(api_key: String) -> Self {
        PolygonProvider {
            api_key,
            client: reqwest::Client::new(),
        }
    }
}

#[async_trait]
impl MarketDataProvider for PolygonProvider {
    async fn get_price(&self, ticker: &str) -> Result<MarketPriceData> {
        let url = format!(
            "https://api.polygon.io/v2/aggs/ticker/{}/prev?adjusted=true&apikey={}",
            ticker, self.api_key
        );

        let response = self.client
            .get(&url)
            .send()
            .await
            .context("Failed to send Polygon request")?;

        if !response.status().is_success() {
            anyhow::bail!("Polygon API error: {}", response.status());
        }

        let json: serde_json::Value = response.json().await
            .context("Failed to parse Polygon response")?;

        if let Some(status) = json.get("status").and_then(|v| v.as_str()) {
            if status != "OK" {
                anyhow::bail!("Polygon API error: {}", status);
            }
        }

        if let Some(results) = json.get("results").and_then(|r| r.as_array()) {
            if let Some(result) = results.first().and_then(|r| r.as_object()) {
                let close = result.get("c")
                    .and_then(|v| v.as_f64())
                    .context("Missing close price")?;

                let open = result.get("o")
                    .and_then(|v| v.as_f64())
                    .unwrap_or(close);

                let change = close - open;
                let change_percent = if open > 0.0 {
                    (change / open) * 100.0
                } else {
                    0.0
                };

                let volume = result.get("v")
                    .and_then(|v| v.as_i64())
                    .unwrap_or(0);

                let timestamp = result.get("t")
                    .and_then(|v| v.as_i64())
                    .unwrap_or(chrono::Utc::now().timestamp_millis() / 1000);

                Ok(MarketPriceData {
                    ticker: ticker.to_string(),
                    price: close,
                    change,
                    change_percent,
                    volume,
                    timestamp,
                })
            } else {
                anyhow::bail!("No results in Polygon response")
            }
        } else {
            anyhow::bail!("Invalid Polygon response format")
        }
    }

    async fn get_prices(&self, tickers: &[String]) -> Result<Vec<MarketPriceData>> {
        // Polygon allows batch requests, but we'll do individual for simplicity
        let mut results = Vec::new();
        for ticker in tickers {
            match self.get_price(ticker).await {
                Ok(price) => results.push(price),
                Err(e) => {
                    eprintln!("Failed to fetch {} from Polygon: {}", ticker, e);
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
        // Map interval to Polygon multiplier
        let (multiplier, timespan) = match interval {
            "1m" | "1M" => (1, "minute"),
            "5m" | "5M" => (5, "minute"),
            "15m" | "15M" => (15, "minute"),
            "1h" | "1H" => (1, "hour"),
            "1d" | "1D" => (1, "day"),
            "1w" | "1W" => (1, "week"),
            _ => (1, "day"),
        };

        let from_date = chrono::DateTime::from_timestamp(from_ts, 0)
            .context("Invalid from timestamp")?
            .format("%Y-%m-%d")
            .to_string();
        let to_date = chrono::DateTime::from_timestamp(to_ts, 0)
            .context("Invalid to timestamp")?
            .format("%Y-%m-%d")
            .to_string();

        let url = format!(
            "https://api.polygon.io/v2/aggs/ticker/{}/range/{}/{}/{}/{}?adjusted=true&sort=asc&apikey={}",
            ticker, multiplier, timespan, from_date, to_date, self.api_key
        );

        let response = self.client
            .get(&url)
            .send()
            .await
            .context("Failed to send Polygon history request")?;

        if !response.status().is_success() {
            anyhow::bail!("Polygon API error: {}", response.status());
        }

        let json: serde_json::Value = response.json().await
            .context("Failed to parse Polygon response")?;

        if let Some(status) = json.get("status").and_then(|v| v.as_str()) {
            if status != "OK" {
                anyhow::bail!("Polygon API error: {}", status);
            }
        }

        if let Some(results) = json.get("results").and_then(|r| r.as_array()) {
            let mut data = Vec::new();
            for result in results {
                if let Some(ohlcv) = result.as_object() {
                    let volume = ohlcv.get("v").and_then(|v| v.as_i64()).unwrap_or(0i64);
                    if let (Some(timestamp_ms), Some(open), Some(high), Some(low), Some(close)) = (
                        ohlcv.get("t").and_then(|v| v.as_i64()),
                        ohlcv.get("o").and_then(|v| v.as_f64()),
                        ohlcv.get("h").and_then(|v| v.as_f64()),
                        ohlcv.get("l").and_then(|v| v.as_f64()),
                        ohlcv.get("c").and_then(|v| v.as_f64()),
                    ) {
                        let timestamp = timestamp_ms / 1000; // Convert ms to seconds
                        if timestamp >= from_ts && timestamp <= to_ts {
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
            Ok(data)
        } else {
            anyhow::bail!("Invalid Polygon response format")
        }
    }

    fn get_name(&self) -> &str {
        "Polygon.io"
    }
}

