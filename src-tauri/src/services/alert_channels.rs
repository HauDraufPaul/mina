use anyhow::{Context, Result};
use serde_json::Value;
use std::collections::HashMap;
use lettre::{
    message::{header::ContentType, Mailbox, Message, SinglePart},
    transport::smtp::authentication::Credentials,
    AsyncSmtpTransport, AsyncTransport, Tokio1Executor,
};

pub struct AlertChannelSender;

impl AlertChannelSender {
    /// Send alert via email
    pub async fn send_email(
        _alert_id: i64,
        alert_title: &str,
        alert_message: &str,
        recipient: &str,
        config: Option<&Value>,
    ) -> Result<()> {
        // Check if SMTP config is available
        if let Some(config) = config {
            if let (Some(smtp_host), Some(smtp_port), Some(smtp_user), Some(smtp_pass)) = (
                config.get("smtp_host").and_then(|v| v.as_str()),
                config.get("smtp_port").and_then(|v| v.as_u64()),
                config.get("smtp_user").and_then(|v| v.as_str()),
                config.get("smtp_pass").and_then(|v| v.as_str()),
            ) {
                // Parse recipient email
                let to_email: Mailbox = recipient
                    .parse()
                    .context(format!("Invalid recipient email address: {}", recipient))?;
                
                // Parse from email (use smtp_user or default)
                let from_email: Mailbox = smtp_user
                    .parse()
                    .context(format!("Invalid sender email address: {}", smtp_user))?;
                
                // Build email message
                let email = Message::builder()
                    .from(from_email)
                    .to(to_email)
                    .subject(alert_title)
                    .singlepart(
                        SinglePart::builder()
                            .header(ContentType::TEXT_PLAIN)
                            .body(alert_message.to_string()),
                    )
                    .context("Failed to build email message")?;
                
                // Create SMTP transport
                let smtp_port_u16 = smtp_port.min(u16::MAX as u64) as u16;
                
                // Determine if we need TLS (check config or default to STARTTLS)
                let use_tls = config
                    .get("use_tls")
                    .and_then(|v| v.as_bool())
                    .unwrap_or(true);
                
                let mailer = if use_tls {
                    AsyncSmtpTransport::<Tokio1Executor>::relay(smtp_host)
                        .context(format!("Failed to create SMTP relay for {}", smtp_host))?
                        .port(smtp_port_u16)
                        .credentials(Credentials::new(smtp_user.to_string(), smtp_pass.to_string()))
                        .build()
                } else {
                    // For non-TLS (not recommended but supported)
                    AsyncSmtpTransport::<Tokio1Executor>::builder_dangerous(smtp_host)
                        .port(smtp_port_u16)
                        .credentials(Credentials::new(smtp_user.to_string(), smtp_pass.to_string()))
                        .build()
                };
                
                // Send email
                mailer
                    .send(email)
                    .await
                    .context("Failed to send email via SMTP")?;
                
                return Ok(());
            }
        }
        
        // Fallback: log email if no SMTP config
        eprintln!("[EMAIL] No SMTP config provided. To: {}, Subject: {}, Body: {}", recipient, alert_title, alert_message);
        Ok(())
    }

