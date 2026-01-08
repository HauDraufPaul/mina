use anyhow::Result;
use async_trait::async_trait;
use serde::{Deserialize, Serialize};

pub mod rss_provider;
pub mod bloomberg_rss;
pub mod reuters_rss;
pub mod other_rss;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NewsItem {
    pub title: String,
    pub content: String,
    pub url: String,
    pub published_at: i64,
    pub source: String,
    pub source_id: Option<String>,
}

#[async_trait]
pub trait NewsProvider: Send + Sync {
    async fn fetch_news(&self, symbols: &[String], since: Option<i64>) -> Result<Vec<NewsItem>>;
    fn get_name(&self) -> &str;
    fn get_rss_urls(&self) -> Vec<String>;
}

// Re-export providers
pub use rss_provider::RSSProvider;
pub use bloomberg_rss::BloombergRSS;
pub use reuters_rss::ReutersRSS;
pub use other_rss::OtherFinancialRSS;

