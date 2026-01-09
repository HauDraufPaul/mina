use crate::storage::portfolio::PortfolioStore;
use crate::storage::market_data::MarketDataStore;
use crate::storage::portfolio_performance::PortfolioPerformanceStore;
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
pub struct PortfolioPerformanceMetrics {
    pub portfolio_id: i64,
    pub current_value: f64,
    pub cost_basis: f64,
    pub total_return: f64,
    pub total_return_percent: f64,
    pub daily_return: Option<f64>,
    pub weekly_return: Option<f64>,
    pub monthly_return: Option<f64>,
    pub quarterly_return: Option<f64>,
    pub yearly_return: Option<f64>,
    pub volatility: Option<f64>, // 30-day volatility
    pub sharpe_ratio: Option<f64>, // Assuming risk-free rate of 0
    pub max_drawdown: Option<f64>,
    pub beta: Option<f64>, // Beta vs S&P 500
    pub alpha: Option<f64>, // Alpha vs S&P 500
    pub win_rate: Option<f64>, // Percentage of holdings with positive returns
    pub largest_gain: Option<f64>,
    pub largest_loss: Option<f64>,
    pub sector_allocation: std::collections::HashMap<String, f64>, // sector -> percentage
    pub top_holdings: Vec<TopHolding>,
    pub performance_history: Vec<PerformanceSnapshot>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TopHolding {
    pub ticker: String,
    pub allocation_percent: f64,
    pub value: f64,
    pub gain_percent: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PerformanceSnapshot {
    pub timestamp: i64,
    pub value: f64,
    pub return_percent: f64,
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

    pub fn calculate_performance_metrics(
        portfolio_store: &PortfolioStore,
        market_data_store: &MarketDataStore,
        performance_store: &PortfolioPerformanceStore,
        portfolio_id: i64,
    ) -> Result<PortfolioPerformanceMetrics> {
        let portfolio_value = Self::calculate_portfolio_value(portfolio_store, market_data_store, portfolio_id)?;
        let now = chrono::Utc::now().timestamp();

        // Calculate time-based returns
        let daily_return = Self::calculate_period_return(performance_store, portfolio_id, now - 86400, now)?;
        let weekly_return = Self::calculate_period_return(performance_store, portfolio_id, now - 7 * 86400, now)?;
        let monthly_return = Self::calculate_period_return(performance_store, portfolio_id, now - 30 * 86400, now)?;
        let quarterly_return = Self::calculate_period_return(performance_store, portfolio_id, now - 90 * 86400, now)?;
        let yearly_return = Self::calculate_period_return(performance_store, portfolio_id, now - 365 * 86400, now)?;

        // Calculate volatility (30-day standard deviation of daily returns)
        let volatility = Self::calculate_volatility(performance_store, portfolio_id, 30)?;

        // Calculate Sharpe ratio (assuming risk-free rate of 0)
        let sharpe_ratio = if let Some(vol) = volatility {
            if vol > 0.0 {
                // Use monthly return for Sharpe calculation
                monthly_return.map(|r| r / vol)
            } else {
                None
            }
        } else {
            None
        };

        // Calculate max drawdown
        let max_drawdown = Self::calculate_max_drawdown(performance_store, portfolio_id)?;

        // Calculate win rate
        let win_rate = if !portfolio_value.holdings.is_empty() {
            let winners = portfolio_value.holdings.iter().filter(|h| h.gain > 0.0).count();
            Some((winners as f64 / portfolio_value.holdings.len() as f64) * 100.0)
        } else {
            None
        };

        // Find largest gain and loss
        let largest_gain = portfolio_value.holdings.iter()
            .map(|h| h.gain)
            .max_by(|a, b| a.partial_cmp(b).unwrap_or(std::cmp::Ordering::Equal));
        let largest_loss = portfolio_value.holdings.iter()
            .map(|h| h.gain)
            .min_by(|a, b| a.partial_cmp(b).unwrap_or(std::cmp::Ordering::Equal));

        // Calculate top holdings by allocation
        let mut holdings_by_value: Vec<_> = portfolio_value.holdings.iter().collect();
        holdings_by_value.sort_by(|a, b| b.current_value.partial_cmp(&a.current_value).unwrap_or(std::cmp::Ordering::Equal));
        
        let top_holdings: Vec<TopHolding> = holdings_by_value.iter().take(10).map(|h| {
            TopHolding {
                ticker: h.ticker.clone(),
                allocation_percent: if portfolio_value.total_value > 0.0 {
                    (h.current_value / portfolio_value.total_value) * 100.0
                } else {
                    0.0
                },
                value: h.current_value,
                gain_percent: h.gain_percent,
            }
        }).collect();

        // Get performance history (last 30 days)
        let performance_history = performance_store.get_snapshots(portfolio_id, Some(now - 30 * 86400), Some(now), Some(30))?
            .into_iter()
            .map(|s| PerformanceSnapshot {
                timestamp: s.timestamp,
                value: s.total_value,
                return_percent: s.return_percent,
            })
            .collect();

        // Sector allocation (placeholder - would need sector data)
        let sector_allocation = std::collections::HashMap::new();

        // Beta and Alpha (placeholder - would need benchmark data)
        let beta = None;
        let alpha = None;

        Ok(PortfolioPerformanceMetrics {
            portfolio_id,
            current_value: portfolio_value.total_value,
            cost_basis: portfolio_value.total_cost,
            total_return: portfolio_value.total_gain,
            total_return_percent: portfolio_value.total_gain_percent,
            daily_return,
            weekly_return,
            monthly_return,
            quarterly_return,
            yearly_return,
            volatility,
            sharpe_ratio,
            max_drawdown,
            beta,
            alpha,
            win_rate,
            largest_gain,
            largest_loss,
            sector_allocation,
            top_holdings,
            performance_history,
        })
    }

    fn calculate_period_return(
        performance_store: &PortfolioPerformanceStore,
        portfolio_id: i64,
        from_ts: i64,
        to_ts: i64,
    ) -> Result<Option<f64>> {
        let snapshots = performance_store.get_snapshots(portfolio_id, Some(from_ts), Some(to_ts), Some(2))?;
        
        if snapshots.len() < 2 {
            return Ok(None);
        }

        let start_value = snapshots.last()
            .ok_or_else(|| anyhow::anyhow!("No last snapshot available"))?
            .total_value;
        let end_value = snapshots.first()
            .ok_or_else(|| anyhow::anyhow!("No first snapshot available"))?
            .total_value;

        if start_value > 0.0 {
            Ok(Some(((end_value - start_value) / start_value) * 100.0))
        } else {
            Ok(None)
        }
    }

    fn calculate_volatility(
        performance_store: &PortfolioPerformanceStore,
        portfolio_id: i64,
        days: i64,
    ) -> Result<Option<f64>> {
        let now = chrono::Utc::now().timestamp();
        let snapshots = performance_store.get_snapshots(portfolio_id, Some(now - days * 86400), Some(now), Some(days as i64))?;
        
        if snapshots.len() < 2 {
            return Ok(None);
        }

        // Calculate daily returns
        let mut returns = Vec::new();
        for i in 1..snapshots.len() {
            let prev_value = snapshots[i].total_value;
            let curr_value = snapshots[i - 1].total_value;
            if prev_value > 0.0 {
                returns.push(((curr_value - prev_value) / prev_value) * 100.0);
            }
        }

        if returns.is_empty() {
            return Ok(None);
        }

        // Calculate standard deviation
        let mean = returns.iter().sum::<f64>() / returns.len() as f64;
        let variance = returns.iter()
            .map(|r| (r - mean).powi(2))
            .sum::<f64>() / returns.len() as f64;
        let std_dev = variance.sqrt();

        Ok(Some(std_dev))
    }

    fn calculate_max_drawdown(
        performance_store: &PortfolioPerformanceStore,
        portfolio_id: i64,
    ) -> Result<Option<f64>> {
        let now = chrono::Utc::now().timestamp();
        let snapshots = performance_store.get_snapshots(portfolio_id, Some(now - 365 * 86400), Some(now), Some(365))?;
        
        if snapshots.len() < 2 {
            return Ok(None);
        }

        let mut max_value = snapshots[0].total_value;
        let mut max_drawdown = 0.0;

        for snapshot in &snapshots {
            if snapshot.total_value > max_value {
                max_value = snapshot.total_value;
            }
            let drawdown = if max_value > 0.0 {
                ((max_value - snapshot.total_value) / max_value) * 100.0
            } else {
                0.0
            };
            if drawdown > max_drawdown {
                max_drawdown = drawdown;
            }
        }

        Ok(Some(max_drawdown))
    }
}
