use crate::storage::{StockNewsStore, StockTicker};
use anyhow::Result;
use std::collections::HashMap;
use std::sync::{Arc, Mutex};

pub struct TickerMatcher {
    tickers: Arc<Mutex<HashMap<String, StockTicker>>>,
    aliases: Arc<Mutex<HashMap<String, String>>>, // company name -> ticker
}

impl TickerMatcher {
    pub fn new(store: &StockNewsStore) -> Result<Self> {
        let tickers_list = store.list_tickers(None)?;
        
        let mut tickers_map = HashMap::new();
        let mut aliases_map = HashMap::new();

        for ticker in tickers_list {
            // Add ticker symbol
            tickers_map.insert(ticker.symbol.clone(), ticker.clone());
            
            // Add company name as alias
            let company_lower = ticker.name.to_lowercase();
            aliases_map.insert(company_lower.clone(), ticker.symbol.clone());
            
            // Add common variations
            // Remove "Inc.", "Corp.", "Ltd.", etc.
            let clean_name = company_lower
                .replace(" inc.", "")
                .replace(" inc", "")
                .replace(" corporation", "")
                .replace(" corp.", "")
                .replace(" corp", "")
                .replace(" ltd.", "")
                .replace(" ltd", "")
                .replace(" ag", "")
                .replace(" se", "")
                .replace(" plc", "")
                .replace(" n.v.", "")
                .replace(" co.", "")
                .trim()
                .to_string();
            
            if !clean_name.is_empty() && clean_name != company_lower {
                aliases_map.insert(clean_name, ticker.symbol.clone());
            }

            // Add short versions (first word of company name for major companies)
            let first_word = company_lower.split_whitespace().next().unwrap_or("");
            if first_word.len() >= 4 && !first_word.contains("the") {
                aliases_map.insert(first_word.to_string(), ticker.symbol.clone());
            }
        }

        Ok(TickerMatcher {
            tickers: Arc::new(Mutex::new(tickers_map)),
            aliases: Arc::new(Mutex::new(aliases_map)),
        })
    }

    /// Match tickers in text and return list of (ticker, confidence)
    pub fn match_tickers(&self, text: &str) -> Vec<(String, f64)> {
        let mut matches: HashMap<String, f64> = HashMap::new();
        let text_lower = text.to_lowercase();
        let tickers = match self.tickers.lock() {
            Ok(guard) => guard,
            Err(e) => {
                eprintln!("Failed to lock tickers: {}", e);
                return Vec::new();
            }
        };
        let aliases = match self.aliases.lock() {
            Ok(guard) => guard,
            Err(e) => {
                eprintln!("Failed to lock aliases: {}", e);
                return Vec::new();
            }
        };

        // 1. Explicit ticker mentions (e.g., "AAPL", "$MSFT", "(GOOGL)")
        for (symbol, _ticker) in tickers.iter() {
            let symbol_upper = symbol.to_uppercase();
            
            // Check for exact ticker matches with common patterns
            let patterns = vec![
                format!(" {} ", symbol_upper),     // " AAPL "
                format!("${}", symbol_upper),       // "$AAPL"
                format!("({})", symbol_upper),      // "(AAPL)"
                format!(" {}:", symbol_upper),      // " AAPL:"
                format!(" {},", symbol_upper),      // " AAPL,"
                format!(" {}.", symbol_upper),      // " AAPL."
            ];

            for pattern in patterns {
                if text.contains(&pattern) {
                    matches.insert(symbol.clone(), 0.95);
                    break;
                }
            }
        }

        // 2. Company name matching
        for (company_name, symbol) in aliases.iter() {
            // Check if company name appears in text
            if text_lower.contains(company_name) {
                // Calculate confidence based on name length and context
                let confidence = if company_name.len() >= 10 {
                    0.85 // High confidence for longer names
                } else if company_name.len() >= 6 {
                    0.75 // Medium confidence
                } else {
                    0.60 // Lower confidence for short names
                };

                // Increase confidence if it's already matched by ticker
                let final_confidence = if matches.contains_key(symbol) {
                    0.95
                } else {
                    confidence
                };

                matches.entry(symbol.clone())
                    .and_modify(|e| *e = f64::max(*e, final_confidence))
                    .or_insert(final_confidence);
            }
        }

        // 3. Context-aware matching (boost confidence for financial context)
        let financial_keywords = vec![
            "stock", "shares", "trading", "market", "earnings", "revenue",
            "profit", "loss", "quarterly", "ceo", "cfo", "announces", "reports",
        ];

        let has_financial_context = financial_keywords.iter()
            .any(|keyword| text_lower.contains(keyword));

        if has_financial_context {
            for (_ticker, confidence) in matches.iter_mut() {
                *confidence = f64::min(*confidence + 0.05, 1.0);
            }
        }

        // Convert to vector and sort by confidence
        let mut result: Vec<(String, f64)> = matches.into_iter().collect();
        result.sort_by(|a, b| b.1.partial_cmp(&a.1)
            .unwrap_or(std::cmp::Ordering::Equal));

        // Only return matches with confidence >= 0.6
        result.into_iter()
            .filter(|(_, conf)| *conf >= 0.6)
            .collect()
    }

    /// Get ticker info
    pub fn get_ticker(&self, symbol: &str) -> Option<StockTicker> {
        let tickers = match self.tickers.lock() {
            Ok(guard) => guard,
            Err(e) => {
                eprintln!("Failed to lock tickers: {}", e);
                return None;
            }
        };
        tickers.get(symbol).cloned()
    }

    /// Refresh ticker list from database
    pub fn refresh(&mut self, store: &StockNewsStore) -> Result<()> {
        let new_matcher = TickerMatcher::new(store)?;
        let new_tickers = new_matcher.tickers.lock()
            .map_err(|e| anyhow::anyhow!("Failed to lock new_matcher.tickers: {}", e))?;
        let new_aliases = new_matcher.aliases.lock()
            .map_err(|e| anyhow::anyhow!("Failed to lock new_matcher.aliases: {}", e))?;
        
        *self.tickers.lock()
            .map_err(|e| anyhow::anyhow!("Failed to lock self.tickers: {}", e))? = new_tickers.clone();
        *self.aliases.lock()
            .map_err(|e| anyhow::anyhow!("Failed to lock self.aliases: {}", e))? = new_aliases.clone();
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_ticker_matching() {
        // This is a placeholder test - in real implementation, we'd need a test database
        // For now, just verify the structure compiles
        let text = "Apple Inc. (AAPL) reports strong earnings";
        // Would need actual TickerMatcher instance with test data
        println!("Test text: {}", text);
    }
}

