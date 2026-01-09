use super::{NewsItem, NewsProvider};
use crate::services::api_key_manager::APIKeyManager;
use anyhow::{Context, Result};
use async_trait::async_trait;
use serde::Deserialize;

#[derive(Debug, Deserialize)]
struct FinnhubResponse {
    #[serde(default)]
    result: Vec<FinnhubArticle>,
}

#[derive(Debug, Deserialize)]
struct FinnhubArticle {
    category: String,
    datetime: i64,
    headline: String,
    id: i64,
    image: Option<String>,
    related: Option<String>,
    source: String,
    summary: String,
    url: String,
}

pub struct FinnhubNewsProvider {
    api_key: Option<String>,
    client: reqwest::Client,
}

impl FinnhubNewsProvider {
    pub fn new(api_key_manager: Option<&APIKeyManager>) -> Self {
        let api_key = api_key_manager
            .and_then(|mgr| mgr.get_key_optional("finnhub").ok().flatten());

        FinnhubNewsProvider {
            api_key,
            client: reqwest::Client::new(),
        }
    }

    async fn fetch_news_internal(
        &self,
        symbol: Option<&str>,
        since: Option<i64>,
    ) -> Result<Vec<NewsItem>> {
        let api_key = self.api_key.as_ref()
            .ok_or_else(|| anyhow::anyhow!("Finnhub API key not configured"))?;

        let url = if let Some(sym) = symbol {
            format!(
                "https://finnhub.io/api/v1/company-news?symbol={}&from={}&to={}&token={}",
                sym,
                if let Some(since_ts) = since {
                    chrono::DateTime::from_timestamp(since_ts, 0)
                        .map(|dt| dt.format("%Y-%m-%d").to_string())
                        .unwrap_or_else(|| chrono::Utc::now().format("%Y-%m-%d").to_string())
                } else {
                    (chrono::Utc::now() - chrono::Duration::days(7))
                        .format("%Y-%m-%d")
                        .to_string()
                },
                chrono::Utc::now().format("%Y-%m-%d").to_string(),
                api_key
            )
        } else {
            format!(
                "https://finnhub.io/api/v1/news?category=general&token={}",
                api_key
            )
        };

        let response = self.client
            .get(&url)
            .send()
            .await
            .context("Failed to send Finnhub request")?;

        if !response.status().is_success() {
            let status = response.status();
            let error_text = response.text().await.unwrap_or_default();
            anyhow::bail!("Finnhub error: {} - {}", status, error_text);
        }

        let articles: Vec<FinnhubArticle> = response.json().await
            .context("Failed to parse Finnhub response")?;

        let mut items = Vec::new();
        for article in articles {
            // Filter by timestamp if provided
            if let Some(since_ts) = since {
                if article.datetime < since_ts {
                    continue;
                }
            }

            let content = if !article.summary.is_empty() {
                article.summary
            } else {
                article.headline.clone()
            };

            items.push(NewsItem {
                title: article.headline,
                content: format!("<p>{}</p>", content),
                url: article.url,
                published_at: article.datetime,
                source: article.source,
                source_id: Some(article.id.to_string()),
            });
        }

        Ok(items)
    }
}

#[async_trait]
impl NewsProvider for FinnhubNewsProvider {
    async fn fetch_news(&self, symbols: &[String], since: Option<i64>) -> Result<Vec<NewsItem>> {
        if self.api_key.is_none() {
            return Ok(Vec::new()); // Skip if no API key
        }

        let mut all_items = Vec::new();

        if symbols.is_empty() {
            // Fetch general market news
            match self.fetch_news_internal(None, since).await {
                Ok(items) => all_items.extend(items),
                Err(e) => eprintln!("Finnhub: Failed to fetch general news: {}", e),
            }
        } else {
            // Fetch news for each symbol
            for symbol in symbols {
                match self.fetch_news_internal(Some(symbol), since).await {
                    Ok(items) => all_items.extend(items),
                    Err(e) => eprintln!("Finnhub: Failed to fetch news for {}: {}", symbol, e),
                }
            }
        }

        Ok(all_items)
    }

    fn get_name(&self) -> &str {
        "Finnhub"
    }

    fn get_rss_urls(&self) -> Vec<String> {
        Vec::new() // Not an RSS provider
    }
}

