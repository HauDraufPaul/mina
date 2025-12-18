use anyhow::{Context, Result};
use rusqlite::Connection;
use std::sync::Arc;
use std::sync::Mutex;
use std::time::{SystemTime, UNIX_EPOCH};

pub fn seed_initial_data(conn: &Arc<Mutex<Connection>>) -> Result<()> {
    let conn_guard = conn.lock()
        .map_err(|e| anyhow::anyhow!("Database lock poisoned: {}", e))?;
    let now = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .context("System time is before UNIX epoch")?
        .as_secs() as i64;

    // Check if rate limit buckets already exist
    let bucket_count: i64 = conn_guard.query_row(
        "SELECT COUNT(*) FROM rate_limit_buckets",
        [],
        |row| row.get(0),
    ).unwrap_or(0); // Safe to unwrap_or here - table might not exist yet

    // Seed rate limit buckets only if table is empty
    if bucket_count == 0 {
        conn_guard.execute(
            "INSERT INTO rate_limit_buckets (name, capacity, tokens, refill_rate, refill_interval, last_refill)
             VALUES 
             ('api_requests', 1000, 1000, 10, 60, ?1),
             ('database_queries', 500, 500, 5, 30, ?1),
             ('file_operations', 200, 200, 2, 60, ?1)",
            [now],
        )?;
    }

    // Check if test suites already exist
    let suite_count: i64 = conn_guard.query_row(
        "SELECT COUNT(*) FROM test_suites",
        [],
        |row| row.get(0),
    ).unwrap_or(0); // Safe to unwrap_or here - table might not exist yet

    // Seed test suites only if table is empty
    if suite_count == 0 {
        conn_guard.execute(
            "INSERT INTO test_suites (name, test_type, created_at)
             VALUES 
             ('Unit Tests', 'unit', ?1),
             ('Integration Tests', 'integration', ?1),
             ('E2E Tests', 'e2e', ?1)",
            [now],
        )?;

        // Get test suite IDs and seed some results
        let unit_suite_id: i64 = conn_guard.query_row(
            "SELECT id FROM test_suites WHERE name = 'Unit Tests'",
            [],
            |row| row.get(0),
        )?;

        let integration_suite_id: i64 = conn_guard.query_row(
            "SELECT id FROM test_suites WHERE name = 'Integration Tests'",
            [],
            |row| row.get(0),
        )?;

        // Seed test results for unit tests
        conn_guard.execute(
            "INSERT INTO test_results (suite_id, name, status, duration, error, executed_at)
             VALUES 
             (?1, 'test_user_creation', 'passed', 0.05, NULL, ?2),
             (?1, 'test_user_validation', 'passed', 0.03, NULL, ?2),
             (?1, 'test_user_deletion', 'failed', 0.12, 'Assertion failed', ?2),
             (?1, 'test_data_serialization', 'passed', 0.08, NULL, ?2)",
            [unit_suite_id, now],
        )?;

        // Seed test results for integration tests
        conn_guard.execute(
            "INSERT INTO test_results (suite_id, name, status, duration, error, executed_at)
             VALUES 
             (?1, 'test_api_endpoint', 'passed', 0.25, NULL, ?2),
             (?1, 'test_database_connection', 'passed', 0.15, NULL, ?2),
             (?1, 'test_file_upload', 'passed', 0.30, NULL, ?2)",
            [integration_suite_id, now],
        )?;
    }

    Ok(())
}