    /// Send alert via SMS
    pub async fn send_sms(
        _alert_id: i64,
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
                
                // Try AWS SNS
                if let Some(aws_region) = config.get("aws_region").and_then(|v| v.as_str()) {
                    if let Some(aws_access_key_id) = config.get("aws_access_key_id").and_then(|v| v.as_str()) {
                        if let Some(aws_secret_access_key) = config.get("aws_secret_access_key").and_then(|v| v.as_str()) {
                            return Self::send_aws_sns_sms(
                                aws_region,
                                aws_access_key_id,
                                aws_secret_access_key,
                                from_number,
                                recipient,
                                alert_message,
                            ).await;
                        }
                    }
                }
                
                // Try Vonage
                if config.get("vonage_api_key").and_then(|v| v.as_str()).is_some() {
                    if let Some(vonage_api_key) = config.get("vonage_api_key").and_then(|v| v.as_str()) {
                        if let Some(vonage_api_secret) = config.get("vonage_api_secret").and_then(|v| v.as_str()) {
                            return Self::send_vonage_sms(
                                vonage_api_key,
                                vonage_api_secret,
                                from_number,
                                recipient,
                                alert_message,
                            ).await;
                        }
                    }
                }
                
                // Try MessageBird
                if let Some(messagebird_api_key) = config.get("messagebird_api_key").and_then(|v| v.as_str()) {
                    return Self::send_messagebird_sms(
                        messagebird_api_key,
                        from_number,
                        recipient,
                        alert_message,
                    ).await;
                }
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

    async fn send_aws_sns_sms(
        region: &str,
        access_key_id: &str,
        secret_access_key: &str,
        from: &str,
        to: &str,
        message: &str,
    ) -> Result<()> {
        use chrono::Utc;
        use hmac::{Hmac, Mac};
        use sha2::{Sha256, Digest};
        use std::collections::BTreeMap;
        
        // AWS SNS Publish API endpoint
        let endpoint = format!("https://sns.{}.amazonaws.com/", region);
        
        // Prepare parameters
        let mut params = BTreeMap::new();
        params.insert("Action", "Publish");
        params.insert("PhoneNumber", to);
        params.insert("Message", message);
        if !from.is_empty() {
            params.insert("MessageAttributes.entry.1.Name", "AWS.SNS.SMS.SenderID");
            params.insert("MessageAttributes.entry.1.Value.DataType", "String");
            params.insert("MessageAttributes.entry.1.Value.StringValue", from);
        }
        params.insert("Version", "2010-03-31");
        
        // Create query string
        let query_string = params.iter()
            .map(|(k, v)| format!("{}={}", urlencoding::encode(k), urlencoding::encode(v)))
            .collect::<Vec<_>>()
            .join("&");
        
        // AWS Signature Version 4
        let now = Utc::now();
        let amz_date = now.format("%Y%m%dT%H%M%SZ").to_string();
        let date_stamp = now.format("%Y%m%d").to_string();
        
        // Create canonical request
        let canonical_uri = "/";
        let canonical_querystring = query_string;
        let canonical_headers = format!(
            "host:sns.{}.amazonaws.com\nx-amz-date:{}\n",
            region, amz_date
        );
        let signed_headers = "host;x-amz-date";
        let payload_hash = {
            let mut hasher = Sha256::new();
            hasher.update(canonical_querystring.as_bytes());
            format!("{:x}", hasher.finalize())
        };
        
        let canonical_request = format!(
            "POST\n{}\n{}\n{}\n{}\n{}",
            canonical_uri, "", canonical_headers, signed_headers, payload_hash
        );
        
        // Create string to sign
        let algorithm = "AWS4-HMAC-SHA256";
        let credential_scope = format!("{}/{}/sns/aws4_request", date_stamp, region);
        let string_to_sign = format!(
            "{}\n{}\n{}\n{}",
            algorithm,
            amz_date,
            credential_scope,
            {
                let mut hasher = Sha256::new();
                hasher.update(canonical_request.as_bytes());
                format!("{:x}", hasher.finalize())
            }
        );
        
        // Calculate signature
        let k_date = {
            let mut mac = Hmac::<Sha256>::new_from_slice(format!("AWS4{}", secret_access_key).as_bytes())
                .map_err(|e| anyhow::anyhow!("HMAC error: {}", e))?;
            mac.update(date_stamp.as_bytes());
            mac.finalize().into_bytes()
        };
        
        let k_region = {
            let mut mac = Hmac::<Sha256>::new_from_slice(&k_date)
                .map_err(|e| anyhow::anyhow!("HMAC error: {}", e))?;
            mac.update(region.as_bytes());
            mac.finalize().into_bytes()
        };
        
        let k_service = {
            let mut mac = Hmac::<Sha256>::new_from_slice(&k_region)
                .map_err(|e| anyhow::anyhow!("HMAC error: {}", e))?;
            mac.update(b"sns");
            mac.finalize().into_bytes()
        };
        
        let k_signing = {
            let mut mac = Hmac::<Sha256>::new_from_slice(&k_service)
                .map_err(|e| anyhow::anyhow!("HMAC error: {}", e))?;
            mac.update(b"aws4_request");
            mac.finalize().into_bytes()
        };
        
        let signature = {
            let mut mac = Hmac::<Sha256>::new_from_slice(&k_signing)
                .map_err(|e| anyhow::anyhow!("HMAC error: {}", e))?;
            mac.update(string_to_sign.as_bytes());
            format!("{:x}", mac.finalize().into_bytes())
        };
        
        // Create authorization header
        let authorization = format!(
            "{} Credential={}/{}, SignedHeaders={}, Signature={}",
            algorithm, access_key_id, credential_scope, signed_headers, signature
        );
        
        // Make request
        let client = reqwest::Client::new();
        let response = client
            .post(&endpoint)
            .header("Authorization", authorization)
            .header("x-amz-date", amz_date)
            .header("Content-Type", "application/x-www-form-urlencoded")
            .body(canonical_querystring)
            .send()
            .await
            .context("Failed to send AWS SNS SMS request")?;
        
        let status = response.status();
        if !status.is_success() {
            let error_text = response.text().await.unwrap_or_default();
            anyhow::bail!("AWS SNS API error: {} - {}", status, error_text);
        }
        
        Ok(())
    }

    async fn send_vonage_sms(
        api_key: &str,
        api_secret: &str,
        from: &str,
        to: &str,
        message: &str,
    ) -> Result<()> {
        let client = reqwest::Client::new();
        let url = "https://rest.nexmo.com/sms/json";
        
        let mut params = HashMap::new();
        params.insert("api_key", api_key);
        params.insert("api_secret", api_secret);
        params.insert("from", from);
        params.insert("to", to);
        params.insert("text", message);
        
        let response = client
            .post(url)
            .form(&params)
            .send()
            .await
            .context("Failed to send Vonage SMS request")?;
        
        let status = response.status();
        if !status.is_success() {
            let error_text = response.text().await.unwrap_or_default();
            anyhow::bail!("Vonage API error: {} - {}", status, error_text);
        }
        
        // Check response body for Vonage-specific error codes
        let response_body: serde_json::Value = response.json().await
            .context("Failed to parse Vonage response")?;
        
        if let Some(messages) = response_body.get("messages").and_then(|v| v.as_array()) {
            if let Some(first_msg) = messages.first() {
                if let Some(status) = first_msg.get("status").and_then(|v| v.as_str()) {
                    if status != "0" {
                        let error_text = first_msg.get("error-text")
                            .and_then(|v| v.as_str())
                            .unwrap_or("Unknown error");
                        anyhow::bail!("Vonage SMS error (status {}): {}", status, error_text);
                    }
                }
            }
        }
        
        Ok(())
    }

    async fn send_messagebird_sms(
        api_key: &str,
        from: &str,
        to: &str,
        message: &str,
    ) -> Result<()> {
        let client = reqwest::Client::new();
        let url = "https://rest.messagebird.com/messages";
        
        let body = serde_json::json!({
            "originator": from,
            "recipients": [to],
            "body": message,
        });
        
        let response = client
            .post(url)
            .header("Authorization", format!("AccessKey {}", api_key))
            .header("Content-Type", "application/json")
            .json(&body)
            .send()
            .await
            .context("Failed to send MessageBird SMS request")?;
        
        let status = response.status();
        if !status.is_success() {
            let error_text = response.text().await.unwrap_or_default();
            anyhow::bail!("MessageBird API error: {} - {}", status, error_text);
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
            // Use desktop notification service
            use crate::services::desktop_notifications::DesktopNotificationService;
            DesktopNotificationService::send_alert_notification(
                &app_handle,
                alert_id,
                alert_title,
                alert_message,
            )
            .await
            .context("Failed to send desktop notification")?;
        } else {
            // Fallback: log push
            eprintln!("[PUSH] Title: {}, Message: {}", alert_title, alert_message);
        }
        
        Ok(())
    }
}

