use anyhow::Result;
use crate::data::sentiment_words::{
    get_positive_words, get_negative_words, get_intensifiers, get_negations, get_financial_context
};
use std::collections::{HashMap, HashSet};

/// VADER-like sentiment analyzer
/// Returns score from -1.0 (very negative) to 1.0 (very positive)
pub struct SentimentAnalyzer {
    positive_words: HashMap<String, f64>,
    negative_words: HashMap<String, f64>,
    intensifiers: HashMap<String, f64>,
    negations: HashSet<String>,
    financial_context: HashMap<String, f64>,
}

impl SentimentAnalyzer {
    pub fn new() -> Self {
        SentimentAnalyzer {
            positive_words: get_positive_words(),
            negative_words: get_negative_words(),
            intensifiers: get_intensifiers(),
            negations: get_negations(),
            financial_context: get_financial_context(),
        }
    }

    /// Analyze sentiment using VADER-like algorithm
    pub fn analyze(&self, text: &str) -> f64 {
        let text_lower = text.to_lowercase();
        let words: Vec<&str> = text_lower
            .split_whitespace()
            .map(|w| w.trim_matches(|c: char| !c.is_alphanumeric() && c != '\''))
            .filter(|w| !w.is_empty())
            .collect();

        let mut scores = Vec::new();
        let mut i = 0;

        while i < words.len() {
            let word = words[i];
            let mut score = 0.0;
            let mut intensity = 1.0;
            let mut is_negated = false;

            // Check for intensifiers (look back 1-2 words)
            if i > 0 {
                let prev_word = words[i - 1];
                if let Some(intensity_mult) = self.intensifiers.get(prev_word) {
                    intensity = *intensity_mult;
                }
            }
            if i > 1 {
                let prev_word = words[i - 2];
                if let Some(intensity_mult) = self.intensifiers.get(prev_word) {
                    intensity = *intensity_mult;
                }
            }

            // Check for negations (look back 1-3 words)
            for j in (i.saturating_sub(3)..i).rev() {
                if self.negations.contains(words[j]) {
                    is_negated = true;
                    break;
                }
            }

            // Check if word is positive or negative
            if let Some(weight) = self.positive_words.get(word) {
                score = *weight;
            } else if let Some(weight) = self.negative_words.get(word) {
                score = *weight;
                // Apply financial context if applicable
                if let Some(financial_weight) = self.financial_context.get(word) {
                    // Use financial weight if it's less extreme
                    if financial_weight.abs() < weight.abs() {
                        score = *financial_weight;
                    }
                }
            }

            // Apply intensifier
            if score != 0.0 {
                score *= intensity;
            }

            // Apply negation (flip sign and reduce magnitude)
            if is_negated && score != 0.0 {
                score = -score * 0.5; // Flip and reduce
            }

            if score != 0.0 {
                scores.push(score);
            }

            i += 1;
        }

        // Calculate compound score
        if scores.is_empty() {
            return 0.0;
        }

        // Sum all scores
        let sum: f64 = scores.iter().sum();
        
        // Normalize to [-1, 1] range using hyperbolic tangent-like function
        let normalized = (sum / (sum.abs() + 15.0)).tanh();
        
        // Clamp to ensure we're in [-1, 1]
        normalized.max(-1.0).min(1.0)
    }

    /// Aggregate sentiment scores over time
    pub fn aggregate_sentiment(scores: &[f64]) -> f64 {
        if scores.is_empty() {
            return 0.0;
        }

        let sum: f64 = scores.iter().sum();
        sum / scores.len() as f64
    }

    /// Calculate sentiment trend (increasing/decreasing)
    pub fn sentiment_trend(scores: &[f64]) -> f64 {
        if scores.len() < 2 {
            return 0.0;
        }

        // Simple linear regression slope
        let n = scores.len() as f64;
        let sum_x: f64 = (0..scores.len()).map(|i| i as f64).sum();
        let sum_y: f64 = scores.iter().sum();
        let sum_xy: f64 = scores.iter().enumerate().map(|(i, &y)| i as f64 * y).sum();
        let sum_x2: f64 = (0..scores.len()).map(|i| (i as f64).powi(2)).sum();

        let slope = (n * sum_xy - sum_x * sum_y) / (n * sum_x2 - sum_x.powi(2));
        slope
    }
}

impl Default for SentimentAnalyzer {
    fn default() -> Self {
        Self::new()
    }
}

// Static method for backward compatibility
impl SentimentAnalyzer {
    /// Static method for backward compatibility
    pub fn analyze_static(text: &str) -> f64 {
        let analyzer = SentimentAnalyzer::new();
        analyzer.analyze(text)
    }
}
