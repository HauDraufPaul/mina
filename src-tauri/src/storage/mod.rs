pub mod database;
pub mod migrations;
pub mod auth;
pub mod vector_store;
pub mod analytics;
pub mod rate_limit;
pub mod migration_tracking;
pub mod ai;
pub mod automation;
pub mod devops;
pub mod osint;
pub mod temporal;
pub mod testing;
pub mod projects;
pub mod seed_data;
pub mod stock_news;

pub use database::{Database, ErrorRecord};
pub use migrations::MigrationManager;
pub use auth::{AuthManager, AuthAttempt};
pub use vector_store::{VectorStore, VectorDocument, CollectionStats};
pub use analytics::{AnalyticsStore, AnalyticsMetrics, Statistics};
pub use rate_limit::{RateLimitStore, RateLimitBucket};
pub use migration_tracking::{MigrationTracker, MigrationRecord};
pub use ai::{AIStore, ChatMessage, Conversation, PromptTemplate};
pub use automation::{AutomationStore, Script, Workflow, WorkflowExecution};
pub use devops::{DevOpsStore, HealthCheck, Alert, PrometheusMetric};
pub use osint::{OSINTStore, RSSFeed, RSSItem, Entity, EntityRelationship};
pub use temporal::{
    TemporalStore,
    TemporalEvent,
    TemporalEventEvidence,
    Watchlist,
    WatchlistItem,
    AlertRule,
    Alert as TemporalAlert,
    BacktestReport,
    EntityGraph,
    EntityGraphNode,
    EntityGraphEdge,
    FeatureDefinition,
    FeatureValue,
    AlertLabel,
};
pub use testing::{TestingStore, TestSuite, TestResult, TestSuiteStats};
pub use projects::{ProjectStore, Project};
pub use stock_news::{StockNewsStore, StockTicker, StockNewsItem, StockNewsTicker};

