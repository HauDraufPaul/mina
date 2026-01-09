use crate::storage::price_alerts::{PriceAlertStore, PriceAlert};
use crate::storage::Database;
use crate::providers::market_data::MarketDataManager;
use crate::services::api_key_manager::APIKeyManager;
use crate::services::rate_limiter::RateLimiter;
use crate::services::automation_event_bus::AutomationEventBus;
use crate::ws::WsServer;
use anyhow::Result;
use std::sync::{Arc, Mutex};
use tauri::{AppHandle, Emitter};

pub struct PriceAlertChecker;

impl PriceAlertChecker {
    /// Start periodic price alert checking
    pub fn start_checking(
        db: Arc<Mutex<Database>>,
        ws_server: Arc<WsServer>,
        api_key_manager: Arc<APIKeyManager>,
        rate_limiter: Arc<Mutex<RateLimiter>>,
        app: AppHandle,
        event_bus: Option<Arc<AutomationEventBus>>,
    ) {
        tauri::async_runtime::spawn(async move {
            let mut interval = tokio::time::interval(tokio::time::Duration::from_secs(30)); // Check every 30 seconds

            loop {
                interval.tick().await;

                if let Err(e) = Self::check_alerts(&db, &ws_server, &api_key_manager, &rate_limiter, &app, event_bus.as_ref()).await {
                    eprintln!("Error checking price alerts: {}", e);
                }
            }
        });
    }

    async fn check_alerts(
        db: &Arc<Mutex<Database>>,
        ws_server: &Arc<WsServer>,
        api_key_manager: &Arc<APIKeyManager>,
        rate_limiter: &Arc<Mutex<RateLimiter>>,
        app: &AppHandle,
        event_bus: Option<&Arc<AutomationEventBus>>,
    ) -> Result<()> {
        // Clone the connection before any await to avoid holding MutexGuard across await
        let conn = {
            let db_guard = db.lock()
                .map_err(|e| anyhow::anyhow!("Database lock poisoned: {}", e))?;
            db_guard.conn.clone()
        };
        let store = PriceAlertStore::new(conn);

        // Get all enabled, non-triggered alerts
        let alerts = store.list_alerts(None, true)?;
        let active_alerts: Vec<PriceAlert> = alerts
            .into_iter()
            .filter(|a| !a.triggered)
            .collect();

        if active_alerts.is_empty() {
            return Ok(());
        }

        // Group by ticker to batch price fetches
        let mut ticker_alerts: std::collections::HashMap<String, Vec<PriceAlert>> = 
            std::collections::HashMap::new();
        for alert in active_alerts {
            ticker_alerts.entry(alert.ticker.clone()).or_insert_with(Vec::new).push(alert);
        }

        // Create market manager for this check
        let market_manager = MarketDataManager::new(Some(&**api_key_manager));

        // Check each ticker
        for (ticker, alerts_for_ticker) in ticker_alerts {
            // Get current price - rate limiter is optional, pass None to avoid holding lock across await
            let price_result = market_manager.get_price(&ticker, None).await;
            
            if let Ok(price_data) = price_result {
                let current_price = price_data.price;

                // Update current price in database
                for alert in &alerts_for_ticker {
                    if let Err(e) = store.update_alert_price(alert.id, current_price) {
                        eprintln!("Failed to update alert price: {}", e);
                    }
                }

                // Check each alert condition
                for alert in alerts_for_ticker {
                    let should_trigger = match alert.condition.as_str() {
                        "above" => current_price > alert.target_price,
                        "below" => current_price < alert.target_price,
                        "cross_above" => {
                            // Triggered if price was below and now above
                            if let Some(prev_price) = alert.current_price {
                                prev_price <= alert.target_price && current_price > alert.target_price
                            } else {
                                current_price > alert.target_price
                            }
                        }
                        "cross_below" => {
                            // Triggered if price was above and now below
                            if let Some(prev_price) = alert.current_price {
                                prev_price >= alert.target_price && current_price < alert.target_price
                            } else {
                                current_price < alert.target_price
                            }
                        }
                        _ => false,
                    };

                    if should_trigger {
                        // Mark as triggered
                        if let Err(e) = store.mark_triggered(alert.id) {
                            eprintln!("Failed to mark alert as triggered: {}", e);
                            continue;
                        }

                        // Emit Tauri event for frontend and desktop notification
                        let alert_message = serde_json::json!({
                            "type": "price_alert",
                            "alert_id": alert.id,
                            "ticker": alert.ticker,
                            "condition": alert.condition,
                            "target_price": alert.target_price,
                            "current_price": current_price,
                            "triggered_at": chrono::Utc::now().timestamp(),
                        });

                        // Send desktop notification
                        use crate::services::desktop_notifications::DesktopNotificationService;
                        let _ = DesktopNotificationService::send_price_alert_notification(
                            &app,
                            &alert.ticker,
                            &alert.condition,
                            alert.target_price,
                            current_price,
                        ).await;
                        
                        // Emit via Tauri event system (frontend will handle WebSocket if needed)
                        use tauri::Emitter;
                        let _ = app.emit("price-alert-triggered", &alert_message);
                        
                        // Also emit via ws-message for consistency
                        let _ = app.emit("ws-message", serde_json::json!({
                            "type": "price-alert",
                            "data": alert_message,
                            "timestamp": chrono::Utc::now().timestamp_millis(),
                        }));
                        
                        // Forward to AutomationEventBus
                        if let Some(bus) = event_bus {
                            let _ = crate::services::EventBridge::emit_price_alert(bus, &alert_message).await;
                        }

                        eprintln!("Price alert triggered: {} {} ${:.2} (current: ${:.2})", 
                            alert.ticker, alert.condition, alert.target_price, current_price);
                    }
                }
            }
        }

        Ok(())
    }
}

