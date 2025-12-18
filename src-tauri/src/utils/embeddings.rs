use std::collections::HashMap;

/// Simple text-based embedding generator using TF-IDF-like approach
/// This is a basic implementation - for production, use proper embedding models
pub struct EmbeddingGenerator {
    dimension: usize,
}

impl EmbeddingGenerator {
    pub fn new(dimension: usize) -> Self {
        EmbeddingGenerator { dimension }
    }

    /// Generate a simple embedding vector from text
    /// Uses a hash-based approach to create a deterministic vector
    pub fn generate(&self, text: &str) -> Vec<f32> {
        let words: Vec<String> = text
            .to_lowercase()
            .split_whitespace()
            .map(|w| w.trim_matches(|c: char| !c.is_alphanumeric()).to_string())
            .filter(|w| !w.is_empty())
            .collect();

        let mut embedding = vec![0.0f32; self.dimension];
        
        // Simple hash-based word embedding
        for word in &words {
            let hash = self.hash_string(word);
            for i in 0..self.dimension {
                let value = ((hash.wrapping_mul(i as u32 + 1)) as f32 / u32::MAX as f32) * 2.0 - 1.0;
                embedding[i] += value;
            }
        }

        // Normalize the vector
        let norm: f32 = embedding.iter().map(|x| x * x).sum::<f32>().sqrt();
        if norm > 0.0 {
            for val in &mut embedding {
                *val /= norm;
            }
        }

        embedding
    }

    /// Simple string hash function
    fn hash_string(&self, s: &str) -> u32 {
        let mut hash: u32 = 0;
        for byte in s.bytes() {
            hash = hash.wrapping_mul(31).wrapping_add(byte as u32);
        }
        hash
    }

    /// Generate embedding with word frequency weighting
    pub fn generate_weighted(&self, text: &str) -> Vec<f32> {
        let words: Vec<String> = text
            .to_lowercase()
            .split_whitespace()
            .map(|w| w.trim_matches(|c: char| !c.is_alphanumeric()).to_string())
            .filter(|w| !w.is_empty())
            .collect();

        // Count word frequencies
        let mut word_counts: HashMap<String, usize> = HashMap::new();
        for word in &words {
            *word_counts.entry(word.clone()).or_insert(0) += 1;
        }

        let total_words = words.len() as f32;
        let mut embedding = vec![0.0f32; self.dimension];

        // Weight by frequency
        for (word, count) in &word_counts {
            let weight = (*count as f32 / total_words).sqrt(); // TF-like weighting
            let hash = self.hash_string(word);
            for i in 0..self.dimension {
                let value = ((hash.wrapping_mul(i as u32 + 1)) as f32 / u32::MAX as f32) * 2.0 - 1.0;
                embedding[i] += value * weight;
            }
        }

        // Normalize
        let norm: f32 = embedding.iter().map(|x| x * x).sum::<f32>().sqrt();
        if norm > 0.0 {
            for val in &mut embedding {
                *val /= norm;
            }
        }

        embedding
    }
}

impl Default for EmbeddingGenerator {
    fn default() -> Self {
        Self::new(384) // Default dimension
    }
}

