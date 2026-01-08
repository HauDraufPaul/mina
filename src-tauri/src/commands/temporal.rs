use crate::storage::temporal::TemporalStore;
use crate::storage::Database;
use serde_json::Value;
use std::sync::Mutex;
use tauri::{Emitter, State};

#[tauri::command]
pub fn temporal_list_events(
    limit: Option<i64>,
    from_ts: Option<i64>,
    to_ts: Option<i64>,
    db: State<'_, Mutex<Database>>,
) -> Result<Vec<crate::storage::temporal::TemporalEvent>, String> {
    let limit = limit.unwrap_or(200).max(1).min(2000);
    let db_guard = db.lock().map_err(|e| format!("Database lock error: {}", e))?;
    let store = TemporalStore::new(db_guard.conn.clone());
    store
        .list_events(limit, from_ts, to_ts)
        .map_err(|e| format!("Failed to list events: {}", e))
}

#[tauri::command]
pub fn temporal_get_event(
    id: i64,
    db: State<'_, Mutex<Database>>,
) -> Result<Option<crate::storage::temporal::TemporalEvent>, String> {
    let db_guard = db.lock().map_err(|e| format!("Database lock error: {}", e))?;
    let store = TemporalStore::new(db_guard.conn.clone());
    store
        .get_event(id)
        .map_err(|e| format!("Failed to get event: {}", e))
}

#[tauri::command]
pub fn temporal_list_event_evidence(
    event_id: i64,
    db: State<'_, Mutex<Database>>,
) -> Result<Vec<crate::storage::temporal::TemporalEventEvidence>, String> {
    let db_guard = db.lock().map_err(|e| format!("Database lock error: {}", e))?;
    let store = TemporalStore::new(db_guard.conn.clone());
    store
        .list_event_evidence(event_id)
        .map_err(|e| format!("Failed to list evidence: {}", e))
}

#[tauri::command]
pub fn temporal_rebuild_events_mvp(
    days_back: Option<i64>,
    app: tauri::AppHandle,
    db: State<'_, Mutex<Database>>,
) -> Result<i64, String> {
    let days_back = days_back.unwrap_or(14).max(1).min(365);
    let db_guard = db.lock().map_err(|e| format!("Database lock error: {}", e))?;
    let store = TemporalStore::new(db_guard.conn.clone());
    let count = store
        .rebuild_events_mvp(days_back)
        .map_err(|e| format!("Failed to rebuild events: {}", e))?;

    // Evaluate rules and emit newly created alerts
    if let Ok(created_alerts) = store.evaluate_alert_rules_mvp(days_back, 500) {
        for alert in created_alerts {
            let _ = app.emit(
                "ws-message",
                serde_json::json!({
                    "type": "temporal-alert",
                    "data": alert,
                    "timestamp": chrono::Utc::now().timestamp_millis()
                }),
            );
        }
    }

    let _ = app.emit(
        "ws-message",
        serde_json::json!({
            "type": "temporal-job-status",
            "data": { "job": "rebuild-events-mvp", "touched_events": count, "days_back": days_back },
            "timestamp": chrono::Utc::now().timestamp_millis()
        }),
    );

    Ok(count)
}

#[tauri::command]
pub fn temporal_rebuild_search_index(
    from_ts: Option<i64>,
    app: tauri::AppHandle,
    db: State<'_, Mutex<Database>>,
) -> Result<i64, String> {
    let db_guard = db.lock().map_err(|e| format!("Database lock error: {}", e))?;
    let store = TemporalStore::new(db_guard.conn.clone());
    let count = store
        .rebuild_search_index(from_ts)
        .map_err(|e| format!("Failed to rebuild search index: {}", e))?;

    let _ = app.emit(
        "ws-message",
        serde_json::json!({
            "type": "temporal-job-status",
            "data": { "job": "rebuild-search-index", "indexed_docs": count },
            "timestamp": chrono::Utc::now().timestamp_millis()
        }),
    );

    Ok(count)
}

#[tauri::command]
pub fn temporal_search(
    query: String,
    limit: Option<i64>,
    db: State<'_, Mutex<Database>>,
) -> Result<Vec<Value>, String> {
    let limit = limit.unwrap_or(50).max(1).min(500);
    let db_guard = db.lock().map_err(|e| format!("Database lock error: {}", e))?;
    let store = TemporalStore::new(db_guard.conn.clone());
    store
        .search(&query, limit)
        .map_err(|e| format!("Failed to search: {}", e))
}

#[tauri::command]
pub fn temporal_list_watchlists(
    db: State<'_, Mutex<Database>>,
) -> Result<Vec<crate::storage::temporal::Watchlist>, String> {
    let db_guard = db.lock().map_err(|e| format!("Database lock error: {}", e))?;
    let store = TemporalStore::new(db_guard.conn.clone());
    store
        .list_watchlists()
        .map_err(|e| format!("Failed to list watchlists: {}", e))
}

