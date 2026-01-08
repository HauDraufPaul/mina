use super::{NewsItem, NewsProvider, RSSProvider};
use anyhow::Result;
use async_trait::async_trait;

pub struct OtherFinancialRSS {
    inner: RSSProvider,
}

impl OtherFinancialRSS {
    pub fn new() -> Self {
        let urls = vec![
            "https://feeds.marketwatch.com/marketwatch/topstories/".to_string(),
            "https://finance.yahoo.com/news/rssindex".to_string(),
            "https://www.cnbc.com/id/100003114/device/rss/rss.html".to_string(),
            "https://feeds.a.dj.com/rss/RSSMarketsMain.xml".to_string(),
            "https://seekingalpha.com/feed.xml".to_string(),
            "https://financialpost.com/feed".to_string(),
            "https://www.barrons.com/feed".to_string(),
            "https://www.investopedia.com/feedbuilder/feed/getFeed?feedName=rss_headline".to_string(),
        ];

        OtherFinancialRSS {
            inner: RSSProvider::new("Other Financial News", urls),
        }
    }
}

impl Default for OtherFinancialRSS {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait]
impl NewsProvider for OtherFinancialRSS {
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

