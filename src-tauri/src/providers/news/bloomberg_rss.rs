use super::{NewsItem, NewsProvider, RSSProvider};
use anyhow::Result;
use async_trait::async_trait;

pub struct BloombergRSS {
    inner: RSSProvider,
}

impl BloombergRSS {
    pub fn new() -> Self {
        let urls = vec![
            "https://feeds.bloomberg.com/markets/news.rss".to_string(),
            "https://feeds.bloomberg.com/technology/news.rss".to_string(),
            "https://feeds.bloomberg.com/business/news.rss".to_string(),
            "https://feeds.bloomberg.com/stocks/news.rss".to_string(),
        ];

        BloombergRSS {
            inner: RSSProvider::new("Bloomberg", urls),
        }
    }
}

impl Default for BloombergRSS {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait]
impl NewsProvider for BloombergRSS {
    async fn fetch_news(&self, symbols: &[String], since: Option<i64>) -> Result<Vec<NewsItem>> {
        self.inner.fetch_news(symbols, since).await
    }

    fn get_name(&self) -> &str {
        self.inner.get_name()
    }

    fn get_rss_urls(&self) -> Vec<String> {
        self.inner.get_rss_urls()
    }
}

