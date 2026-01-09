use crate::storage::economic_calendar::{EconomicCalendarStore, EconomicEvent, EventImpactHistory};
use crate::services::economic_calendar::EconomicCalendarService;
use crate::storage::Database;
use std::sync::Mutex;
use tauri::State;

#[tauri::command]
pub fn create_economic_event(
    name: String,
    country: String,
    event_type: String,
    scheduled_at: i64,
    forecast_value: Option<f64>,
    previous_value: Option<f64>,
    db: State<'_, Mutex<Database>>,
) -> Result<i64, String> {
    let db_guard = db.lock().map_err(|e| format!("Database lock error: {}", e))?;
    let store = EconomicCalendarStore::new(db_guard.conn.clone());
    
    let impact_score = EconomicCalendarService::calculate_impact_score(&event_type, &country);
    
    store
        .create_event(&name, &country, &event_type, scheduled_at, forecast_value, previous_value, impact_score)
        .map_err(|e| format!("Failed to create event: {}", e))
}

#[tauri::command]
pub fn list_economic_events(
    from_ts: i64,
    to_ts: i64,
    country: Option<String>,
    event_type: Option<String>,
    db: State<'_, Mutex<Database>>,
) -> Result<Vec<EconomicEvent>, String> {
    let db_guard = db.lock().map_err(|e| format!("Database lock error: {}", e))?;
    let store = EconomicCalendarStore::new(db_guard.conn.clone());
    
    store
        .list_events(from_ts, to_ts, country.as_deref(), event_type.as_deref())
        .map_err(|e| format!("Failed to list events: {}", e))
}

#[tauri::command]
pub fn get_economic_event(
    id: i64,
    db: State<'_, Mutex<Database>>,
) -> Result<Option<EconomicEvent>, String> {
    let db_guard = db.lock().map_err(|e| format!("Database lock error: {}", e))?;
    let store = EconomicCalendarStore::new(db_guard.conn.clone());
    
    store
        .get_event(id)
        .map_err(|e| format!("Failed to get event: {}", e))
}

#[tauri::command]
pub fn get_event_impact_prediction(
    event_id: i64,
    db: State<'_, Mutex<Database>>,
) -> Result<f64, String> {
    let db_guard = db.lock().map_err(|e| format!("Database lock error: {}", e))?;
    let store = EconomicCalendarStore::new(db_guard.conn.clone());
    
    EconomicCalendarService::predict_market_reaction(&store, event_id)
        .map_err(|e| format!("Failed to predict impact: {}", e))
}

#[tauri::command]
pub fn record_event_outcome(
    event_id: i64,
    actual_value: f64,
    market_reaction: f64,
    db: State<'_, Mutex<Database>>,
) -> Result<i64, String> {
    let db_guard = db.lock().map_err(|e| format!("Database lock error: {}", e))?;
    let store = EconomicCalendarStore::new(db_guard.conn.clone());
    
    store
        .record_event_outcome(event_id, actual_value, market_reaction)
        .map_err(|e| format!("Failed to record outcome: {}", e))
}

#[tauri::command]
pub fn get_event_impact_history(
    event_id: i64,
    db: State<'_, Mutex<Database>>,
) -> Result<Vec<EventImpactHistory>, String> {
    let db_guard = db.lock().map_err(|e| format!("Database lock error: {}", e))?;
    let store = EconomicCalendarStore::new(db_guard.conn.clone());
    
    store
        .get_impact_history(event_id)
        .map_err(|e| format!("Failed to get impact history: {}", e))
}
