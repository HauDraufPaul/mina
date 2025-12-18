use anyhow::Result;
use rusqlite::{Connection, params, OptionalExtension};
use serde::{Deserialize, Serialize};
use std::sync::{Arc, Mutex};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TestSuite {
    pub id: i64,
    pub name: String,
    pub test_type: String, // "unit", "integration", "e2e"
    pub created_at: i64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TestResult {
    pub id: i64,
    pub suite_id: i64,
    pub name: String,
    pub status: String, // "passed", "failed", "running", "pending"
    pub duration: Option<f64>,
    pub error: Option<String>,
    pub executed_at: i64,
}

pub struct TestingStore {
    conn: Arc<Mutex<Connection>>,
}

impl TestingStore {
    pub fn new(conn: Arc<Mutex<Connection>>) -> Self {
        let store = TestingStore { conn };
        store.init_schema().unwrap();
        store
    }

    fn init_schema(&self) -> Result<()> {
        let conn = self.conn.lock().unwrap();

        conn.execute(
            "CREATE TABLE IF NOT EXISTS test_suites (
                id INTEGER PRIMARY KEY AUTOINCREMENT,
                name TEXT NOT NULL UNIQUE,
                test_type TEXT NOT NULL,
                created_at INTEGER NOT NULL
            )",
            [],
        )?;

        conn.execute(
            "CREATE TABLE IF NOT EXISTS test_results (
                id INTEGER PRIMARY KEY AUTOINCREMENT,
                suite_id INTEGER NOT NULL,
                name TEXT NOT NULL,
                status TEXT NOT NULL,
                duration REAL,
                error TEXT,
                executed_at INTEGER NOT NULL,
                FOREIGN KEY (suite_id) REFERENCES test_suites(id)
            )",
            [],
        )?;

        conn.execute(
            "CREATE INDEX IF NOT EXISTS idx_test_results_suite ON test_results(suite_id)",
            [],
        )?;

        conn.execute(
            "CREATE INDEX IF NOT EXISTS idx_test_results_executed ON test_results(executed_at)",
            [],
        )?;

        Ok(())
    }

    pub fn create_suite(&self, name: &str, test_type: &str) -> Result<i64> {
        let conn = self.conn.lock().unwrap();
        let now = chrono::Utc::now().timestamp();

        conn.execute(
            "INSERT OR IGNORE INTO test_suites (name, test_type, created_at)
             VALUES (?1, ?2, ?3)",
            params![name, test_type, now],
        )?;

        // Get the ID
        let id: i64 = conn.query_row(
            "SELECT id FROM test_suites WHERE name = ?1",
            params![name],
            |row| row.get(0),
        )?;

        Ok(id)
    }

    pub fn list_suites(&self) -> Result<Vec<TestSuite>> {
        let conn = self.conn.lock().unwrap();
        let mut stmt = conn.prepare(
            "SELECT id, name, test_type, created_at FROM test_suites ORDER BY name"
        )?;

        let rows = stmt.query_map([], |row| {
            Ok(TestSuite {
                id: row.get(0)?,
                name: row.get(1)?,
                test_type: row.get(2)?,
                created_at: row.get(3)?,
            })
        })?;

        let mut suites = Vec::new();
        for row in rows {
            suites.push(row?);
        }
        Ok(suites)
    }

    pub fn save_test_result(
        &self,
        suite_id: i64,
        name: &str,
        status: &str,
        duration: Option<f64>,
        error: Option<&str>,
    ) -> Result<i64> {
        let conn = self.conn.lock().unwrap();
        let executed_at = chrono::Utc::now().timestamp();

        conn.execute(
            "INSERT INTO test_results (suite_id, name, status, duration, error, executed_at)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6)",
            params![suite_id, name, status, duration, error, executed_at],
        )?;

        Ok(conn.last_insert_rowid())
    }

    pub fn get_suite_results(&self, suite_id: i64) -> Result<Vec<TestResult>> {
        let conn = self.conn.lock().unwrap();
        let mut stmt = conn.prepare(
            "SELECT id, suite_id, name, status, duration, error, executed_at
             FROM test_results
             WHERE suite_id = ?1
             ORDER BY executed_at DESC"
        )?;

        let rows = stmt.query_map(params![suite_id], |row| {
            Ok(TestResult {
                id: row.get(0)?,
                suite_id: row.get(1)?,
                name: row.get(2)?,
                status: row.get(3)?,
                duration: row.get(4)?,
                error: row.get(5)?,
                executed_at: row.get(6)?,
            })
        })?;

        let mut results = Vec::new();
        for row in rows {
            results.push(row?);
        }
        Ok(results)
    }

    pub fn get_suite_stats(&self, suite_id: i64) -> Result<TestSuiteStats> {
        let conn = self.conn.lock().unwrap();
        
        let total: i64 = conn.query_row(
            "SELECT COUNT(*) FROM test_results WHERE suite_id = ?1",
            params![suite_id],
            |row| row.get(0),
        )?;

        let passed: i64 = conn.query_row(
            "SELECT COUNT(*) FROM test_results WHERE suite_id = ?1 AND status = 'passed'",
            params![suite_id],
            |row| row.get(0),
        )?;

        let failed: i64 = conn.query_row(
            "SELECT COUNT(*) FROM test_results WHERE suite_id = ?1 AND status = 'failed'",
            params![suite_id],
            |row| row.get(0),
        )?;

        let avg_duration: Option<f64> = conn
            .query_row(
                "SELECT AVG(duration) FROM test_results WHERE suite_id = ?1 AND duration IS NOT NULL",
                params![suite_id],
                |row| row.get(0),
            )
            .optional()?;

        Ok(TestSuiteStats {
            total: total as usize,
            passed: passed as usize,
            failed: failed as usize,
            duration: avg_duration.unwrap_or(0.0),
        })
    }
}

#[derive(Debug, Serialize, Deserialize)]
pub struct TestSuiteStats {
    pub total: usize,
    pub passed: usize,
    pub failed: usize,
    pub duration: f64,
}

