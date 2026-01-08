use super::{NewsItem, NewsProvider};
use anyhow::{Context, Result};
use async_trait::async_trait;
use rss::Channel;
use std::io::Cursor;

pub struct RSSProvider {
    name: String,
    urls: Vec<String>,
}

impl RSSProvider {
    pub fn new(name: impl Into<String>, urls: Vec<String>) -> Self {
        RSSProvider {
            name: name.into(),
            urls,
        }
    }

    async fn fetch_feed(&self, url: &str) -> Result<Vec<NewsItem>> {
        let client = reqwest::Client::builder()
            .timeout(std::time::Duration::from_secs(15))
            .user_agent("Mozilla/5.0 (Macintosh; Intel Mac OS X 10_15_7) AppleWebKit/537.36")
            .build()?;

        let response = client.get(url).send().await?;
        
        if !response.status().is_success() {
            return Err(anyhow::anyhow!(
                "Failed to fetch RSS feed from {}: HTTP {}",
                url,
                response.status()
            ));
        }

        let content = response.text().await?;
        let channel = Channel::read_from(Cursor::new(content.as_bytes()))
            .context("Failed to parse RSS feed")?;

        let mut items = Vec::new();
        let now = chrono::Utc::now().timestamp();

        for item in channel.items() {
            let title = item.title().unwrap_or("Untitled").to_string();
            let link = item.link().unwrap_or("").to_string();
            
            // Try to get content from various RSS fields
            let description = item.description().unwrap_or("").to_string();
            
            // Check if description contains HTML (likely full content)
            let content = if description.contains("<p>") || 
                             description.contains("<div>") || 
                             description.contains("<article>") ||
                             (description.len() > 1000 && description.contains("<")) {
                description
            } else if description.len() > 500 {
                format!("<p>{}</p>", description)
            } else {
                // For short descriptions, we'll just use what we have
                if !description.is_empty() {
                    format!("<p>{}</p>", description)
                } else {
                    format!("<p>No content preview available.</p><p><a href=\"{}\" target=\"_blank\" rel=\"noopener noreferrer\">Read full article →</a></p>", link)
                }
            };

            // Parse published date
            let published_at = item.pub_date()
                .and_then(|date| {
                    chrono::DateTime::parse_from_rfc2822(date)
                        .or_else(|_| chrono::DateTime::parse_from_rfc3339(date))
                        .ok()
                })
                .map(|dt| dt.timestamp())
                .unwrap_or(now);

            items.push(NewsItem {
                title,
                content,
                url: link,
                published_at,
                source: self.name.clone(),
                source_id: None,
            });
        }

        Ok(items)
    }
}

#[async_trait]
impl NewsProvider for RSSProvider {
    async fn fetch_news(&self, _symbols: &[String], since: Option<i64>) -> Result<Vec<NewsItem>> {
        let mut all_items = Vec::new();

        for url in &self.urls {
            match self.fetch_feed(url).await {
                Ok(items) => {
                    // Filter by timestamp if provided
                    let filtered_items: Vec<NewsItem> = if let Some(since_ts) = since {
                        items.into_iter()
                            .filter(|item| item.published_at >= since_ts)
                            .collect()
                    } else {
                        items
                    };
                    
                    all_items.extend(filtered_items);
                }
                Err(e) => {
                    eprintln!("Warning: Failed to fetch RSS feed from {}: {}", url, e);
                    // Continue with other feeds even if one fails
                }
            }
        }

        Ok(all_items)
    }

    fn get_name(&self) -> &str {
        &self.name
    }

    fn get_rss_urls(&self) -> Vec<String> {
        self.urls.clone()
    }
}

