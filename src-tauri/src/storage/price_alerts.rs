use anyhow::Result;
use rusqlite::{params, Connection, OptionalExtension};
use serde::{Deserialize, Serialize};
use std::sync::{Arc, Mutex};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PriceAlert {
    pub id: i64,
    pub ticker: String,
    pub condition: String, // "above", "below", "cross_above", "cross_below"
    pub target_price: f64,
    pub current_price: Option<f64>,
    pub triggered: bool,
    pub triggered_at: Option<i64>,
    pub enabled: bool,
    pub created_at: i64,
    pub updated_at: i64,
}

pub struct PriceAlertStore {
    conn: Arc<Mutex<Connection>>,
}

impl PriceAlertStore {
    pub fn new(conn: Arc<Mutex<Connection>>) -> Self {
        let store = PriceAlertStore { conn };
        if let Err(e) = store.init_schema() {
            eprintln!("WARNING: PriceAlertStore schema initialization failed: {}", e);
        }
        store
    }

    fn init_schema(&self) -> Result<()> {
        let conn = self.conn.lock()
            .map_err(|e| anyhow::anyhow!("Database lock poisoned: {}", e))?;

        conn.execute(
            "CREATE TABLE IF NOT EXISTS price_alerts (
                id INTEGER PRIMARY KEY AUTOINCREMENT,
                ticker TEXT NOT NULL,
                condition TEXT NOT NULL,
                target_price REAL NOT NULL,
                current_price REAL,
                triggered INTEGER NOT NULL DEFAULT 0,
                triggered_at INTEGER,
                enabled INTEGER NOT NULL DEFAULT 1,
                created_at INTEGER NOT NULL,
                updated_at INTEGER NOT NULL
            )",
            [],
        )?;

        conn.execute(
            "CREATE INDEX IF NOT EXISTS idx_price_alerts_ticker ON price_alerts(ticker)",
            [],
        )?;

        conn.execute(
            "CREATE INDEX IF NOT EXISTS idx_price_alerts_enabled ON price_alerts(enabled)",
            [],
        )?;

        conn.execute(
            "CREATE INDEX IF NOT EXISTS idx_price_alerts_triggered ON price_alerts(triggered)",
            [],
        )?;

