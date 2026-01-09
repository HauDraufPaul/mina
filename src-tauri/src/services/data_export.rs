use crate::storage::{
    Database, StockNewsStore, TemporalStore, PortfolioStore, MarketDataStore,
    EconomicCalendarStore,
};
use anyhow::Result;
use serde::{Deserialize, Serialize};
use std::sync::Mutex;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ExportFormat {
    Csv,
    Json,
    Excel, // Placeholder for future Excel export
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ExportDataType {
    Portfolio,
    MarketData,
    News,
    Alerts,
    EconomicCalendar,
    TemporalEvents,
    PriceAlerts,
}

pub struct DataExportService;

impl DataExportService {
    /// Export data to CSV format
    pub fn export_to_csv(data: &[serde_json::Value], headers: &[&str]) -> String {
        let mut csv = String::new();
        
        // Write headers
        csv.push_str(&headers.join(","));
        csv.push('\n');
        
        // Write rows
        for row in data {
            let values: Vec<String> = headers.iter()
                .map(|header| {
                    row.get(header)
                        .map(|v| {
                            match v {
                                serde_json::Value::String(s) => format!("\"{}\"", s.replace("\"", "\"\"")),
                                serde_json::Value::Number(n) => n.to_string(),
                                serde_json::Value::Bool(b) => b.to_string(),
                                serde_json::Value::Null => String::new(),
                                _ => v.to_string(),
                            }
                        })
                        .unwrap_or_else(String::new)
                })
                .collect();
            csv.push_str(&values.join(","));
            csv.push('\n');
        }
        
        csv
    }

    /// Export portfolio data
    pub fn export_portfolio(
        portfolio_id: i64,
        format: ExportFormat,
        db: &Mutex<Database>,
    ) -> Result<String> {
        let db_guard = db.lock().map_err(|e| anyhow::anyhow!("Database lock error: {}", e))?;
        let store = PortfolioStore::new(db_guard.conn.clone());
        
        let holdings = store.list_holdings(portfolio_id)?;
        let transactions = store.list_transactions(portfolio_id, None)?;
        
        match format {
            ExportFormat::Csv => {
                let mut csv = String::new();
                
                // Export holdings
                csv.push_str("Type,ID,Ticker,Quantity,Purchase Price,Purchase Date\n");
                for holding in &holdings {
                    csv.push_str(&format!(
                        "Holding,{},{},{},{},{}\n",
                        holding.id,
                        holding.ticker,
                        holding.quantity,
                        holding.purchase_price,
                        chrono::DateTime::from_timestamp(holding.purchase_date, 0)
                            .map(|dt| dt.format("%Y-%m-%d").to_string())
                            .unwrap_or_else(|| "unknown".to_string())
                    ));
                }
                
                // Export transactions
                for transaction in &transactions {
                    csv.push_str(&format!(
                        "Transaction,{},{},{},{},{},{},{}\n",
                        transaction.id,
                        transaction.ticker,
                        transaction.transaction_type,
                        transaction.quantity,
                        transaction.price,
                        transaction.fees,
                        chrono::DateTime::from_timestamp(transaction.transaction_date, 0)
                            .map(|dt| dt.format("%Y-%m-%d %H:%M:%S").to_string())
                            .unwrap_or_else(|| "unknown".to_string())
                    ));
                }
                
                Ok(csv)
            }
            ExportFormat::Json => {
                let json = serde_json::json!({
                    "portfolio_id": portfolio_id,
                    "holdings": holdings,
                    "transactions": transactions,
                    "exported_at": chrono::Utc::now().timestamp(),
                });
                Ok(serde_json::to_string_pretty(&json)?)
            }
            ExportFormat::Excel => {
                // Excel export would require additional dependencies
                anyhow::bail!("Excel export not yet implemented")
            }
        }
    }

    /// Export market data
    pub fn export_market_data(
        tickers: &[String],
        from_ts: Option<i64>,
        to_ts: Option<i64>,
        format: ExportFormat,
        db: &Mutex<Database>,
    ) -> Result<String> {
        let db_guard = db.lock().map_err(|e| anyhow::anyhow!("Database lock error: {}", e))?;
        let store = MarketDataStore::new(db_guard.conn.clone());
        
        let mut all_data = Vec::new();
        
        for ticker in tickers {
            let history = store.get_price_history(
                ticker,
                from_ts.unwrap_or(0),
                to_ts.unwrap_or(i64::MAX),
                Some(10000),
            )?;
            
            for h in history {
                all_data.push(serde_json::json!({
                    "ticker": ticker,
                    "timestamp": h.timestamp,
                    "open": h.open,
                    "high": h.high,
                    "low": h.low,
                    "close": h.close,
                    "volume": h.volume,
                }));
            }
        }
        
        match format {
            ExportFormat::Csv => {
                let headers = vec!["ticker", "timestamp", "open", "high", "low", "close", "volume"];
                Ok(Self::export_to_csv(&all_data, &headers))
            }
            ExportFormat::Json => {
                Ok(serde_json::to_string_pretty(&all_data)?)
            }
            ExportFormat::Excel => {
                anyhow::bail!("Excel export not yet implemented")
            }
        }
    }

    /// Export news data
    pub fn export_news(
        tickers: Option<Vec<String>>,
        from_ts: Option<i64>,
        to_ts: Option<i64>,
        format: ExportFormat,
        db: &Mutex<Database>,
    ) -> Result<String> {
        let db_guard = db.lock().map_err(|e| anyhow::anyhow!("Database lock error: {}", e))?;
        let store = StockNewsStore::new(db_guard.conn.clone());
        
        let news = store.get_news(tickers, 10000, from_ts)?;
        
        // Filter by to_ts if provided
        let news: Vec<_> = if let Some(to) = to_ts {
            news.into_iter().filter(|n| n.published_at <= to).collect()
        } else {
            news
        };
        
        let data: Vec<serde_json::Value> = news.iter().map(|item| {
            serde_json::json!({
                "id": item.id,
                "title": item.title,
                "content": item.content,
                "url": item.url,
                "source": item.source,
                "published_at": item.published_at,
                "sentiment": item.sentiment,
                "tickers": item.tickers,
            })
        }).collect();
        
        match format {
            ExportFormat::Csv => {
                let headers = vec!["id", "title", "source", "published_at", "sentiment", "tickers", "url"];
                Ok(Self::export_to_csv(&data, &headers))
            }
            ExportFormat::Json => {
                Ok(serde_json::to_string_pretty(&data)?)
            }
            ExportFormat::Excel => {
                anyhow::bail!("Excel export not yet implemented")
            }
        }
    }

    /// Export alerts
    pub fn export_alerts(
        limit: Option<i64>,
        format: ExportFormat,
        db: &Mutex<Database>,
    ) -> Result<String> {
        let db_guard = db.lock().map_err(|e| anyhow::anyhow!("Database lock error: {}", e))?;
        let store = TemporalStore::new(db_guard.conn.clone());
        
        let alerts = store.list_alerts(limit.unwrap_or(10000), None, None)?;
        
        let data: Vec<serde_json::Value> = alerts.iter().map(|alert| {
            serde_json::json!({
                "id": alert.id,
                "rule_id": alert.rule_id,
                "fired_at": alert.fired_at,
                "status": alert.status,
                "payload": alert.payload_json,
            })
        }).collect();
        
        match format {
            ExportFormat::Csv => {
                let headers = vec!["id", "rule_id", "fired_at", "status"];
                Ok(Self::export_to_csv(&data, &headers))
            }
            ExportFormat::Json => {
                Ok(serde_json::to_string_pretty(&data)?)
            }
            ExportFormat::Excel => {
                anyhow::bail!("Excel export not yet implemented")
            }
        }
    }

    /// Export economic calendar
    pub fn export_economic_calendar(
        from_ts: Option<i64>,
        to_ts: Option<i64>,
        format: ExportFormat,
        db: &Mutex<Database>,
    ) -> Result<String> {
        let db_guard = db.lock().map_err(|e| anyhow::anyhow!("Database lock error: {}", e))?;
        let store = EconomicCalendarStore::new(db_guard.conn.clone());
        
        let from_ts = from_ts.unwrap_or(0);
        let to_ts = to_ts.unwrap_or(i64::MAX);
        let events = store.list_events(from_ts, to_ts, None, None)?;
        
        let data: Vec<serde_json::Value> = events.iter().map(|event| {
            serde_json::json!({
                "id": event.id,
                "name": event.name,
                "country": event.country,
                "event_type": event.event_type,
                "scheduled_at": event.scheduled_at,
                "forecast_value": event.forecast_value,
                "actual_value": event.actual_value,
                "previous_value": event.previous_value,
                "impact_score": event.impact_score,
            })
        }).collect();
        
        match format {
            ExportFormat::Csv => {
                let headers = vec!["id", "name", "country", "event_type", "scheduled_at", "forecast_value", "actual_value", "impact_score"];
                Ok(Self::export_to_csv(&data, &headers))
            }
            ExportFormat::Json => {
                Ok(serde_json::to_string_pretty(&data)?)
            }
            ExportFormat::Excel => {
                anyhow::bail!("Excel export not yet implemented")
            }
        }
    }

    /// Export temporal events
    pub fn export_temporal_events(
        limit: Option<i64>,
        from_ts: Option<i64>,
        to_ts: Option<i64>,
        format: ExportFormat,
        db: &Mutex<Database>,
    ) -> Result<String> {
        let db_guard = db.lock().map_err(|e| anyhow::anyhow!("Database lock error: {}", e))?;
        let store = TemporalStore::new(db_guard.conn.clone());
        
        let events = store.list_events(
            limit.unwrap_or(10000),
            from_ts,
            to_ts,
        )?;
        
        let data: Vec<serde_json::Value> = events.iter().map(|event| {
            serde_json::json!({
                "id": event.id,
                "title": event.title,
                "summary": event.summary,
                "start_ts": event.start_ts,
                "end_ts": event.end_ts,
                "sentiment_score": event.sentiment_score,
                "event_type": event.event_type,
                "confidence": event.confidence,
            })
        }).collect();
        
        match format {
            ExportFormat::Csv => {
                let headers = vec!["id", "title", "summary", "start_ts", "end_ts", "sentiment_score"];
                Ok(Self::export_to_csv(&data, &headers))
            }
            ExportFormat::Json => {
                Ok(serde_json::to_string_pretty(&data)?)
            }
            ExportFormat::Excel => {
                anyhow::bail!("Excel export not yet implemented")
            }
        }
    }
}

