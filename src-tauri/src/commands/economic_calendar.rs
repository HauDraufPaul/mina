use crate::storage::economic_calendar::{EconomicCalendarStore, EconomicEvent, EventImpactHistory};
use crate::services::economic_calendar::EconomicCalendarService;
use crate::providers::economic_calendar::EconomicCalendarProvider;
use crate::services::api_key_manager::APIKeyManager;
use crate::storage::Database;
use chrono::{DateTime, Utc};
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

#[tauri::command]
pub async fn sync_economic_events(
    from_ts: i64,
    to_ts: i64,
    db: State<'_, Mutex<Database>>,
    api_key_manager: State<'_, Mutex<APIKeyManager>>,
) -> Result<usize, String> {
    // Get API key for Trading Economics
    let api_key = {
        let key_mgr = api_key_manager.lock().map_err(|e| format!("API key manager lock error: {}", e))?;
        key_mgr.get_key_optional("trading_economics").ok().flatten()
    };
    
    let provider = EconomicCalendarProvider::new(api_key);
    
    let from_date = DateTime::from_timestamp(from_ts, 0)
        .ok_or_else(|| "Invalid from timestamp".to_string())?;
    let to_date = DateTime::from_timestamp(to_ts, 0)
        .ok_or_else(|| "Invalid to timestamp".to_string())?;
    
    // Fetch events from API
    let events_data = provider.fetch_events(from_date, to_date).await
        .map_err(|e| format!("Failed to fetch events: {}", e))?;
    
    // Store events in database
    let db_guard = db.lock().map_err(|e| format!("Database lock error: {}", e))?;
    let store = EconomicCalendarStore::new(db_guard.conn.clone());
    
    let mut synced_count = 0;
    for event_data in events_data {
        let impact_score = EconomicCalendarProvider::impact_to_score(&event_data.impact);
        
        // Parse forecast and previous values
        let forecast_value = event_data.forecast_value
            .and_then(|v| v.parse::<f64>().ok());
        let previous_value = event_data.previous_value
            .and_then(|v| v.parse::<f64>().ok());
        
        // Check if event already exists (by name, country, scheduled_at)
        let existing = store.list_events(
            event_data.scheduled_at - 3600,
            event_data.scheduled_at + 3600,
            Some(&event_data.country),
            Some(&event_data.event_type),
        ).ok()
            .and_then(|events| {
                events.into_iter()
                    .find(|e| e.name == event_data.title)
            });
        
        if existing.is_none() {
            if let Ok(_) = store.create_event(
                &event_data.title,
                &event_data.country,
                &event_data.event_type,
                event_data.scheduled_at,
                forecast_value,
                previous_value,
                impact_score,
            ) {
                synced_count += 1;
            }
        }
    }
    
    Ok(synced_count)
}
