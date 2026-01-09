use crate::storage::portfolio::PortfolioStore;
use crate::storage::market_data::MarketDataStore;
use anyhow::Result;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PortfolioValue {
    pub total_value: f64,
    pub total_cost: f64,
    pub total_gain: f64,
    pub total_gain_percent: f64,
    pub holdings: Vec<HoldingValue>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HoldingValue {
    pub ticker: String,
    pub quantity: f64,
    pub cost_basis: f64,
    pub current_value: f64,
    pub gain: f64,
    pub gain_percent: f64,
    pub current_price: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ImpactAnalysis {
    pub portfolio_id: i64,
    pub event_id: i64,
    pub total_impact: f64,
    pub impact_percent: f64,
    pub affected_holdings: Vec<HoldingImpact>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HoldingImpact {
    pub ticker: String,
    pub quantity: f64,
    pub price_change: f64,
    pub price_change_percent: f64,
    pub impact: f64,
    pub impact_percent: f64,
}

pub struct PortfolioAnalyzer;

impl PortfolioAnalyzer {
    pub fn calculate_portfolio_value(
        portfolio_store: &PortfolioStore,
        market_data_store: &MarketDataStore,
        portfolio_id: i64,
    ) -> Result<PortfolioValue> {
        let holdings = portfolio_store.list_holdings(portfolio_id)?;
        let mut total_value = 0.0;
        let mut total_cost = 0.0;
        let mut holding_values = Vec::new();

        for holding in &holdings {
            let cost_basis = holding.quantity * holding.purchase_price;
            total_cost += cost_basis;

            // Get current price
            let current_price = if let Ok(Some(price)) = market_data_store.get_price(&holding.ticker) {
                price.price
            } else {
                holding.purchase_price // Fallback to purchase price if no current price
            };

            let current_value = holding.quantity * current_price;
            total_value += current_value;

            let gain = current_value - cost_basis;
            let gain_percent = if cost_basis > 0.0 {
                (gain / cost_basis) * 100.0
            } else {
                0.0
            };

            holding_values.push(HoldingValue {
                ticker: holding.ticker.clone(),
                quantity: holding.quantity,
                cost_basis,
                current_value,
                gain,
                gain_percent,
                current_price,
            });
        }

        let total_gain = total_value - total_cost;
        let total_gain_percent = if total_cost > 0.0 {
            (total_gain / total_cost) * 100.0
        } else {
            0.0
        };

        Ok(PortfolioValue {
            total_value,
            total_cost,
            total_gain,
            total_gain_percent,
            holdings: holding_values,
        })
    }

    pub fn analyze_event_impact(
        portfolio_store: &PortfolioStore,
        market_data_store: &MarketDataStore,
        portfolio_id: i64,
        event_id: i64,
        price_changes: &std::collections::HashMap<String, f64>, // ticker -> price change percent
    ) -> Result<ImpactAnalysis> {
        let holdings = portfolio_store.list_holdings(portfolio_id)?;
        let mut total_impact = 0.0;
        let mut affected_holdings = Vec::new();

        for holding in &holdings {
            if let Some(price_change_percent) = price_changes.get(&holding.ticker) {
                let current_price = if let Ok(Some(price)) = market_data_store.get_price(&holding.ticker) {
                    price.price
                } else {
                    holding.purchase_price
                };

                let price_change = current_price * (price_change_percent / 100.0);
                let new_value = holding.quantity * (current_price + price_change);
                let current_value = holding.quantity * current_price;
                let impact = new_value - current_value;
                let impact_percent = if current_value > 0.0 {
                    (impact / current_value) * 100.0
                } else {
                    0.0
                };

                total_impact += impact;

                affected_holdings.push(HoldingImpact {
                    ticker: holding.ticker.clone(),
                    quantity: holding.quantity,
                    price_change,
                    price_change_percent: *price_change_percent,
                    impact,
                    impact_percent,
                });
            }
        }

        let portfolio_value = Self::calculate_portfolio_value(portfolio_store, market_data_store, portfolio_id)?;
        let impact_percent = if portfolio_value.total_value > 0.0 {
            (total_impact / portfolio_value.total_value) * 100.0
        } else {
            0.0
        };

        Ok(ImpactAnalysis {
            portfolio_id,
            event_id,
            total_impact,
            impact_percent,
            affected_holdings,
        })
    }
}
