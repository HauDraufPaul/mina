pub mod database;
pub mod migrations;
pub mod auth;
pub mod vector_store;

pub use database::{Database, ErrorRecord};
pub use migrations::MigrationManager;
pub use auth::{AuthManager, AuthAttempt};
pub use vector_store::{VectorStore, VectorDocument, CollectionStats};

