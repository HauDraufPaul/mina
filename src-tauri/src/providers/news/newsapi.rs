use super::{NewsItem, NewsProvider};
use crate::services::api_key_manager::APIKeyManager;
use anyhow::{Context, Result};
use async_trait::async_trait;
use serde::Deserialize;

#[derive(Debug, Deserialize)]
struct NewsAPIResponse {
    status: String,
    total_results: Option<u32>,
    articles: Vec<NewsAPIArticle>,
}

#[derive(Debug, Deserialize)]
struct NewsAPIArticle {
    title: String,
    description: Option<String>,
    content: Option<String>,
    url: String,
    published_at: String,
    source: NewsAPISource,
}

#[derive(Debug, Deserialize)]
struct NewsAPISource {
    name: String,
}

pub struct NewsAPIProvider {
    api_key: Option<String>,
    client: reqwest::Client,
}

impl NewsAPIProvider {
    pub fn new(api_key_manager: Option<&APIKeyManager>) -> Self {
        let api_key = api_key_manager
            .and_then(|mgr| mgr.get_key_optional("newsapi").ok().flatten());

        NewsAPIProvider {
            api_key,
            client: reqwest::Client::new(),
        }
    }

    async fn fetch_news_internal(
        &self,
        query: &str,
        since: Option<i64>,
    ) -> Result<Vec<NewsItem>> {
        let api_key = self.api_key.as_ref()
            .ok_or_else(|| anyhow::anyhow!("NewsAPI key not configured"))?;

        let mut url = format!(
            "https://newsapi.org/v2/everything?q={}&apiKey={}&sortBy=publishedAt&language=en",
            urlencoding::encode(query),
            api_key // API key usually doesn't need encoding
        );

        // Add date filter if provided
        if let Some(since_ts) = since {
            let since_date = chrono::DateTime::from_timestamp(since_ts, 0)
                .ok_or_else(|| anyhow::anyhow!("Invalid timestamp"))?
                .format("%Y-%m-%d");
            url.push_str(&format!("&from={}", since_date));
        }

        let response = self.client
            .get(&url)
            .send()
            .await
            .context("Failed to send NewsAPI request")?;

        if !response.status().is_success() {
            let status = response.status();
            let error_text = response.text().await.unwrap_or_default();
            anyhow::bail!("NewsAPI error: {} - {}", status, error_text);
        }

        let api_response: NewsAPIResponse = response.json().await
            .context("Failed to parse NewsAPI response")?;

        if api_response.status != "ok" {
            anyhow::bail!("NewsAPI returned error status: {:?}", api_response.status);
        }

        let mut items = Vec::new();
        for article in api_response.articles {
            let published_at = chrono::DateTime::parse_from_rfc3339(&article.published_at)
                .or_else(|_| chrono::DateTime::parse_from_rfc2822(&article.published_at))
                .map(|dt| dt.timestamp())
                .unwrap_or_else(|_| chrono::Utc::now().timestamp());

            // Filter by timestamp if provided
            if let Some(since_ts) = since {
                if published_at < since_ts {
                    continue;
                }
            }

            let content = article.content
                .or(article.description)
                .unwrap_or_else(|| article.title.clone());

            items.push(NewsItem {
                title: article.title,
                content: format!("<p>{}</p>", content),
                url: article.url,
                published_at,
                source: article.source.name,
                source_id: None,
            });
        }

        Ok(items)
    }
}

#[async_trait]
impl NewsProvider for NewsAPIProvider {
    async fn fetch_news(&self, symbols: &[String], since: Option<i64>) -> Result<Vec<NewsItem>> {
        if self.api_key.is_none() {
            return Ok(Vec::new()); // Skip if no API key
        }

        let mut all_items = Vec::new();

        if symbols.is_empty() {
            // Fetch general financial news
            let items = self.fetch_news_internal("finance OR stock OR market OR trading", since).await?;
            all_items.extend(items);
        } else {
            // Fetch news for each symbol
            for symbol in symbols {
                let query = format!("{} OR ${}", symbol, symbol);
                match self.fetch_news_internal(&query, since).await {
                    Ok(items) => all_items.extend(items),
                    Err(e) => eprintln!("NewsAPI: Failed to fetch news for {}: {}", symbol, e),
                }
            }
        }

        Ok(all_items)
    }

    fn get_name(&self) -> &str {
        "NewsAPI"
    }

    fn get_rss_urls(&self) -> Vec<String> {
        Vec::new() // Not an RSS provider
    }
}