        Ok(())
    }

    pub fn create_alert(
        &self,
        ticker: &str,
        condition: &str,
        target_price: f64,
    ) -> Result<i64> {
        let conn = self.conn.lock()
            .map_err(|e| anyhow::anyhow!("Database lock poisoned: {}", e))?;
        let now = chrono::Utc::now().timestamp();

        conn.execute(
            "INSERT INTO price_alerts (ticker, condition, target_price, enabled, created_at, updated_at)
             VALUES (?1, ?2, ?3, 1, ?4, ?5)",
            params![ticker, condition, target_price, now, now],
        )?;

        Ok(conn.last_insert_rowid())
    }

    pub fn get_alert(&self, id: i64) -> Result<Option<PriceAlert>> {
        let conn = self.conn.lock()
            .map_err(|e| anyhow::anyhow!("Database lock poisoned: {}", e))?;

        conn.query_row(
            "SELECT id, ticker, condition, target_price, current_price, triggered, triggered_at, enabled, created_at, updated_at
             FROM price_alerts WHERE id = ?1",
            params![id],
            |row| {
                Ok(PriceAlert {
                    id: row.get(0)?,
                    ticker: row.get(1)?,
                    condition: row.get(2)?,
                    target_price: row.get(3)?,
                    current_price: row.get(4)?,
                    triggered: row.get::<_, i64>(5)? == 1,
                    triggered_at: row.get(6)?,
                    enabled: row.get::<_, i64>(7)? == 1,
                    created_at: row.get(8)?,
                    updated_at: row.get(9)?,
                })
            },
        )
        .optional()
        .map_err(anyhow::Error::from)
    }

    pub fn list_alerts(
        &self,
        ticker: Option<&str>,
        enabled_only: bool,
    ) -> Result<Vec<PriceAlert>> {
        let conn = self.conn.lock()
            .map_err(|e| anyhow::anyhow!("Database lock poisoned: {}", e))?;

        let mut sql = "SELECT id, ticker, condition, target_price, current_price, triggered, triggered_at, enabled, created_at, updated_at
                       FROM price_alerts WHERE 1=1".to_string();
        let mut params_vec: Vec<Box<dyn rusqlite::ToSql>> = Vec::new();

        if let Some(t) = ticker {
            sql.push_str(" AND ticker = ?");
            params_vec.push(Box::new(t));
        }

        if enabled_only {
            sql.push_str(" AND enabled = 1");
        }

        sql.push_str(" ORDER BY created_at DESC");

        let mut stmt = conn.prepare(&sql)?;
        let rows = stmt.query_map(rusqlite::params_from_iter(params_vec.iter().map(|p| p.as_ref())), |row| {
            Ok(PriceAlert {
                id: row.get(0)?,
                ticker: row.get(1)?,
                condition: row.get(2)?,
                target_price: row.get(3)?,
                current_price: row.get(4)?,
                triggered: row.get::<_, i64>(5)? == 1,
                triggered_at: row.get(6)?,
                enabled: row.get::<_, i64>(7)? == 1,
                created_at: row.get(8)?,
                updated_at: row.get(9)?,
            })
        })?;

        let mut alerts = Vec::new();
        for row in rows {
            alerts.push(row?);
        }

        Ok(alerts)
    }

    pub fn update_alert_price(&self, id: i64, current_price: f64) -> Result<()> {
        let conn = self.conn.lock()
            .map_err(|e| anyhow::anyhow!("Database lock poisoned: {}", e))?;
        let now = chrono::Utc::now().timestamp();

        conn.execute(
            "UPDATE price_alerts SET current_price = ?1, updated_at = ?2 WHERE id = ?3",
            params![current_price, now, id],
        )?;

        Ok(())
    }

    pub fn mark_triggered(&self, id: i64) -> Result<()> {
        let conn = self.conn.lock()
            .map_err(|e| anyhow::anyhow!("Database lock poisoned: {}", e))?;
        let now = chrono::Utc::now().timestamp();

        conn.execute(
            "UPDATE price_alerts SET triggered = 1, triggered_at = ?1, enabled = 0, updated_at = ?1 WHERE id = ?2",
            params![now, id],
        )?;

        Ok(())
    }

    pub fn update_alert(
        &self,
        id: i64,
        condition: Option<&str>,
        target_price: Option<f64>,
        enabled: Option<bool>,
    ) -> Result<()> {
        let conn = self.conn.lock()
            .map_err(|e| anyhow::anyhow!("Database lock poisoned: {}", e))?;
        let now = chrono::Utc::now().timestamp();

        let mut updates = Vec::new();
        let mut params_vec: Vec<Box<dyn rusqlite::ToSql>> = Vec::new();

        if let Some(c) = condition {
            updates.push("condition = ?");
            params_vec.push(Box::new(c));
        }
        if let Some(tp) = target_price {
            updates.push("target_price = ?");
            params_vec.push(Box::new(tp));
        }
        if let Some(e) = enabled {
            updates.push("enabled = ?");
            params_vec.push(Box::new(if e { 1 } else { 0 }));
        }

        if updates.is_empty() {
            return Ok(());
        }

        updates.push("updated_at = ?");
        params_vec.push(Box::new(now));
        params_vec.push(Box::new(id));

        let sql = format!("UPDATE price_alerts SET {} WHERE id = ?", updates.join(", "));
        let mut stmt = conn.prepare(&sql)?;
        stmt.execute(rusqlite::params_from_iter(params_vec.iter().map(|p| p.as_ref())))?;

        Ok(())
    }

    pub fn delete_alert(&self, id: i64) -> Result<()> {
        let conn = self.conn.lock()
            .map_err(|e| anyhow::anyhow!("Database lock poisoned: {}", e))?;

        conn.execute("DELETE FROM price_alerts WHERE id = ?1", params![id])?;

        Ok(())
    }

    pub fn get_active_alerts_for_ticker(&self, ticker: &str) -> Result<Vec<PriceAlert>> {
        self.list_alerts(Some(ticker), true)
    }
}

