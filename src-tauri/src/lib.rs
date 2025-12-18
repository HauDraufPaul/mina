use serde::{Deserialize, Serialize};
use std::sync::Mutex;
use tauri::Manager;

mod storage;
mod providers;
mod commands;
mod ws;

use storage::Database;
use providers::{SystemProvider, NetworkProvider, ProcessProvider, HomebrewProvider};
use ws::WsServer;

#[derive(Debug, Serialize, Deserialize)]
pub struct SystemMetrics {
    pub cpu: CpuMetrics,
    pub memory: MemoryMetrics,
    pub disk: DiskMetrics,
    pub network: NetworkMetrics,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct CpuMetrics {
    pub usage: f64,
    pub cores: usize,
    pub frequency: u64,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct MemoryMetrics {
    pub total: u64,
    pub used: u64,
    pub free: u64,
    pub usage: f64,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct DiskMetrics {
    pub total: u64,
    pub used: u64,
    pub free: u64,
    pub usage: f64,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct NetworkMetrics {
    pub rx: u64,
    pub tx: u64,
    #[serde(rename = "rxSpeed")]
    pub rx_speed: f64,
    #[serde(rename = "txSpeed")]
    pub tx_speed: f64,
}


#[tauri::command]
fn get_recent_errors(limit: i32, app: tauri::AppHandle) -> Result<Vec<storage::ErrorRecord>, String> {
    let db = app.try_state::<Mutex<Database>>()
        .ok_or("Database not found")?;
    let db = db.lock().map_err(|e| format!("Database lock error: {}", e))?;
    db.get_recent_errors(limit)
        .map_err(|e| format!("Failed to get errors: {}", e))
}

#[tauri::command]
fn save_error(
    error_type: String,
    message: String,
    stack_trace: Option<String>,
    source: Option<String>,
    severity: String,
    app: tauri::AppHandle,
) -> Result<i64, String> {
    let db = app.try_state::<Mutex<Database>>()
        .ok_or("Database not found")?;
    let db = db.lock().map_err(|e| format!("Database lock error: {}", e))?;
    db.save_error(
        &error_type,
        &message,
        stack_trace.as_deref(),
        source.as_deref(),
        &severity,
    )
    .map_err(|e| format!("Failed to save error: {}", e))
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_opener::init())
        .setup(|app| {
            // Initialize database
            let db = Database::new(app.handle())
                .map_err(|e| {
                    eprintln!("Failed to initialize database: {}", e);
                    e
                })?;
            
            app.manage(Mutex::new(db));
            
            // Initialize providers
            app.manage(Mutex::new(SystemProvider::new()));
            app.manage(Mutex::new(NetworkProvider::new()));
            app.manage(Mutex::new(ProcessProvider::new()));
            app.manage(Mutex::new(HomebrewProvider::new()));
            
            // Initialize WebSocket server
            let ws_server = WsServer::new();
            ws_server.start_broadcast(app.handle().clone());
            app.manage(Mutex::new(ws_server));
            
            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            commands::system::get_system_metrics,
            commands::network::get_network_interfaces,
            commands::network::get_network_connections,
            commands::process::get_processes,
            commands::process::get_process,
            commands::process::kill_process,
            commands::config::get_config,
            commands::config::set_config,
            commands::ws::get_ws_connection_count,
            commands::ws::get_ws_topics,
            commands::ws::publish_ws_message,
            commands::auth::set_pin,
            commands::auth::verify_pin,
            commands::auth::create_session,
            commands::auth::validate_session,
            commands::auth::get_auth_attempts,
            commands::auth::check_permission,
            commands::packages::is_homebrew_available,
            commands::packages::list_installed_packages,
            commands::packages::list_outdated_packages,
            commands::packages::get_package_dependencies,
            commands::packages::list_services,
            commands::packages::start_service,
            commands::packages::stop_service,
            commands::packages::get_cache_size,
            commands::vector_store::create_collection,
            commands::vector_store::list_collections,
            commands::vector_store::get_collection_stats,
            commands::vector_store::cleanup_expired_vectors,
            get_recent_errors,
            save_error
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}

