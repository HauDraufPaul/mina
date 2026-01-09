pub mod ticker_matcher;
pub mod news_aggregator;
pub mod portfolio_analyzer;
pub mod economic_calendar;
pub mod alert_escalator;
pub mod sentiment_analyzer;
pub mod market_data_stream;
pub mod alert_escalation_checker;

pub use ticker_matcher::TickerMatcher;
pub use news_aggregator::NewsAggregator;
pub use portfolio_analyzer::{PortfolioAnalyzer, PortfolioValue, HoldingValue, ImpactAnalysis, HoldingImpact};
pub use economic_calendar::EconomicCalendarService;
pub use sentiment_analyzer::SentimentAnalyzer;

