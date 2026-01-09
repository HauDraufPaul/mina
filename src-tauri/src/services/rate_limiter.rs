use std::collections::HashMap;
use std::sync::{Arc, Mutex};
use std::time::{Duration, Instant};

#[derive(Debug, Clone)]
pub struct RateLimit {
    pub max_requests: u32,
    pub window: Duration,
}

pub struct RateLimiter {
    limits: HashMap<String, RateLimit>,
    requests: Arc<Mutex<HashMap<String, Vec<Instant>>>>,
}

impl RateLimiter {
    pub fn new() -> Self {
        RateLimiter {
            limits: HashMap::new(),
            requests: Arc::new(Mutex::new(HashMap::new())),
        }
    }

    /// Register a rate limit for a provider
    pub fn register_limit(&mut self, provider: String, max_requests: u32, window_secs: u64) {
        self.limits.insert(
            provider,
            RateLimit {
                max_requests,
                window: Duration::from_secs(window_secs),
            },
        );
    }

    /// Check if a request can be made for a provider
    pub fn can_make_request(&self, provider: &str) -> bool {
        let limit = match self.limits.get(provider) {
            Some(l) => l,
            None => return true, // No limit registered, allow request
        };

        let mut requests = match self.requests.lock() {
            Ok(r) => r,
            Err(_) => return false, // Mutex poisoned, deny to be safe
        };

        let now = Instant::now();
        let window_start = now - limit.window;

        // Remove old requests outside the window
        if let Some(provider_requests) = requests.get_mut(provider) {
            provider_requests.retain(|&timestamp| timestamp > window_start);
            
            // Check if under limit
            provider_requests.len() < limit.max_requests as usize
        } else {
            // No previous requests, allow
            true
        }
    }

    /// Record a request for a provider
    pub fn record_request(&self, provider: &str) {
        let mut requests = match self.requests.lock() {
            Ok(r) => r,
            Err(_) => return, // Mutex poisoned, skip recording
        };

        let now = Instant::now();
        requests
            .entry(provider.to_string())
            .or_insert_with(Vec::new)
            .push(now);
    }

    /// Wait if needed until a request can be made
    pub async fn wait_if_needed(&self, provider: &str) {
        while !self.can_make_request(provider) {
            // Calculate time until next request can be made
            let limit = match self.limits.get(provider) {
                Some(l) => l,
                None => return, // No limit, no need to wait
            };

            let requests = match self.requests.lock() {
                Ok(r) => r,
                Err(_) => return,
            };

            if let Some(provider_requests) = requests.get(provider) {
                if let Some(oldest) = provider_requests.first() {
                    let elapsed = oldest.elapsed();
                    if elapsed < limit.window {
                        let wait_time = limit.window - elapsed;
                        tokio::time::sleep(wait_time).await;
                    }
                }
            } else {
                return;
            }
        }
    }

    /// Get remaining requests in current window
    pub fn get_remaining_requests(&self, provider: &str) -> Option<u32> {
        let limit = self.limits.get(provider)?;
        let requests = self.requests.lock().ok()?;
        
        let now = Instant::now();
        let window_start = now - limit.window;
        
        if let Some(provider_requests) = requests.get(provider) {
            let count = provider_requests
                .iter()
                .filter(|&&timestamp| timestamp > window_start)
                .count();
            Some(limit.max_requests.saturating_sub(count as u32))
        } else {
            Some(limit.max_requests)
        }
    }

    /// Clear old requests (cleanup)
    pub fn cleanup(&self) {
        let mut requests = match self.requests.lock() {
            Ok(r) => r,
            Err(_) => return,
        };

        let now = Instant::now();
        for (provider, provider_requests) in requests.iter_mut() {
            if let Some(limit) = self.limits.get(provider) {
                let window_start = now - limit.window;
                provider_requests.retain(|&timestamp| timestamp > window_start);
            }
        }
    }
}

impl Default for RateLimiter {
    fn default() -> Self {
        Self::new()
    }
}

impl Clone for RateLimiter {
    fn clone(&self) -> Self {
        RateLimiter {
            limits: self.limits.clone(),
            requests: self.requests.clone(),
        }
    }
}

