use anyhow::Result;
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};
use std::collections::HashMap;
use std::sync::{Arc, Mutex};
use tauri::AppHandle;

use crate::storage::Database;
use crate::storage::automation::AutomationStore;
use crate::services::workflow_engine::WorkflowEngine;

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type")]
pub enum AutomationEvent {
    #[serde(rename = "market_price_change")]
    MarketPriceChange {
        ticker: String,
        old_price: f64,
        new_price: f64,
        change_percent: f64,
    },
    #[serde(rename = "alert_fired")]
    AlertFired {
        alert_id: i64,
        rule_id: i64,
        event_id: Option<i64>,
        payload: Value,
    },
    #[serde(rename = "news_published")]
    NewsPublished {
        news_id: i64,
        ticker: Option<String>,
        title: String,
        source: String,
    },
    #[serde(rename = "portfolio_value_change")]
    PortfolioValueChange {
        portfolio_id: i64,
        old_value: f64,
        new_value: f64,
        change_percent: f64,
    },
    #[serde(rename = "temporal_event_created")]
    TemporalEventCreated {
        event_id: i64,
        title: String,
        event_type: String,
    },
}

#[derive(Clone)]
struct EventSubscription {
    workflow_id: i64,
    event_type: String,
    filter: Option<Value>,
}

pub struct AutomationEventBus {
    subscribers: Arc<Mutex<HashMap<String, Vec<EventSubscription>>>>,
    db: Arc<Mutex<Database>>,
    workflow_engine: Arc<WorkflowEngine>,
    app: AppHandle,
}

impl AutomationEventBus {
    pub fn new(
        db: Arc<Mutex<Database>>,
        workflow_engine: Arc<WorkflowEngine>,
        app: AppHandle,
    ) -> Self {
        let bus = AutomationEventBus {
            subscribers: Arc::new(Mutex::new(HashMap::new())),
            db,
            workflow_engine,
            app,
        };
        bus.reload_subscriptions();
        bus
    }

    pub fn reload_subscriptions(&self) {
        let db_guard = self.db.lock().unwrap();
        let store = AutomationStore::new(db_guard.conn.clone()).unwrap();
        
        let workflows = store.list_workflows().unwrap_or_default();
        let mut subscribers_guard = self.subscribers.lock().unwrap();
        subscribers_guard.clear();

        for workflow in workflows {
            if !workflow.enabled || workflow.trigger_type != "event" {
                continue;
            }

            // Parse trigger config
            let trigger_config: Value = serde_json::from_str(&workflow.trigger_config)
                .unwrap_or(json!({}));

            if let Some(event_type) = trigger_config.get("event").and_then(|v| v.as_str()) {
                let filter = trigger_config.get("filter").cloned();
                
                subscribers_guard
                    .entry(event_type.to_string())
                    .or_insert_with(Vec::new)
                    .push(EventSubscription {
                        workflow_id: workflow.id,
                        event_type: event_type.to_string(),
                        filter,
                    });
            }
        }
    }

    pub async fn emit(&self, event: AutomationEvent) -> Result<()> {
        let event_type = match &event {
            AutomationEvent::MarketPriceChange { .. } => "market_price_change",
            AutomationEvent::AlertFired { .. } => "alert_fired",
            AutomationEvent::NewsPublished { .. } => "news_published",
            AutomationEvent::PortfolioValueChange { .. } => "portfolio_value_change",
            AutomationEvent::TemporalEventCreated { .. } => "temporal_event_created",
        };

        let subscribers_guard = self.subscribers.lock().unwrap();
        let subscribers = subscribers_guard.get(event_type).cloned().unwrap_or_default();
        drop(subscribers_guard);

        // Convert event to JSON for filtering
        let event_json: Value = serde_json::to_value(&event)?;

        // Execute matching workflows
        for subscription in subscribers {
            // Check filter if present
            let should_execute = if let Some(filter) = &subscription.filter {
                self.matches_filter(&event_json, filter)
            } else {
                true
            };

            if should_execute {
                let engine = self.workflow_engine.clone();
                let event_data = serde_json::to_value(&event)?;
                
                tauri::async_runtime::spawn(async move {
                    if let Err(e) = engine.execute_workflow(subscription.workflow_id, Some(event_data)).await {
                        eprintln!("Failed to execute event-triggered workflow {}: {}", subscription.workflow_id, e);
                    }
                });
            }
        }

        Ok(())
    }

    fn matches_filter(&self, event: &Value, filter: &Value) -> bool {
        // Simple JSON path matching
        // For MVP, support basic equality checks
        if let Some(filter_obj) = filter.as_object() {
            for (key, filter_value) in filter_obj {
                let event_value = self.get_json_path(event, key);
                
                // Simple comparison
                if event_value != *filter_value {
                    return false;
                }
            }
        }
        true
    }

    fn get_json_path(&self, value: &Value, path: &str) -> Value {
        // Simple path resolution (e.g., "payload.sentiment")
        let parts: Vec<&str> = path.split('.').collect();
        let mut current = value;
        
        for part in parts {
            if let Some(obj) = current.as_object() {
                if let Some(next) = obj.get(part) {
                    current = next;
                } else {
                    return Value::Null;
                }
            } else {
                return Value::Null;
            }
        }
        
        current.clone()
    }
}

