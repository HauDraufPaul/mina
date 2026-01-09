use crate::storage::temporal::{TemporalStore, Alert, AlertRule};
use crate::services::alert_escalator::AlertEscalator;
use crate::storage::Database;
use std::sync::{Arc, Mutex};
use tokio::time::{interval, Duration};
use anyhow::Result;

pub struct AlertEscalationChecker;

impl AlertEscalationChecker {
    /// Start periodic escalation checks for time-based escalations
    pub fn start_periodic_checks(
        db: Arc<Mutex<Database>>,
        app: tauri::AppHandle,
    ) {
        tauri::async_runtime::spawn(async move {
            let mut interval = interval(Duration::from_secs(60)); // Check every minute

            loop {
                interval.tick().await;

                if let Err(e) = Self::check_pending_escalations(&db) {
                    eprintln!("Error checking pending escalations: {}", e);
                }
            }
        });
    }

    fn check_pending_escalations(db: &Arc<Mutex<Database>>) -> Result<()> {
        let db_guard = db.lock().map_err(|e| anyhow::anyhow!("Database lock error: {}", e))?;
        let store = TemporalStore::new(db_guard.conn.clone());
        
        // Get all unacknowledged alerts
        let alerts = store.list_alerts(1000, None, None)?;
        let rules = store.list_alert_rules()?;
        
        let rules_map: std::collections::HashMap<i64, &AlertRule> = rules
            .iter()
            .map(|r| (r.id, r))
            .collect();

        for alert in alerts {
            if alert.status == "new" || alert.status == "snoozed" {
                if let Some(rule) = rules_map.get(&alert.rule_id) {
                    if let Err(e) = AlertEscalator::check_and_escalate(&store, &alert, rule) {
                        eprintln!("Failed to check escalation for alert {}: {}", alert.id, e);
                    }
                }
            }
        }

        Ok(())
    }
}

