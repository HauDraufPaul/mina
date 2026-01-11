use anyhow::Result;
use rusqlite::{Connection, params};
use serde::{Deserialize, Serialize};
use std::sync::{Arc, Mutex, MutexGuard};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HealthCheck {
    pub id: i64,
    pub name: String,
    pub url: String,
    pub status: String, // "healthy", "unhealthy", "unknown"
    pub last_check: i64,
    pub response_time: Option<i64>, // milliseconds
    pub error: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Alert {
    pub id: i64,
    pub name: String,
    pub severity: String, // "critical", "warning", "info"
    pub message: String,
    pub source: String,
    pub created_at: i64,
    pub resolved_at: Option<i64>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PrometheusMetric {
    pub name: String,
    pub value: f64,
    pub labels: String, // JSON
    pub timestamp: i64,
}

pub struct DevOpsStore {
    conn: Arc<Mutex<Connection>>,
}

impl DevOpsStore {
    pub fn new(conn: Arc<Mutex<Connection>>) -> Self {
        let store = DevOpsStore { conn };
        if let Err(e) = store.init_schema() {
            eprintln!("WARNING: DevOpsStore schema initialization failed: {}", e);
        }
        store
    }

    /// Safely lock the database connection, recovering from poisoned locks
    fn lock_conn(&self) -> Result<MutexGuard<'_, Connection>> {
        match self.conn.lock() {
            Ok(guard) => Ok(guard),
            Err(err) => {
                eprintln!("WARNING: Database lock was poisoned, recovering...");
                // Extract the guard from the PoisonError - it's still valid
                Ok(err.into_inner())
            }
        }
    }

    fn init_schema(&self) -> Result<()> {
        let conn = self.lock_conn()?;

        conn.execute(
            "CREATE TABLE IF NOT EXISTS health_checks (
                id INTEGER PRIMARY KEY AUTOINCREMENT,
                name TEXT NOT NULL UNIQUE,
                url TEXT NOT NULL,
                status TEXT NOT NULL,
                last_check INTEGER NOT NULL,
                response_time INTEGER,
                error TEXT
            )",
            [],
        )?;

        conn.execute(
            "CREATE TABLE IF NOT EXISTS alerts (
                id INTEGER PRIMARY KEY AUTOINCREMENT,
                name TEXT NOT NULL,
                severity TEXT NOT NULL,
                message TEXT NOT NULL,
                source TEXT NOT NULL,
                created_at INTEGER NOT NULL,
                resolved_at INTEGER
            )",
            [],
        )?;

        conn.execute(
            "CREATE TABLE IF NOT EXISTS prometheus_metrics (
                id INTEGER PRIMARY KEY AUTOINCREMENT,
                name TEXT NOT NULL,
                value REAL NOT NULL,
                labels TEXT NOT NULL,
                timestamp INTEGER NOT NULL
            )",
            [],
        )?;

        conn.execute(
            "CREATE INDEX IF NOT EXISTS idx_health_checks_name ON health_checks(name)",
            [],
        )?;

        // Migration: Ensure alerts table has required columns
        let table_exists: bool = conn.query_row(
            "SELECT COUNT(*) > 0 FROM sqlite_master WHERE type='table' AND name='alerts'",
            [],
            |row| Ok(row.get::<_, i64>(0)? > 0),
        ).unwrap_or(false);
        
        if table_exists {
            // Get existing columns
            let mut stmt = conn.prepare("SELECT name FROM pragma_table_info('alerts')")?;
            let columns: Vec<String> = stmt.query_map([], |row| {
                Ok(row.get::<_, String>(0)?)
            })?.collect::<Result<Vec<_>, _>>()?;
            
            let needs_migration = !columns.contains(&"name".to_string());
            
            if needs_migration {
                // Table exists but has old schema - recreate it
                conn.execute("DROP INDEX IF EXISTS idx_alerts_created_at", [])?;
                conn.execute("DROP TABLE IF EXISTS alerts", [])?;
                conn.execute(
                    "CREATE TABLE alerts (
                        id INTEGER PRIMARY KEY AUTOINCREMENT,
                        name TEXT NOT NULL,
                        severity TEXT NOT NULL,
                        message TEXT NOT NULL,
                        source TEXT NOT NULL,
                        created_at INTEGER NOT NULL,
                        resolved_at INTEGER
                    )",
                    [],
                )?;
            }
        }

        conn.execute(
            "CREATE INDEX IF NOT EXISTS idx_alerts_created_at ON alerts(created_at)",
            [],
        )?;

        conn.execute(
            "CREATE INDEX IF NOT EXISTS idx_prometheus_timestamp ON prometheus_metrics(timestamp)",
            [],
        )?;

        // Initialize default health checks if none exist
        self.init_default_health_checks_with_conn(&conn)?;

        Ok(())
    }

    // Private helper that works with an already-locked connection
    fn init_default_health_checks_with_conn(&self, conn: &MutexGuard<Connection>) -> Result<()> {
        // Always migrate existing health checks with old URLs to new ones
        // This ensures existing health checks get updated even if they already exist
        if let Err(e) = self.migrate_health_check_urls_with_conn(conn) {
            eprintln!("WARNING: Failed to migrate health check URLs: {}", e);
        }
        
        // Check if any health checks exist
        let count: i64 = match conn.query_row(
            "SELECT COUNT(*) FROM health_checks",
            [],
            |row| row.get(0),
        ) {
            Ok(c) => c,
            Err(_) => return Ok(()), // Table doesn't exist yet, skip
        };

        // Only add defaults if no health checks exist
        if count == 0 {
            eprintln!("MINA: Adding default health checks...");
            let now = chrono::Utc::now().timestamp();
            
            // Default health checks - all HTTP endpoints
            // Database and Redis are checked via the MINA health check service which wraps their native protocols
            let default_checks = vec![
                ("Localhost", "http://localhost:3000/health"),
                ("API Server", "http://localhost:8080/health"),
                ("Database", "http://127.0.0.1:5433/health/database"),
                ("Redis", "http://127.0.0.1:5433/health/redis"),
                ("Elasticsearch", "http://localhost:9200/_cluster/health"),
            ];

            for (name, url) in default_checks {
                if let Err(e) = conn.execute(
                    "INSERT OR IGNORE INTO health_checks (name, url, status, last_check)
                     VALUES (?1, ?2, 'unknown', ?3)",
                    params![name, url, now],
                ) {
                    eprintln!("Warning: Failed to insert default health check {}: {}", name, e);
                } else {
                    eprintln!("MINA: Added default health check: {}", name);
                }
            }
            eprintln!("MINA: Default health checks initialization complete");
        }

        Ok(())
    }

    /// Migrate health check URLs from old format to new format
    fn migrate_health_check_urls_with_conn(&self, conn: &MutexGuard<Connection>) -> Result<()> {
        // Update Database health check URL if it's using the old format
        conn.execute(
            "UPDATE health_checks 
             SET url = 'http://127.0.0.1:5433/health/database'
             WHERE name = 'Database' AND url LIKE '%:5432%'",
            [],
        )?;

        // Update Redis health check URL if it's using the old format
        conn.execute(
            "UPDATE health_checks 
             SET url = 'http://127.0.0.1:5433/health/redis'
             WHERE name = 'Redis' AND url LIKE '%:6379%'",
            [],
        )?;

        Ok(())
    }

    // Public API that locks the connection
    pub fn init_default_health_checks(&self) -> Result<()> {
        let conn = self.conn.lock()
            .map_err(|e| anyhow::anyhow!("Database lock poisoned: {}", e))?;
        self.init_default_health_checks_with_conn(&conn)
    }

    pub fn create_health_check(&self, name: &str, url: &str) -> Result<i64> {
        let conn = self.conn.lock()
            .map_err(|e| anyhow::anyhow!("Database lock poisoned: {}", e))?;
        let now = chrono::Utc::now().timestamp();

        conn.execute(
            "INSERT OR REPLACE INTO health_checks (name, url, status, last_check)
             VALUES (?1, ?2, 'unknown', ?3)",
            params![name, url, now],
        )?;

        Ok(conn.last_insert_rowid())
    }

    pub fn update_health_check(
        &self,
        name: &str,
        status: &str,
        response_time: Option<i64>,
        error: Option<&str>,
    ) -> Result<()> {
        let conn = self.conn.lock()
            .map_err(|e| anyhow::anyhow!("Database lock poisoned: {}", e))?;
        let now = chrono::Utc::now().timestamp();

        conn.execute(
            "UPDATE health_checks 
             SET status = ?1, last_check = ?2, response_time = ?3, error = ?4
             WHERE name = ?5",
            params![status, now, response_time, error, name],
        )?;

        Ok(())
    }

    pub fn list_health_checks(&self) -> Result<Vec<HealthCheck>> {
        let conn = self.conn.lock()
            .map_err(|e| anyhow::anyhow!("Database lock poisoned: {}", e))?;
        let mut stmt = conn.prepare(
            "SELECT id, name, url, status, last_check, response_time, error FROM health_checks ORDER BY name"
        )?;

        let rows = stmt.query_map([], |row| {
            Ok(HealthCheck {
                id: row.get(0)?,
                name: row.get(1)?,
                url: row.get(2)?,
                status: row.get(3)?,
                last_check: row.get(4)?,
                response_time: row.get(5)?,
                error: row.get(6)?,
            })
        })?;

        let mut checks = Vec::new();
        for row in rows {
            checks.push(row?);
        }
        Ok(checks)
    }

    pub fn create_alert(
        &self,
        name: &str,
        severity: &str,
        message: &str,
        source: &str,
    ) -> Result<i64> {
        let conn = self.conn.lock()
            .map_err(|e| anyhow::anyhow!("Database lock poisoned: {}", e))?;
        let now = chrono::Utc::now().timestamp();

        conn.execute(
            "INSERT INTO alerts (name, severity, message, source, created_at)
             VALUES (?1, ?2, ?3, ?4, ?5)",
            params![name, severity, message, source, now],
        )?;

        Ok(conn.last_insert_rowid())
    }

    pub fn list_alerts(&self, limit: i32, unresolved_only: bool) -> Result<Vec<Alert>> {
        let conn = self.conn.lock()
            .map_err(|e| anyhow::anyhow!("Database lock poisoned: {}", e))?;
        let query = if unresolved_only {
            "SELECT id, name, severity, message, source, created_at, resolved_at
             FROM alerts
             WHERE resolved_at IS NULL
             ORDER BY created_at DESC
             LIMIT ?1"
        } else {
            "SELECT id, name, severity, message, source, created_at, resolved_at
             FROM alerts
             ORDER BY created_at DESC
             LIMIT ?1"
        };

        let mut stmt = conn.prepare(query)?;
        let rows = stmt.query_map(params![limit], |row| {
            Ok(Alert {
                id: row.get(0)?,
                name: row.get(1)?,
                severity: row.get(2)?,
                message: row.get(3)?,
                source: row.get(4)?,
                created_at: row.get(5)?,
                resolved_at: row.get(6)?,
            })
        })?;

        let mut alerts = Vec::new();
        for row in rows {
            alerts.push(row?);
        }
        Ok(alerts)
    }

    pub fn resolve_alert(&self, id: i64) -> Result<()> {
        let conn = self.conn.lock()
            .map_err(|e| anyhow::anyhow!("Database lock poisoned: {}", e))?;
        let now = chrono::Utc::now().timestamp();

        conn.execute(
            "UPDATE alerts SET resolved_at = ?1 WHERE id = ?2",
            params![now, id],
        )?;

        Ok(())
    }

    pub fn save_prometheus_metric(&self, name: &str, value: f64, labels: &str) -> Result<()> {
        let conn = self.conn.lock()
            .map_err(|e| anyhow::anyhow!("Database lock poisoned: {}", e))?;
        let timestamp = chrono::Utc::now().timestamp();

        conn.execute(
            "INSERT INTO prometheus_metrics (name, value, labels, timestamp)
             VALUES (?1, ?2, ?3, ?4)",
            params![name, value, labels, timestamp],
        )?;

        Ok(())
    }

    pub fn get_prometheus_metrics(&self, name: &str, start_time: i64, end_time: i64) -> Result<Vec<PrometheusMetric>> {
        let conn = self.conn.lock()
            .map_err(|e| anyhow::anyhow!("Database lock poisoned: {}", e))?;
        let mut stmt = conn.prepare(
            "SELECT name, value, labels, timestamp
             FROM prometheus_metrics
             WHERE name = ?1 AND timestamp >= ?2 AND timestamp <= ?3
             ORDER BY timestamp ASC"
        )?;

        let rows = stmt.query_map(params![name, start_time, end_time], |row| {
            Ok(PrometheusMetric {
                name: row.get(0)?,
                value: row.get(1)?,
                labels: row.get(2)?,
                timestamp: row.get(3)?,
            })
        })?;

        let mut metrics = Vec::new();
        for row in rows {
            metrics.push(row?);
        }
        Ok(metrics)
    }
}