#[tauri::command]
pub fn temporal_create_watchlist(
    name: String,
    db: State<'_, Mutex<Database>>,
) -> Result<i64, String> {
    let db_guard = db.lock().map_err(|e| format!("Database lock error: {}", e))?;
    let store = TemporalStore::new(db_guard.conn.clone());
    store
        .create_watchlist(&name)
        .map_err(|e| format!("Failed to create watchlist: {}", e))
}

#[tauri::command]
pub fn temporal_add_watchlist_item(
    watchlist_id: i64,
    item_type: String,
    value: String,
    weight: Option<f64>,
    enabled: Option<bool>,
    db: State<'_, Mutex<Database>>,
) -> Result<i64, String> {
    let weight = weight.unwrap_or(1.0);
    let enabled = enabled.unwrap_or(true);
    let db_guard = db.lock().map_err(|e| format!("Database lock error: {}", e))?;
    let store = TemporalStore::new(db_guard.conn.clone());
    store
        .add_watchlist_item(watchlist_id, &item_type, &value, weight, enabled)
        .map_err(|e| format!("Failed to add watchlist item: {}", e))
}

#[tauri::command]
pub fn temporal_list_watchlist_items(
    watchlist_id: i64,
    db: State<'_, Mutex<Database>>,
) -> Result<Vec<crate::storage::temporal::WatchlistItem>, String> {
    let db_guard = db.lock().map_err(|e| format!("Database lock error: {}", e))?;
    let store = TemporalStore::new(db_guard.conn.clone());
    store
        .list_watchlist_items(watchlist_id)
        .map_err(|e| format!("Failed to list watchlist items: {}", e))
}

#[tauri::command]
pub fn temporal_create_alert_rule(
    name: String,
    enabled: Option<bool>,
    watchlist_id: Option<i64>,
    rule_json: Value,
    schedule: Option<String>,
    db: State<'_, Mutex<Database>>,
) -> Result<i64, String> {
    let enabled = enabled.unwrap_or(true);
    let db_guard = db.lock().map_err(|e| format!("Database lock error: {}", e))?;
    let store = TemporalStore::new(db_guard.conn.clone());
    store
        .create_alert_rule(&name, enabled, watchlist_id, &rule_json, schedule.as_deref())
        .map_err(|e| format!("Failed to create alert rule: {}", e))
}

#[tauri::command]
pub fn temporal_list_alert_rules(
    db: State<'_, Mutex<Database>>,
) -> Result<Vec<crate::storage::temporal::AlertRule>, String> {
    let db_guard = db.lock().map_err(|e| format!("Database lock error: {}", e))?;
    let store = TemporalStore::new(db_guard.conn.clone());
    store
        .list_alert_rules()
        .map_err(|e| format!("Failed to list alert rules: {}", e))
}

#[tauri::command]
pub fn temporal_list_alerts(
    limit: Option<i64>,
    from_ts: Option<i64>,
    to_ts: Option<i64>,
    db: State<'_, Mutex<Database>>,
) -> Result<Vec<crate::storage::temporal::Alert>, String> {
    let limit = limit.unwrap_or(200).max(1).min(2000);
    let db_guard = db.lock().map_err(|e| format!("Database lock error: {}", e))?;
    let store = TemporalStore::new(db_guard.conn.clone());
    store
        .list_alerts(limit, from_ts, to_ts)
        .map_err(|e| format!("Failed to list alerts: {}", e))
}

#[tauri::command]
pub fn temporal_ack_alert(
    alert_id: i64,
    db: State<'_, Mutex<Database>>,
) -> Result<(), String> {
    let db_guard = db.lock().map_err(|e| format!("Database lock error: {}", e))?;
    let store = TemporalStore::new(db_guard.conn.clone());
    store
        .update_alert_status(alert_id, "ack", None)
        .map_err(|e| format!("Failed to ack alert: {}", e))
}

#[tauri::command]
pub fn temporal_snooze_alert(
    alert_id: i64,
    snooze_seconds: i64,
    db: State<'_, Mutex<Database>>,
) -> Result<(), String> {
    let snoozed_until = chrono::Utc::now().timestamp() + snooze_seconds.max(60).min(7 * 24 * 3600);
    let db_guard = db.lock().map_err(|e| format!("Database lock error: {}", e))?;
    let store = TemporalStore::new(db_guard.conn.clone());
    store
        .update_alert_status(alert_id, "snoozed", Some(snoozed_until))
        .map_err(|e| format!("Failed to snooze alert: {}", e))
}

#[tauri::command]
pub fn temporal_resolve_alert(
    alert_id: i64,
    db: State<'_, Mutex<Database>>,
) -> Result<(), String> {
    let db_guard = db.lock().map_err(|e| format!("Database lock error: {}", e))?;
    let store = TemporalStore::new(db_guard.conn.clone());
    store
        .update_alert_status(alert_id, "resolved", None)
        .map_err(|e| format!("Failed to resolve alert: {}", e))
}

