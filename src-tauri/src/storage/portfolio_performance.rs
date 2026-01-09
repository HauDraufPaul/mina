use anyhow::Result;
use rusqlite::{params, Connection};
use serde::{Deserialize, Serialize};
use std::sync::{Arc, Mutex};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PortfolioSnapshot {
    pub id: i64,
    pub portfolio_id: i64,
    pub timestamp: i64,
    pub total_value: f64,
    pub total_cost: f64,
    pub return_percent: f64,
}

pub struct PortfolioPerformanceStore {
    conn: Arc<Mutex<Connection>>,
}

impl PortfolioPerformanceStore {
    pub fn new(conn: Arc<Mutex<Connection>>) -> Self {
        let store = PortfolioPerformanceStore { conn };
        if let Err(e) = store.init_schema() {
            eprintln!("WARNING: PortfolioPerformanceStore schema initialization failed: {}", e);
        }
        store
    }

    fn init_schema(&self) -> Result<()> {
        let conn = self.conn.lock()
            .map_err(|e| anyhow::anyhow!("Database lock poisoned: {}", e))?;

        conn.execute(
            "CREATE TABLE IF NOT EXISTS portfolio_snapshots (
                id INTEGER PRIMARY KEY AUTOINCREMENT,
                portfolio_id INTEGER NOT NULL,
                timestamp INTEGER NOT NULL,
                total_value REAL NOT NULL,
                total_cost REAL NOT NULL,
                return_percent REAL NOT NULL,
                created_at INTEGER NOT NULL,
                FOREIGN KEY (portfolio_id) REFERENCES portfolios(id) ON DELETE CASCADE
            )",
            [],
        )?;

        conn.execute(
            "CREATE INDEX IF NOT EXISTS idx_portfolio_snapshots_portfolio ON portfolio_snapshots(portfolio_id, timestamp DESC)",
            [],
        )?;

        Ok(())
    }

    pub fn save_snapshot(
        &self,
        portfolio_id: i64,
        total_value: f64,
        total_cost: f64,
        return_percent: f64,
    ) -> Result<i64> {
        let conn = self.conn.lock()
            .map_err(|e| anyhow::anyhow!("Database lock poisoned: {}", e))?;
        let now = chrono::Utc::now().timestamp();
        let timestamp = now;

        conn.execute(
            "INSERT INTO portfolio_snapshots (portfolio_id, timestamp, total_value, total_cost, return_percent, created_at)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6)",
            params![portfolio_id, timestamp, total_value, total_cost, return_percent, now],
        )?;

        Ok(conn.last_insert_rowid())
    }

    pub fn get_snapshots(
        &self,
        portfolio_id: i64,
        from_ts: Option<i64>,
        to_ts: Option<i64>,
        limit: Option<i64>,
    ) -> Result<Vec<PortfolioSnapshot>> {
        let conn = self.conn.lock()
            .map_err(|e| anyhow::anyhow!("Database lock poisoned: {}", e))?;
        let limit = limit.unwrap_or(1000).max(1).min(10000);

        let mut query = "SELECT id, portfolio_id, timestamp, total_value, total_cost, return_percent
                         FROM portfolio_snapshots
                         WHERE portfolio_id = ?1".to_string();
        let mut params_vec: Vec<Box<dyn rusqlite::ToSql>> = vec![Box::new(portfolio_id)];

        if let Some(from) = from_ts {
            query.push_str(" AND timestamp >= ?");
            params_vec.push(Box::new(from));
        }
        if let Some(to) = to_ts {
            query.push_str(" AND timestamp <= ?");
            params_vec.push(Box::new(to));
        }

        query.push_str(" ORDER BY timestamp DESC LIMIT ?");
        params_vec.push(Box::new(limit));

        let mut stmt = conn.prepare(&query)?;
        let rows = stmt.query_map(rusqlite::params_from_iter(params_vec.iter().map(|p| p.as_ref())), |row| {
            Ok(PortfolioSnapshot {
                id: row.get(0)?,
                portfolio_id: row.get(1)?,
                timestamp: row.get(2)?,
                total_value: row.get(3)?,
                total_cost: row.get(4)?,
                return_percent: row.get(5)?,
            })
        })?;

        let mut snapshots = Vec::new();
        for row in rows {
            snapshots.push(row?);
        }

        Ok(snapshots)
    }

    pub fn get_latest_snapshot(&self, portfolio_id: i64) -> Result<Option<PortfolioSnapshot>> {
        let snapshots = self.get_snapshots(portfolio_id, None, None, Some(1))?;
        Ok(snapshots.into_iter().next())
    }
}

