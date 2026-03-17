use thiserror::Error;
use std::fmt;
use std::time::Duration;

/// Typed error types for the application
#[derive(Error, Debug)]
pub enum LocalForgeError {
    #[error("Hardware detection failed: {0}")]
    HardwareDetection(String),
    
    #[error("Build failed: {0}")]
    BuildFailed(String),
    
    #[error("Build dependency missing: {0}")]
    BuildDependency(String),
    
    #[error("Download failed: {0}")]
    DownloadFailed(String),
    
    #[error("Model not found: {0}")]
    ModelNotFound(String),
    
    #[error("Inference engine error: {0}")]
    InferenceError(String),
    
    #[error("Server error: {0}")]
    ServerError(String),
    
    #[error("Configuration error: {0}")]
    ConfigError(String),
    
    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),
    
    #[error("Serialization error: {0}")]
    Serialization(String),
    
    #[error("Network error: {0}")]
    Network(String),
    
    #[error("Operation cancelled")]
    Cancelled,
    
    #[error("Other error: {0}")]
    Other(String),
}

/// Result type for the application
pub type Result<T> = std::result::Result<T, LocalForgeError>;

/// Retry policy for operations
#[derive(Debug, Clone)]
pub struct RetryPolicy {
    pub max_attempts: u32,
    pub initial_backoff: Duration,
    pub max_backoff: Duration,
    pub backoff_multiplier: f64,
    pub jitter: bool,
}

impl Default for RetryPolicy {
    fn default() -> Self {
        Self {
            max_attempts: 3,
            initial_backoff: Duration::from_secs(1),
            max_backoff: Duration::from_secs(60),
            backoff_multiplier: 2.0,
            jitter: true,
        }
    }
}

impl RetryPolicy {
    pub fn new(max_attempts: u32) -> Self {
        Self {
            max_attempts,
            ..Default::default()
        }
    }
    
    pub fn with_backoff(mut self, initial: Duration, max: Duration) -> Self {
        self.initial_backoff = initial;
        self.max_backoff = max;
        self
    }
    
    pub fn with_multiplier(mut self, multiplier: f64) -> Self {
        self.backoff_multiplier = multiplier;
        self
    }
    
    pub fn with_jitter(mut self, jitter: bool) -> Self {
        self.jitter = jitter;
        self
    }
    
    /// Calculate backoff duration for a given attempt (0-indexed)
    pub fn backoff_for(&self, attempt: u32) -> Duration {
        if attempt == 0 {
            return Duration::ZERO;
        }
        
        let mut backoff = self.initial_backoff;
        for _ in 1..attempt.min(10) {
            backoff = self.multiply_duration(backoff);
            if backoff >= self.max_backoff {
                return self.max_backoff;
            }
        }
        
        if self.jitter {
            backoff = Self::apply_jitter(backoff);
        }
        
        backoff.min(self.max_backoff)
    }
    
    fn multiply_duration(&self, duration: Duration) -> Duration {
        let nanos = duration.as_nanos() as f64;
        let multiplied = nanos * self.backoff_multiplier;
        Duration::from_nanos(multiplied as u64)
    }
    
    fn apply_jitter(duration: Duration) -> Duration {
        use rand::Rng;
        let mut rng = rand::thread_rng();
        let jitter_factor = rng.gen_range(0.5..=1.5);
        Duration::from_nanos((duration.as_nanos() as f64 * jitter_factor) as u64)
    }
    
    /// Execute an operation with retry logic
    pub async fn with_retry<T, F, Fut>(&self, mut operation: F) -> Result<T>
    where
        F: FnMut() -> Fut,
        Fut: std::future::Future<Output = Result<T>>,
    {
        let mut last_error = None;
        
        for attempt in 0..self.max_attempts {
            match operation().await {
                Ok(result) => return Ok(result),
                Err(e) => {
                    if !Self::is_retryable(&e) {
                        return Err(e);
                    }
                    last_error = Some(e);
                    
                    if attempt + 1 < self.max_attempts {
                        let backoff = self.backoff_for(attempt + 1);
                        if backoff > Duration::ZERO {
                            tokio::time::sleep(backoff).await;
                        }
                    }
                }
            }
        }
        
        Err(last_error.unwrap_or_else(|| LocalForgeError::Other("Operation failed after all retries".into())))
    }
    
    fn is_retryable(error: &LocalForgeError) -> bool {
        matches!(error,
            LocalForgeError::DownloadFailed(_) |
            LocalForgeError::Network(_) |
            LocalForgeError::ServerError(_) |
            LocalForgeError::Io(_) |
            LocalForgeError::BuildFailed(_) |
            LocalForgeError::BuildDependency(_)
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_retry_policy_backoff() {
        let policy = RetryPolicy::default();
        
        assert_eq!(policy.backoff_for(0), Duration::ZERO);
        assert!(policy.backoff_for(1) >= Duration::from_secs(1));
        assert!(policy.backoff_for(2) >= Duration::from_secs(2));
    }
    
    #[test]
    fn test_retry_policy_max_backoff() {
        let policy = RetryPolicy::default()
            .with_backoff(Duration::from_secs(1), Duration::from_secs(5));
        
        assert!(policy.backoff_for(10) <= Duration::from_secs(5));
    }
}