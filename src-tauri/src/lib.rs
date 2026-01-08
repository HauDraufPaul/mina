use serde::{Deserialize, Serialize};
use std::sync::Mutex;
use tauri::Manager;
use anyhow::Context;

mod storage;
mod providers;
mod commands;
mod ws;
mod utils;

use storage::Database;
use storage::{RateLimitStore, TestingStore, AnalyticsStore, VectorStore, AIStore, AutomationStore, DevOpsStore, OSINTStore, ProjectStore, MigrationTracker};
use providers::{SystemProvider, NetworkProvider, ProcessProvider, HomebrewProvider, SystemUtilsProvider, OllamaProvider};
use ws::WsServer;
use std::path::PathBuf;
use tokio::sync::RwLock;
use std::sync::Arc;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SystemMetrics {
    pub cpu: CpuMetrics,
    pub memory: MemoryMetrics,
    pub disk: DiskMetrics,
    pub network: NetworkMetrics,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CpuMetrics {
    pub usage: f64,
    pub cores: usize,
    pub frequency: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MemoryMetrics {
    pub total: u64,
    pub used: u64,
    pub free: u64,
    pub usage: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DiskMetrics {
    pub total: u64,
    pub used: u64,
    pub free: u64,
    pub usage: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NetworkMetrics {
    pub rx: u64,
    pub tx: u64,
    #[serde(rename = "rxSpeed")]
    pub rx_speed: f64,
    #[serde(rename = "txSpeed")]
    pub tx_speed: f64,
}


#[tauri::command]
fn get_recent_errors(limit: i32, db: tauri::State<'_, Mutex<Database>>) -> Result<Vec<storage::ErrorRecord>, String> {
    let db_guard = db.lock().map_err(|e| format!("Database lock error: {}", e))?;
    db_guard.get_recent_errors(limit)
        .map_err(|e| format!("Failed to get errors: {}", e))
}

#[tauri::command]
fn save_error(
    error_type: String,
    message: String,
    stack_trace: Option<String>,
    source: Option<String>,
    severity: String,
    db: tauri::State<'_, Mutex<Database>>,
) -> Result<i64, String> {
    let db_guard = db.lock().map_err(|e| format!("Database lock error: {}", e))?;
    db_guard.save_error(
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
            eprintln!("MINA: Starting setup...");
            
            // Initialize database first
            let db = match Database::new(app.handle()) {
                Ok(db) => {
                    eprintln!("MINA: Database initialized successfully");
                    db
                }
                Err(e) => {
                    eprintln!("ERROR: Failed to initialize database: {}", e);
                    return Err(e.into());
                }
            };
            
            // Initialize all store schemas by creating instances
            // This ensures all tables are created
            eprintln!("MINA: Initializing stores...");
            
            eprintln!("MINA: Initializing RateLimitStore...");
            RateLimitStore::new(db.conn.clone())
                .map_err(|e| {
                    eprintln!("MINA: Failed to initialize RateLimitStore: {}", e);
                    e
                })?;
            eprintln!("MINA: RateLimitStore initialized");
            
            eprintln!("MINA: Initializing TestingStore...");
            let _ = TestingStore::new(db.conn.clone());
            eprintln!("MINA: TestingStore initialized");
            
            eprintln!("MINA: Initializing AnalyticsStore...");
            let _ = AnalyticsStore::new(db.conn.clone());
            eprintln!("MINA: AnalyticsStore initialized");
            
            eprintln!("MINA: Initializing VectorStore...");
            let _ = VectorStore::new(db.conn.clone());
            eprintln!("MINA: VectorStore initialized");
            
            eprintln!("MINA: Initializing AIStore...");
            if let Err(e) = AIStore::new(db.conn.clone()) {
                eprintln!("WARNING: Failed to initialize AIStore: {}", e);
            } else {
                eprintln!("MINA: AIStore initialized");
            }
            
            eprintln!("MINA: Initializing AutomationStore...");
            if let Err(e) = AutomationStore::new(db.conn.clone()) {
                eprintln!("WARNING: Failed to initialize AutomationStore: {}", e);
            } else {
                eprintln!("MINA: AutomationStore initialized");
            }
            
            eprintln!("MINA: Initializing DevOpsStore...");
            let _ = DevOpsStore::new(db.conn.clone());
            eprintln!("MINA: DevOpsStore initialized");
            
            eprintln!("MINA: Initializing OSINTStore...");
            // Initialize OSINTStore - this will create default feeds if needed
            // Now safe - won't panic on errors
            let _ = OSINTStore::new(db.conn.clone());
            eprintln!("MINA: OSINTStore initialized");
            
            eprintln!("MINA: Initializing ProjectStore...");
            let _ = ProjectStore::new(db.conn.clone());
            eprintln!("MINA: ProjectStore initialized");
            
            eprintln!("MINA: Initializing MigrationTracker...");
            let _ = MigrationTracker::new(db.conn.clone());
            eprintln!("MINA: MigrationTracker initialized");
            
            // Seed initial data
            if let Err(e) = storage::seed_data::seed_initial_data(&db.conn) {
                eprintln!("Warning: Failed to seed initial data: {}", e);
            }
            
            eprintln!("MINA: Stores initialized");
            
            app.manage(Mutex::new(db));
            
            // Initialize providers
            app.manage(std::sync::Mutex::new(SystemProvider::new()));
            app.manage(std::sync::Mutex::new(NetworkProvider::new()));
            app.manage(std::sync::Mutex::new(ProcessProvider::new()));
            app.manage(tokio::sync::Mutex::new(HomebrewProvider::new()));
            app.manage(std::sync::Mutex::new(SystemUtilsProvider::new()));
            
            // Initialize Ollama provider
            eprintln!("MINA: Initializing Ollama provider...");
            if let Ok(app_data_dir) = app.path().app_data_dir() {
                let models_folder = app_data_dir.join("models");
                if let Err(e) = std::fs::create_dir_all(&models_folder) {
                    eprintln!("WARNING: Failed to create models folder: {}", e);
                } else {
                    let ollama_provider = OllamaProvider::new(models_folder);
                    let ollama_state = Arc::new(RwLock::new(ollama_provider));
                    app.manage(ollama_state);
                }
            } else {
                eprintln!("WARNING: Failed to get app data dir for Ollama");
            }
            
            // Initialize WebSocket server
            eprintln!("MINA: Initializing WebSocket server...");
            let ws_server = WsServer::new();
            ws_server.start_broadcast(app.handle().clone());
            app.manage(Mutex::new(ws_server));
            
            eprintln!("MINA: Setup complete, showing window...");
            
            // Ensure window is visible and focused after all initialization
            if let Some(window) = app.get_webview_window("main") {
                eprintln!("MINA: Main window found, showing...");
                if let Err(e) = window.show() {
                    eprintln!("ERROR: Failed to show window: {:?}", e);
                }
                if let Err(e) = window.set_focus() {
                    eprintln!("WARNING: Failed to focus window: {:?}", e);
                }
                if let Err(e) = window.center() {
                    eprintln!("WARNING: Failed to center window: {:?}", e);
                }
                eprintln!("MINA: Window should now be visible");
            } else {
                eprintln!("ERROR: Main window not found after setup!");
                eprintln!("MINA: Available windows: {:?}", app.webview_windows().keys().collect::<Vec<_>>());
            }
            
            Ok(())
        })
        .on_window_event(|window, event| {
            // Handle window events
            if let tauri::WindowEvent::CloseRequested { .. } = event {
                // Allow window to close
            }
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
            commands::analytics::save_metric,
            commands::analytics::get_metrics,
            commands::analytics::get_statistics,
            commands::rate_limit::create_rate_limit_bucket,
            commands::rate_limit::list_rate_limit_buckets,
            commands::rate_limit::get_rate_limit_bucket,
            commands::rate_limit::consume_rate_limit_token,
            commands::rate_limit::refill_rate_limit_bucket,
            commands::migration::list_migrations,
            commands::migration::get_latest_migration_version,
            commands::system_utils::get_disk_info,
            commands::system_utils::get_system_info,
            commands::system_utils::prevent_sleep,
            commands::vector_search::search_vectors,
            commands::embeddings::generate_embedding,
            commands::ai::create_conversation,
            commands::ai::list_conversations,
            commands::ai::add_chat_message,
            commands::ai::get_chat_messages,
            commands::ai::create_prompt_template,
            commands::ai::list_prompt_templates,
            commands::ai::get_prompt_template,
            commands::automation::create_script,
            commands::automation::list_scripts,
            commands::automation::get_script,
            commands::automation::create_workflow,
            commands::automation::list_workflows,
            commands::automation::record_workflow_execution,
            commands::automation::get_workflow_executions,
            commands::devops::create_health_check,
            commands::devops::update_health_check,
            commands::devops::list_health_checks,
            commands::devops::create_alert,
            commands::devops::list_alerts,
            commands::devops::resolve_alert,
            commands::devops::save_prometheus_metric,
            commands::devops::get_prometheus_metrics,
            commands::osint::create_rss_feed,
            commands::osint::list_rss_feeds,
            commands::osint::update_rss_feed,
            commands::osint::delete_rss_feed,
            commands::osint::save_rss_item,
            commands::osint::get_recent_rss_items,
            commands::osint::create_entity,
            commands::osint::list_entities,
            commands::osint::create_entity_relationship,
            commands::osint::fetch_rss_feeds,
            commands::osint::get_rss_item,
            commands::osint::mark_article_read,
            commands::osint::toggle_article_favorite,
            commands::osint::toggle_article_saved,
            commands::osint::set_article_folder,
            commands::osint::create_article_folder,
            commands::osint::list_article_folders,
            commands::osint::delete_article_folder,
            commands::osint::get_filtered_articles,
            commands::osint::get_article_entities,
            commands::osint::extract_entities_from_article,
            commands::osint::fetch_full_article,
            commands::testing::create_test_suite,
            commands::testing::list_test_suites,
            commands::testing::save_test_result,
            commands::testing::get_suite_results,
            commands::testing::get_suite_stats,
            commands::projects::create_project,
            commands::projects::update_project,
            commands::projects::list_projects,
            commands::projects::get_project,
            commands::projects::delete_project,
            commands::ollama::check_ollama_status,
            commands::ollama::list_ollama_models,
            commands::ollama::get_ollama_model_info,
            commands::ollama::load_model_from_file,
            commands::ollama::chat_with_ollama,
            commands::ollama::scan_models_folder,
            commands::ollama::get_models_folder_path,
            get_recent_errors,
            save_error
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}

