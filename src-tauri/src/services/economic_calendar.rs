use crate::storage::economic_calendar::{EconomicCalendarStore, EconomicEvent, EventImpactHistory};
use anyhow::Result;

pub struct EconomicCalendarService;

impl EconomicCalendarService {
    /// Calculate impact score based on historical data
    /// Simple heuristic: higher impact for major events, GDP, interest rates
    pub fn calculate_impact_score(event_type: &str, country: &str) -> f64 {
        let base_score = match event_type.to_uppercase().as_str() {
            "GDP" | "INTEREST RATE" | "FOMC" => 0.8,
            "CPI" | "INFLATION" | "UNEMPLOYMENT" => 0.6,
            "RETAIL SALES" | "MANUFACTURING" => 0.4,
            _ => 0.3,
        };

        // Major economies have higher impact
        let country_multiplier = match country.to_uppercase().as_str() {
            "US" | "USA" | "UNITED STATES" => 1.0,
            "EU" | "EUROZONE" | "GERMANY" | "FRANCE" => 0.8,
            "UK" | "UNITED KINGDOM" => 0.7,
            "CHINA" | "JAPAN" => 0.9,
            _ => 0.5,
        };

        base_score * country_multiplier
    }

    /// Predict market reaction based on historical similar events
    pub fn predict_market_reaction(
        store: &EconomicCalendarStore,
        event_id: i64,
    ) -> Result<f64> {
        let event = store
            .get_event(event_id)?
            .ok_or_else(|| anyhow::anyhow!("Event not found"))?;

        // Get historical similar events
        let from_ts = event.scheduled_at - (365 * 24 * 3600); // 1 year back
        let to_ts = event.scheduled_at - 1;
        let similar_events = store.list_events(from_ts, to_ts, Some(&event.country), Some(&event.event_type))?;

        // Calculate average market reaction from similar events
        let mut total_reaction = 0.0;
        let mut count = 0;

        for similar_event in &similar_events {
            if let Some(_) = similar_event.actual_value {
                let history = store.get_impact_history(similar_event.id)?;
                for h in history {
                    total_reaction += h.market_reaction.abs();
                    count += 1;
                }
            }
        }

        if count > 0 {
            Ok(total_reaction / count as f64)
        } else {
            // Fallback to impact score as proxy
            Ok(event.impact_score * 2.0)
        }
    }

    /// Update impact score based on historical accuracy
    pub fn update_impact_score_from_history(
        store: &EconomicCalendarStore,
        event_type: &str,
        country: &str,
    ) -> Result<f64> {
        // Get recent events of this type
        let now = chrono::Utc::now().timestamp();
        let from_ts = now - (365 * 24 * 3600); // 1 year
        let events = store.list_events(from_ts, now, Some(country), Some(event_type))?;

        let mut total_impact = 0.0;
        let mut count = 0;

        for event in &events {
            if event.actual_value.is_some() {
                let history = store.get_impact_history(event.id)?;
                for h in history {
                    total_impact += h.market_reaction.abs();
                    count += 1;
                }
            }
        }

        if count > 0 {
            Ok(total_impact / count as f64)
        } else {
            Ok(Self::calculate_impact_score(event_type, country))
        }
    }
}
