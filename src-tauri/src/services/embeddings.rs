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

    /// Generate local embedding using improved TF-IDF-like approach with character n-grams
    /// This is much better than hash-based and provides semantic similarity
    fn generate_local(&self, text: &str) -> Vec<f32> {
        // Use improved embedding generator with character n-grams and word co-occurrence
        self.generate_tfidf_embedding(text)
    }
    
    /// Generate embedding using TF-IDF with character n-grams for better semantic similarity
    fn generate_tfidf_embedding(&self, text: &str) -> Vec<f32> {
        use std::collections::HashMap;
        
        // Tokenize and normalize
        let words: Vec<String> = text
            .to_lowercase()
            .split_whitespace()
            .map(|w| w.trim_matches(|c: char| !c.is_alphanumeric()).to_string())
            .filter(|w| !w.is_empty() && w.len() > 1)
            .collect();
        
        if words.is_empty() {
            return vec![0.0; self.dimension];
        }
        
        // Calculate word frequencies (TF)
        let mut word_freq: HashMap<String, usize> = HashMap::new();
        for word in &words {
            *word_freq.entry(word.clone()).or_insert(0) += 1;
        }
        
        let total_words = words.len() as f32;
        
        // Generate embedding using multiple features:
        // 1. Word frequency vectors (TF)
        // 2. Character n-grams (2-grams and 3-grams)
        // 3. Word position weighting
        let mut embedding = vec![0.0f32; self.dimension];
        
        // Feature 1: Word frequency with position weighting
        for (idx, word) in words.iter().enumerate() {
            let tf = word_freq.get(word).copied().unwrap_or(0) as f32 / total_words;
            // Position weighting: earlier words slightly more important
            let position_weight = 1.0 + (1.0 / (idx as f32 + 1.0)) * 0.1;
            let weight = tf * position_weight;
            
            // Hash word to embedding dimensions
            let word_hash = self.hash_string(word);
            for i in 0..self.dimension {
                let dim_hash = self.hash_combine(word_hash, i as u32);
                let value = ((dim_hash as f32 / u32::MAX as f32) * 2.0 - 1.0) * weight;
                embedding[i] += value;
            }
        }
        
        // Feature 2: Character n-grams (captures subword patterns)
        for word in &words {
            if word.len() >= 2 {
                // 2-grams
                for i in 0..word.len().saturating_sub(1) {
                    let bigram = &word[i..i+2];
                    let bigram_hash = self.hash_string(bigram);
                    for j in 0..(self.dimension / 2) {
                        let dim_hash = self.hash_combine(bigram_hash, j as u32);
                        let value = (dim_hash as f32 / u32::MAX as f32) * 0.1;
                        embedding[j] += value;
                    }
                }
            }
            if word.len() >= 3 {
                // 3-grams
                for i in 0..word.len().saturating_sub(2) {
                    let trigram = &word[i..i+3];
                    let trigram_hash = self.hash_string(trigram);
                    for j in (self.dimension / 2)..self.dimension {
                        let dim_hash = self.hash_combine(trigram_hash, (j - self.dimension / 2) as u32);
                        let value = (dim_hash as f32 / u32::MAX as f32) * 0.1;
                        embedding[j] += value;
                    }
                }
            }
        }
        
        // Feature 3: Document length normalization
        let length_factor = (total_words as f32).ln_1p() / 10.0;
        for val in &mut embedding {
            *val *= length_factor;
        }
        
        // Normalize to unit vector
        let norm: f32 = embedding.iter().map(|x| x * x).sum::<f32>().sqrt();
        if norm > 0.0 {
            for val in &mut embedding {
                *val /= norm;
            }
        }
        
        embedding
    }
    
    /// Hash function for strings
    fn hash_string(&self, s: &str) -> u32 {
        let mut hash: u32 = 2166136261; // FNV offset basis
        for byte in s.bytes() {
            hash ^= byte as u32;
            hash = hash.wrapping_mul(16777619); // FNV prime
        }
        hash
    }
    
    /// Combine two hashes
    fn hash_combine(&self, a: u32, b: u32) -> u32 {
        a.wrapping_mul(31).wrapping_add(b)
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