#[tauri::command]
pub fn temporal_run_backtest_mvp(
    from_ts: i64,
    to_ts: i64,
    db: State<'_, Mutex<Database>>,
) -> Result<crate::storage::temporal::BacktestReport, String> {
    let db_guard = db.lock().map_err(|e| format!("Database lock error: {}", e))?;
    let store = TemporalStore::new(db_guard.conn.clone());
    store
        .run_backtest_mvp(from_ts, to_ts)
        .map_err(|e| format!("Failed to run backtest: {}", e))
}

#[tauri::command]
pub fn temporal_get_entity_graph_mvp(
    days_back: Option<i64>,
    max_nodes: Option<i64>,
    max_edges: Option<i64>,
    db: State<'_, Mutex<Database>>,
) -> Result<crate::storage::temporal::EntityGraph, String> {
    let days_back = days_back.unwrap_or(14).max(1).min(365);
    let max_nodes = max_nodes.unwrap_or(60).max(10).min(500) as usize;
    let max_edges = max_edges.unwrap_or(200).max(10).min(2000) as usize;
    let db_guard = db.lock().map_err(|e| format!("Database lock error: {}", e))?;
    let store = TemporalStore::new(db_guard.conn.clone());
    store
        .get_entity_graph_mvp(days_back, max_nodes, max_edges)
        .map_err(|e| format!("Failed to build graph: {}", e))
}

#[tauri::command]
pub fn temporal_create_feature_definition(
    name: String,
    expression: String,
    description: Option<String>,
    db: State<'_, Mutex<Database>>,
) -> Result<i64, String> {
    let db_guard = db.lock().map_err(|e| format!("Database lock error: {}", e))?;
    let store = TemporalStore::new(db_guard.conn.clone());
    store
        .create_feature_definition(&name, &expression, description.as_deref())
        .map_err(|e| format!("Failed to create feature: {}", e))
}

#[tauri::command]
pub fn temporal_list_feature_definitions(
    db: State<'_, Mutex<Database>>,
) -> Result<Vec<crate::storage::temporal::FeatureDefinition>, String> {
    let db_guard = db.lock().map_err(|e| format!("Database lock error: {}", e))?;
    let store = TemporalStore::new(db_guard.conn.clone());
    store
        .list_feature_definitions()
        .map_err(|e| format!("Failed to list features: {}", e))
}

#[tauri::command]
pub fn temporal_compute_feature_mvp(
    feature_id: i64,
    days_back: Option<i64>,
    app: tauri::AppHandle,
    db: State<'_, Mutex<Database>>,
) -> Result<i64, String> {
    let days_back = days_back.unwrap_or(30).max(1).min(365);
    let db_guard = db.lock().map_err(|e| format!("Database lock error: {}", e))?;
    let store = TemporalStore::new(db_guard.conn.clone());
    let inserted = store
        .compute_feature_mvp(feature_id, days_back)
        .map_err(|e| format!("Failed to compute feature: {}", e))?;

    let _ = app.emit(
        "ws-message",
        serde_json::json!({
            "type": "temporal-job-status",
            "data": { "job": "compute-feature-mvp", "feature_id": feature_id, "inserted": inserted, "days_back": days_back },
            "timestamp": chrono::Utc::now().timestamp_millis()
        }),
    );

    Ok(inserted)
}

#[tauri::command]
pub fn temporal_list_feature_values(
    feature_id: i64,
    from_ts: Option<i64>,
    to_ts: Option<i64>,
    limit: Option<i64>,
    db: State<'_, Mutex<Database>>,
) -> Result<Vec<crate::storage::temporal::FeatureValue>, String> {
    let limit = limit.unwrap_or(365).max(1).min(5000);
    let db_guard = db.lock().map_err(|e| format!("Database lock error: {}", e))?;
    let store = TemporalStore::new(db_guard.conn.clone());
    store
        .list_feature_values(feature_id, from_ts, to_ts, limit)
        .map_err(|e| format!("Failed to list feature values: {}", e))
}

#[tauri::command]
pub fn temporal_set_alert_label(
    alert_id: i64,
    label: i64,
    note: Option<String>,
    db: State<'_, Mutex<Database>>,
) -> Result<(), String> {
    let db_guard = db.lock().map_err(|e| format!("Database lock error: {}", e))?;
    let store = TemporalStore::new(db_guard.conn.clone());
    store
        .set_alert_label(alert_id, label, note.as_deref())
        .map_err(|e| format!("Failed to set label: {}", e))
}

#[tauri::command]
pub fn temporal_get_alert_label(
    alert_id: i64,
    db: State<'_, Mutex<Database>>,
) -> Result<Option<crate::storage::temporal::AlertLabel>, String> {
    let db_guard = db.lock().map_err(|e| format!("Database lock error: {}", e))?;
    let store = TemporalStore::new(db_guard.conn.clone());
    store
        .get_alert_label(alert_id)
        .map_err(|e| format!("Failed to get label: {}", e))
}


