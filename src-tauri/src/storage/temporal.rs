use anyhow::Result;
use rusqlite::{params, Connection, OptionalExtension};
use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::collections::{HashMap, HashSet};
use std::sync::{Arc, Mutex};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TemporalEvent {
    pub id: i64,
    pub title: String,
    pub summary: String,
    pub start_ts: i64,
    pub end_ts: i64,
    pub event_type: String,
    pub confidence: f64,
    pub severity: f64,
    pub novelty_score: f64,
    pub volume_score: f64,
    pub sentiment_score: f64,
    pub cluster_key: String,
    pub created_at: i64,
    pub updated_at: i64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TemporalEventEvidence {
    pub event_id: i64,
    pub rss_item_id: i64,
    pub weight: f64,
    pub snippet: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Watchlist {
    pub id: i64,
    pub name: String,
    pub created_at: i64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WatchlistItem {
    pub id: i64,
    pub watchlist_id: i64,
    pub item_type: String, // entity|keyword|domain|source
    pub value: String,
    pub weight: f64,
    pub enabled: bool,
    pub created_at: i64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AlertRule {
    pub id: i64,
    pub name: String,
    pub enabled: bool,
    pub watchlist_id: Option<i64>,
    pub rule_json: Value,
    pub schedule: Option<String>,
    pub escalation_config: Option<Value>, // Escalation configuration JSON
    pub created_at: i64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Alert {
    pub id: i64,
    pub rule_id: i64,
    pub fired_at: i64,
    pub event_id: Option<i64>,
    pub payload_json: Value,
    pub status: String, // new|ack|snoozed|resolved
    pub snoozed_until: Option<i64>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BacktestReport {
    pub from_ts: i64,
    pub to_ts: i64,
    pub total_alerts: i64,
    pub acked_alerts: i64,
    pub snoozed_alerts: i64,
    pub resolved_alerts: i64,
    pub helpful_alerts: i64,
    pub unhelpful_alerts: i64,
    pub by_rule: HashMap<i64, i64>,
    pub by_rule_helpful: HashMap<i64, i64>,
    pub by_rule_unhelpful: HashMap<i64, i64>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AlertLabel {
    pub alert_id: i64,
    pub label: i64, // 1 helpful, -1 unhelpful
    pub note: Option<String>,
    pub created_at: i64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EntityGraphNode {
    pub id: String,
    pub label: String,
    pub count: i64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EntityGraphEdge {
    pub source: String,
    pub target: String,
    pub weight: i64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EntityGraph {
    pub nodes: Vec<EntityGraphNode>,
    pub edges: Vec<EntityGraphEdge>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FeatureDefinition {
    pub id: i64,
    pub name: String,
    pub expression: String,
    pub description: Option<String>,
    pub created_at: i64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FeatureValue {
    pub id: i64,
    pub feature_id: i64,
    pub ts: i64,
    pub subject_type: String,
    pub subject_value: String,
    pub value: f64,
}

pub struct TemporalStore {
    pub conn: Arc<Mutex<Connection>>,
}

impl TemporalStore {
    pub fn new(conn: Arc<Mutex<Connection>>) -> Self {
        let store = TemporalStore { conn };
        if let Err(e) = store.init_schema() {
            eprintln!("WARNING: TemporalStore schema initialization failed: {}", e);
        }
        store
    }

    fn init_schema(&self) -> Result<()> {
        let conn = self.conn.lock()
            .map_err(|e| anyhow::anyhow!("Database lock poisoned: {}", e))?;

        conn.execute(
            "CREATE TABLE IF NOT EXISTS temporal_events (
                id INTEGER PRIMARY KEY AUTOINCREMENT,
                title TEXT NOT NULL,
                summary TEXT NOT NULL,
                start_ts INTEGER NOT NULL,
                end_ts INTEGER NOT NULL,
                event_type TEXT NOT NULL DEFAULT 'news',
                confidence REAL NOT NULL DEFAULT 0.5,
                severity REAL NOT NULL DEFAULT 0.0,
                novelty_score REAL NOT NULL DEFAULT 0.0,
                volume_score REAL NOT NULL DEFAULT 0.0,
                sentiment_score REAL NOT NULL DEFAULT 0.0,
                cluster_key TEXT NOT NULL UNIQUE,
                created_at INTEGER NOT NULL,
                updated_at INTEGER NOT NULL
            )",
            [],
        )?;

        conn.execute(
            "CREATE TABLE IF NOT EXISTS temporal_event_evidence (
                event_id INTEGER NOT NULL,
                rss_item_id INTEGER NOT NULL,
                weight REAL NOT NULL DEFAULT 1.0,
                snippet TEXT,
                PRIMARY KEY(event_id, rss_item_id),
                FOREIGN KEY (event_id) REFERENCES temporal_events(id) ON DELETE CASCADE,
                FOREIGN KEY (rss_item_id) REFERENCES rss_items(id) ON DELETE CASCADE
            )",
            [],
        )?;

        conn.execute(
            "CREATE TABLE IF NOT EXISTS watchlists (
                id INTEGER PRIMARY KEY AUTOINCREMENT,
                name TEXT NOT NULL UNIQUE,
                created_at INTEGER NOT NULL
            )",
            [],
        )?;

        conn.execute(
            "CREATE TABLE IF NOT EXISTS watchlist_items (
                id INTEGER PRIMARY KEY AUTOINCREMENT,
                watchlist_id INTEGER NOT NULL,
                item_type TEXT NOT NULL,
                value TEXT NOT NULL,
                weight REAL NOT NULL DEFAULT 1.0,
                enabled INTEGER NOT NULL DEFAULT 1,
                created_at INTEGER NOT NULL,
                FOREIGN KEY (watchlist_id) REFERENCES watchlists(id) ON DELETE CASCADE,
                UNIQUE(watchlist_id, item_type, value)
            )",
            [],
        )?;

        conn.execute(
            "CREATE TABLE IF NOT EXISTS alert_rules (
                id INTEGER PRIMARY KEY AUTOINCREMENT,
                name TEXT NOT NULL,
                enabled INTEGER NOT NULL DEFAULT 1,
                watchlist_id INTEGER,
                rule_json TEXT NOT NULL,
                schedule TEXT,
                escalation_config TEXT,
                created_at INTEGER NOT NULL,
                FOREIGN KEY (watchlist_id) REFERENCES watchlists(id) ON DELETE SET NULL
            )",
            [],
        )?;

        conn.execute(
            "CREATE TABLE IF NOT EXISTS alerts (
                id INTEGER PRIMARY KEY AUTOINCREMENT,
                rule_id INTEGER NOT NULL,
                fired_at INTEGER NOT NULL,
                event_id INTEGER,
                payload_json TEXT NOT NULL,
                status TEXT NOT NULL DEFAULT 'new',
                snoozed_until INTEGER,
                FOREIGN KEY (rule_id) REFERENCES alert_rules(id) ON DELETE CASCADE,
                FOREIGN KEY (event_id) REFERENCES temporal_events(id) ON DELETE SET NULL
            )",
            [],
        )?;

        conn.execute(
            "CREATE TABLE IF NOT EXISTS alert_labels (
                alert_id INTEGER PRIMARY KEY,
                label INTEGER NOT NULL,
                note TEXT,
                created_at INTEGER NOT NULL,
                FOREIGN KEY (alert_id) REFERENCES alerts(id) ON DELETE CASCADE
            )",
            [],
        )?;

        conn.execute(
            "CREATE TABLE IF NOT EXISTS alert_escalations (
                id INTEGER PRIMARY KEY AUTOINCREMENT,
                alert_id INTEGER NOT NULL,
                escalated_at INTEGER NOT NULL,
                escalation_level INTEGER NOT NULL,
                channel TEXT NOT NULL,
                sent INTEGER NOT NULL DEFAULT 0,
                error_message TEXT,
                FOREIGN KEY (alert_id) REFERENCES alerts(id) ON DELETE CASCADE
            )",
            [],
        )?;

        conn.execute(
            "CREATE INDEX IF NOT EXISTS idx_escalations_alert ON alert_escalations(alert_id)",
            [],
        )?;

        conn.execute(
            "CREATE TABLE IF NOT EXISTS feature_definitions (
                id INTEGER PRIMARY KEY AUTOINCREMENT,
                name TEXT NOT NULL UNIQUE,
                expression TEXT NOT NULL,
                description TEXT,
                created_at INTEGER NOT NULL
            )",
            [],
        )?;

        conn.execute(
            "CREATE TABLE IF NOT EXISTS feature_values (
                id INTEGER PRIMARY KEY AUTOINCREMENT,
                feature_id INTEGER NOT NULL,
                ts INTEGER NOT NULL,
                subject_type TEXT NOT NULL DEFAULT 'global',
                subject_value TEXT NOT NULL DEFAULT 'global',
                value REAL NOT NULL,
                FOREIGN KEY (feature_id) REFERENCES feature_definitions(id) ON DELETE CASCADE
            )",
            [],
        )?;

        // MVP search index (rebuildable)
        conn.execute(
            "CREATE VIRTUAL TABLE IF NOT EXISTS fts_documents USING fts5(
                doc_type,
                doc_id UNINDEXED,
                title,
                content,
                ts UNINDEXED
            )",
            [],
        )?;

        conn.execute(
            "CREATE INDEX IF NOT EXISTS idx_temporal_events_start_ts ON temporal_events(start_ts)",
            [],
        )?;
        // Migration: Ensure alerts table has required columns
        // Check if alerts table exists and verify its schema
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
            
            let needs_migration = !columns.contains(&"rule_id".to_string()) || 
                                  !columns.contains(&"fired_at".to_string());
            
            if needs_migration {
                // Table exists but has old schema - recreate it
                // Drop dependent objects first
                conn.execute("DROP INDEX IF EXISTS idx_alerts_fired_at", [])?;
                conn.execute("DROP TABLE IF EXISTS alert_labels", [])?;
                conn.execute("DROP TABLE IF EXISTS alert_escalations", [])?;
                
                // Recreate alerts table with correct schema
                conn.execute("DROP TABLE IF EXISTS alerts", [])?;
                conn.execute(
                    "CREATE TABLE alerts (
                        id INTEGER PRIMARY KEY AUTOINCREMENT,
                        rule_id INTEGER NOT NULL,
                        fired_at INTEGER NOT NULL,
                        event_id INTEGER,
                        payload_json TEXT NOT NULL,
                        status TEXT NOT NULL DEFAULT 'new',
                        snoozed_until INTEGER,
                        FOREIGN KEY (rule_id) REFERENCES alert_rules(id) ON DELETE CASCADE,
                        FOREIGN KEY (event_id) REFERENCES temporal_events(id) ON DELETE SET NULL
                    )",
                    [],
                )?;
                
                // Recreate dependent tables
                conn.execute(
                    "CREATE TABLE IF NOT EXISTS alert_labels (
                        alert_id INTEGER PRIMARY KEY,
                        label INTEGER NOT NULL,
                        note TEXT,
                        created_at INTEGER NOT NULL,
                        FOREIGN KEY (alert_id) REFERENCES alerts(id) ON DELETE CASCADE
                    )",
                    [],
                )?;
                
                conn.execute(
                    "CREATE TABLE IF NOT EXISTS alert_escalations (
                        id INTEGER PRIMARY KEY AUTOINCREMENT,
                        alert_id INTEGER NOT NULL,
                        escalated_at INTEGER NOT NULL,
                        escalation_level INTEGER NOT NULL,
                        channel TEXT NOT NULL,
                        sent INTEGER NOT NULL DEFAULT 0,
                        error_message TEXT,
                        FOREIGN KEY (alert_id) REFERENCES alerts(id) ON DELETE CASCADE
                    )",
                    [],
                )?;
                
                conn.execute(
                    "CREATE INDEX IF NOT EXISTS idx_escalations_alert ON alert_escalations(alert_id)",
                    [],
                )?;
            }
        }

        conn.execute(
            "CREATE INDEX IF NOT EXISTS idx_alerts_fired_at ON alerts(fired_at)",
            [],
        )?;
        conn.execute(
            "CREATE INDEX IF NOT EXISTS idx_watchlist_items_watchlist ON watchlist_items(watchlist_id)",
            [],
        )?;

        // Ensure a default watchlist exists
        let now = chrono::Utc::now().timestamp();
        conn.execute(
            "INSERT OR IGNORE INTO watchlists (name, created_at) VALUES (?1, ?2)",
            params!["Default", now],
        )?;

        Ok(())
    }

    pub fn list_events(&self, limit: i64, from_ts: Option<i64>, to_ts: Option<i64>) -> Result<Vec<TemporalEvent>> {
        let conn = self.conn.lock()
            .map_err(|e| anyhow::anyhow!("Database lock poisoned: {}", e))?;
        let row_mapper = |row: &rusqlite::Row<'_>| {
            Ok(TemporalEvent {
                id: row.get(0)?,
                title: row.get(1)?,
                summary: row.get(2)?,
                start_ts: row.get(3)?,
                end_ts: row.get(4)?,
                event_type: row.get(5)?,
                confidence: row.get(6)?,
                severity: row.get(7)?,
                novelty_score: row.get(8)?,
                volume_score: row.get(9)?,
                sentiment_score: row.get(10)?,
                cluster_key: row.get(11)?,
                created_at: row.get(12)?,
                updated_at: row.get(13)?,
            })
        };

        let mut out: Vec<TemporalEvent> = Vec::new();

        match (from_ts, to_ts) {
            (Some(f), Some(t)) => {
                let mut stmt = conn.prepare(
                    "SELECT id, title, summary, start_ts, end_ts, event_type, confidence, severity, novelty_score, volume_score, sentiment_score, cluster_key, created_at, updated_at
                     FROM temporal_events
                     WHERE end_ts >= ?1 AND start_ts <= ?2
                     ORDER BY start_ts DESC
                     LIMIT ?3",
                )?;
                let rows = stmt.query_map(params![f, t, limit], row_mapper)?;
                for r in rows {
                    out.push(r?);
                }
            }
            (Some(f), None) => {
                let mut stmt = conn.prepare(
                    "SELECT id, title, summary, start_ts, end_ts, event_type, confidence, severity, novelty_score, volume_score, sentiment_score, cluster_key, created_at, updated_at
                     FROM temporal_events
                     WHERE end_ts >= ?1
                     ORDER BY start_ts DESC
                     LIMIT ?2",
                )?;
                let rows = stmt.query_map(params![f, limit], row_mapper)?;
                for r in rows {
                    out.push(r?);
                }
            }
            (None, Some(t)) => {
                let mut stmt = conn.prepare(
                    "SELECT id, title, summary, start_ts, end_ts, event_type, confidence, severity, novelty_score, volume_score, sentiment_score, cluster_key, created_at, updated_at
                     FROM temporal_events
                     WHERE start_ts <= ?1
                     ORDER BY start_ts DESC
                     LIMIT ?2",
                )?;
                let rows = stmt.query_map(params![t, limit], row_mapper)?;
                for r in rows {
                    out.push(r?);
                }
            }
            (None, None) => {
                let mut stmt = conn.prepare(
                    "SELECT id, title, summary, start_ts, end_ts, event_type, confidence, severity, novelty_score, volume_score, sentiment_score, cluster_key, created_at, updated_at
                     FROM temporal_events
                     ORDER BY start_ts DESC
                     LIMIT ?1",
                )?;
                let rows = stmt.query_map(params![limit], row_mapper)?;
                for r in rows {
                    out.push(r?);
                }
            }
        }

        Ok(out)
    }

    pub fn get_event(&self, id: i64) -> Result<Option<TemporalEvent>> {
        let conn = self.conn.lock()
            .map_err(|e| anyhow::anyhow!("Database lock poisoned: {}", e))?;
        conn.query_row(
            "SELECT id, title, summary, start_ts, end_ts, event_type, confidence, severity, novelty_score, volume_score, sentiment_score, cluster_key, created_at, updated_at
             FROM temporal_events WHERE id = ?1",
            params![id],
            |row| {
                Ok(TemporalEvent {
                    id: row.get(0)?,
                    title: row.get(1)?,
                    summary: row.get(2)?,
                    start_ts: row.get(3)?,
                    end_ts: row.get(4)?,
                    event_type: row.get(5)?,
                    confidence: row.get(6)?,
                    severity: row.get(7)?,
                    novelty_score: row.get(8)?,
                    volume_score: row.get(9)?,
                    sentiment_score: row.get(10)?,
                    cluster_key: row.get(11)?,
                    created_at: row.get(12)?,
                    updated_at: row.get(13)?,
                })
            },
        )
        .optional()
        .map_err(Into::into)
    }

    pub fn list_event_evidence(&self, event_id: i64) -> Result<Vec<TemporalEventEvidence>> {
        let conn = self.conn.lock()
            .map_err(|e| anyhow::anyhow!("Database lock poisoned: {}", e))?;
        let mut stmt = conn.prepare(
            "SELECT event_id, rss_item_id, weight, snippet
             FROM temporal_event_evidence
             WHERE event_id = ?1
             ORDER BY weight DESC",
        )?;
        let rows = stmt.query_map(params![event_id], |row| {
            Ok(TemporalEventEvidence {
                event_id: row.get(0)?,
                rss_item_id: row.get(1)?,
                weight: row.get(2)?,
                snippet: row.get(3)?,
            })
        })?;
        let mut out = Vec::new();
        for r in rows {
            out.push(r?);
        }
        Ok(out)
    }

    pub fn create_watchlist(&self, name: &str) -> Result<i64> {
        let conn = self.conn.lock()
            .map_err(|e| anyhow::anyhow!("Database lock poisoned: {}", e))?;
        let now = chrono::Utc::now().timestamp();
        conn.execute(
            "INSERT INTO watchlists (name, created_at) VALUES (?1, ?2)",
            params![name, now],
        )?;
        Ok(conn.last_insert_rowid())
    }

    pub fn list_watchlists(&self) -> Result<Vec<Watchlist>> {
        let conn = self.conn.lock()
            .map_err(|e| anyhow::anyhow!("Database lock poisoned: {}", e))?;
        let mut stmt = conn.prepare("SELECT id, name, created_at FROM watchlists ORDER BY name ASC")?;
        let rows = stmt.query_map([], |row| {
            Ok(Watchlist {
                id: row.get(0)?,
                name: row.get(1)?,
                created_at: row.get(2)?,
            })
        })?;
        let mut out = Vec::new();
        for r in rows {
            out.push(r?);
        }
        Ok(out)
    }

    pub fn add_watchlist_item(
        &self,
        watchlist_id: i64,
        item_type: &str,
        value: &str,
        weight: f64,
        enabled: bool,
    ) -> Result<i64> {
        let conn = self.conn.lock()
            .map_err(|e| anyhow::anyhow!("Database lock poisoned: {}", e))?;
        let now = chrono::Utc::now().timestamp();
        conn.execute(
            "INSERT OR REPLACE INTO watchlist_items (watchlist_id, item_type, value, weight, enabled, created_at)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6)",
            params![watchlist_id, item_type, value, weight, if enabled { 1 } else { 0 }, now],
        )?;
        Ok(conn.last_insert_rowid())
    }

    pub fn list_watchlist_items(&self, watchlist_id: i64) -> Result<Vec<WatchlistItem>> {
        let conn = self.conn.lock()
            .map_err(|e| anyhow::anyhow!("Database lock poisoned: {}", e))?;
        let mut stmt = conn.prepare(
            "SELECT id, watchlist_id, item_type, value, weight, enabled, created_at
             FROM watchlist_items WHERE watchlist_id = ?1
             ORDER BY item_type ASC, weight DESC, value ASC",
        )?;
        let rows = stmt.query_map(params![watchlist_id], |row| {
            Ok(WatchlistItem {
                id: row.get(0)?,
                watchlist_id: row.get(1)?,
                item_type: row.get(2)?,
                value: row.get(3)?,
                weight: row.get(4)?,
                enabled: row.get::<_, i64>(5)? == 1,
                created_at: row.get(6)?,
            })
        })?;
        let mut out = Vec::new();
        for r in rows {
            out.push(r?);
        }
        Ok(out)
    }

    pub fn create_alert_rule(
        &self,
        name: &str,
        enabled: bool,
        watchlist_id: Option<i64>,
        rule_json: &Value,
        schedule: Option<&str>,
        escalation_config: Option<&Value>,
    ) -> Result<i64> {
        let conn = self.conn.lock()
            .map_err(|e| anyhow::anyhow!("Database lock poisoned: {}", e))?;
        let now = chrono::Utc::now().timestamp();
        let escalation_config_str = escalation_config.map(|v| v.to_string());
        conn.execute(
            "INSERT INTO alert_rules (name, enabled, watchlist_id, rule_json, schedule, escalation_config, created_at)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7)",
            params![name, if enabled { 1 } else { 0 }, watchlist_id, rule_json.to_string(), schedule, escalation_config_str, now],
        )?;
        Ok(conn.last_insert_rowid())
    }

    pub fn list_alert_rules(&self) -> Result<Vec<AlertRule>> {
        let conn = self.conn.lock()
            .map_err(|e| anyhow::anyhow!("Database lock poisoned: {}", e))?;
        let mut stmt = conn.prepare(
            "SELECT id, name, enabled, watchlist_id, rule_json, schedule, escalation_config, created_at
             FROM alert_rules ORDER BY created_at DESC",
        )?;
        let rows = stmt.query_map([], |row| {
            let rule_json_str: String = row.get(4)?;
            let rule_json: Value = serde_json::from_str(&rule_json_str).unwrap_or(Value::Null);
            let escalation_config_str: Option<String> = row.get(6)?;
            let escalation_config = escalation_config_str
                .and_then(|s| serde_json::from_str(&s).ok());
            Ok(AlertRule {
                id: row.get(0)?,
                name: row.get(1)?,
                enabled: row.get::<_, i64>(2)? == 1,
                watchlist_id: row.get(3)?,
                rule_json,
                schedule: row.get(5)?,
                escalation_config,
                created_at: row.get(7)?,
            })
        })?;
        let mut out = Vec::new();
        for r in rows {
            out.push(r?);
        }
        Ok(out)
    }

    pub fn list_alerts(&self, limit: i64, from_ts: Option<i64>, to_ts: Option<i64>) -> Result<Vec<Alert>> {
        let conn = self.conn.lock()
            .map_err(|e| anyhow::anyhow!("Database lock poisoned: {}", e))?;
        let row_mapper = |row: &rusqlite::Row<'_>| {
            let payload_str: String = row.get(4)?;
            let payload_json: Value = serde_json::from_str(&payload_str)
                .unwrap_or_else(|_| Value::Null);
            Ok(Alert {
                id: row.get(0)?,
                rule_id: row.get(1)?,
                fired_at: row.get(2)?,
                event_id: row.get(3)?,
                payload_json,
                status: row.get(5)?,
                snoozed_until: row.get(6)?,
            })
        };

        let mut out: Vec<Alert> = Vec::new();
        match (from_ts, to_ts) {
            (Some(f), Some(t)) => {
                let mut stmt = conn.prepare(
                    "SELECT id, rule_id, fired_at, event_id, payload_json, status, snoozed_until
                     FROM alerts
                     WHERE fired_at >= ?1 AND fired_at <= ?2
                     ORDER BY fired_at DESC
                     LIMIT ?3",
                )?;
                let rows = stmt.query_map(params![f, t, limit], row_mapper)?;
                for r in rows {
                    out.push(r?);
                }
            }
            (Some(f), None) => {
                let mut stmt = conn.prepare(
                    "SELECT id, rule_id, fired_at, event_id, payload_json, status, snoozed_until
                     FROM alerts
                     WHERE fired_at >= ?1
                     ORDER BY fired_at DESC
                     LIMIT ?2",
                )?;
                let rows = stmt.query_map(params![f, limit], row_mapper)?;
                for r in rows {
                    out.push(r?);
                }
            }
            (None, Some(t)) => {
                let mut stmt = conn.prepare(
                    "SELECT id, rule_id, fired_at, event_id, payload_json, status, snoozed_until
                     FROM alerts
                     WHERE fired_at <= ?1
                     ORDER BY fired_at DESC
                     LIMIT ?2",
                )?;
                let rows = stmt.query_map(params![t, limit], row_mapper)?;
                for r in rows {
                    out.push(r?);
                }
            }
            (None, None) => {
                let mut stmt = conn.prepare(
                    "SELECT id, rule_id, fired_at, event_id, payload_json, status, snoozed_until
                     FROM alerts
                     ORDER BY fired_at DESC
                     LIMIT ?1",
                )?;
                let rows = stmt.query_map(params![limit], row_mapper)?;
                for r in rows {
                    out.push(r?);
                }
            }
        }

        Ok(out)
    }

    pub fn update_alert_status(&self, alert_id: i64, status: &str, snoozed_until: Option<i64>) -> Result<()> {
        let conn = self.conn.lock()
            .map_err(|e| anyhow::anyhow!("Database lock poisoned: {}", e))?;
        conn.execute(
            "UPDATE alerts SET status = ?1, snoozed_until = ?2 WHERE id = ?3",
            params![status, snoozed_until, alert_id],
        )?;
        Ok(())
    }

    fn check_alert_escalation(&self, alert: &Alert, rule: &AlertRule) -> Result<()> {
        // Spawn async task to check and escalate alert
        // We clone what we need since we're in a sync context
        let store_clone = TemporalStore::new(self.conn.clone());
        let alert_clone = alert.clone();
        let rule_clone = rule.clone();
        
        // Spawn async task for escalation (no AppHandle available here, so desktop notifications won't work)
        tauri::async_runtime::spawn(async move {
            use crate::services::alert_escalator::AlertEscalator;
            if let Err(e) = AlertEscalator::check_and_escalate(&store_clone, &alert_clone, &rule_clone, None).await {
                eprintln!("Failed to escalate alert {}: {}", alert_clone.id, e);
            }
        });
        
        Ok(())
    }

    fn create_alert_if_new(&self, rule_id: i64, event_id: Option<i64>, payload_json: &Value) -> Result<Option<Alert>> {
        let conn = self.conn.lock()
            .map_err(|e| anyhow::anyhow!("Database lock poisoned: {}", e))?;
        let now = chrono::Utc::now().timestamp();

        // Avoid spamming duplicates: if an alert exists for same rule+event in last 6 hours and not resolved, skip.
        if let Some(eid) = event_id {
            let existing: Option<i64> = conn
                .query_row(
                    "SELECT id FROM alerts
                     WHERE rule_id = ?1 AND event_id = ?2 AND fired_at >= ?3 AND status != 'resolved'
                     ORDER BY fired_at DESC
                     LIMIT 1",
                    params![rule_id, eid, now - 6 * 3600],
                    |row| row.get(0),
                )
                .optional()?;
            if existing.is_some() {
                return Ok(None);
            }
        }

        conn.execute(
            "INSERT INTO alerts (rule_id, fired_at, event_id, payload_json, status)
             VALUES (?1, ?2, ?3, ?4, 'new')",
            params![rule_id, now, event_id, payload_json.to_string()],
        )?;
        let id = conn.last_insert_rowid();
        Ok(Some(Alert {
            id,
            rule_id,
            fired_at: now,
            event_id,
            payload_json: payload_json.clone(),
            status: "new".to_string(),
            snoozed_until: None,
        }))
    }

    pub fn evaluate_alert_rules_mvp(&self, days_back: i64, limit_events: i64) -> Result<Vec<Alert>> {
        let conn = self.conn.lock()
            .map_err(|e| anyhow::anyhow!("Database lock poisoned: {}", e))?;
        let now = chrono::Utc::now().timestamp();
        let from_ts = now - days_back.max(1).min(365) * 24 * 3600;

        // Load enabled rules
        let mut rules_stmt = conn.prepare(
            "SELECT id, name, enabled, watchlist_id, rule_json, schedule, escalation_config, created_at
             FROM alert_rules
             WHERE enabled = 1
             ORDER BY created_at DESC",
        )?;
        let rule_rows = rules_stmt.query_map([], |row| {
            let rule_json_str: String = row.get(4)?;
            let rule_json: Value = serde_json::from_str(&rule_json_str).unwrap_or(Value::Null);
            let escalation_config_str: Option<String> = row.get(6)?;
            let escalation_config = escalation_config_str
                .and_then(|s| serde_json::from_str(&s).ok());
            Ok(AlertRule {
                id: row.get(0)?,
                name: row.get(1)?,
                enabled: row.get::<_, i64>(2)? == 1,
                watchlist_id: row.get(3)?,
                rule_json,
                schedule: row.get(5)?,
                escalation_config,
                created_at: row.get(7)?,
            })
        })?;
        let mut rules: Vec<AlertRule> = Vec::new();
        for r in rule_rows {
            rules.push(r?);
        }

        if rules.is_empty() {
            return Ok(Vec::new());
        }

        // Load recent events
        let mut events_stmt = conn.prepare(
            "SELECT id, title, summary, start_ts, end_ts, event_type, confidence, severity, novelty_score, volume_score, sentiment_score, cluster_key, created_at, updated_at
             FROM temporal_events
             WHERE end_ts >= ?1
             ORDER BY start_ts DESC
             LIMIT ?2",
        )?;
        let event_rows = events_stmt.query_map(params![from_ts, limit_events], |row| {
            Ok(TemporalEvent {
                id: row.get(0)?,
                title: row.get(1)?,
                summary: row.get(2)?,
                start_ts: row.get(3)?,
                end_ts: row.get(4)?,
                event_type: row.get(5)?,
                confidence: row.get(6)?,
                severity: row.get(7)?,
                novelty_score: row.get(8)?,
                volume_score: row.get(9)?,
                sentiment_score: row.get(10)?,
                cluster_key: row.get(11)?,
                created_at: row.get(12)?,
                updated_at: row.get(13)?,
            })
        })?;

        let mut created: Vec<Alert> = Vec::new();

        for er in event_rows {
            let event = er?;

            // Entities for event (derived from evidence -> extracted_entities)
            let mut ent_stmt = conn.prepare(
                "SELECT DISTINCT ee.name
                 FROM temporal_event_evidence te
                 JOIN extracted_entities ee ON ee.article_id = te.rss_item_id
                 WHERE te.event_id = ?1",
            )?;
            let ent_rows = ent_stmt.query_map(params![event.id], |row| Ok(row.get::<_, String>(0)?))?;
            let mut entities: HashSet<String> = HashSet::new();
            for e in ent_rows {
                entities.insert(e?.to_lowercase());
            }

            // Sources for event (rss_feeds.name)
            let mut src_stmt = conn.prepare(
                "SELECT DISTINCT f.name
                 FROM temporal_event_evidence te
                 JOIN rss_items i ON i.id = te.rss_item_id
                 JOIN rss_feeds f ON f.id = i.feed_id
                 WHERE te.event_id = ?1",
            )?;
            let src_rows = src_stmt.query_map(params![event.id], |row| Ok(row.get::<_, String>(0)?))?;
            let mut sources: HashSet<String> = HashSet::new();
            for s in src_rows {
                sources.insert(s?.to_lowercase());
            }

            let haystack = format!("{} {}", event.title.to_lowercase(), event.summary.to_lowercase());

            for rule in &rules {
                if rule.rule_json.is_null() {
                    continue;
                }
                if rule_matches_mvp(&rule.rule_json, &haystack, &entities, &sources, &event) {
                    let payload = serde_json::json!({
                        "rule": { "id": rule.id, "name": rule.name },
                        "event": { "id": event.id, "title": event.title, "start_ts": event.start_ts, "end_ts": event.end_ts },
                        "scores": { "sentiment": event.sentiment_score, "novelty": event.novelty_score, "volume": event.volume_score }
                    });
                    if let Some(alert) = self.create_alert_if_new(rule.id, Some(event.id), &payload)? {
                        // Trigger escalation check for new alert
                        if let Err(e) = self.check_alert_escalation(&alert, &rule) {
                            eprintln!("Failed to check escalation for alert {}: {}", alert.id, e);
                        }
                        created.push(alert);
                    }
                }
            }
        }

        Ok(created)
    }

    pub fn rebuild_search_index(&self, from_ts: Option<i64>) -> Result<i64> {
        let conn = self.conn.lock()
            .map_err(|e| anyhow::anyhow!("Database lock poisoned: {}", e))?;
        conn.execute("DELETE FROM fts_documents", [])?;

        // Index RSS items
        let mut inserted = 0i64;
        if let Some(ts) = from_ts {
            let mut stmt = conn.prepare(
                "SELECT id, title, content, published_at FROM rss_items WHERE published_at >= ?1 ORDER BY published_at DESC",
            )?;
            let rows = stmt.query_map(params![ts], |row| {
                Ok((
                    row.get::<_, i64>(0)?,
                    row.get::<_, String>(1)?,
                    row.get::<_, String>(2)?,
                    row.get::<_, i64>(3)?,
                ))
            })?;
            for r in rows {
                let (id, title, content, ts) = r?;
                conn.execute(
                    "INSERT INTO fts_documents (doc_type, doc_id, title, content, ts) VALUES (?1, ?2, ?3, ?4, ?5)",
                    params!["rss_item", id, title, content, ts],
                )?;
                inserted += 1;
            }
        } else {
            let mut stmt = conn.prepare(
                "SELECT id, title, content, published_at FROM rss_items ORDER BY published_at DESC",
            )?;
            let rows = stmt.query_map([], |row| {
                Ok((
                    row.get::<_, i64>(0)?,
                    row.get::<_, String>(1)?,
                    row.get::<_, String>(2)?,
                    row.get::<_, i64>(3)?,
                ))
            })?;
            for r in rows {
                let (id, title, content, ts) = r?;
                conn.execute(
                    "INSERT INTO fts_documents (doc_type, doc_id, title, content, ts) VALUES (?1, ?2, ?3, ?4, ?5)",
                    params!["rss_item", id, title, content, ts],
                )?;
                inserted += 1;
            }
        }

        // Index Temporal events
        let mut evt_stmt = conn.prepare(
            "SELECT id, title, summary, start_ts FROM temporal_events ORDER BY start_ts DESC",
        )?;
        let evt_rows = evt_stmt.query_map([], |row| {
            Ok((row.get::<_, i64>(0)?, row.get::<_, String>(1)?, row.get::<_, String>(2)?, row.get::<_, i64>(3)?))
        })?;
        for r in evt_rows {
            let (id, title, summary, ts) = r?;
            conn.execute(
                "INSERT INTO fts_documents (doc_type, doc_id, title, content, ts) VALUES (?1, ?2, ?3, ?4, ?5)",
                params!["temporal_event", id, title, summary, ts],
            )?;
            inserted += 1;
        }

        Ok(inserted)
    }

    pub fn search(&self, query: &str, limit: i64) -> Result<Vec<Value>> {
        let conn = self.conn.lock()
            .map_err(|e| anyhow::anyhow!("Database lock poisoned: {}", e))?;
        let mut stmt = conn.prepare(
            "SELECT doc_type, doc_id, title, snippet(fts_documents, 3, '[', ']', '…', 12) as snippet, ts
             FROM fts_documents
             WHERE fts_documents MATCH ?1
             ORDER BY ts DESC
             LIMIT ?2",
        )?;
        let rows = stmt.query_map(params![query, limit], |row| {
            Ok(serde_json::json!({
                "doc_type": row.get::<_, String>(0)?,
                "doc_id": row.get::<_, i64>(1)?,
                "title": row.get::<_, String>(2)?,
                "snippet": row.get::<_, String>(3)?,
                "ts": row.get::<_, i64>(4)?,
            }))
        })?;
        let mut out = Vec::new();
        for r in rows {
            out.push(r?);
        }
        Ok(out)
    }

    // =========================
    // Event formation (MVP)
    // =========================
    pub fn rebuild_events_mvp(&self, days_back: i64) -> Result<i64> {
        let conn = self.conn.lock()
            .map_err(|e| anyhow::anyhow!("Database lock poisoned: {}", e))?;
        let now = chrono::Utc::now().timestamp();
        let from_ts = now - days_back * 24 * 3600;

        // Strategy:
        // - For each rss_item in range, find its top extracted entity (by confidence)
        // - Cluster key = YYYY-MM-DD + '|' + top_entity_name (or 'misc')
        // - Upsert event per cluster key, attach evidence rows
        let mut stmt = conn.prepare(
            "SELECT id, title, content, published_at
             FROM rss_items
             WHERE published_at >= ?1
             ORDER BY published_at DESC",
        )?;
        let rows = stmt.query_map(params![from_ts], |row| {
            Ok((
                row.get::<_, i64>(0)?,
                row.get::<_, String>(1)?,
                row.get::<_, String>(2)?,
                row.get::<_, i64>(3)?,
            ))
        })?;

        let mut touched_events = 0i64;
        for r in rows {
            let (rss_id, title, content, published_at) = r?;

            let top_entity: Option<String> = conn
                .query_row(
                    "SELECT name
                     FROM extracted_entities
                     WHERE article_id = ?1
                     ORDER BY confidence DESC
                     LIMIT 1",
                    params![rss_id],
                    |row| row.get(0),
                )
                .optional()?;

            let date_key = chrono::NaiveDateTime::from_timestamp_opt(published_at, 0)
                .unwrap_or_else(|| chrono::Utc::now().naive_utc())
                .date()
                .format("%Y-%m-%d")
                .to_string();

            let entity_key = top_entity.clone().unwrap_or_else(|| "misc".to_string());
            let cluster_key = format!("{}|{}", date_key, entity_key);

            // Sentiment (very light)
            let sentiment_score = compute_sentiment_light(&content);

            // Upsert event
            let existing: Option<(i64, i64, i64)> = conn
                .query_row(
                    "SELECT id, start_ts, end_ts FROM temporal_events WHERE cluster_key = ?1",
                    params![cluster_key],
                    |row| Ok((row.get(0)?, row.get(1)?, row.get(2)?)),
                )
                .optional()?;

            let event_id = if let Some((eid, start_ts, end_ts)) = existing {
                let new_start = std::cmp::min(start_ts, published_at);
                let new_end = std::cmp::max(end_ts, published_at);
                conn.execute(
                    "UPDATE temporal_events
                     SET end_ts = ?1,
                         start_ts = ?2,
                         updated_at = ?3,
                         volume_score = volume_score + 1,
                         sentiment_score = (sentiment_score + ?4) / 2.0
                     WHERE id = ?5",
                    params![new_end, new_start, now, sentiment_score, eid],
                )?;
                eid
            } else {
                let summary = summarize_light(&title, &content);
                let event_title = format!("{}: {}", entity_key, truncate(&title, 80));
                conn.execute(
                    "INSERT INTO temporal_events
                     (title, summary, start_ts, end_ts, event_type, confidence, severity, novelty_score, volume_score, sentiment_score, cluster_key, created_at, updated_at)
                     VALUES (?1, ?2, ?3, ?4, 'news', 0.5, 0.0, 0.0, 1.0, ?5, ?6, ?7, ?8)",
                    params![event_title, summary, published_at, published_at, sentiment_score, cluster_key, now, now],
                )?;
                touched_events += 1;
                conn.last_insert_rowid()
            };

            // Evidence link (idempotent)
            let snippet = Some(truncate(&content, 220));
            conn.execute(
                "INSERT OR IGNORE INTO temporal_event_evidence (event_id, rss_item_id, weight, snippet)
                 VALUES (?1, ?2, 1.0, ?3)",
                params![event_id, rss_id, snippet],
            )?;
        }

        // Recompute novelty score (unique entities count / 10 capped)
        let mut evt_stmt = conn.prepare("SELECT id FROM temporal_events WHERE updated_at >= ?1")?;
        let evt_rows = evt_stmt.query_map(params![from_ts], |row| Ok(row.get::<_, i64>(0)?))?;
        for r in evt_rows {
            let eid = r?;
            let mut ent_stmt = conn.prepare(
                "SELECT DISTINCT ee.name
                 FROM temporal_event_evidence te
                 JOIN extracted_entities ee ON ee.article_id = te.rss_item_id
                 WHERE te.event_id = ?1",
            )?;
            let ent_rows = ent_stmt.query_map(params![eid], |row| Ok(row.get::<_, String>(0)?))?;
            let mut uniq = HashSet::new();
            for e in ent_rows {
                uniq.insert(e?);
            }
            let novelty = (uniq.len() as f64 / 10.0).min(1.0);
            conn.execute(
                "UPDATE temporal_events SET novelty_score = ?1 WHERE id = ?2",
                params![novelty, eid],
            )?;
        }

        Ok(touched_events)
    }

    pub fn run_backtest_mvp(&self, from_ts: i64, to_ts: i64) -> Result<BacktestReport> {
        let conn = self.conn.lock()
            .map_err(|e| anyhow::anyhow!("Database lock poisoned: {}", e))?;
        let total_alerts: i64 = conn.query_row(
            "SELECT COUNT(*) FROM alerts WHERE fired_at BETWEEN ?1 AND ?2",
            params![from_ts, to_ts],
            |row| row.get(0),
        )?;
        let acked_alerts: i64 = conn.query_row(
            "SELECT COUNT(*) FROM alerts WHERE status = 'ack' AND fired_at BETWEEN ?1 AND ?2",
            params![from_ts, to_ts],
            |row| row.get(0),
        )?;
        let snoozed_alerts: i64 = conn.query_row(
            "SELECT COUNT(*) FROM alerts WHERE status = 'snoozed' AND fired_at BETWEEN ?1 AND ?2",
            params![from_ts, to_ts],
            |row| row.get(0),
        )?;
        let resolved_alerts: i64 = conn.query_row(
            "SELECT COUNT(*) FROM alerts WHERE status = 'resolved' AND fired_at BETWEEN ?1 AND ?2",
            params![from_ts, to_ts],
            |row| row.get(0),
        )?;

        let mut stmt = conn.prepare(
            "SELECT rule_id, COUNT(*) FROM alerts
             WHERE fired_at BETWEEN ?1 AND ?2
             GROUP BY rule_id",
        )?;
        let rows = stmt.query_map(params![from_ts, to_ts], |row| Ok((row.get::<_, i64>(0)?, row.get::<_, i64>(1)?)))?;
        let mut by_rule = HashMap::new();
        for r in rows {
            let (rule_id, count) = r?;
            by_rule.insert(rule_id, count);
        }

        let helpful_alerts: i64 = conn.query_row(
            "SELECT COUNT(*) FROM alert_labels l
             JOIN alerts a ON a.id = l.alert_id
             WHERE l.label = 1 AND a.fired_at BETWEEN ?1 AND ?2",
            params![from_ts, to_ts],
            |row| row.get(0),
        )?;
        let unhelpful_alerts: i64 = conn.query_row(
            "SELECT COUNT(*) FROM alert_labels l
             JOIN alerts a ON a.id = l.alert_id
             WHERE l.label = -1 AND a.fired_at BETWEEN ?1 AND ?2",
            params![from_ts, to_ts],
            |row| row.get(0),
        )?;

        let mut by_rule_helpful: HashMap<i64, i64> = HashMap::new();
        let mut by_rule_unhelpful: HashMap<i64, i64> = HashMap::new();

        let mut hstmt = conn.prepare(
            "SELECT a.rule_id, COUNT(*)
             FROM alert_labels l
             JOIN alerts a ON a.id = l.alert_id
             WHERE l.label = 1 AND a.fired_at BETWEEN ?1 AND ?2
             GROUP BY a.rule_id",
        )?;
        let hrows = hstmt.query_map(params![from_ts, to_ts], |row| Ok((row.get::<_, i64>(0)?, row.get::<_, i64>(1)?)))?;
        for r in hrows {
            let (rid, cnt) = r?;
            by_rule_helpful.insert(rid, cnt);
        }

        let mut ustmt = conn.prepare(
            "SELECT a.rule_id, COUNT(*)
             FROM alert_labels l
             JOIN alerts a ON a.id = l.alert_id
             WHERE l.label = -1 AND a.fired_at BETWEEN ?1 AND ?2
             GROUP BY a.rule_id",
        )?;
        let urows = ustmt.query_map(params![from_ts, to_ts], |row| Ok((row.get::<_, i64>(0)?, row.get::<_, i64>(1)?)))?;
        for r in urows {
            let (rid, cnt) = r?;
            by_rule_unhelpful.insert(rid, cnt);
        }

        Ok(BacktestReport {
            from_ts,
            to_ts,
            total_alerts,
            acked_alerts,
            snoozed_alerts,
            resolved_alerts,
            helpful_alerts,
            unhelpful_alerts,
            by_rule,
            by_rule_helpful,
            by_rule_unhelpful,
        })
    }

    pub fn set_alert_label(&self, alert_id: i64, label: i64, note: Option<&str>) -> Result<()> {
        let conn = self.conn.lock()
            .map_err(|e| anyhow::anyhow!("Database lock poisoned: {}", e))?;
        let now = chrono::Utc::now().timestamp();
        let label = if label >= 1 { 1 } else { -1 };
        conn.execute(
            "INSERT OR REPLACE INTO alert_labels (alert_id, label, note, created_at)
             VALUES (?1, ?2, ?3, ?4)",
            params![alert_id, label, note, now],
        )?;
        Ok(())
    }

    pub fn get_alert_label(&self, alert_id: i64) -> Result<Option<AlertLabel>> {
        let conn = self.conn.lock()
            .map_err(|e| anyhow::anyhow!("Database lock poisoned: {}", e))?;
        conn.query_row(
            "SELECT alert_id, label, note, created_at FROM alert_labels WHERE alert_id = ?1",
            params![alert_id],
            |row| {
                Ok(AlertLabel {
                    alert_id: row.get(0)?,
                    label: row.get(1)?,
                    note: row.get(2)?,
                    created_at: row.get(3)?,
                })
            },
        )
        .optional()
        .map_err(Into::into)
    }

    pub fn get_entity_graph_mvp(&self, days_back: i64, max_nodes: usize, max_edges: usize) -> Result<EntityGraph> {
        let conn = self.conn.lock()
            .map_err(|e| anyhow::anyhow!("Database lock poisoned: {}", e))?;
        let now = chrono::Utc::now().timestamp();
        let from_ts = now - days_back.max(1).min(365) * 24 * 3600;

        // Collect entities per article (limited by time)
        let mut stmt = conn.prepare(
            "SELECT i.id as article_id, ee.name as entity_name
             FROM rss_items i
             JOIN extracted_entities ee ON ee.article_id = i.id
             WHERE i.published_at >= ?1",
        )?;
        let rows = stmt.query_map(params![from_ts], |row| {
            Ok((row.get::<_, i64>(0)?, row.get::<_, String>(1)?))
        })?;

        let mut by_article: HashMap<i64, Vec<String>> = HashMap::new();
        let mut node_counts: HashMap<String, i64> = HashMap::new();

        for r in rows {
            let (article_id, name) = r?;
            let key = name.to_lowercase();
            by_article.entry(article_id).or_default().push(key.clone());
            *node_counts.entry(key).or_insert(0) += 1;
        }

        // Build co-mention edges
        let mut edge_counts: HashMap<(String, String), i64> = HashMap::new();
        for (_article_id, mut ents) in by_article {
            ents.sort();
            ents.dedup();
            for i in 0..ents.len() {
                for j in (i + 1)..ents.len() {
                    let a = ents[i].clone();
                    let b = ents[j].clone();
                    let (s, t) = if a <= b { (a, b) } else { (b, a) };
                    *edge_counts.entry((s, t)).or_insert(0) += 1;
                }
            }
        }

        // Select top nodes
        let mut nodes_vec: Vec<(String, i64)> = node_counts.into_iter().collect();
        nodes_vec.sort_by(|a, b| b.1.cmp(&a.1));
        nodes_vec.truncate(max_nodes.max(10));

        let allowed: HashSet<String> = nodes_vec.iter().map(|(k, _)| k.clone()).collect();

        let nodes: Vec<EntityGraphNode> = nodes_vec
            .into_iter()
            .map(|(id, count)| EntityGraphNode {
                label: id.clone(),
                id,
                count,
            })
            .collect();

        // Select top edges among allowed nodes
        let mut edges_vec: Vec<((String, String), i64)> = edge_counts
            .into_iter()
            .filter(|((s, t), _)| allowed.contains(s) && allowed.contains(t))
            .collect();
        edges_vec.sort_by(|a, b| b.1.cmp(&a.1));
        edges_vec.truncate(max_edges.max(10));

        let edges: Vec<EntityGraphEdge> = edges_vec
            .into_iter()
            .map(|((source, target), weight)| EntityGraphEdge { source, target, weight })
            .collect();

        Ok(EntityGraph { nodes, edges })
    }

    // =========================
    // Workbench: Features (MVP)
    // =========================
    pub fn create_feature_definition(&self, name: &str, expression: &str, description: Option<&str>) -> Result<i64> {
        let conn = self.conn.lock()
            .map_err(|e| anyhow::anyhow!("Database lock poisoned: {}", e))?;
        let now = chrono::Utc::now().timestamp();
        conn.execute(
            "INSERT INTO feature_definitions (name, expression, description, created_at)
             VALUES (?1, ?2, ?3, ?4)",
            params![name, expression, description, now],
        )?;
        Ok(conn.last_insert_rowid())
    }

    pub fn list_feature_definitions(&self) -> Result<Vec<FeatureDefinition>> {
        let conn = self.conn.lock()
            .map_err(|e| anyhow::anyhow!("Database lock poisoned: {}", e))?;
        let mut stmt = conn.prepare(
            "SELECT id, name, expression, description, created_at
             FROM feature_definitions
             ORDER BY created_at DESC",
        )?;
        let rows = stmt.query_map([], |row| {
            Ok(FeatureDefinition {
                id: row.get(0)?,
                name: row.get(1)?,
                expression: row.get(2)?,
                description: row.get(3)?,
                created_at: row.get(4)?,
            })
        })?;
        let mut out = Vec::new();
        for r in rows {
            out.push(r?);
        }
        Ok(out)
    }

    pub fn list_feature_values(&self, feature_id: i64, from_ts: Option<i64>, to_ts: Option<i64>, limit: i64) -> Result<Vec<FeatureValue>> {
        let conn = self.conn.lock()
            .map_err(|e| anyhow::anyhow!("Database lock poisoned: {}", e))?;
        let limit = limit.max(1).min(5000);

        let mut out: Vec<FeatureValue> = Vec::new();
        match (from_ts, to_ts) {
            (Some(f), Some(t)) => {
                let mut stmt = conn.prepare(
                    "SELECT id, feature_id, ts, subject_type, subject_value, value
                     FROM feature_values
                     WHERE feature_id = ?1 AND ts BETWEEN ?2 AND ?3
                     ORDER BY ts ASC
                     LIMIT ?4",
                )?;
                let rows = stmt.query_map(params![feature_id, f, t, limit], |row| {
                    Ok(FeatureValue {
                        id: row.get(0)?,
                        feature_id: row.get(1)?,
                        ts: row.get(2)?,
                        subject_type: row.get(3)?,
                        subject_value: row.get(4)?,
                        value: row.get(5)?,
                    })
                })?;
                for r in rows {
                    out.push(r?);
                }
            }
            (Some(f), None) => {
                let mut stmt = conn.prepare(
                    "SELECT id, feature_id, ts, subject_type, subject_value, value
                     FROM feature_values
                     WHERE feature_id = ?1 AND ts >= ?2
                     ORDER BY ts ASC
                     LIMIT ?3",
                )?;
                let rows = stmt.query_map(params![feature_id, f, limit], |row| {
                    Ok(FeatureValue {
                        id: row.get(0)?,
                        feature_id: row.get(1)?,
                        ts: row.get(2)?,
                        subject_type: row.get(3)?,
                        subject_value: row.get(4)?,
                        value: row.get(5)?,
                    })
                })?;
                for r in rows {
                    out.push(r?);
                }
            }
            (None, Some(t)) => {
                let mut stmt = conn.prepare(
                    "SELECT id, feature_id, ts, subject_type, subject_value, value
                     FROM feature_values
                     WHERE feature_id = ?1 AND ts <= ?2
                     ORDER BY ts ASC
                     LIMIT ?3",
                )?;
                let rows = stmt.query_map(params![feature_id, t, limit], |row| {
                    Ok(FeatureValue {
                        id: row.get(0)?,
                        feature_id: row.get(1)?,
                        ts: row.get(2)?,
                        subject_type: row.get(3)?,
                        subject_value: row.get(4)?,
                        value: row.get(5)?,
                    })
                })?;
                for r in rows {
                    out.push(r?);
                }
            }
            (None, None) => {
                let mut stmt = conn.prepare(
                    "SELECT id, feature_id, ts, subject_type, subject_value, value
                     FROM feature_values
                     WHERE feature_id = ?1
                     ORDER BY ts ASC
                     LIMIT ?2",
                )?;
                let rows = stmt.query_map(params![feature_id, limit], |row| {
                    Ok(FeatureValue {
                        id: row.get(0)?,
                        feature_id: row.get(1)?,
                        ts: row.get(2)?,
                        subject_type: row.get(3)?,
                        subject_value: row.get(4)?,
                        value: row.get(5)?,
                    })
                })?;
                for r in rows {
                    out.push(r?);
                }
            }
        }
        Ok(out)
    }

    // Expression DSL (MVP):
    // - alerts_count(<days>)
    // - events_count(<days>)
    // - avg_sentiment(<days>)
    // Materializes daily buckets into feature_values.
    pub fn compute_feature_mvp(&self, feature_id: i64, days_back: i64) -> Result<i64> {
        let conn = self.conn.lock()
            .map_err(|e| anyhow::anyhow!("Database lock poisoned: {}", e))?;
        let days_back = days_back.max(1).min(365);

        let def: FeatureDefinition = conn.query_row(
            "SELECT id, name, expression, description, created_at FROM feature_definitions WHERE id = ?1",
            params![feature_id],
            |row| {
                Ok(FeatureDefinition {
                    id: row.get(0)?,
                    name: row.get(1)?,
                    expression: row.get(2)?,
                    description: row.get(3)?,
                    created_at: row.get(4)?,
                })
            },
        )?;

        // Clear existing values for this feature in the computed range (simple strategy)
        let now = chrono::Utc::now().timestamp();
        let from_ts = now - days_back * 24 * 3600;
        conn.execute(
            "DELETE FROM feature_values WHERE feature_id = ?1 AND ts >= ?2",
            params![feature_id, from_ts],
        )?;

        let mut inserted = 0i64;
        for day in (0..days_back).rev() {
            let day_end = now - day * 24 * 3600;
            let day_start = day_end - 24 * 3600;

            let value = if def.expression.starts_with("alerts_count") {
                conn.query_row(
                    "SELECT COUNT(*) FROM alerts WHERE fired_at BETWEEN ?1 AND ?2",
                    params![day_start, day_end],
                    |row| row.get::<_, i64>(0),
                )? as f64
            } else if def.expression.starts_with("events_count") {
                conn.query_row(
                    "SELECT COUNT(*) FROM temporal_events WHERE start_ts BETWEEN ?1 AND ?2",
                    params![day_start, day_end],
                    |row| row.get::<_, i64>(0),
                )? as f64
            } else if def.expression.starts_with("avg_sentiment") {
                conn.query_row(
                    "SELECT COALESCE(AVG(sentiment_score), 0.0) FROM temporal_events WHERE start_ts BETWEEN ?1 AND ?2",
                    params![day_start, day_end],
                    |row| row.get::<_, f64>(0),
                )?
            } else {
                0.0
            };

            conn.execute(
                "INSERT INTO feature_values (feature_id, ts, subject_type, subject_value, value)
                 VALUES (?1, ?2, 'global', 'global', ?3)",
                params![feature_id, day_end, value],
            )?;
            inserted += 1;
        }

        Ok(inserted)
    }
}

