use crate::storage::portfolio::{PortfolioStore, Portfolio, Holding, Transaction};
use crate::storage::market_data::MarketDataStore;
use crate::services::portfolio_analyzer::PortfolioAnalyzer;
use crate::storage::Database;
use std::collections::HashMap;
use std::sync::Mutex;
use tauri::State;

#[tauri::command]
pub fn create_portfolio(
    name: String,
    db: State<'_, Mutex<Database>>,
) -> Result<i64, String> {
    let db_guard = db.lock().map_err(|e| format!("Database lock error: {}", e))?;
    let store = PortfolioStore::new(db_guard.conn.clone());
    store
        .create_portfolio(&name)
        .map_err(|e| format!("Failed to create portfolio: {}", e))
}

#[tauri::command]
pub fn list_portfolios(
    db: State<'_, Mutex<Database>>,
) -> Result<Vec<Portfolio>, String> {
    let db_guard = db.lock().map_err(|e| format!("Database lock error: {}", e))?;
    let store = PortfolioStore::new(db_guard.conn.clone());
    store
        .list_portfolios()
        .map_err(|e| format!("Failed to list portfolios: {}", e))
}

#[tauri::command]
pub fn get_portfolio(
    id: i64,
    db: State<'_, Mutex<Database>>,
) -> Result<Option<Portfolio>, String> {
    let db_guard = db.lock().map_err(|e| format!("Database lock error: {}", e))?;
    let store = PortfolioStore::new(db_guard.conn.clone());
    store
        .get_portfolio(id)
        .map_err(|e| format!("Failed to get portfolio: {}", e))
}

#[tauri::command]
pub fn add_holding(
    portfolio_id: i64,
    ticker: String,
    quantity: f64,
    purchase_price: f64,
    purchase_date: i64,
    db: State<'_, Mutex<Database>>,
) -> Result<i64, String> {
    let db_guard = db.lock().map_err(|e| format!("Database lock error: {}", e))?;
    let store = PortfolioStore::new(db_guard.conn.clone());
    store
        .add_holding(portfolio_id, &ticker, quantity, purchase_price, purchase_date)
        .map_err(|e| format!("Failed to add holding: {}", e))
}

#[tauri::command]
pub fn list_holdings(
    portfolio_id: i64,
    db: State<'_, Mutex<Database>>,
) -> Result<Vec<Holding>, String> {
    let db_guard = db.lock().map_err(|e| format!("Database lock error: {}", e))?;
    let store = PortfolioStore::new(db_guard.conn.clone());
    store
        .list_holdings(portfolio_id)
        .map_err(|e| format!("Failed to list holdings: {}", e))
}

#[tauri::command]
pub fn get_holdings_by_ticker(
    ticker: String,
    db: State<'_, Mutex<Database>>,
) -> Result<Vec<Holding>, String> {
    let db_guard = db.lock().map_err(|e| format!("Database lock error: {}", e))?;
    let store = PortfolioStore::new(db_guard.conn.clone());
    store
        .get_holdings_by_ticker(&ticker)
        .map_err(|e| format!("Failed to get holdings: {}", e))
}

#[tauri::command]
pub fn update_holding(
    holding_id: i64,
    quantity: Option<f64>,
    purchase_price: Option<f64>,
    db: State<'_, Mutex<Database>>,
) -> Result<(), String> {
    let db_guard = db.lock().map_err(|e| format!("Database lock error: {}", e))?;
    let store = PortfolioStore::new(db_guard.conn.clone());
    store
        .update_holding(holding_id, quantity, purchase_price)
        .map_err(|e| format!("Failed to update holding: {}", e))
}

#[tauri::command]
pub fn delete_holding(
    holding_id: i64,
    db: State<'_, Mutex<Database>>,
) -> Result<(), String> {
    let db_guard = db.lock().map_err(|e| format!("Database lock error: {}", e))?;
    let store = PortfolioStore::new(db_guard.conn.clone());
    store
        .delete_holding(holding_id)
        .map_err(|e| format!("Failed to delete holding: {}", e))
}

#[tauri::command]
pub fn get_portfolio_value(
    portfolio_id: i64,
    db: State<'_, Mutex<Database>>,
) -> Result<crate::services::portfolio_analyzer::PortfolioValue, String> {
    let db_guard = db.lock().map_err(|e| format!("Database lock error: {}", e))?;
    let portfolio_store = PortfolioStore::new(db_guard.conn.clone());
    let market_data_store = MarketDataStore::new(db_guard.conn.clone());
    
    PortfolioAnalyzer::calculate_portfolio_value(&portfolio_store, &market_data_store, portfolio_id)
        .map_err(|e| format!("Failed to calculate portfolio value: {}", e))
}

#[tauri::command]
pub fn get_portfolio_impact(
    portfolio_id: i64,
    event_id: i64,
    price_changes: HashMap<String, f64>, // ticker -> price change percent
    db: State<'_, Mutex<Database>>,
) -> Result<crate::services::portfolio_analyzer::ImpactAnalysis, String> {
    let db_guard = db.lock().map_err(|e| format!("Database lock error: {}", e))?;
    let portfolio_store = PortfolioStore::new(db_guard.conn.clone());
    let market_data_store = MarketDataStore::new(db_guard.conn.clone());
    
    PortfolioAnalyzer::analyze_event_impact(&portfolio_store, &market_data_store, portfolio_id, event_id, &price_changes)
        .map_err(|e| format!("Failed to analyze impact: {}", e))
}

#[tauri::command]
pub fn add_transaction(
    portfolio_id: i64,
    ticker: String,
    transaction_type: String,
    quantity: f64,
    price: f64,
    transaction_date: i64,
    fees: f64,
    notes: Option<String>,
    db: State<'_, Mutex<Database>>,
) -> Result<i64, String> {
    let db_guard = db.lock().map_err(|e| format!("Database lock error: {}", e))?;
    let store = PortfolioStore::new(db_guard.conn.clone());
    store
        .add_transaction(
            portfolio_id,
            &ticker,
            &transaction_type,
            quantity,
            price,
            transaction_date,
            fees,
            notes.as_deref(),
        )
        .map_err(|e| format!("Failed to add transaction: {}", e))
}

#[tauri::command]
pub fn list_transactions(
    portfolio_id: i64,
    limit: Option<i64>,
    db: State<'_, Mutex<Database>>,
) -> Result<Vec<Transaction>, String> {
    let db_guard = db.lock().map_err(|e| format!("Database lock error: {}", e))?;
    let store = PortfolioStore::new(db_guard.conn.clone());
    store
        .list_transactions(portfolio_id, limit)
        .map_err(|e| format!("Failed to list transactions: {}", e))
}

#[tauri::command]
pub fn delete_portfolio(
    id: i64,
    db: State<'_, Mutex<Database>>,
) -> Result<(), String> {
    let db_guard = db.lock().map_err(|e| format!("Database lock error: {}", e))?;
    let store = PortfolioStore::new(db_guard.conn.clone());
    store
        .delete_portfolio(id)
        .map_err(|e| format!("Failed to delete portfolio: {}", e))
}
