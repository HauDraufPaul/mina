use anyhow::{Context, Result};
use rusqlite::{params, Connection, OptionalExtension};
use serde::{Deserialize, Serialize};
use std::sync::{Arc, Mutex};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EconomicEvent {
    pub id: i64,
    pub name: String,
    pub country: String,
    pub event_type: String, // GDP, CPI, Interest Rate, etc.
    pub scheduled_at: i64,
    pub actual_value: Option<f64>,
    pub forecast_value: Option<f64>,
    pub previous_value: Option<f64>,
    pub impact_score: f64, // Predicted impact
    pub created_at: i64,
    pub updated_at: i64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EventImpactHistory {
    pub id: i64,
    pub event_id: i64,
    pub actual_value: f64,
    pub market_reaction: f64, // Market movement percentage
    pub recorded_at: i64,
}

pub struct EconomicCalendarStore {
    pub conn: Arc<Mutex<Connection>>,
}

impl EconomicCalendarStore {
    pub fn new(conn: Arc<Mutex<Connection>>) -> Self {
        let store = EconomicCalendarStore { conn };
        if let Err(e) = store.init_schema() {
            eprintln!("WARNING: EconomicCalendarStore schema initialization failed: {}", e);
        }
        store
    }

    fn init_schema(&self) -> Result<()> {
        let conn = self.conn.lock()
            .map_err(|e| anyhow::anyhow!("Database lock poisoned: {}", e))?;

        conn.execute(
            "CREATE TABLE IF NOT EXISTS economic_events (
                id INTEGER PRIMARY KEY AUTOINCREMENT,
                name TEXT NOT NULL,
                country TEXT NOT NULL,
                event_type TEXT NOT NULL,
                scheduled_at INTEGER NOT NULL,
                actual_value REAL,
                forecast_value REAL,
                previous_value REAL,
                impact_score REAL NOT NULL DEFAULT 0.0,
                created_at INTEGER NOT NULL,
                updated_at INTEGER NOT NULL
            )",
            [],
        )?;

        conn.execute(
            "CREATE TABLE IF NOT EXISTS event_impact_history (
                id INTEGER PRIMARY KEY AUTOINCREMENT,
                event_id INTEGER NOT NULL,
                actual_value REAL NOT NULL,
                market_reaction REAL NOT NULL,
                recorded_at INTEGER NOT NULL,
                FOREIGN KEY (event_id) REFERENCES economic_events(id) ON DELETE CASCADE
            )",
            [],
        )?;

        conn.execute(
            "CREATE INDEX IF NOT EXISTS idx_economic_events_scheduled ON economic_events(scheduled_at DESC)",
            [],
        )?;
        conn.execute(
            "CREATE INDEX IF NOT EXISTS idx_economic_events_country ON economic_events(country)",
            [],
        )?;
        conn.execute(
            "CREATE INDEX IF NOT EXISTS idx_economic_events_type ON economic_events(event_type)",
            [],
        )?;
        conn.execute(
            "CREATE INDEX IF NOT EXISTS idx_impact_history_event ON event_impact_history(event_id)",
            [],
        )?;

        Ok(())
    }

    pub fn create_event(
        &self,
        name: &str,
        country: &str,
        event_type: &str,
        scheduled_at: i64,
        forecast_value: Option<f64>,
        previous_value: Option<f64>,
        impact_score: f64,
    ) -> Result<i64> {
        let conn = self.conn.lock()
            .map_err(|e| anyhow::anyhow!("Database lock poisoned: {}", e))?;
        let now = chrono::Utc::now().timestamp();

        conn.execute(
            "INSERT INTO economic_events (name, country, event_type, scheduled_at, forecast_value, previous_value, impact_score, created_at, updated_at)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9)",
            params![name, country, event_type, scheduled_at, forecast_value, previous_value, impact_score, now, now],
        )?;

        Ok(conn.last_insert_rowid())
    }

    pub fn list_events(&self, from_ts: i64, to_ts: i64, country: Option<&str>, event_type: Option<&str>) -> Result<Vec<EconomicEvent>> {
        let conn = self.conn.lock()
            .map_err(|e| anyhow::anyhow!("Database lock poisoned: {}", e))?;
        
        let mut query = "SELECT id, name, country, event_type, scheduled_at, actual_value, forecast_value, previous_value, impact_score, created_at, updated_at
                         FROM economic_events
                         WHERE scheduled_at >= ?1 AND scheduled_at <= ?2".to_string();
        let mut params_vec: Vec<Box<dyn rusqlite::ToSql>> = vec![Box::new(from_ts), Box::new(to_ts)];

        if let Some(c) = country {
            query.push_str(" AND country = ?3");
            params_vec.push(Box::new(c));
        }

        if let Some(et) = event_type {
            let param_num = if country.is_some() { "?4" } else { "?3" };
            query.push_str(&format!(" AND event_type = {}", param_num));
            params_vec.push(Box::new(et));
        }

        query.push_str(" ORDER BY scheduled_at ASC");

        let mut stmt = conn.prepare(&query)?;
        let rows = stmt.query_map(rusqlite::params_from_iter(params_vec.iter()), |row| {
            Ok(EconomicEvent {
                id: row.get(0)?,
                name: row.get(1)?,
                country: row.get(2)?,
                event_type: row.get(3)?,
                scheduled_at: row.get(4)?,
                actual_value: row.get(5)?,
                forecast_value: row.get(6)?,
                previous_value: row.get(7)?,
                impact_score: row.get(8)?,
                created_at: row.get(9)?,
                updated_at: row.get(10)?,
            })
        })?;

        let mut events = Vec::new();
        for row in rows {
            events.push(row?);
        }

        Ok(events)
    }

    pub fn get_event(&self, id: i64) -> Result<Option<EconomicEvent>> {
        let conn = self.conn.lock()
            .map_err(|e| anyhow::anyhow!("Database lock poisoned: {}", e))?;

        let event = conn
            .query_row(
                "SELECT id, name, country, event_type, scheduled_at, actual_value, forecast_value, previous_value, impact_score, created_at, updated_at
                 FROM economic_events
                 WHERE id = ?1",
                params![id],
                |row| {
                    Ok(EconomicEvent {
                        id: row.get(0)?,
                        name: row.get(1)?,
                        country: row.get(2)?,
                        event_type: row.get(3)?,
                        scheduled_at: row.get(4)?,
                        actual_value: row.get(5)?,
                        forecast_value: row.get(6)?,
                        previous_value: row.get(7)?,
                        impact_score: row.get(8)?,
                        created_at: row.get(9)?,
                        updated_at: row.get(10)?,
                    })
                },
            )
            .optional()?;

        Ok(event)
    }

    pub fn update_event(
        &self,
        id: i64,
        actual_value: Option<f64>,
        impact_score: Option<f64>,
    ) -> Result<()> {
        let conn = self.conn.lock()
            .map_err(|e| anyhow::anyhow!("Database lock poisoned: {}", e))?;
        let now = chrono::Utc::now().timestamp();

        if let Some(av) = actual_value {
            conn.execute(
                "UPDATE economic_events SET actual_value = ?1, updated_at = ?2 WHERE id = ?3",
                params![av, now, id],
            )?;
        }

        if let Some(is) = impact_score {
            conn.execute(
                "UPDATE economic_events SET impact_score = ?1, updated_at = ?2 WHERE id = ?3",
                params![is, now, id],
            )?;
        }

        Ok(())
    }

    pub fn record_event_outcome(
        &self,
        event_id: i64,
        actual_value: f64,
        market_reaction: f64,
    ) -> Result<i64> {
        let conn = self.conn.lock()
            .map_err(|e| anyhow::anyhow!("Database lock poisoned: {}", e))?;
        let now = chrono::Utc::now().timestamp();

        // Update event with actual value
        conn.execute(
            "UPDATE economic_events SET actual_value = ?1, updated_at = ?2 WHERE id = ?3",
            params![actual_value, now, event_id],
        )?;

        // Record in impact history
        conn.execute(
            "INSERT INTO event_impact_history (event_id, actual_value, market_reaction, recorded_at)
             VALUES (?1, ?2, ?3, ?4)",
            params![event_id, actual_value, market_reaction, now],
        )?;

        Ok(conn.last_insert_rowid())
    }

    pub fn get_impact_history(&self, event_id: i64) -> Result<Vec<EventImpactHistory>> {
        let conn = self.conn.lock()
            .map_err(|e| anyhow::anyhow!("Database lock poisoned: {}", e))?;
        let mut stmt = conn.prepare(
            "SELECT id, event_id, actual_value, market_reaction, recorded_at
             FROM event_impact_history
             WHERE event_id = ?1
             ORDER BY recorded_at DESC",
        )?;

        let rows = stmt.query_map(params![event_id], |row| {
            Ok(EventImpactHistory {
                id: row.get(0)?,
                event_id: row.get(1)?,
                actual_value: row.get(2)?,
                market_reaction: row.get(3)?,
                recorded_at: row.get(4)?,
            })
        })?;

        let mut history = Vec::new();
        for row in rows {
            history.push(row?);
        }

        Ok(history)
    }
}
