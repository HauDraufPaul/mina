use anyhow::Result;
use rusqlite::{params, Connection, OptionalExtension};
use serde::{Deserialize, Serialize};
use std::sync::{Arc, Mutex};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MarketPrice {
    pub ticker: String,
    pub price: f64,
    pub change: f64,
    pub change_percent: f64,
    pub volume: i64,
    pub timestamp: i64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PriceHistory {
    pub id: i64,
    pub ticker: String,
    pub timestamp: i64,
    pub open: f64,
    pub high: f64,
    pub low: f64,
    pub close: f64,
    pub volume: i64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MarketSnapshot {
    pub ticker: String,
    pub price: f64,
    pub change: f64,
    pub change_percent: f64,
    pub volume: i64,
    pub high_52w: Option<f64>,
    pub low_52w: Option<f64>,
    pub market_cap: Option<i64>,
    pub timestamp: i64,
}

pub struct MarketDataStore {
    pub conn: Arc<Mutex<Connection>>,
}

impl MarketDataStore {
    pub fn new(conn: Arc<Mutex<Connection>>) -> Self {
        let store = MarketDataStore { conn };
        if let Err(e) = store.init_schema() {
            eprintln!("WARNING: MarketDataStore schema initialization failed: {}", e);
        }
        store
    }

    fn init_schema(&self) -> Result<()> {
        let conn = self.conn.lock().unwrap();

        // Market prices table (current/latest prices)
        conn.execute(
            "CREATE TABLE IF NOT EXISTS market_prices (
                ticker TEXT NOT NULL PRIMARY KEY,
                price REAL NOT NULL,
                change REAL NOT NULL DEFAULT 0.0,
                change_percent REAL NOT NULL DEFAULT 0.0,
                volume INTEGER NOT NULL DEFAULT 0,
                timestamp INTEGER NOT NULL,
                updated_at INTEGER NOT NULL
            )",
            [],
        )?;

        // Price history table (OHLCV data)
        conn.execute(
            "CREATE TABLE IF NOT EXISTS price_history (
                id INTEGER PRIMARY KEY AUTOINCREMENT,
                ticker TEXT NOT NULL,
                timestamp INTEGER NOT NULL,
                open REAL NOT NULL,
                high REAL NOT NULL,
                low REAL NOT NULL,
                close REAL NOT NULL,
                volume INTEGER NOT NULL DEFAULT 0,
                created_at INTEGER NOT NULL,
                UNIQUE(ticker, timestamp)
            )",
            [],
        )?;

        // Market snapshots (extended data)
        conn.execute(
            "CREATE TABLE IF NOT EXISTS market_snapshots (
                ticker TEXT NOT NULL PRIMARY KEY,
                price REAL NOT NULL,
                change REAL NOT NULL DEFAULT 0.0,
                change_percent REAL NOT NULL DEFAULT 0.0,
                volume INTEGER NOT NULL DEFAULT 0,
                high_52w REAL,
                low_52w REAL,
                market_cap INTEGER,
                timestamp INTEGER NOT NULL,
                updated_at INTEGER NOT NULL
            )",
            [],
        )?;

        // Create indexes
        conn.execute(
            "CREATE INDEX IF NOT EXISTS idx_price_history_ticker_ts ON price_history(ticker, timestamp DESC)",
            [],
        )?;
        conn.execute(
            "CREATE INDEX IF NOT EXISTS idx_price_history_ts ON price_history(timestamp DESC)",
            [],
        )?;

        Ok(())
    }

    pub fn upsert_price(&self, price: &MarketPrice) -> Result<()> {
        let conn = self.conn.lock().unwrap();
        let now = chrono::Utc::now().timestamp();

        conn.execute(
            "INSERT OR REPLACE INTO market_prices (ticker, price, change, change_percent, volume, timestamp, updated_at)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7)",
            params![
                price.ticker,
                price.price,
                price.change,
                price.change_percent,
                price.volume,
                price.timestamp,
                now
            ],
        )?;

        Ok(())
    }

    pub fn get_price(&self, ticker: &str) -> Result<Option<MarketPrice>> {
        let conn = self.conn.lock().unwrap();

        let price = conn
            .query_row(
                "SELECT ticker, price, change, change_percent, volume, timestamp
                 FROM market_prices
                 WHERE ticker = ?1",
                params![ticker],
                |row| {
                    Ok(MarketPrice {
                        ticker: row.get(0)?,
                        price: row.get(1)?,
                        change: row.get(2)?,
                        change_percent: row.get(3)?,
                        volume: row.get(4)?,
                        timestamp: row.get(5)?,
                    })
                },
            )
            .optional()?;

        Ok(price)
    }

    pub fn get_prices(&self, tickers: &[String]) -> Result<Vec<MarketPrice>> {
        let conn = self.conn.lock().unwrap();
        let placeholders = tickers.iter().map(|_| "?").collect::<Vec<_>>().join(",");
        let query = format!(
            "SELECT ticker, price, change, change_percent, volume, timestamp
             FROM market_prices
             WHERE ticker IN ({})",
            placeholders
        );

        let mut stmt = conn.prepare(&query)?;
        let rows = stmt.query_map(
            rusqlite::params_from_iter(tickers.iter()),
            |row| {
                Ok(MarketPrice {
                    ticker: row.get(0)?,
                    price: row.get(1)?,
                    change: row.get(2)?,
                    change_percent: row.get(3)?,
                    volume: row.get(4)?,
                    timestamp: row.get(5)?,
                })
            },
        )?;

        let mut prices = Vec::new();
        for row in rows {
            prices.push(row?);
        }

        Ok(prices)
    }

    pub fn insert_price_history(&self, history: &PriceHistory) -> Result<i64> {
        let conn = self.conn.lock().unwrap();
        let now = chrono::Utc::now().timestamp();

        conn.execute(
            "INSERT OR REPLACE INTO price_history (ticker, timestamp, open, high, low, close, volume, created_at)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8)",
            params![
                history.ticker,
                history.timestamp,
                history.open,
                history.high,
                history.low,
                history.close,
                history.volume,
                now
            ],
        )?;

        Ok(conn.last_insert_rowid())
    }

    pub fn get_price_history(
        &self,
        ticker: &str,
        from_ts: i64,
        to_ts: i64,
        limit: Option<i64>,
    ) -> Result<Vec<PriceHistory>> {
        let conn = self.conn.lock().unwrap();
        let limit = limit.unwrap_or(1000).max(1).min(10000);

        let mut stmt = conn.prepare(
            "SELECT id, ticker, timestamp, open, high, low, close, volume
             FROM price_history
             WHERE ticker = ?1 AND timestamp >= ?2 AND timestamp <= ?3
             ORDER BY timestamp ASC
             LIMIT ?4",
        )?;

        let rows = stmt.query_map(params![ticker, from_ts, to_ts, limit], |row| {
            Ok(PriceHistory {
                id: row.get(0)?,
                ticker: row.get(1)?,
                timestamp: row.get(2)?,
                open: row.get(3)?,
                high: row.get(4)?,
                low: row.get(5)?,
                close: row.get(6)?,
                volume: row.get(7)?,
            })
        })?;

        let mut history = Vec::new();
        for row in rows {
            history.push(row?);
        }

        Ok(history)
    }

    pub fn upsert_snapshot(&self, snapshot: &MarketSnapshot) -> Result<()> {
        let conn = self.conn.lock().unwrap();
        let now = chrono::Utc::now().timestamp();

        conn.execute(
            "INSERT OR REPLACE INTO market_snapshots 
             (ticker, price, change, change_percent, volume, high_52w, low_52w, market_cap, timestamp, updated_at)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10)",
            params![
                snapshot.ticker,
                snapshot.price,
                snapshot.change,
                snapshot.change_percent,
                snapshot.volume,
                snapshot.high_52w,
                snapshot.low_52w,
                snapshot.market_cap,
                snapshot.timestamp,
                now
            ],
        )?;

        Ok(())
    }

    pub fn get_snapshot(&self, ticker: &str) -> Result<Option<MarketSnapshot>> {
        let conn = self.conn.lock().unwrap();

        let snapshot = conn
            .query_row(
                "SELECT ticker, price, change, change_percent, volume, high_52w, low_52w, market_cap, timestamp
                 FROM market_snapshots
                 WHERE ticker = ?1",
                params![ticker],
                |row| {
                    Ok(MarketSnapshot {
                        ticker: row.get(0)?,
                        price: row.get(1)?,
                        change: row.get(2)?,
                        change_percent: row.get(3)?,
                        volume: row.get(4)?,
                        high_52w: row.get(5)?,
                        low_52w: row.get(6)?,
                        market_cap: row.get(7)?,
                        timestamp: row.get(8)?,
                    })
                },
            )
            .optional()?;

        Ok(snapshot)
    }
}
