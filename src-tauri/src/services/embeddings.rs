use anyhow::{Result, Context};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::{Arc, Mutex};

#[derive(Debug, Clone, Serialize, Deserialize)]
struct OpenAIEmbeddingRequest {
    model: String,
    input: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct OpenAIEmbeddingResponse {
    data: Vec<OpenAIEmbeddingData>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct OpenAIEmbeddingData {
    embedding: Vec<f32>,
}

/// Embedding service that supports multiple providers
pub struct EmbeddingService {
    openai_client: Option<reqwest::Client>,
    openai_api_key: Option<String>,
    openai_base_url: String,
    cache: Arc<Mutex<HashMap<String, Vec<f32>>>>,
    dimension: usize,
}

impl EmbeddingService {
    pub fn new() -> Self {
        EmbeddingService {
            openai_client: Some(reqwest::Client::new()),
            openai_api_key: None,
            openai_base_url: "https://api.openai.com/v1".to_string(),
            cache: Arc::new(Mutex::new(HashMap::new())),
            dimension: 1536, // OpenAI text-embedding-3-small dimension
        }
    }

    /// Set OpenAI API key
    pub fn set_openai_key(&mut self, api_key: String) {
        self.openai_api_key = Some(api_key);
    }

    /// Generate embedding for text, trying OpenAI first, then falling back to local
    pub async fn generate(&self, text: &str) -> Result<Vec<f32>> {
        // Check cache first
        {
            let cache = self.cache.lock()
                .map_err(|e| anyhow::anyhow!("Cache lock error: {}", e))?;
            if let Some(cached) = cache.get(text) {
                return Ok(cached.clone());
            }
        }

        // Try OpenAI first if API key is available
        if let Some(ref api_key) = self.openai_api_key {
            match self.generate_openai(text, api_key).await {
                Ok(embedding) => {
                    // Cache the result
                    let mut cache = self.cache.lock()
                        .map_err(|e| anyhow::anyhow!("Cache lock error: {}", e))?;
                    cache.insert(text.to_string(), embedding.clone());
                    return Ok(embedding);
                }
                Err(e) => {
                    eprintln!("OpenAI embedding failed: {}, falling back to local", e);
                }
            }
        }

        // Fallback to local hash-based embedding
        let embedding = self.generate_local(text);
        
        // Cache the result
        let mut cache = self.cache.lock()
            .map_err(|e| anyhow::anyhow!("Cache lock error: {}", e))?;
        cache.insert(text.to_string(), embedding.clone());
        
        Ok(embedding)
    }

    /// Generate embedding using OpenAI API
    async fn generate_openai(&self, text: &str, api_key: &str) -> Result<Vec<f32>> {
        let client = self.openai_client.as_ref()
            .context("OpenAI client not initialized")?;

        let request = OpenAIEmbeddingRequest {
            model: "text-embedding-3-small".to_string(),
            input: vec![text.to_string()],
        };

        let response = client
            .post(&format!("{}/embeddings", self.openai_base_url))
            .header("Authorization", format!("Bearer {}", api_key))
            .header("Content-Type", "application/json")
            .json(&request)
            .send()
            .await
            .context("Failed to send OpenAI request")?;

        if !response.status().is_success() {
            let status = response.status();
            let error_text = response.text().await.unwrap_or_default();
            anyhow::bail!("OpenAI API error ({}): {}", status, error_text);
        }

        let embedding_response: OpenAIEmbeddingResponse = response
            .json()
            .await
            .context("Failed to parse OpenAI response")?;

        if let Some(data) = embedding_response.data.first() {
            Ok(data.embedding.clone())
        } else {
            anyhow::bail!("No embedding data in OpenAI response");
        }
    }

    /// Generate local hash-based embedding (fallback)
    fn generate_local(&self, text: &str) -> Vec<f32> {
        use crate::utils::embeddings::EmbeddingGenerator;
        let generator = EmbeddingGenerator::new(self.dimension);
        generator.generate_weighted(text)
    }

    /// Get the dimension of embeddings
    pub fn dimension(&self) -> usize {
        self.dimension
    }

    /// Clear the cache
    pub fn clear_cache(&self) -> Result<()> {
        let mut cache = self.cache.lock()
            .map_err(|e| anyhow::anyhow!("Cache lock error: {}", e))?;
        cache.clear();
        Ok(())
    }
}

impl Default for EmbeddingService {
    fn default() -> Self {
        Self::new()
    }
}

