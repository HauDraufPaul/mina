use crate::providers::news::{
    BloombergRSS, NewsItem, NewsProvider, OtherFinancialRSS, ReutersRSS,
    NewsAPIProvider, AlphaVantageNewsProvider, FinnhubNewsProvider,
};
use crate::services::{TickerMatcher, SentimentAnalyzer};
use crate::services::api_key_manager::APIKeyManager;
use crate::storage::{StockNewsItem, StockNewsStore};
use anyhow::Result;
use std::collections::{HashMap, HashSet};
use std::sync::{Arc, Mutex};
use tauri::{AppHandle, Emitter};

pub struct NewsAggregator {
    providers: Vec<Box<dyn NewsProvider>>,
    ticker_matcher: Arc<Mutex<TickerMatcher>>,
    store: Arc<Mutex<StockNewsStore>>,
}

impl NewsAggregator {
    pub fn new(store: Arc<Mutex<StockNewsStore>>) -> Result<Self> {
        Self::new_with_api_keys(store, None)
    }

    pub fn new_with_api_keys(store: Arc<Mutex<StockNewsStore>>, api_key_manager: Option<&APIKeyManager>) -> Result<Self> {
        let mut providers: Vec<Box<dyn NewsProvider>> = vec![
            Box::new(BloombergRSS::new()),
            Box::new(ReutersRSS::new()),
            Box::new(OtherFinancialRSS::new()),
        ];

        // Add API-based providers if API keys are available
        if let Some(key_mgr) = api_key_manager {
            providers.push(Box::new(NewsAPIProvider::new(Some(key_mgr))));
            providers.push(Box::new(AlphaVantageNewsProvider::new(Some(key_mgr))));
            providers.push(Box::new(FinnhubNewsProvider::new(Some(key_mgr))));
        }

        // Initialize ticker matcher
        let ticker_matcher = {
            let store_guard = store.lock()
                .map_err(|e| anyhow::anyhow!("Failed to lock store: {}", e))?;
            TickerMatcher::new(&store_guard)?
        };

        Ok(NewsAggregator {
            providers,
            ticker_matcher: Arc::new(Mutex::new(ticker_matcher)),
            store,
        })
    }

    /// Fetch news from all providers
    pub async fn fetch_all_news(&self, symbols: &[String], since: Option<i64>) -> Result<Vec<StockNewsItem>> {
        let mut all_items = Vec::new();

        // Fetch from all providers in parallel
        let mut futures = Vec::new();
        for provider in &self.providers {
            let provider_name = provider.get_name().to_string();
            let symbols_clone = symbols.to_vec();
            let since_clone = since;
            
            // Create a future for each provider
            let future = async move {
                match provider.fetch_news(&symbols_clone, since_clone).await {
                    Ok(items) => {
                        eprintln!("Fetched {} items from {}", items.len(), provider_name);
                        Ok(items)
                    }
                    Err(e) => {
                        eprintln!("Failed to fetch from {}: {}", provider_name, e);
                        Err(e)
                    }
                }
            };
            
            futures.push(future);
        }

        // Wait for all futures to complete
        let results = futures::future::join_all(futures).await;

        // Collect all items
        for result in results {
            if let Ok(items) = result {
                all_items.extend(items);
            }
        }

        // Deduplicate by URL
        let unique_items = self.deduplicate_news(all_items);

        // Match tickers and save to database
        let mut stock_news_items = Vec::new();
        for item in unique_items {
            if let Ok(stock_item) = self.process_news_item(item).await {
                stock_news_items.push(stock_item);
            }
        }

        Ok(stock_news_items)
    }

