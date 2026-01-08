use super::{NewsItem, NewsProvider, RSSProvider};
use anyhow::Result;
use async_trait::async_trait;

pub struct ReutersRSS {
    inner: RSSProvider,
}

impl ReutersRSS {
    pub fn new() -> Self {
        let urls = vec![
            "https://feeds.reuters.com/reuters/businessNews".to_string(),
            "https://feeds.reuters.com/reuters/marketsNews".to_string(),
            "https://feeds.reuters.com/reuters/technologyNews".to_string(),
            "https://feeds.reuters.com/reuters/finance".to_string(),
        ];

        ReutersRSS {
            inner: RSSProvider::new("Reuters", urls),
        }
    }
}

impl Default for ReutersRSS {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait]
impl NewsProvider for ReutersRSS {
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

