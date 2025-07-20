pub mod api;
pub mod config;
pub mod contracts;
pub mod db;
pub mod middleware;
pub mod services;

#[cfg(test)]
mod tests {
    use super::*;
    use crate::middleware::rate_limiter::{RedisRateLimiter, RateLimitResult};

    #[test]
    fn test_rate_limiter_creation() {
        // Test that we can create a rate limiter (even if Redis isn't running)
        let result = RedisRateLimiter::new("redis://localhost:6379");
        assert!(result.is_ok());
    }

    #[test]
    fn test_rate_limit_result() {
        let result = RateLimitResult {
            allowed: true,
            limit: 100,
            remaining: 99,
            reset_time: 1234567890,
            retry_after: None,
        };
        
        assert!(result.allowed);
        assert_eq!(result.limit, 100);
        assert_eq!(result.remaining, 99);
        assert_eq!(result.reset_time, 1234567890);
        assert!(result.retry_after.is_none());
    }

    #[test]
    fn test_rate_limit_config() {
        use crate::middleware::rate_limiter::get_rate_limit_config;
        
        // Test different endpoint configurations
        let (limit, window) = get_rate_limit_config("/api/users/username");
        assert_eq!(limit, 5);
        assert_eq!(window, 3600);
        
        let (limit, window) = get_rate_limit_config("/api/transactions");
        assert_eq!(limit, 100);
        assert_eq!(window, 60);
        
        let (limit, window) = get_rate_limit_config("/api/analytics");
        assert_eq!(limit, 50);
        assert_eq!(window, 60);
        
        let (limit, window) = get_rate_limit_config("/api/other");
        assert_eq!(limit, 200);
        assert_eq!(window, 60);
    }
}