    /// Process a single news item: match tickers, save to DB
    async fn process_news_item(&self, item: NewsItem) -> Result<StockNewsItem> {
        // Match tickers in title and content
        let text = format!("{} {}", item.title, item.content);
        let ticker_matches = {
            let matcher = self.ticker_matcher.lock()
                .map_err(|e| anyhow::anyhow!("Failed to lock ticker_matcher: {}", e))?;
            matcher.match_tickers(&text)
        };

        // Calculate sentiment score using VADER-like analyzer
        let analyzer = SentimentAnalyzer::new();
        let sentiment_score = analyzer.analyze(&text);

        // Save to database
        let news_id = {
            let store = self.store.lock()
                .map_err(|e| anyhow::anyhow!("Failed to lock store: {}", e))?;
            store.create_news_item_with_sentiment(
                &item.title,
                &item.content,
                &item.url,
                &item.source,
                item.source_id.as_deref(),
                item.published_at,
                sentiment_score,
            )?
        };

        // Associate tickers
        let mut associated_tickers = Vec::new();
        {
            let store = self.store.lock()
                .map_err(|e| anyhow::anyhow!("Failed to lock store: {}", e))?;
            for (ticker, confidence) in &ticker_matches {
                store.associate_ticker(news_id, ticker, *confidence)?;
                associated_tickers.push(ticker.clone());
            }
        }

        // Calculate relevance score (simple: based on number of ticker matches)
        let relevance_score = if !ticker_matches.is_empty() {
            1.0
        } else {
            0.5
        };

        Ok(StockNewsItem {
            id: news_id,
            title: item.title,
            content: item.content,
            url: item.url,
            source: item.source,
            source_id: item.source_id,
            published_at: item.published_at,
            fetched_at: chrono::Utc::now().timestamp(),
            sentiment: Some(sentiment_score),
            relevance_score,
            tickers: associated_tickers,
            created_at: chrono::Utc::now().timestamp(),
        })
    }

    /// Deduplicate news items by URL
    fn deduplicate_news(&self, items: Vec<NewsItem>) -> Vec<NewsItem> {
        let mut seen_urls = HashSet::new();
        let mut unique_items = Vec::new();

        for item in items {
            if !seen_urls.contains(&item.url) {
                seen_urls.insert(item.url.clone());
                unique_items.push(item);
            }
        }

        unique_items
    }

    /// Start real-time news streaming
    pub async fn start_realtime_stream(&self, app: AppHandle) -> Result<()> {
        let store = self.store.clone();
        let ticker_matcher = self.ticker_matcher.clone();
        let providers = self.providers.iter().map(|p| p.get_name().to_string()).collect::<Vec<_>>();

        // Spawn a background task that polls RSS feeds
        tauri::async_runtime::spawn(async move {
            let mut last_fetch_time = chrono::Utc::now().timestamp();
            let poll_interval = std::time::Duration::from_secs(60); // Poll every 60 seconds

            loop {
                tokio::time::sleep(poll_interval).await;

                eprintln!("NewsAggregator: Polling RSS feeds...");

                // Fetch news since last fetch
                // Note: API key manager would need to be passed here if available
                // For now, use None - providers will skip if no key
                let aggregator = {
                    match store.lock() {
                        Ok(_guard) => {},
                        Err(e) => {
                            eprintln!("Failed to lock store: {}", e);
                            continue;
                        }
                    }
                    match NewsAggregator::new(store.clone()) {
                        Ok(agg) => agg,
                        Err(e) => {
                            eprintln!("Failed to create news aggregator: {}", e);
                            continue;
                        }
                    }
                };

                match aggregator.fetch_all_news(&[], Some(last_fetch_time)).await {
                    Ok(new_items) => {
                        eprintln!("NewsAggregator: Found {} new items", new_items.len());

                        if !new_items.is_empty() {
                            // Emit events for new news items
                            for item in &new_items {
                                let _ = app.emit(
                                    "ws-message",
                                    serde_json::json!({
                                        "type": "stock-news",
                                        "data": item,
                                        "timestamp": chrono::Utc::now().timestamp_millis()
                                    }),
                                );
                            }

                            // Also emit batch event
                            if new_items.len() <= 10 {
                                let _ = app.emit(
                                    "ws-message",
                                    serde_json::json!({
                                        "type": "stock-news-batch",
                                        "data": new_items,
                                        "timestamp": chrono::Utc::now().timestamp_millis()
                                    }),
                                );
                            }
                        }

                        last_fetch_time = chrono::Utc::now().timestamp();
                    }
                    Err(e) => {
                        eprintln!("NewsAggregator: Error fetching news: {}", e);
                    }
                }
            }
        });

        Ok(())
    }

    /// Match tickers in text (convenience method)
    pub fn match_tickers(&self, text: &str) -> Vec<(String, f64)> {
        let matcher = match self.ticker_matcher.lock() {
            Ok(guard) => guard,
            Err(e) => {
                eprintln!("Failed to lock ticker_matcher: {}", e);
                return Vec::new();
            }
        };
        matcher.match_tickers(text)
    }
}

