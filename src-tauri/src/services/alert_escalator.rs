use crate::storage::temporal::{TemporalStore, Alert, AlertRule, AlertEscalation};
use anyhow::Result;
use serde_json::Value;

pub struct AlertEscalator;

impl AlertEscalator {
    /// Check if alert should be escalated and create escalation records
    pub fn check_and_escalate(
        store: &TemporalStore,
        alert: &Alert,
        rule: &AlertRule,
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
                                    if let Err(e) = Self::send_escalation(store, escalation_id, channel, alert, level_config) {
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

    fn send_escalation(
        store: &TemporalStore,
        escalation_id: i64,
        channel: &str,
        alert: &Alert,
        level_config: &Value,
    ) -> Result<()> {
        match channel {
            "email" => {
                // TODO: Implement email sending
                // For now, just log
                eprintln!("Email escalation for alert {}: {}", alert.id, alert.payload_json);
                Ok(())
            }
            "sms" => {
                // TODO: Implement SMS sending
                eprintln!("SMS escalation for alert {}: {}", alert.id, alert.payload_json);
                Ok(())
            }
            "webhook" => {
                if let Some(url) = level_config.get("webhook_url").and_then(|v| v.as_str()) {
                    // TODO: Implement webhook POST
                    eprintln!("Webhook escalation for alert {} to {}: {}", alert.id, url, alert.payload_json);
                    Ok(())
                } else {
                    anyhow::bail!("Webhook URL not configured")
                }
            }
            "push" => {
                // TODO: Implement push notification
                eprintln!("Push escalation for alert {}: {}", alert.id, alert.payload_json);
                Ok(())
            }
            _ => {
                anyhow::bail!("Unknown channel: {}", channel)
            }
        }
    }
}
