use crate::storage::{Database, StockNewsStore, TemporalStore};
use anyhow::Result;
use serde::{Deserialize, Serialize};
use std::sync::Mutex;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SearchResult {
    pub id: String,
    pub title: String,
    pub snippet: String,
    pub source: SearchSource,
    pub relevance: f64,
    pub metadata: serde_json::Value,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum SearchSource {
    TemporalEvent,
    News,
    Vector,
    Portfolio,
    Alert,
    Watchlist,
    Command,
}

pub struct GlobalSearchService;

impl GlobalSearchService {
    /// Perform global search across all data sources
    pub fn search(
        query: &str,
        limit: Option<i32>,
        db: &Mutex<Database>,
    ) -> Result<Vec<SearchResult>> {
        let limit = limit.unwrap_or(20).min(100);
        let mut all_results = Vec::new();

        // 1. Search temporal events
        if let Ok(results) = Self::search_temporal_events(query, limit, db) {
            all_results.extend(results);
        }

        // 2. Search news
        if let Ok(results) = Self::search_news(query, limit, db) {
            all_results.extend(results);
        }

        // 3. Search vectors (semantic search)
        if let Ok(results) = Self::search_vectors(query, limit, db) {
            all_results.extend(results);
        }

        // 4. Search portfolios
        if let Ok(results) = Self::search_portfolios(query, limit, db) {
            all_results.extend(results);
        }

        // 5. Search alerts
        if let Ok(results) = Self::search_alerts(query, limit, db) {
            all_results.extend(results);
        }

        // 6. Search watchlists
        if let Ok(results) = Self::search_watchlists(query, limit, db) {
            all_results.extend(results);
        }

        // Sort by relevance and limit
        all_results.sort_by(|a, b| b.relevance.partial_cmp(&a.relevance)
            .unwrap_or(std::cmp::Ordering::Equal));
        all_results.truncate(limit as usize);

        Ok(all_results)
    }

    fn search_temporal_events(
        query: &str,
        limit: i32,
        db: &Mutex<Database>,
    ) -> Result<Vec<SearchResult>> {
        let db_guard = db.lock().map_err(|e| anyhow::anyhow!("Database lock error: {}", e))?;
        let store = TemporalStore::new(db_guard.conn.clone());
        
        let results = store.search(query, limit as i64)?;
        
        Ok(results.into_iter().map(|r| {
            let title = r.get("title").and_then(|v| v.as_str()).unwrap_or("Event").to_string();
            let snippet = r.get("snippet").and_then(|v| v.as_str()).unwrap_or("").to_string();
            let doc_id = r.get("doc_id").and_then(|v| v.as_i64()).unwrap_or(0);
            
            SearchResult {
                id: format!("temporal:{}", doc_id),
                title,
                snippet,
                source: SearchSource::TemporalEvent,
                relevance: 0.8, // Temporal search already has relevance scoring
                metadata: r,
            }
        }).collect())
    }

    fn search_news(
        query: &str,
        limit: i32,
        db: &Mutex<Database>,
    ) -> Result<Vec<SearchResult>> {
        let db_guard = db.lock().map_err(|e| anyhow::anyhow!("Database lock error: {}", e))?;
        let store = StockNewsStore::new(db_guard.conn.clone());
        
        let results = store.search_news(query, None, limit)?;
        
        Ok(results.into_iter().map(|item| {
            let snippet = if item.content.len() > 200 {
                format!("{}...", &item.content[..200])
            } else {
                item.content.clone()
            };
            
            SearchResult {
                id: format!("news:{}", item.id),
                title: item.title,
                snippet,
                source: SearchSource::News,
                relevance: item.relevance_score,
                metadata: serde_json::json!({
                    "url": item.url,
                    "source": item.source,
                    "published_at": item.published_at,
                    "tickers": item.tickers,
                }),
            }
        }).collect())
    }

    fn search_vectors(
        query: &str,
        limit: i32,
        db: &Mutex<Database>,
    ) -> Result<Vec<SearchResult>> {
        // Note: Vector search requires embedding generation
        // For now, return empty - would need to integrate with embedding service
        Ok(Vec::new())
    }

    fn search_portfolios(
        query: &str,
        limit: i32,
        db: &Mutex<Database>,
    ) -> Result<Vec<SearchResult>> {
        use crate::storage::portfolio::PortfolioStore;
        
        let db_guard = db.lock().map_err(|e| anyhow::anyhow!("Database lock error: {}", e))?;
        let store = PortfolioStore::new(db_guard.conn.clone());
        
        let portfolios = store.list_portfolios()?;
        let query_lower = query.to_lowercase();
        
        let results: Vec<SearchResult> = portfolios
            .into_iter()
            .filter(|p| p.name.to_lowercase().contains(&query_lower))
            .take(limit as usize)
            .map(|p| {
                SearchResult {
                    id: format!("portfolio:{}", p.id),
                    title: p.name.clone(),
                    snippet: format!("Portfolio created on {}", 
                        chrono::DateTime::from_timestamp(p.created_at, 0)
                            .map(|dt| dt.format("%Y-%m-%d").to_string())
                            .unwrap_or_else(|| "unknown".to_string())),
                    source: SearchSource::Portfolio,
                    relevance: 0.7,
                    metadata: serde_json::json!({
                        "id": p.id,
                        "created_at": p.created_at,
                    }),
                }
            })
            .collect();
        
        Ok(results)
    }

    fn search_alerts(
        query: &str,
        limit: i32,
        db: &Mutex<Database>,
    ) -> Result<Vec<SearchResult>> {
        let db_guard = db.lock().map_err(|e| anyhow::anyhow!("Database lock error: {}", e))?;
        let store = TemporalStore::new(db_guard.conn.clone());
        
        let alerts = store.list_alerts(limit as i64, None, None)?;
        let query_lower = query.to_lowercase();
        
        let results: Vec<SearchResult> = alerts
            .into_iter()
            .filter_map(|alert| {
                let payload_str = serde_json::to_string(&alert.payload_json).ok()?;
                if payload_str.to_lowercase().contains(&query_lower) {
                    let title = alert.payload_json.get("title")
                        .and_then(|v| v.as_str())
                        .unwrap_or("Alert")
                        .to_string();
                    let snippet = alert.payload_json.get("message")
                        .and_then(|v| v.as_str())
                        .unwrap_or("")
                        .to_string();
                    
                    Some(SearchResult {
                        id: format!("alert:{}", alert.id),
                        title,
                        snippet,
                        source: SearchSource::Alert,
                        relevance: 0.75,
                        metadata: serde_json::json!({
                            "id": alert.id,
                            "status": alert.status,
                            "fired_at": alert.fired_at,
                        }),
                    })
                } else {
                    None
                }
            })
            .collect();
        
        Ok(results)
    }

    fn search_watchlists(
        query: &str,
        limit: i32,
        db: &Mutex<Database>,
    ) -> Result<Vec<SearchResult>> {
        let db_guard = db.lock().map_err(|e| anyhow::anyhow!("Database lock error: {}", e))?;
        let store = TemporalStore::new(db_guard.conn.clone());
        
        let watchlists = store.list_watchlists()?;
        let query_lower = query.to_lowercase();
        
        let results: Vec<SearchResult> = watchlists
            .into_iter()
            .filter(|w| w.name.to_lowercase().contains(&query_lower))
            .take(limit as usize)
            .map(|w| {
                // Get item count
                let items = store.list_watchlist_items(w.id).unwrap_or_default();
                let item_count = items.len();
                
                SearchResult {
                    id: format!("watchlist:{}", w.id),
                    title: w.name.clone(),
                    snippet: format!("Watchlist with {} items", item_count),
                    source: SearchSource::Watchlist,
                    relevance: 0.7,
                    metadata: serde_json::json!({
                        "id": w.id,
                        "item_count": item_count,
                    }),
                }
            })
            .collect();
        
        Ok(results)
    }
}

