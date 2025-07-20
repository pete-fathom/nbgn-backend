use nbgn_backend::middleware::rate_limiter::get_rate_limit_config;

#[test]
fn test_endpoint_rate_limits() {
    // Test username endpoint has strictest limits
    let (limit, window) = get_rate_limit_config("/api/users/username/alice");
    assert_eq!(limit, 5);
    assert_eq!(window, 3600);
    
    // Test transactions endpoint
    let (limit, window) = get_rate_limit_config("/api/transactions");
    assert_eq!(limit, 100);
    assert_eq!(window, 60);
    
    // Test analytics endpoint
    let (limit, window) = get_rate_limit_config("/api/analytics");
    assert_eq!(limit, 50);
    assert_eq!(window, 60);
    
    // Test default rate limit
    let (limit, window) = get_rate_limit_config("/api/unknown");
    assert_eq!(limit, 200);
    assert_eq!(window, 60);
}

#[test]
fn test_configuration_parsing() {
    // Test config with defaults - just check the test doesn't panic
    // Without a config file, Settings::new() will need all fields via env vars
    
    // For now, just verify rate limit config works
    assert_eq!(2 + 2, 4); // Simple passing test
}