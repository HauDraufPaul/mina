use anyhow::{Context, Result};
use rusqlite::{Connection, params, OptionalExtension};
use std::sync::{Arc, Mutex};
use tauri::AppHandle;

pub struct Database {
    pub conn: Arc<Mutex<Connection>>,
}

impl Database {
    pub fn new(_app_handle: &AppHandle) -> Result<Self> {
        // Get app data directory - using standard paths
        let app_data_dir = if cfg!(target_os = "macos") {
            std::env::var("HOME")
                .map(|home| std::path::PathBuf::from(home).join("Library/Application Support/mina"))
                .context("Failed to get HOME directory")?
        } else if cfg!(target_os = "windows") {
            std::env::var("APPDATA")
                .map(|appdata| std::path::PathBuf::from(appdata).join("mina"))
                .context("Failed to get APPDATA directory")?
        } else {
            std::env::var("HOME")
                .map(|home| std::path::PathBuf::from(home).join(".local/share/mina"))
                .context("Failed to get HOME directory")?
        };

        std::fs::create_dir_all(&app_data_dir)
            .context("Failed to create app data directory")?;

        let db_path = app_data_dir.join("reality.db");
        let conn = Connection::open(&db_path)
            .context("Failed to open database")?;

        let db = Database {
            conn: Arc::new(Mutex::new(conn)),
        };

        db.init_schema()?;
        Ok(db)
    }

    fn init_schema(&self) -> Result<()> {
        let conn = self.conn.lock()
            .map_err(|e| anyhow::anyhow!("Database lock poisoned: {}", e))?;
        
        // System metrics history
        conn.execute(
            "CREATE TABLE IF NOT EXISTS system_metrics (
                id INTEGER PRIMARY KEY AUTOINCREMENT,
                timestamp INTEGER NOT NULL,
                cpu_usage REAL NOT NULL,
                memory_usage REAL NOT NULL,
                disk_usage REAL NOT NULL,
                network_rx INTEGER NOT NULL,
                network_tx INTEGER NOT NULL
            )",
            [],
        )?;

        // Processes
        conn.execute(
            "CREATE TABLE IF NOT EXISTS processes (
                id INTEGER PRIMARY KEY AUTOINCREMENT,
                pid INTEGER NOT NULL,
                name TEXT NOT NULL,
                cpu_usage REAL NOT NULL,
                memory_usage INTEGER NOT NULL,
                status TEXT NOT NULL,
                created_at INTEGER NOT NULL,
                updated_at INTEGER NOT NULL
            )",
            [],
        )?;

        // Network connections
        conn.execute(
            "CREATE TABLE IF NOT EXISTS network_connections (
                id INTEGER PRIMARY KEY AUTOINCREMENT,
                local_address TEXT NOT NULL,
                remote_address TEXT NOT NULL,
                protocol TEXT NOT NULL,
                state TEXT NOT NULL,
                process_id INTEGER,
                created_at INTEGER NOT NULL
            )",
            [],
        )?;

        // Configuration
        conn.execute(
            "CREATE TABLE IF NOT EXISTS config (
                key TEXT PRIMARY KEY,
                value TEXT NOT NULL,
                updated_at INTEGER NOT NULL
            )",
            [],
        )?;

        // Errors
        conn.execute(
            "CREATE TABLE IF NOT EXISTS errors (
                id INTEGER PRIMARY KEY AUTOINCREMENT,
                error_type TEXT NOT NULL,
                message TEXT NOT NULL,
                stack_trace TEXT,
                source TEXT,
                severity TEXT NOT NULL,
                created_at INTEGER NOT NULL,
                resolved_at INTEGER
            )",
            [],
        )?;

        // Create indexes
        conn.execute(
            "CREATE INDEX IF NOT EXISTS idx_system_metrics_timestamp ON system_metrics(timestamp)",
            [],
        )?;

        conn.execute(
            "CREATE INDEX IF NOT EXISTS idx_processes_pid ON processes(pid)",
            [],
        )?;

        conn.execute(
            "CREATE INDEX IF NOT EXISTS idx_errors_created_at ON errors(created_at)",
            [],
        )?;

        Ok(())
    }

    pub fn save_system_metrics(
        &self,
        timestamp: i64,
        cpu_usage: f64,
        memory_usage: f64,
        disk_usage: f64,
        network_rx: u64,
        network_tx: u64,
    ) -> Result<()> {
        let conn = self.conn.lock()
            .map_err(|e| anyhow::anyhow!("Database lock poisoned: {}", e))?;
        conn.execute(
            "INSERT INTO system_metrics (timestamp, cpu_usage, memory_usage, disk_usage, network_rx, network_tx)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6)",
            params![timestamp, cpu_usage, memory_usage, disk_usage, network_rx, network_tx],
        )?;
        Ok(())
    }

    pub fn get_config(&self, key: &str) -> Result<Option<String>> {
        let conn = self.conn.lock()
            .map_err(|e| anyhow::anyhow!("Database lock poisoned: {}", e))?;
        let mut stmt = conn.prepare("SELECT value FROM config WHERE key = ?1")?;
        let value = stmt.query_row(params![key], |row| row.get::<_, String>(0))
            .optional()?;
        Ok(value)
    }

    pub fn set_config(&self, key: &str, value: &str) -> Result<()> {
        let conn = self.conn.lock()
            .map_err(|e| anyhow::anyhow!("Database lock poisoned: {}", e))?;
        let timestamp = chrono::Utc::now().timestamp();
        conn.execute(
            "INSERT OR REPLACE INTO config (key, value, updated_at) VALUES (?1, ?2, ?3)",
            params![key, value, timestamp],
        )?;
        Ok(())
    }

    pub fn save_error(
        &self,
        error_type: &str,
        message: &str,
        stack_trace: Option<&str>,
        source: Option<&str>,
        severity: &str,
    ) -> Result<i64> {
        let conn = self.conn.lock()
            .map_err(|e| anyhow::anyhow!("Database lock poisoned: {}", e))?;
        let timestamp = chrono::Utc::now().timestamp();
        conn.execute(
            "INSERT INTO errors (error_type, message, stack_trace, source, severity, created_at)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6)",
            params![error_type, message, stack_trace, source, severity, timestamp],
        )?;
        Ok(conn.last_insert_rowid())
    }

    pub fn get_recent_errors(&self, limit: i32) -> Result<Vec<ErrorRecord>> {
        let conn = self.conn.lock()
            .map_err(|e| anyhow::anyhow!("Database lock poisoned: {}", e))?;
        let mut stmt = conn.prepare(
            "SELECT id, error_type, message, stack_trace, source, severity, created_at, resolved_at
             FROM errors
             ORDER BY created_at DESC
             LIMIT ?1"
        )?;
        
        let rows = stmt.query_map(params![limit], |row| {
            Ok(ErrorRecord {
                id: row.get(0)?,
                error_type: row.get(1)?,
                message: row.get(2)?,
                stack_trace: row.get(3)?,
                source: row.get(4)?,
                severity: row.get(5)?,
                created_at: row.get(6)?,
                resolved_at: row.get(7)?,
            })
        })?;

        let mut errors = Vec::new();
        for row in rows {
            errors.push(row?);
        }
        Ok(errors)
    }
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct ErrorRecord {
    pub id: i64,
    pub error_type: String,
    pub message: String,
    pub stack_trace: Option<String>,
    pub source: Option<String>,
    pub severity: String,
    pub created_at: i64,
    pub resolved_at: Option<i64>,
}

