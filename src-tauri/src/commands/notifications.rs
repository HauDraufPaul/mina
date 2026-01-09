use crate::services::desktop_notifications::DesktopNotificationService;
use crate::services::desktop_notifications::NotificationOptions;
use tauri::{AppHandle, Manager};

#[tauri::command]
pub async fn send_notification(
    title: String,
    body: String,
    icon: Option<String>,
    sound: Option<String>,
    tag: Option<String>,
    data: Option<serde_json::Value>,
    app: AppHandle,
) -> Result<(), String> {
    let options = NotificationOptions {
        title,
        body,
        icon,
        sound,
        tag,
        data,
    };
    
    DesktopNotificationService::send(&app, options)
        .await
        .map_err(|e| format!("Failed to send notification: {}", e))
}

#[tauri::command]
pub async fn send_alert_notification(
    alert_id: i64,
    title: String,
    message: String,
    app: AppHandle,
) -> Result<(), String> {
    DesktopNotificationService::send_alert_notification(&app, alert_id, &title, &message)
        .await
        .map_err(|e| format!("Failed to send alert notification: {}", e))
}

#[tauri::command]
pub async fn send_price_alert_notification(
    ticker: String,
    condition: String,
    target_price: f64,
    current_price: f64,
    app: AppHandle,
) -> Result<(), String> {
    DesktopNotificationService::send_price_alert_notification(
        &app,
        &ticker,
        &condition,
        target_price,
        current_price,
    )
    .await
    .map_err(|e| format!("Failed to send price alert notification: {}", e))
}

