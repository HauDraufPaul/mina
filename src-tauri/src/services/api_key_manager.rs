use crate::storage::api_keys::APIKeyStore;
use anyhow::{Context, Result};
use std::sync::Arc;

pub struct APIKeyManager {
    store: APIKeyStore,
}

impl APIKeyManager {
    pub fn new(store: APIKeyStore) -> Self {
        APIKeyManager { store }
    }

    /// Get API key for a provider, returning error if not found
    pub fn get_key(&self, provider: &str) -> Result<String> {
        self.store.get_key(provider)?
            .ok_or_else(|| anyhow::anyhow!("API key not found for provider: {}", provider))
    }

    /// Get API key for a provider, returning None if not found (no error)
    pub fn get_key_optional(&self, provider: &str) -> Result<Option<String>> {
        self.store.get_key(provider)
    }

    /// Check if a provider has a stored key
    pub fn has_key(&self, provider: &str) -> Result<bool> {
        self.store.has_key(provider)
    }

    /// Store API key for a provider
    pub fn store_key(&self, provider: &str, key: &str) -> Result<()> {
        self.store.store_key(provider, key)
            .context(format!("Failed to store API key for provider: {}", provider))
    }

    /// Delete API key for a provider
    pub fn delete_key(&self, provider: &str) -> Result<()> {
        self.store.delete_key(provider)
            .context(format!("Failed to delete API key for provider: {}", provider))
    }

    /// List all providers with stored keys
    pub fn list_providers(&self) -> Result<Vec<String>> {
        self.store.list_providers()
            .context("Failed to list API key providers")
    }
}

