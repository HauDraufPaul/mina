use anyhow::{Context, Result};
use rusqlite::{Connection, params};
use serde::{Deserialize, Serialize};
use std::sync::{Arc, Mutex};

#[derive(Debug, Serialize, Deserialize)]
pub struct AnalyticsMetrics {
    pub timestamp: i64,
    pub metric_type: String,
    pub value: f64,
    pub metadata: Option<String>,
}

pub struct AnalyticsStore {
    conn: Arc<Mutex<Connection>>,
}

impl AnalyticsStore {
    pub fn new(conn: Arc<Mutex<Connection>>) -> Self {
        let store = AnalyticsStore { conn };
        store.init_schema().unwrap();
        store
    }

    fn init_schema(&self) -> Result<()> {
        let conn = self.conn.lock().unwrap();

        conn.execute(
            "CREATE TABLE IF NOT EXISTS analytics_metrics (
                id INTEGER PRIMARY KEY AUTOINCREMENT,
                timestamp INTEGER NOT NULL,
                metric_type TEXT NOT NULL,
                value REAL NOT NULL,
                metadata TEXT
            )",
            [],
        )?;

        conn.execute(
            "CREATE INDEX IF NOT EXISTS idx_analytics_timestamp ON analytics_metrics(timestamp)",
            [],
        )?;

        conn.execute(
            "CREATE INDEX IF NOT EXISTS idx_analytics_type ON analytics_metrics(metric_type)",
            [],
        )?;

        Ok(())
    }

    pub fn save_metric(&self, metric_type: &str, value: f64, metadata: Option<&str>) -> Result<()> {
        let conn = self.conn.lock().unwrap();
        let timestamp = chrono::Utc::now().timestamp();

        conn.execute(
            "INSERT INTO analytics_metrics (timestamp, metric_type, value, metadata)
             VALUES (?1, ?2, ?3, ?4)",
            params![timestamp, metric_type, value, metadata],
        )?;

        Ok(())
    }

    pub fn get_metrics(
        &self,
        metric_type: &str,
        start_time: Option<i64>,
        end_time: Option<i64>,
        limit: Option<i32>,
    ) -> Result<Vec<AnalyticsMetrics>> {
        let conn = self.conn.lock().unwrap();
        let mut query = "SELECT timestamp, metric_type, value, metadata FROM analytics_metrics WHERE metric_type = ?1".to_string();
        
        if start_time.is_some() {
            query.push_str(" AND timestamp >= ?2");
        }
        if end_time.is_some() {
            query.push_str(" AND timestamp <= ?3");
        }
        query.push_str(" ORDER BY timestamp DESC");
        if let Some(lim) = limit {
            query.push_str(&format!(" LIMIT {}", lim));
        }

        let mut metrics = Vec::new();
        match (start_time, end_time) {
            (Some(start), Some(end)) => {
                let mut stmt = conn.prepare(&query)?;
                let rows = stmt.query_map(params![metric_type, start, end], |row| {
                    Ok(AnalyticsMetrics {
                        timestamp: row.get(0)?,
                        metric_type: row.get(1)?,
                        value: row.get(2)?,
                        metadata: row.get(3)?,
                    })
                })?;
                for row in rows {
                    metrics.push(row?);
                }
            }
            (Some(start), None) => {
                let mut stmt = conn.prepare(&query)?;
                let rows = stmt.query_map(params![metric_type, start], |row| {
                    Ok(AnalyticsMetrics {
                        timestamp: row.get(0)?,
                        metric_type: row.get(1)?,
                        value: row.get(2)?,
                        metadata: row.get(3)?,
                    })
                })?;
                for row in rows {
                    metrics.push(row?);
                }
            }
            (None, Some(end)) => {
                let mut stmt = conn.prepare(&query)?;
                let rows = stmt.query_map(params![metric_type, end], |row| {
                    Ok(AnalyticsMetrics {
                        timestamp: row.get(0)?,
                        metric_type: row.get(1)?,
                        value: row.get(2)?,
                        metadata: row.get(3)?,
                    })
                })?;
                for row in rows {
                    metrics.push(row?);
                }
            }
            (None, None) => {
                let mut stmt = conn.prepare(&query)?;
                let rows = stmt.query_map(params![metric_type], |row| {
                    Ok(AnalyticsMetrics {
                        timestamp: row.get(0)?,
                        metric_type: row.get(1)?,
                        value: row.get(2)?,
                        metadata: row.get(3)?,
                    })
                })?;
                for row in rows {
                    metrics.push(row?);
                }
            }
        }
        Ok(metrics)
    }

    pub fn get_statistics(&self, metric_type: &str, start_time: Option<i64>, end_time: Option<i64>) -> Result<Statistics> {
        let conn = self.conn.lock().unwrap();
        let mut query = "SELECT AVG(value), MIN(value), MAX(value), COUNT(*) FROM analytics_metrics WHERE metric_type = ?1".to_string();
        
        if start_time.is_some() {
            query.push_str(" AND timestamp >= ?2");
        }
        if end_time.is_some() {
            query.push_str(" AND timestamp <= ?3");
        }

        let mut stmt = conn.prepare(&query)?;
        let row = match (start_time, end_time) {
            (Some(start), Some(end)) => {
                stmt.query_row(params![metric_type, start, end], |row| {
                    Ok((
                        row.get::<_, Option<f64>>(0)?,
                        row.get::<_, Option<f64>>(1)?,
                        row.get::<_, Option<f64>>(2)?,
                        row.get::<_, i64>(3)?,
                    ))
                })?
            }
            (Some(start), None) => {
                stmt.query_row(params![metric_type, start], |row| {
                    Ok((
                        row.get::<_, Option<f64>>(0)?,
                        row.get::<_, Option<f64>>(1)?,
                        row.get::<_, Option<f64>>(2)?,
                        row.get::<_, i64>(3)?,
                    ))
                })?
            }
            (None, Some(end)) => {
                stmt.query_row(params![metric_type, end], |row| {
                    Ok((
                        row.get::<_, Option<f64>>(0)?,
                        row.get::<_, Option<f64>>(1)?,
                        row.get::<_, Option<f64>>(2)?,
                        row.get::<_, i64>(3)?,
                    ))
                })?
            }
            (None, None) => {
                stmt.query_row(params![metric_type], |row| {
                    Ok((
                        row.get::<_, Option<f64>>(0)?,
                        row.get::<_, Option<f64>>(1)?,
                        row.get::<_, Option<f64>>(2)?,
                        row.get::<_, i64>(3)?,
                    ))
                })?
            }
        };

        let (avg, min, max, count) = row;
        
        // Calculate standard deviation
        let mut std_dev = 0.0;
        if let Some(mean) = avg {
            // Simplified std dev calculation - get all values and calculate
            let all_metrics = self.get_metrics(metric_type, start_time, end_time, None)?;
            if !all_metrics.is_empty() {
                let variance: f64 = all_metrics.iter()
                    .map(|m| (m.value - mean).powi(2))
                    .sum::<f64>() / all_metrics.len() as f64;
                std_dev = variance.sqrt();
            }
        }

        Ok(Statistics {
            mean: avg.unwrap_or(0.0),
            min: min.unwrap_or(0.0),
            max: max.unwrap_or(0.0),
            std_dev,
            count: count as usize,
        })
    }
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Statistics {
    pub mean: f64,
    pub min: f64,
    pub max: f64,
    pub std_dev: f64,
    pub count: usize,
}
