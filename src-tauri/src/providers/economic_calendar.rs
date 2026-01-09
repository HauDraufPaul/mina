use anyhow::{Context, Result};
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EconomicEventData {
    pub event_type: String,
    pub country: String,
    pub title: String,
    pub description: Option<String>,
    pub scheduled_at: i64,
    pub forecast_value: Option<String>,
    pub previous_value: Option<String>,
    pub impact: String, // "high", "medium", "low"
    pub currency: Option<String>,
}

#[derive(Debug, Serialize, Deserialize)]
struct TradingEconomicsResponse {
    #[serde(rename = "Calendar")]
    calendar: Vec<TradingEconomicsEvent>,
}

#[derive(Debug, Serialize, Deserialize)]
struct TradingEconomicsEvent {
    #[serde(rename = "Country")]
    country: String,
    #[serde(rename = "Category")]
    category: String,
    #[serde(rename = "Event")]
    event: String,
    #[serde(rename = "Reference")]
    reference: Option<String>,
    #[serde(rename = "Source")]
    source: Option<String>,
    #[serde(rename = "Importance")]
    importance: Option<String>,
    #[serde(rename = "Date")]
    date: String,
    #[serde(rename = "Forecast")]
    forecast: Option<String>,
    #[serde(rename = "Previous")]
    previous: Option<String>,
    #[serde(rename = "Actual")]
    actual: Option<String>,
}

pub struct EconomicCalendarProvider {
    api_key: Option<String>,
    client: reqwest::Client,
}

impl EconomicCalendarProvider {
    pub fn new(api_key: Option<String>) -> Self {
        EconomicCalendarProvider {
            api_key,
            client: reqwest::Client::new(),
        }
    }

    /// Fetch economic events from Trading Economics API
    pub async fn fetch_events(
        &self,
        from_date: DateTime<Utc>,
        to_date: DateTime<Utc>,
    ) -> Result<Vec<EconomicEventData>> {
        if let Some(api_key) = &self.api_key {
            self.fetch_trading_economics(from_date, to_date, api_key).await
        } else {
            // Fallback to free API or mock data
            self.fetch_free_api(from_date, to_date).await
        }
    }

    async fn fetch_trading_economics(
        &self,
        from_date: DateTime<Utc>,
        to_date: DateTime<Utc>,
        api_key: &str,
    ) -> Result<Vec<EconomicEventData>> {
        let url = format!(
            "https://api.tradingeconomics.com/calendar?c={}&d1={}&d2={}",
            api_key,
            from_date.format("%Y-%m-%d"),
            to_date.format("%Y-%m-%d")
        );

        let response = self.client
            .get(&url)
            .send()
            .await
            .context("Failed to send Trading Economics request")?;

        if !response.status().is_success() {
            anyhow::bail!("Trading Economics API error: {}", response.status());
        }

        let events: Vec<TradingEconomicsEvent> = response.json().await
            .context("Failed to parse Trading Economics response")?;

        let mut result = Vec::new();
        for event in events {
            let scheduled_at = chrono::NaiveDate::parse_from_str(&event.date, "%Y-%m-%d")
                .or_else(|_| chrono::NaiveDate::parse_from_str(&event.date, "%Y-%m-%d %H:%M:%S"))
                .map(|d| d.and_hms_opt(0, 0, 0)
                    .unwrap_or_else(|| d.and_hms_opt(12, 0, 0)
                        .unwrap_or_else(|| d.and_hms_opt(0, 0, 0)
                            .expect("Failed to create datetime from date"))))
                .map(|dt| DateTime::<Utc>::from_naive_utc_and_offset(dt, Utc))
                .context("Failed to parse date")?
                .timestamp();

            let impact = match event.importance.as_deref() {
                Some("High") | Some("high") => "high",
                Some("Medium") | Some("medium") => "medium",
                _ => "low",
            };

            result.push(EconomicEventData {
                event_type: event.category,
                country: event.country,
                title: event.event,
                description: event.reference,
                scheduled_at,
                forecast_value: event.forecast,
                previous_value: event.previous,
                impact: impact.to_string(),
                currency: None, // Trading Economics doesn't always provide this
            });
        }

        Ok(result)
    }

    async fn fetch_free_api(
        &self,
        from_date: DateTime<Utc>,
        to_date: DateTime<Utc>,
    ) -> Result<Vec<EconomicEventData>> {
        // Use ForexFactory or other free API
        // For now, return empty - can be enhanced later
        Ok(Vec::new())
    }

    /// Map impact string to numeric score
    pub fn impact_to_score(impact: &str) -> f64 {
        match impact.to_lowercase().as_str() {
            "high" => 0.8,
            "medium" => 0.5,
            "low" => 0.3,
            _ => 0.3,
        }
    }
}

