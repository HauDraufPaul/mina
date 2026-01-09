use super::{NewsItem, NewsProvider};
use crate::services::api_key_manager::APIKeyManager;
use anyhow::{Context, Result};
use async_trait::async_trait;
use serde::Deserialize;

#[derive(Debug, Deserialize)]
struct AlphaVantageResponse {
    #[serde(rename = "feed")]
    feed: Vec<AlphaVantageArticle>,
}

#[derive(Debug, Deserialize)]
struct AlphaVantageArticle {
    title: String,
    url: String,
    #[serde(rename = "time_published")]
    time_published: String,
    authors: Option<String>,
    summary: String,
    #[serde(rename = "source")]
    source: String,
    #[serde(rename = "category_within_source")]
    category: Option<String>,
    #[serde(rename = "source_domain")]
    source_domain: Option<String>,
    #[serde(rename = "topics")]
    topics: Option<Vec<AlphaVantageTopic>>,
    #[serde(rename = "overall_sentiment_score")]
    sentiment_score: Option<f64>,
    #[serde(rename = "overall_sentiment_label")]
    sentiment_label: Option<String>,
}

#[derive(Debug, Deserialize)]
struct AlphaVantageTopic {
    topic: String,
    #[serde(rename = "relevance_score")]
    relevance_score: Option<String>,
}

pub struct AlphaVantageNewsProvider {
    api_key: Option<String>,
    client: reqwest::Client,
}

impl AlphaVantageNewsProvider {
    pub fn new(api_key_manager: Option<&APIKeyManager>) -> Self {
        let api_key = api_key_manager
            .and_then(|mgr| mgr.get_key_optional("alpha_vantage").ok().flatten());

        AlphaVantageNewsProvider {
            api_key,
            client: reqwest::Client::new(),
        }
    }

    async fn fetch_news_internal(
        &self,
        tickers: &[String],
        since: Option<i64>,
    ) -> Result<Vec<NewsItem>> {
        let api_key = self.api_key.as_ref()
            .ok_or_else(|| anyhow::anyhow!("Alpha Vantage API key not configured"))?;

        // Build tickers parameter
        let tickers_param = if tickers.is_empty() {
            "TOP_GAINERS,TOP_LOSERS".to_string() // Default to market movers
        } else {
            tickers.join(",")
        };

        let url = format!(
            "https://www.alphavantage.co/query?function=NEWS_SENTIMENT&tickers={}&apikey={}&limit=1000",
            urlencoding::encode(&tickers_param),
            api_key // API key usually doesn't need encoding
        );

        let response = self.client
            .get(&url)
            .send()
            .await
            .context("Failed to send Alpha Vantage request")?;

        if !response.status().is_success() {
            let status = response.status();
            let error_text = response.text().await.unwrap_or_default();
            anyhow::bail!("Alpha Vantage error: {} - {}", status, error_text);
        }

        let json: serde_json::Value = response.json().await
            .context("Failed to parse Alpha Vantage response")?;

        // Check for API error messages
        if let Some(error_msg) = json.get("Error Message").and_then(|v| v.as_str()) {
            anyhow::bail!("Alpha Vantage API error: {}", error_msg);
        }

        if let Some(note) = json.get("Note").and_then(|v| v.as_str()) {
            eprintln!("Alpha Vantage note: {}", note);
            return Ok(Vec::new()); // Rate limit or similar
        }

        let api_response: AlphaVantageResponse = serde_json::from_value(json)
            .context("Failed to parse Alpha Vantage response structure")?;

        let mut items = Vec::new();
        for article in api_response.feed {
            // Parse time_published (format: YYYYMMDDTHHMMSS)
            let published_at = if article.time_published.len() >= 8 {
                let date_str = &article.time_published[0..8];
                let time_str = if article.time_published.len() >= 15 {
                    &article.time_published[9..15]
                } else {
                    "000000"
                };
                
                format!("{}-{}-{}T{}:{}:{}Z",
                    &date_str[0..4], &date_str[4..6], &date_str[6..8],
                    &time_str[0..2], &time_str[2..4], &time_str[4..6]
                )
                .parse::<chrono::DateTime<chrono::Utc>>()
                .map(|dt| dt.timestamp())
                .unwrap_or_else(|_| chrono::Utc::now().timestamp())
            } else {
                chrono::Utc::now().timestamp()
            };

            // Filter by timestamp if provided
            if let Some(since_ts) = since {
                if published_at < since_ts {
                    continue;
                }
            }

            let content = if !article.summary.is_empty() {
                article.summary
            } else {
                article.title.clone()
            };

            items.push(NewsItem {
                title: article.title,
                content: format!("<p>{}</p>", content),
                url: article.url,
                published_at,
                source: article.source,
                source_id: article.source_domain,
            });
        }

        Ok(items)
    }
}

#[async_trait]
impl NewsProvider for AlphaVantageNewsProvider {
    async fn fetch_news(&self, symbols: &[String], since: Option<i64>) -> Result<Vec<NewsItem>> {
        if self.api_key.is_none() {
            return Ok(Vec::new()); // Skip if no API key
        }

        self.fetch_news_internal(symbols, since).await
    }

    fn get_name(&self) -> &str {
        "Alpha Vantage News"
    }

    fn get_rss_urls(&self) -> Vec<String> {
        Vec::new() // Not an RSS provider
    }
}

