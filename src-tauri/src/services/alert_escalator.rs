use crate::storage::temporal::{TemporalStore, Alert, AlertRule, AlertEscalation};
use crate::services::alert_channels::AlertChannelSender;
use anyhow::Result;
use serde_json::Value;
use tauri::AppHandle;

pub struct AlertEscalator;

impl AlertEscalator {
    /// Check if alert should be escalated and create escalation records
    pub async fn check_and_escalate(
        store: &TemporalStore,
        alert: &Alert,
        rule: &AlertRule,
        app: Option<tauri::AppHandle>,
    ) -> Result<Vec<AlertEscalation>> {
        let mut escalations = Vec::new();

        // Check if rule has escalation config
        if let Some(escalation_config) = &rule.escalation_config {
            if let Some(levels) = escalation_config.get("levels").and_then(|v| v.as_array()) {
                for (level_idx, level_config) in levels.iter().enumerate() {
                    let escalation_level = level_idx as i32 + 1;
                    
                    // Check if this level should trigger
                    if Self::should_escalate(store, alert, escalation_level, level_config)? {
                        // Get channels for this level
                        if let Some(channels) = level_config.get("channels").and_then(|v| v.as_array()) {
                            for channel_value in channels {
                                if let Some(channel) = channel_value.as_str() {
                                    let escalation_id = store.create_escalation(
                                        alert.id,
                                        escalation_level,
                                        channel,
                                    )?;
                                    
                                    // Try to send escalation
                                    if let Err(e) = Self::send_escalation(store, escalation_id, channel, alert, level_config, app.clone()).await {
                                        eprintln!("Failed to send escalation: {}", e);
                                        store.mark_escalation_sent(escalation_id, Some(&format!("{}", e)))?;
                                    } else {
                                        store.mark_escalation_sent(escalation_id, None)?;
                                    }
                                    
                                    if let Ok(escalation) = store.get_alert_escalations(alert.id) {
                                        if let Some(esc) = escalation.iter().find(|e| e.id == escalation_id) {
                                            escalations.push(esc.clone());
                                        }
                                    }
                                }
                            }
                        }
                    }
                }
            }
        }

        Ok(escalations)
    }

    fn should_escalate(
        store: &TemporalStore,
        alert: &Alert,
        escalation_level: i32,
        level_config: &Value,
    ) -> Result<bool> {
        // Check time-based escalation
        if let Some(delay_minutes) = level_config.get("delay_minutes").and_then(|v| v.as_i64()) {
            let now = chrono::Utc::now().timestamp();
            let delay_seconds = delay_minutes * 60;
            
            // Check if alert is still unacknowledged after delay
            if alert.status == "new" && (now - alert.fired_at) >= delay_seconds {
                // Check if already escalated at this level
                let existing = store.get_alert_escalations(alert.id)?;
                if !existing.iter().any(|e| e.escalation_level == escalation_level) {
                    return Ok(true);
                }
            }
        }

        // Check if alert was manually escalated
        if let Some(manual) = level_config.get("manual").and_then(|v| v.as_bool()) {
            if manual {
                // Manual escalation - would be triggered by user action
                return Ok(false);
            }
        }

        Ok(false)
    }

    pub async fn send_escalation(
        _store: &TemporalStore,
        _escalation_id: i64,
        channel: &str,
        alert: &Alert,
        level_config: &Value,
        app: Option<tauri::AppHandle>,
    ) -> Result<()> {
        // Alert payload is already a Value
        let alert_payload = &alert.payload_json;
        
        let alert_title = alert_payload.get("title")
            .and_then(|v| v.as_str())
            .unwrap_or("Alert")
            .to_string();
        
        let alert_message = alert_payload.get("message")
            .and_then(|v| v.as_str())
            .map(|s| s.to_string())
            .unwrap_or_else(|| {
                serde_json::to_string(alert_payload).unwrap_or_else(|_| "Alert triggered".to_string())
            });
        
        match channel {
            "email" => {
                let recipient = level_config.get("email")
                    .and_then(|v| v.as_str())
                    .ok_or_else(|| anyhow::anyhow!("Email recipient not configured"))?;
                
                AlertChannelSender::send_email(
                    alert.id,
                    &alert_title,
                    &alert_message,
                    recipient,
                    level_config.get("email_config"),
                ).await
            }
            "sms" => {
                let recipient = level_config.get("phone")
                    .and_then(|v| v.as_str())
                    .ok_or_else(|| anyhow::anyhow!("Phone number not configured"))?;
                
                AlertChannelSender::send_sms(
                    alert.id,
                    &alert_message,
                    recipient,
                    level_config.get("sms_config"),
                ).await
            }
            "webhook" => {
                let url = level_config.get("webhook_url")
                    .and_then(|v| v.as_str())
                    .ok_or_else(|| anyhow::anyhow!("Webhook URL not configured"))?;
                
                AlertChannelSender::send_webhook(
                    alert.id,
                    &alert_payload,
                    url,
                    level_config.get("webhook_config"),
                ).await
            }
            "push" => {
                AlertChannelSender::send_push(
                    alert.id,
                    &alert_title,
                    &alert_message,
                    app,
                ).await
            }
            _ => {
                anyhow::bail!("Unknown channel: {}", channel)
            }
        }
    }
}
