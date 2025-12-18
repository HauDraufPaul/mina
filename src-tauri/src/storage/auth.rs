use anyhow::{Context, Result};
use rusqlite::{Connection, params, OptionalExtension};
use sha2::{Sha256, Digest};
use std::sync::{Arc, Mutex};

pub struct AuthManager {
    conn: Arc<Mutex<Connection>>,
}

impl AuthManager {
    pub fn new(conn: Arc<Mutex<Connection>>) -> Self {
        let auth = AuthManager { conn };
        auth.init_schema().unwrap();
        auth
    }

    fn init_schema(&self) -> Result<()> {
        let conn = self.conn.lock().unwrap();
        
        conn.execute(
            "CREATE TABLE IF NOT EXISTS auth_sessions (
                id TEXT PRIMARY KEY,
                user_id TEXT NOT NULL,
                created_at INTEGER NOT NULL,
                expires_at INTEGER NOT NULL,
                last_activity INTEGER NOT NULL
            )",
            [],
        )?;

        conn.execute(
            "CREATE TABLE IF NOT EXISTS auth_attempts (
                id INTEGER PRIMARY KEY AUTOINCREMENT,
                user_id TEXT NOT NULL,
                success INTEGER NOT NULL,
                ip_address TEXT,
                created_at INTEGER NOT NULL
            )",
            [],
        )?;

        conn.execute(
            "CREATE TABLE IF NOT EXISTS permissions (
                id INTEGER PRIMARY KEY AUTOINCREMENT,
                user_id TEXT NOT NULL,
                resource TEXT NOT NULL,
                action TEXT NOT NULL,
                granted INTEGER NOT NULL,
                created_at INTEGER NOT NULL
            )",
            [],
        )?;

        conn.execute(
            "CREATE INDEX IF NOT EXISTS idx_auth_sessions_user_id ON auth_sessions(user_id)",
            [],
        )?;

        conn.execute(
            "CREATE INDEX IF NOT EXISTS idx_auth_attempts_user_id ON auth_attempts(user_id)",
            [],
        )?;

        Ok(())
    }

    pub fn set_pin(&self, user_id: &str, pin: &str) -> Result<()> {
        let conn = self.conn.lock().unwrap();
        let hashed = Self::hash_pin(pin);
        
        conn.execute(
            "INSERT OR REPLACE INTO config (key, value, updated_at) VALUES (?1, ?2, ?3)",
            params![format!("pin_{}", user_id), hashed, chrono::Utc::now().timestamp()],
        )?;
        
        Ok(())
    }

    pub fn verify_pin(&self, user_id: &str, pin: &str) -> Result<bool> {
        let conn = self.conn.lock().unwrap();
        let hashed = Self::hash_pin(pin);
        
        let stored: Option<String> = conn
            .query_row(
                "SELECT value FROM config WHERE key = ?1",
                params![format!("pin_{}", user_id)],
                |row| row.get::<_, String>(0),
            )
            .optional()?;
        
        Ok(stored.map(|s| s == hashed).unwrap_or(false))
    }

    pub fn create_session(&self, user_id: &str) -> Result<String> {
        let conn = self.conn.lock().unwrap();
        let session_id = uuid::Uuid::new_v4().to_string();
        let now = chrono::Utc::now().timestamp();
        let expires_at = now + 3600; // 1 hour
        
        conn.execute(
            "INSERT INTO auth_sessions (id, user_id, created_at, expires_at, last_activity)
             VALUES (?1, ?2, ?3, ?4, ?5)",
            params![session_id, user_id, now, expires_at, now],
        )?;
        
        Ok(session_id)
    }

    pub fn validate_session(&self, session_id: &str) -> Result<bool> {
        let conn = self.conn.lock().unwrap();
        let now = chrono::Utc::now().timestamp();
        
        let count: i64 = match conn
            .query_row(
                "SELECT COUNT(*) FROM auth_sessions WHERE id = ?1 AND expires_at > ?2",
                params![session_id, now],
                |row| row.get(0),
            )
            .optional() {
            Ok(Some(val)) => val,
            Ok(None) => 0,
            Err(_) => 0,
        };
        
        let exists = count > 0;
        
        if exists {
            // Update last activity
            conn.execute(
                "UPDATE auth_sessions SET last_activity = ?1 WHERE id = ?2",
                params![now, session_id],
            )?;
        }
        
        Ok(exists)
    }

    pub fn log_auth_attempt(&self, user_id: &str, success: bool, ip_address: Option<&str>) -> Result<()> {
        let conn = self.conn.lock().unwrap();
        let now = chrono::Utc::now().timestamp();
        
        conn.execute(
            "INSERT INTO auth_attempts (user_id, success, ip_address, created_at)
             VALUES (?1, ?2, ?3, ?4)",
            params![user_id, if success { 1 } else { 0 }, ip_address, now],
        )?;
        
        Ok(())
    }

    pub fn get_recent_attempts(&self, limit: i32) -> Result<Vec<AuthAttempt>> {
        let conn = self.conn.lock().unwrap();
        let mut stmt = conn.prepare(
            "SELECT id, user_id, success, ip_address, created_at
             FROM auth_attempts
             ORDER BY created_at DESC
             LIMIT ?1"
        )?;
        
        let rows = stmt.query_map(params![limit], |row| {
            Ok(AuthAttempt {
                id: row.get(0)?,
                user_id: row.get(1)?,
                success: row.get::<_, i64>(2)? == 1,
                ip_address: row.get(3)?,
                created_at: row.get(4)?,
            })
        })?;

        let mut attempts = Vec::new();
        for row in rows {
            attempts.push(row?);
        }
        Ok(attempts)
    }

    pub fn grant_permission(&self, user_id: &str, resource: &str, action: &str) -> Result<()> {
        let conn = self.conn.lock().unwrap();
        let now = chrono::Utc::now().timestamp();
        
        conn.execute(
            "INSERT INTO permissions (user_id, resource, action, granted, created_at)
             VALUES (?1, ?2, ?3, ?4, ?5)",
            params![user_id, resource, action, 1, now],
        )?;
        
        Ok(())
    }

    pub fn check_permission(&self, user_id: &str, resource: &str, action: &str) -> Result<bool> {
        let conn = self.conn.lock().unwrap();
        
        let count: i64 = match conn
            .query_row(
                "SELECT COUNT(*) FROM permissions 
                 WHERE user_id = ?1 AND resource = ?2 AND action = ?3 AND granted = 1",
                params![user_id, resource, action],
                |row| row.get(0),
            )
            .optional() {
            Ok(Some(val)) => val,
            Ok(None) => 0,
            Err(_) => 0,
        };
        
        let granted = count > 0;
        
        Ok(granted)
    }

    fn hash_pin(pin: &str) -> String {
        let mut hasher = Sha256::new();
        hasher.update(pin.as_bytes());
        format!("{:x}", hasher.finalize())
    }
}

#[derive(Debug, serde::Serialize)]
pub struct AuthAttempt {
    pub id: i64,
    pub user_id: String,
    pub success: bool,
    pub ip_address: Option<String>,
    pub created_at: i64,
}

