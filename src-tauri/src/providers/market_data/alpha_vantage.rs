use crate::providers::market_data::{MarketDataProvider, MarketPriceData, OHLCVData};
use anyhow::{Context, Result};
use async_trait::async_trait;

pub struct AlphaVantageProvider {
    api_key: String,
    client: reqwest::Client,
}

impl AlphaVantageProvider {
    pub fn new(api_key: String) -> Self {
        AlphaVantageProvider {
            api_key,
            client: reqwest::Client::new(),
        }
    }
}

#[async_trait]
impl MarketDataProvider for AlphaVantageProvider {
    async fn get_price(&self, ticker: &str) -> Result<MarketPriceData> {
        let url = format!(
            "https://www.alphavantage.co/query?function=GLOBAL_QUOTE&symbol={}&apikey={}",
            ticker, self.api_key
        );

        let response = self.client
            .get(&url)
            .send()
            .await
            .context("Failed to send Alpha Vantage request")?;

        if !response.status().is_success() {
            anyhow::bail!("Alpha Vantage API error: {}", response.status());
        }

        let json: serde_json::Value = response.json().await
            .context("Failed to parse Alpha Vantage response")?;

        // Check for API error messages
        if let Some(error_msg) = json.get("Error Message").and_then(|v| v.as_str()) {
            anyhow::bail!("Alpha Vantage API error: {}", error_msg);
        }

        if let Some(note) = json.get("Note").and_then(|v| v.as_str()) {
            anyhow::bail!("Alpha Vantage API limit: {}", note);
        }

        // Parse Global Quote
        if let Some(quote) = json.get("Global Quote").and_then(|q| q.as_object()) {
            let price = quote.get("05. price")
                .and_then(|v| v.as_str())
                .and_then(|s| s.parse::<f64>().ok())
                .context("Missing or invalid price")?;

            let previous_close = quote.get("08. previous close")
                .and_then(|v| v.as_str())
                .and_then(|s| s.parse::<f64>().ok())
                .unwrap_or(price);

            let change = price - previous_close;
            let change_percent = if previous_close > 0.0 {
                (change / previous_close) * 100.0
            } else {
                0.0
            };

            let volume = quote.get("06. volume")
                .and_then(|v| v.as_str())
                .and_then(|s| s.parse::<i64>().ok())
                .unwrap_or(0);

            Ok(MarketPriceData {
                ticker: ticker.to_string(),
                price,
                change,
                change_percent,
                volume,
                timestamp: chrono::Utc::now().timestamp(),
            })
        } else {
            anyhow::bail!("Invalid Alpha Vantage response format")
        }
    }

    async fn get_prices(&self, tickers: &[String]) -> Result<Vec<MarketPriceData>> {
        // Alpha Vantage free tier only allows 5 calls per minute
        // So we'll fetch sequentially with delays
        let mut results = Vec::new();
        for ticker in tickers {
            match self.get_price(ticker).await {
                Ok(price) => results.push(price),
                Err(e) => {
                    eprintln!("Failed to fetch {} from Alpha Vantage: {}", ticker, e);
                }
            }
            // Rate limit: 1 request per 12 seconds (5 per minute)
            if tickers.len() > 1 {
                tokio::time::sleep(tokio::time::Duration::from_secs(12)).await;
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
        // Map interval to Alpha Vantage function
        let function = match interval {
            "1d" | "1D" => "TIME_SERIES_DAILY",
            "1w" | "1W" => "TIME_SERIES_WEEKLY",
            "1m" | "1M" => "TIME_SERIES_MONTHLY",
            _ => "TIME_SERIES_INTRADAY",
        };

        let mut url = format!(
            "https://www.alphavantage.co/query?function={}&symbol={}&apikey={}",
            function, ticker, self.api_key
        );

        if interval.starts_with("1") && interval.len() > 1 {
            if let Ok(minutes) = interval[1..].parse::<u32>() {
                url.push_str(&format!("&interval={}min", minutes));
            }
        }

        let response = self.client
            .get(&url)
            .send()
            .await
            .context("Failed to send Alpha Vantage history request")?;

        if !response.status().is_success() {
            anyhow::bail!("Alpha Vantage API error: {}", response.status());
        }

        let json: serde_json::Value = response.json().await
            .context("Failed to parse Alpha Vantage response")?;

        // Check for errors
        if let Some(error_msg) = json.get("Error Message").and_then(|v| v.as_str()) {
            anyhow::bail!("Alpha Vantage API error: {}", error_msg);
        }

        // Parse time series data
        let time_series_key = if function == "TIME_SERIES_INTRADAY" {
            format!("Time Series ({})", if interval.contains("min") { 
                format!("{}min", interval.chars().skip(1).collect::<String>())
            } else { "1min".to_string() })
        } else if function == "TIME_SERIES_DAILY" {
            "Time Series (Daily)".to_string()
        } else if function == "TIME_SERIES_WEEKLY" {
            "Weekly Time Series".to_string()
        } else {
            "Monthly Time Series".to_string()
        };

        if let Some(series) = json.get(&time_series_key).and_then(|s| s.as_object()) {
            let mut data = Vec::new();
            for (date_str, values) in series {
                if let Some(ohlcv) = values.as_object() {
                    // Parse date string (format: YYYY-MM-DD)
                    if let Ok(naive_date) = chrono::NaiveDate::parse_from_str(date_str, "%Y-%m-%d") {
                        if let Some(naive_datetime) = naive_date.and_hms_opt(0, 0, 0) {
                            let ts = naive_datetime.and_utc().timestamp();
                            if ts >= from_ts && ts <= to_ts {
                                let volume = ohlcv.get("5. volume")
                                    .and_then(|v| v.as_str())
                                    .and_then(|s| s.parse::<i64>().ok())
                                    .unwrap_or(0);
                                if let (Some(open), Some(high), Some(low), Some(close)) = (
                                    ohlcv.get("1. open").and_then(|v| v.as_str()).and_then(|s| s.parse::<f64>().ok()),
                                    ohlcv.get("2. high").and_then(|v| v.as_str()).and_then(|s| s.parse::<f64>().ok()),
                                    ohlcv.get("3. low").and_then(|v| v.as_str()).and_then(|s| s.parse::<f64>().ok()),
                                    ohlcv.get("4. close").and_then(|v| v.as_str()).and_then(|s| s.parse::<f64>().ok()),
                                ) {
                                    data.push(OHLCVData {
                                        timestamp: ts,
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
                }
            }
            data.sort_by_key(|d| d.timestamp);
            Ok(data)
        } else {
            anyhow::bail!("Invalid Alpha Vantage time series format")
        }
    }

    fn get_name(&self) -> &str {
        "Alpha Vantage"
    }
}

