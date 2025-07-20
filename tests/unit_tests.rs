use nbgn_backend::middleware::rate_limiter::get_rate_limit_config;
use nbgn_backend::db::models::UserProfile;
use chrono::Utc;

#[test]
fn test_rate_limit_configurations() {
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
fn test_model_serialization() {
    let profile = UserProfile {
        address: "0x123".to_string(),
        username: Some("testuser".to_string()),
        created_at: Utc::now(),
        total_minted: "1000".to_string(),
        total_redeemed: "500".to_string(),
        total_burned: "100".to_string(),
        transaction_count: 10,
    };
    
    let json = serde_json::to_string(&profile).unwrap();
    assert!(json.contains("\"address\":\"0x123\""));
    assert!(json.contains("\"username\":\"testuser\""));
    assert!(json.contains("\"transaction_count\":10"));
}