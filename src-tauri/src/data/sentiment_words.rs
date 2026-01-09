use std::collections::{HashMap, HashSet};

/// Positive sentiment words with weights
pub fn get_positive_words() -> HashMap<String, f64> {
    let mut words = HashMap::new();
    let positive_list = vec![
        ("surge", 0.8), ("rally", 0.9), ("gain", 0.7), ("profit", 0.8),
        ("growth", 0.7), ("beat", 0.8), ("exceed", 0.7), ("strong", 0.6),
        ("positive", 0.7), ("upgrade", 0.8), ("bullish", 0.9), ("outperform", 0.8),
        ("breakthrough", 0.9), ("success", 0.7), ("win", 0.7), ("soar", 0.8),
        ("climb", 0.7), ("rise", 0.6), ("increase", 0.6), ("improve", 0.6),
        ("excellent", 0.8), ("outstanding", 0.8), ("exceptional", 0.9),
        ("great", 0.7), ("good", 0.5), ("better", 0.6), ("best", 0.7),
        ("optimistic", 0.7), ("confident", 0.6), ("promising", 0.7),
        ("momentum", 0.7), ("strength", 0.6), ("resilient", 0.7),
        ("robust", 0.7), ("solid", 0.6), ("stable", 0.5), ("secure", 0.6),
        ("thrive", 0.8), ("flourish", 0.8), ("prosper", 0.8), ("boom", 0.8),
        ("surpass", 0.8), ("outpace", 0.8), ("accelerate", 0.7),
        ("recovery", 0.7), ("rebound", 0.7), ("revival", 0.7),
    ];
    
    for (word, weight) in positive_list {
        words.insert(word.to_string(), weight);
    }
    words
}

/// Negative sentiment words with weights
pub fn get_negative_words() -> HashMap<String, f64> {
    let mut words = HashMap::new();
    let negative_list = vec![
        ("crash", -0.9), ("plunge", -0.8), ("drop", -0.7), ("loss", -0.8),
        ("decline", -0.7), ("miss", -0.8), ("fall", -0.7), ("weak", -0.6),
        ("negative", -0.7), ("downgrade", -0.8), ("bearish", -0.9), ("underperform", -0.8),
        ("failure", -0.8), ("concern", -0.6), ("risk", -0.5), ("warn", -0.7),
        ("collapse", -0.9), ("slump", -0.8), ("sink", -0.7), ("tumble", -0.8),
        ("deteriorate", -0.8), ("worsen", -0.7), ("weaken", -0.7), ("struggle", -0.7),
        ("crisis", -0.9), ("recession", -0.8), ("depression", -0.9), ("turmoil", -0.8),
        ("volatile", -0.6), ("uncertain", -0.6), ("unstable", -0.7), ("risky", -0.6),
        ("poor", -0.6), ("bad", -0.5), ("worse", -0.6), ("worst", -0.7),
        ("terrible", -0.8), ("awful", -0.8), ("horrible", -0.8), ("disastrous", -0.9),
        ("pessimistic", -0.7), ("doubtful", -0.6), ("skeptical", -0.6),
        ("threat", -0.7), ("danger", -0.7), ("hazard", -0.6), ("peril", -0.7),
        ("bankrupt", -0.9), ("default", -0.9), ("insolvent", -0.9),
        ("layoff", -0.8), ("cut", -0.6), ("reduce", -0.6), ("decrease", -0.6),
    ];
    
    for (word, weight) in negative_list {
        words.insert(word.to_string(), weight);
    }
    words
}

/// Intensifiers that amplify sentiment
pub fn get_intensifiers() -> HashMap<String, f64> {
    let mut words = HashMap::new();
    let intensifier_list = vec![
        ("very", 1.25), ("extremely", 1.5), ("incredibly", 1.4), ("exceptionally", 1.4),
        ("absolutely", 1.3), ("completely", 1.2), ("totally", 1.2), ("utterly", 1.3),
        ("highly", 1.2), ("significantly", 1.3), ("substantially", 1.3), ("dramatically", 1.4),
        ("massively", 1.4), ("enormously", 1.4), ("tremendously", 1.3), ("immensely", 1.3),
        ("remarkably", 1.3), ("notably", 1.2), ("particularly", 1.2), ("especially", 1.2),
        ("really", 1.15), ("quite", 1.1), ("rather", 1.1), ("pretty", 1.1),
        ("somewhat", 0.9), ("slightly", 0.8), ("a bit", 0.8), ("a little", 0.8),
    ];
    
    for (word, weight) in intensifier_list {
        words.insert(word.to_string(), weight);
    }
    words
}

/// Negation words that flip sentiment
pub fn get_negations() -> HashSet<String> {
    let mut words = HashSet::new();
    let negation_list = vec![
        "not", "no", "never", "none", "nobody", "nothing", "nowhere", "neither",
        "cannot", "can't", "won't", "wouldn't", "shouldn't", "couldn't", "isn't",
        "aren't", "wasn't", "weren't", "hasn't", "haven't", "hadn't", "doesn't",
        "don't", "didn't", "without", "lack", "lacks", "lacking", "absence",
        "absent", "missing", "fail", "fails", "failed", "failure",
    ];
    
    for word in negation_list {
        words.insert(word.to_string());
    }
    words
}

/// Financial-specific context words that need special handling
pub fn get_financial_context() -> HashMap<String, f64> {
    let mut words = HashMap::new();
    // Words that might be negative in general but neutral/positive in finance
    let financial_list = vec![
        ("crash", -0.5), // Less negative in finance (market crash is expected sometimes)
        ("correction", -0.3), // Market correction is normal
        ("volatility", -0.2), // Can be positive for traders
    ];
    
    for (word, weight) in financial_list {
        words.insert(word.to_string(), weight);
    }
    words
}

