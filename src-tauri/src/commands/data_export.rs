use crate::services::data_export::{DataExportService, ExportFormat, ExportDataType};
use crate::storage::Database;
use std::sync::Mutex;
use tauri::State;

#[tauri::command]
pub fn export_data(
    data_type: String,
    format: String,
    portfolio_id: Option<i64>,
    tickers: Option<Vec<String>>,
    from_ts: Option<i64>,
    to_ts: Option<i64>,
    limit: Option<i64>,
    db: State<'_, Mutex<Database>>,
) -> Result<String, String> {
    let export_format = match format.to_lowercase().as_str() {
        "csv" => ExportFormat::Csv,
        "json" => ExportFormat::Json,
        "excel" => ExportFormat::Excel,
        _ => return Err("Invalid format. Use 'csv', 'json', or 'excel'".to_string()),
    };

    let export_data_type = match data_type.to_lowercase().as_str() {
        "portfolio" => ExportDataType::Portfolio,
        "market_data" | "marketdata" => ExportDataType::MarketData,
        "news" => ExportDataType::News,
        "alerts" => ExportDataType::Alerts,
        "economic_calendar" | "economiccalendar" => ExportDataType::EconomicCalendar,
        "temporal_events" | "temporalevents" => ExportDataType::TemporalEvents,
        "price_alerts" | "pricealerts" => ExportDataType::PriceAlerts,
        _ => return Err(format!("Invalid data type: {}. Supported types: portfolio, market_data, news, alerts, economic_calendar, temporal_events, price_alerts", data_type)),
    };

    match export_data_type {
        ExportDataType::Portfolio => {
            let portfolio_id = portfolio_id.ok_or_else(|| "portfolio_id required for portfolio export".to_string())?;
            DataExportService::export_portfolio(portfolio_id, export_format, &db)
                .map_err(|e| format!("Failed to export portfolio: {}", e))
        }
        ExportDataType::MarketData => {
            let tickers = tickers.unwrap_or_default();
            if tickers.is_empty() {
                return Err("tickers required for market data export".to_string());
            }
            DataExportService::export_market_data(&tickers, from_ts, to_ts, export_format, &db)
                .map_err(|e| format!("Failed to export market data: {}", e))
        }
        ExportDataType::News => {
            DataExportService::export_news(tickers, from_ts, to_ts, export_format, &db)
                .map_err(|e| format!("Failed to export news: {}", e))
        }
        ExportDataType::Alerts => {
            DataExportService::export_alerts(limit, export_format, &db)
                .map_err(|e| format!("Failed to export alerts: {}", e))
        }
        ExportDataType::EconomicCalendar => {
            DataExportService::export_economic_calendar(from_ts, to_ts, export_format, &db)
                .map_err(|e| format!("Failed to export economic calendar: {}", e))
        }
        ExportDataType::TemporalEvents => {
            DataExportService::export_temporal_events(limit, from_ts, to_ts, export_format, &db)
                .map_err(|e| format!("Failed to export temporal events: {}", e))
        }
        ExportDataType::PriceAlerts => {
            // Price alerts export would use PriceAlertStore
            // For now, return error
            Err("Price alerts export not yet implemented".to_string())
        }
    }
}

