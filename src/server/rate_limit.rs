use std::collections::HashMap;
use std::sync::Arc;
use std::time::{Duration, Instant};
use tokio::sync::RwLock;

/// Rate limiter using a sliding window per API key.
pub struct RateLimiter {
    /// Max requests per window
    max_requests: u32,
    /// Window duration
    window: Duration,
    /// Per-key request timestamps
    state: Arc<RwLock<HashMap<String, Vec<Instant>>>>,
}

impl RateLimiter {
    pub fn new(max_requests: u32, window: Duration) -> Self {
        Self {
            max_requests,
            window,
            state: Arc::new(RwLock::new(HashMap::new())),
        }
    }

    /// Default: 60 requests per minute
    pub fn default_limits() -> Self {
        Self::new(60, Duration::from_secs(60))
    }

    /// Check if a request is allowed for the given key.
    /// Returns `Ok(remaining)` if allowed, `Err(retry_after)` if rate limited.
    pub async fn check(&self, key: &str) -> Result<u32, Duration> {
        let mut state = self.state.write().await;
        let now = Instant::now();
        let window_start = now - self.window;

        let timestamps = state.entry(key.to_string()).or_default();

        // Remove expired entries
        timestamps.retain(|t| *t > window_start);

        if timestamps.len() as u32 >= self.max_requests {
            // Rate limited — calculate retry-after
            let oldest = timestamps.first().copied().unwrap_or(now);
            let retry_after = self.window.saturating_sub(now.duration_since(oldest));
            return Err(retry_after);
        }

        timestamps.push(now);
        let remaining = self.max_requests - timestamps.len() as u32;
        Ok(remaining)
    }

    /// Get current usage for a key (requests in window).
    pub async fn usage(&self, key: &str) -> u32 {
        let state = self.state.read().await;
        let now = Instant::now();
        let window_start = now - self.window;

        state
            .get(key)
            .map(|ts| ts.iter().filter(|t| **t > window_start).count() as u32)
            .unwrap_or(0)
    }

    /// Clean up expired entries for all keys.
    pub async fn cleanup(&self) {
        let mut state = self.state.write().await;
        let now = Instant::now();
        let window_start = now - self.window;

        state.retain(|_, timestamps| {
            timestamps.retain(|t| *t > window_start);
            !timestamps.is_empty()
        });
    }
}

impl Clone for RateLimiter {
    fn clone(&self) -> Self {
        Self {
            max_requests: self.max_requests,
            window: self.window,
            state: self.state.clone(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_allows_under_limit() {
        let limiter = RateLimiter::new(5, Duration::from_secs(60));
        for _ in 0..5 {
            assert!(limiter.check("test-key").await.is_ok());
        }
    }

    #[tokio::test]
    async fn test_blocks_over_limit() {
        let limiter = RateLimiter::new(2, Duration::from_secs(60));
        assert!(limiter.check("test-key").await.is_ok());
        assert!(limiter.check("test-key").await.is_ok());
        assert!(limiter.check("test-key").await.is_err());
    }

    #[tokio::test]
    async fn test_different_keys_independent() {
        let limiter = RateLimiter::new(1, Duration::from_secs(60));
        assert!(limiter.check("key-a").await.is_ok());
        assert!(limiter.check("key-b").await.is_ok());
        assert!(limiter.check("key-a").await.is_err());
    }

    #[tokio::test]
    async fn test_usage_tracking() {
        let limiter = RateLimiter::new(10, Duration::from_secs(60));
        assert_eq!(limiter.usage("key").await, 0);
        limiter.check("key").await.unwrap();
        limiter.check("key").await.unwrap();
        assert_eq!(limiter.usage("key").await, 2);
    }
}
