use anyhow::Result;

/// Simple sentiment analyzer using keyword-based approach
/// Returns score from -1.0 (very negative) to 1.0 (very positive)
pub struct SentimentAnalyzer;

impl SentimentAnalyzer {
    pub fn analyze(text: &str) -> f64 {
        let text_lower = text.to_lowercase();
        
        // Positive keywords with weights
        let positive_keywords: Vec<(&str, f64)> = vec![
            ("surge", 0.8),
            ("rally", 0.9),
            ("gain", 0.7),
            ("profit", 0.8),
            ("growth", 0.7),
            ("beat", 0.8),
            ("exceed", 0.7),
            ("strong", 0.6),
            ("positive", 0.7),
            ("upgrade", 0.8),
            ("bullish", 0.9),
            ("outperform", 0.8),
            ("breakthrough", 0.9),
            ("success", 0.7),
            ("win", 0.7),
        ];

        // Negative keywords with weights
        let negative_keywords: Vec<(&str, f64)> = vec![
            ("crash", -0.9),
            ("plunge", -0.8),
            ("drop", -0.7),
            ("loss", -0.8),
            ("decline", -0.7),
            ("miss", -0.8),
            ("fall", -0.7),
            ("weak", -0.6),
            ("negative", -0.7),
            ("downgrade", -0.8),
            ("bearish", -0.9),
            ("underperform", -0.8),
            ("failure", -0.8),
            ("concern", -0.6),
            ("risk", -0.5),
            ("warn", -0.7),
        ];

        let mut score = 0.0;
        let mut count = 0.0;

        // Check positive keywords
        for (keyword, weight) in &positive_keywords {
            if text_lower.contains(keyword) {
                score += weight;
                count += 1.0;
            }
        }

        // Check negative keywords
        for (keyword, weight) in &negative_keywords {
            if text_lower.contains(keyword) {
                score += weight;
                count += 1.0;
            }
        }

        // Normalize score
        if count > 0.0 {
            let max_count = if count > 1.0 { count } else { 1.0 };
            score / max_count
        } else {
            0.0 // Neutral if no keywords found
        }
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