fn truncate(s: &str, max: usize) -> String {
    let mut out = s.trim().to_string();
    if out.len() > max {
        out.truncate(max);
        out.push_str("…");
    }
    out
}

fn summarize_light(title: &str, content: &str) -> String {
    let cleaned = content.replace('\n', " ").replace('\r', " ");
    format!("{} — {}", truncate(title, 90), truncate(&cleaned, 280))
}

fn compute_sentiment_light(content: &str) -> f64 {
    // Very lightweight lexicon scoring: [-1, 1]
    // This is intentionally simple; we can improve later.
    let pos = ["beat", "beats", "surge", "record", "growth", "upgrade", "bullish", "profit", "profits", "win", "strong"];
    let neg = ["miss", "misses", "plunge", "fraud", "probe", "lawsuit", "downgrade", "bearish", "loss", "losses", "weak", "default"];
    let lower = content.to_lowercase();
    let mut score: f64 = 0.0;
    for w in pos {
        if lower.contains(w) {
            score += 1.0;
        }
    }
    for w in neg {
        if lower.contains(w) {
            score -= 1.0;
        }
    }
    if score > 0.0 {
        (score / 5.0).min(1.0_f64)
    } else if score < 0.0 {
        (score / 5.0).max(-1.0_f64)
    } else {
        0.0
    }
}

