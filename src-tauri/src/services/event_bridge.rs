use anyhow::Result;
use serde_json::Value;
use std::sync::Arc;
use crate::services::AutomationEventBus;

/// Helper to emit events to AutomationEventBus from event emission points
pub struct EventBridge;

impl EventBridge {
    /// Emit a price alert event to the AutomationEventBus
    pub async fn emit_price_alert(event_bus: &Arc<AutomationEventBus>, alert_data: &Value) -> Result<()> {
        if let Some(alert_id) = alert_data.get("alert_id").and_then(|v| v.as_i64()) {
            let _ = event_bus.emit(crate::services::AutomationEvent::AlertFired {
                alert_id,
                rule_id: 0, // Price alerts don't have rule_id
                event_id: None,
                payload: alert_data.clone(),
            }).await;
        }
        Ok(())
    }

    /// Emit a temporal alert event to the AutomationEventBus
    pub async fn emit_temporal_alert(event_bus: &Arc<AutomationEventBus>, alert_data: &Value) -> Result<()> {
        if let Some(alert_id) = alert_data.get("id").and_then(|v| v.as_i64()) {
            let _ = event_bus.emit(crate::services::AutomationEvent::AlertFired {
                alert_id,
                rule_id: alert_data.get("rule_id")
                    .and_then(|v| v.get("id"))
                    .and_then(|v| v.as_i64())
                    .unwrap_or(0),
                event_id: alert_data.get("event_id").and_then(|v| v.as_i64()),
                payload: alert_data.clone(),
            }).await;
        }
        Ok(())
    }

    /// Emit a news published event to the AutomationEventBus
    pub async fn emit_news_published(event_bus: &Arc<AutomationEventBus>, news_data: &Value) -> Result<()> {
        if let Some(news_id) = news_data.get("id").and_then(|v| v.as_i64()) {
            let _ = event_bus.emit(crate::services::AutomationEvent::NewsPublished {
                news_id,
                ticker: news_data.get("ticker").and_then(|v| v.as_str()).map(|s| s.to_string()),
                title: news_data.get("title").and_then(|v| v.as_str()).unwrap_or("").to_string(),
                source: news_data.get("source").and_then(|v| v.as_str()).unwrap_or("").to_string(),
            }).await;
        }
        Ok(())
    }
}

