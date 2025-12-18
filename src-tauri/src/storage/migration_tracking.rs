use anyhow::{Context, Result};
use rusqlite::{Connection, params, OptionalExtension};
use serde::{Deserialize, Serialize};
use std::sync::{Arc, Mutex};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MigrationRecord {
    pub version: i32,
    pub name: String,
    pub applied_at: i64,
    pub status: String,
}

pub struct MigrationTracker {
    conn: Arc<Mutex<Connection>>,
}

impl MigrationTracker {
    pub fn new(conn: Arc<Mutex<Connection>>) -> Self {
        let tracker = MigrationTracker { conn };
        tracker.init_schema().unwrap();
        tracker
    }

    fn init_schema(&self) -> Result<()> {
        let conn = self.conn.lock().unwrap();

        conn.execute(
            "CREATE TABLE IF NOT EXISTS migration_history (
                version INTEGER PRIMARY KEY,
                name TEXT NOT NULL,
                applied_at INTEGER NOT NULL,
                status TEXT NOT NULL DEFAULT 'applied'
            )",
            [],
        )?;

        Ok(())
    }

    pub fn list_migrations(&self) -> Result<Vec<MigrationRecord>> {
        let conn = self.conn.lock().unwrap();
        let mut stmt = conn.prepare(
            "SELECT version, name, applied_at, status FROM migration_history ORDER BY version"
        )?;

        let rows = stmt.query_map([], |row| {
            Ok(MigrationRecord {
                version: row.get(0)?,
                name: row.get(1)?,
                applied_at: row.get(2)?,
                status: row.get(3)?,
            })
        })?;

        let mut migrations = Vec::new();
        for row in rows {
            migrations.push(row?);
        }
        Ok(migrations)
    }

    pub fn record_migration(&self, version: i32, name: &str, status: &str) -> Result<()> {
        let conn = self.conn.lock().unwrap();
        let timestamp = chrono::Utc::now().timestamp();

        conn.execute(
            "INSERT OR REPLACE INTO migration_history (version, name, applied_at, status)
             VALUES (?1, ?2, ?3, ?4)",
            params![version, name, timestamp, status],
        )?;

        Ok(())
    }

    pub fn get_latest_version(&self) -> Result<i32> {
        let conn = self.conn.lock().unwrap();
        
        let version: Option<i32> = conn
            .query_row(
                "SELECT MAX(version) FROM migration_history",
                [],
                |row| row.get(0),
            )
            .optional()?;

        Ok(version.unwrap_or(0))
    }
}