fn rule_matches_mvp(
    rule_json: &Value,
    haystack_lower: &str,
    entities_lower: &HashSet<String>,
    sources_lower: &HashSet<String>,
    event: &TemporalEvent,
) -> bool {
    // Try enhanced rule engine first
    use crate::services::alert_rule_engine::AlertRuleEngine;
    if let Ok(result) = AlertRuleEngine::rule_matches(rule_json, haystack_lower, entities_lower, sources_lower, event) {
        return result;
    }
    
    // Fallback to legacy MVP format
    // Expected format:
    // { "any": [cond...], "all": [cond...] }
    // Each cond is { "type": "...", ... }
    let any_conds = rule_json.get("any").and_then(|v| v.as_array()).cloned().unwrap_or_default();
    let all_conds = rule_json.get("all").and_then(|v| v.as_array()).cloned().unwrap_or_default();

    let any_pass = if any_conds.is_empty() {
        true
    } else {
        any_conds.iter().any(|c| condition_matches_mvp(c, haystack_lower, entities_lower, sources_lower, event))
    };

    let all_pass = all_conds
        .iter()
        .all(|c| condition_matches_mvp(c, haystack_lower, entities_lower, sources_lower, event));

    any_pass && all_pass
}

fn condition_matches_mvp(
    cond: &Value,
    haystack_lower: &str,
    entities_lower: &HashSet<String>,
    sources_lower: &HashSet<String>,
    event: &TemporalEvent,
) -> bool {
    let t = cond.get("type").and_then(|v| v.as_str()).unwrap_or("");
    match t {
        "contains_keyword" => cond
            .get("keyword")
            .and_then(|v| v.as_str())
            .map(|kw| haystack_lower.contains(&kw.to_lowercase()))
            .unwrap_or(false),
        "mentions_entity" => cond
            .get("entity")
            .and_then(|v| v.as_str())
            .map(|e| entities_lower.contains(&e.to_lowercase()))
            .unwrap_or(false),
        "source_in" => {
            if let Some(arr) = cond.get("sources").and_then(|v| v.as_array()) {
                arr.iter()
                    .filter_map(|v| v.as_str())
                    .any(|s| sources_lower.contains(&s.to_lowercase()))
            } else {
                false
            }
        }
        "sentiment_below" => cond
            .get("value")
            .and_then(|v| v.as_f64())
            .map(|v| event.sentiment_score <= v)
            .unwrap_or(false),
        "sentiment_above" => cond
            .get("value")
            .and_then(|v| v.as_f64())
            .map(|v| event.sentiment_score >= v)
            .unwrap_or(false),
        "volume_spike" => cond
            .get("value")
            .and_then(|v| v.as_f64())
            .map(|v| event.volume_score >= v)
            .unwrap_or(false),
        "novelty_above" => cond
            .get("value")
            .and_then(|v| v.as_f64())
            .map(|v| event.novelty_score >= v)
            .unwrap_or(false),
        "co_mention" => {
            let a = cond.get("entityA").and_then(|v| v.as_str()).unwrap_or("").to_lowercase();
            let b = cond.get("entityB").and_then(|v| v.as_str()).unwrap_or("").to_lowercase();
            !a.is_empty() && !b.is_empty() && entities_lower.contains(&a) && entities_lower.contains(&b)
        }
        _ => false,
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AlertEscalation {
    pub id: i64,
    pub alert_id: i64,
    pub escalated_at: i64,
    pub escalation_level: i32,
    pub channel: String, // email|sms|push|webhook
    pub sent: bool,
    pub error_message: Option<String>,
}

impl TemporalStore {
    pub fn create_escalation(
        &self,
        alert_id: i64,
        escalation_level: i32,
        channel: &str,
    ) -> Result<i64> {
        let conn = self.conn.lock()
            .map_err(|e| anyhow::anyhow!("Database lock poisoned: {}", e))?;
        let now = chrono::Utc::now().timestamp();

        conn.execute(
            "INSERT INTO alert_escalations (alert_id, escalated_at, escalation_level, channel, sent)
             VALUES (?1, ?2, ?3, ?4, 0)",
            params![alert_id, now, escalation_level, channel],
        )?;

        Ok(conn.last_insert_rowid())
    }

    pub fn mark_escalation_sent(&self, escalation_id: i64, error_message: Option<&str>) -> Result<()> {
        let conn = self.conn.lock()
            .map_err(|e| anyhow::anyhow!("Database lock poisoned: {}", e))?;
        conn.execute(
            "UPDATE alert_escalations SET sent = 1, error_message = ?1 WHERE id = ?2",
            params![error_message, escalation_id],
        )?;
        Ok(())
    }

    pub fn get_alert_escalations(&self, alert_id: i64) -> Result<Vec<AlertEscalation>> {
        let conn = self.conn.lock()
            .map_err(|e| anyhow::anyhow!("Database lock poisoned: {}", e))?;
        let mut stmt = conn.prepare(
            "SELECT id, alert_id, escalated_at, escalation_level, channel, sent, error_message
             FROM alert_escalations
             WHERE alert_id = ?1
             ORDER BY escalated_at ASC",
        )?;

        let rows = stmt.query_map(params![alert_id], |row| {
            Ok(AlertEscalation {
                id: row.get(0)?,
                alert_id: row.get(1)?,
                escalated_at: row.get(2)?,
                escalation_level: row.get(3)?,
                channel: row.get(4)?,
                sent: row.get::<_, i64>(5)? == 1,
                error_message: row.get(6)?,
            })
        })?;

        let mut escalations = Vec::new();
        for row in rows {
            escalations.push(row?);
        }

        Ok(escalations)
    }
}


