#[cfg(test)]
mod tests {
    use nbgn_backend::middleware::rate_limiter::{RedisRateLimiter, get_rate_limit_config};

    #[test]
    fn test_rate_limiter_creation() {
        // Test that we can create a rate limiter (even if Redis isn't running)
        let result = RedisRateLimiter::new("redis://localhost:6379");
        assert!(result.is_ok());
    }

    #[test]
    fn test_rate_limit_config() {
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

    #[test]
    fn test_redis_url_parsing() {
        // Test various Redis URL formats
        assert!(RedisRateLimiter::new("redis://localhost:6379").is_ok());
        assert!(RedisRateLimiter::new("redis://127.0.0.1:6379").is_ok());
        assert!(RedisRateLimiter::new("redis://redis.example.com:6379").is_ok());
    }
}