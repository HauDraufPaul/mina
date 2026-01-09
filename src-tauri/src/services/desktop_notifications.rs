use anyhow::Result;
use serde::{Deserialize, Serialize};
use tauri::{AppHandle, Manager, Emitter};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NotificationOptions {
    pub title: String,
    pub body: String,
    pub icon: Option<String>,
    pub sound: Option<String>,
    pub tag: Option<String>, // For grouping/updating notifications
    pub data: Option<serde_json::Value>,
}

pub struct DesktopNotificationService;

impl DesktopNotificationService {
    /// Send a desktop notification
    pub async fn send(
        app: &AppHandle,
        options: NotificationOptions,
    ) -> Result<()> {
        // Try to use Tauri's notification plugin if available
        // Otherwise, fall back to emitting an event for the frontend to handle
        
        // Emit event for frontend to handle with browser notifications
        app.emit("desktop-notification", &options)
            .map_err(|e| anyhow::anyhow!("Failed to emit notification event: {}", e))?;
        
        Ok(())
    }

    /// Send a notification for an alert
    pub async fn send_alert_notification(
        app: &AppHandle,
        alert_id: i64,
        title: &str,
        message: &str,
    ) -> Result<()> {
        let options = NotificationOptions {
            title: title.to_string(),
            body: message.to_string(),
            icon: Some("alert".to_string()),
            sound: Some("default".to_string()),
            tag: Some(format!("alert-{}", alert_id)),
            data: Some(serde_json::json!({
                "type": "alert",
                "alert_id": alert_id,
            })),
        };
        
        Self::send(app, options).await
    }

    /// Send a notification for a price alert
    pub async fn send_price_alert_notification(
        app: &AppHandle,
        ticker: &str,
        condition: &str,
        target_price: f64,
        current_price: f64,
    ) -> Result<()> {
        let title = format!("Price Alert: {}", ticker);
        let body = format!(
            "{} {} ${:.2} (Current: ${:.2})",
            ticker, condition, target_price, current_price
        );
        
        let options = NotificationOptions {
            title,
            body,
            icon: Some("price-alert".to_string()),
            sound: Some("default".to_string()),
            tag: Some(format!("price-alert-{}", ticker)),
            data: Some(serde_json::json!({
                "type": "price_alert",
                "ticker": ticker,
                "condition": condition,
                "target_price": target_price,
                "current_price": current_price,
            })),
        };
        
        Self::send(app, options).await
    }

    /// Send a notification for news
    pub async fn send_news_notification(
        app: &AppHandle,
        title: &str,
        source: &str,
        tickers: &[String],
    ) -> Result<()> {
        let body = if tickers.is_empty() {
            format!("From {}", source)
        } else {
            format!("From {} • {}", source, tickers.join(", "))
        };
        
        let options = NotificationOptions {
            title: title.to_string(),
            body,
            icon: Some("news".to_string()),
            sound: None,
            tag: None,
            data: Some(serde_json::json!({
                "type": "news",
                "source": source,
                "tickers": tickers,
            })),
        };
        
        Self::send(app, options).await
    }

    /// Send a notification for portfolio updates
    pub async fn send_portfolio_notification(
        app: &AppHandle,
        portfolio_name: &str,
        message: &str,
    ) -> Result<()> {
        let options = NotificationOptions {
            title: format!("Portfolio: {}", portfolio_name),
            body: message.to_string(),
            icon: Some("portfolio".to_string()),
            sound: None,
            tag: Some(format!("portfolio-{}", portfolio_name)),
            data: Some(serde_json::json!({
                "type": "portfolio",
                "portfolio_name": portfolio_name,
            })),
        };
        
        Self::send(app, options).await
    }
}

