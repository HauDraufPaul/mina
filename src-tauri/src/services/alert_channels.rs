use anyhow::{Context, Result};
use serde_json::Value;
use std::collections::HashMap;
use tauri::Emitter;

pub struct AlertChannelSender;

impl AlertChannelSender {
    /// Send alert via email
    pub async fn send_email(
        alert_id: i64,
        alert_title: &str,
        alert_message: &str,
        recipient: &str,
        config: Option<&Value>,
    ) -> Result<()> {
        // For now, use SMTP (can be enhanced with SendGrid, Mailgun, etc.)
        // Check if SMTP config is available
        if let Some(config) = config {
            if let (Some(smtp_host), Some(smtp_port), Some(smtp_user), Some(smtp_pass)) = (
                config.get("smtp_host").and_then(|v| v.as_str()),
                config.get("smtp_port").and_then(|v| v.as_u64()),
                config.get("smtp_user").and_then(|v| v.as_str()),
                config.get("smtp_pass").and_then(|v| v.as_str()),
            ) {
                // TODO: Implement SMTP sending with lettre
                // For now, log the email
                eprintln!("[EMAIL] To: {}, Subject: {}, Body: {}", recipient, alert_title, alert_message);
                eprintln!("[EMAIL] SMTP: {}:{}", smtp_host, smtp_port);
                return Ok(());
            }
        }
        
        // Fallback: log email
        eprintln!("[EMAIL] To: {}, Subject: {}, Body: {}", recipient, alert_title, alert_message);
        Ok(())
    }

    /// Send alert via SMS
    pub async fn send_sms(
        alert_id: i64,
        alert_message: &str,
        recipient: &str,
        config: Option<&Value>,
    ) -> Result<()> {
        // Check for Twilio or other SMS provider config
        if let Some(config) = config {
            if let (Some(api_key), Some(api_secret), Some(from_number)) = (
                config.get("api_key").and_then(|v| v.as_str()),
                config.get("api_secret").and_then(|v| v.as_str()),
                config.get("from_number").and_then(|v| v.as_str()),
            ) {
                // Try Twilio first
                if let Some(twilio_account_sid) = config.get("twilio_account_sid").and_then(|v| v.as_str()) {
                    return Self::send_twilio_sms(
                        twilio_account_sid,
                        api_key,
                        api_secret,
                        from_number,
                        recipient,
                        alert_message,
                    ).await;
                }
                
                // TODO: Add other SMS providers (AWS SNS, etc.)
            }
        }
        
        // Fallback: log SMS
        eprintln!("[SMS] To: {}, Message: {}", recipient, alert_message);
        Ok(())
    }

    async fn send_twilio_sms(
        account_sid: &str,
        api_key: &str,
        api_secret: &str,
        from: &str,
        to: &str,
        message: &str,
    ) -> Result<()> {
        let client = reqwest::Client::new();
        let url = format!("https://api.twilio.com/2010-04-01/Accounts/{}/Messages.json", account_sid);
        
        let mut params = HashMap::new();
        params.insert("From", from);
        params.insert("To", to);
        params.insert("Body", message);
        
        let response = client
            .post(&url)
            .basic_auth(api_key, Some(api_secret))
            .form(&params)
            .send()
            .await
            .context("Failed to send Twilio SMS request")?;
        
        let status = response.status();
        if !status.is_success() {
            let error_text = response.text().await.unwrap_or_default();
            anyhow::bail!("Twilio API error: {} - {}", status, error_text);
        }
        
        Ok(())
    }

    /// Send alert via webhook
    pub async fn send_webhook(
        alert_id: i64,
        alert_data: &Value,
        webhook_url: &str,
        config: Option<&Value>,
    ) -> Result<()> {
        let client = reqwest::Client::new();
        
        // Build request body
        let mut body = serde_json::json!({
            "alert_id": alert_id,
            "timestamp": chrono::Utc::now().timestamp(),
            "data": alert_data,
        });
        
        // Add custom headers if configured
        let mut request = client.post(webhook_url);
        if let Some(config) = config {
            if let Some(headers) = config.get("headers").and_then(|v| v.as_object()) {
                for (key, value) in headers {
                    if let Some(header_value) = value.as_str() {
                        request = request.header(key, header_value);
                    }
                }
            }
            
            // Add custom payload fields
            if let Some(custom_fields) = config.get("custom_fields").and_then(|v| v.as_object()) {
                for (key, value) in custom_fields {
                    body[key] = value.clone();
                }
            }
        }
        
        let response = request
            .json(&body)
            .send()
            .await
            .context("Failed to send webhook request")?;
        
        let status = response.status();
        if !status.is_success() {
            let error_text = response.text().await.unwrap_or_default();
            anyhow::bail!("Webhook error: {} - {}", status, error_text);
        }
        
        Ok(())
    }

    /// Send alert via push notification (desktop)
    pub async fn send_push(
        alert_id: i64,
        alert_title: &str,
        alert_message: &str,
        app: Option<tauri::AppHandle>,
    ) -> Result<()> {
        if let Some(app_handle) = app {
            // Emit event to all windows
            app_handle
                .emit("alert-notification", serde_json::json!({
                    "id": alert_id,
                    "title": alert_title,
                    "message": alert_message,
                }))
                .map_err(|e| anyhow::anyhow!("Failed to emit notification: {}", e))?;
        } else {
            // Fallback: log push
            eprintln!("[PUSH] Title: {}, Message: {}", alert_title, alert_message);
        }
        
        Ok(())
    }
}

