use actix_web::{web, App};
use ethers::prelude::*;
use nbgn_backend::{
    api::routes::configure_routes,
    contracts::nbgn::NBGNContract,
    middleware::rate_limiter::{RedisRateLimiter, RateLimiterMiddleware},
    services::cache::CacheService,
};
use sqlx::{postgres::PgPoolOptions, PgPool};
use std::sync::Arc;
use wiremock::{MockServer, Mock, ResponseTemplate};
use wiremock::matchers::{method, path};

pub struct TestApp {
    pub pool: PgPool,
    pub cache: CacheService,
    pub rate_limiter: RedisRateLimiter,
    pub mock_server: MockServer,
    pub contract: NBGNContract,
}

impl TestApp {
    pub async fn new() -> Self {
        // Setup test database
        let database_url = std::env::var("TEST_DATABASE_URL")
            .unwrap_or_else(|_| "postgres://postgres:password@localhost/nbgn_backend_test".to_string());
        
        let pool = PgPoolOptions::new()
            .max_connections(5)
            .connect(&database_url)
            .await
            .expect("Failed to connect to test database");
        
        // Run migrations
        sqlx::migrate!("./migrations")
            .run(&pool)
            .await
            .expect("Failed to run migrations");
        
        // Clean database before tests
        sqlx::query("TRUNCATE TABLE transactions, users, daily_stats, sync_status CASCADE")
            .execute(&pool)
            .await
            .expect("Failed to truncate tables");
        
        // Setup Redis
        let redis_url = std::env::var("TEST_REDIS_URL")
            .unwrap_or_else(|_| "redis://localhost:6379/1".to_string());
        
        let cache = CacheService::new(&redis_url)
            .expect("Failed to connect to Redis");
        
        let rate_limiter = RedisRateLimiter::new(&redis_url)
            .expect("Failed to create rate limiter");
        
        // Setup mock Ethereum provider
        let mock_server = MockServer::start().await;
        let provider = Provider::<Http>::try_from(mock_server.uri())
            .expect("Failed to create mock provider");
        let provider = Arc::new(provider);
        
        // Create mock contract
        let contract_address = "0x0000000000000000000000000000000000000001"
            .parse::<Address>()
            .unwrap();
        
        let contract = nbgn_backend::contracts::nbgn::get_contract(contract_address, provider)
            .expect("Failed to create contract");
        
        Self {
            pool,
            cache,
            rate_limiter,
            mock_server,
            contract,
        }
    }
    
    pub async fn cleanup(&self) {
        // Clean up test data
        sqlx::query("TRUNCATE TABLE transactions, users, daily_stats, sync_status CASCADE")
            .execute(&self.pool)
            .await
            .expect("Failed to cleanup test data");
    }
    
    pub fn create_app(&self) -> actix_web::App<
        impl actix_web::dev::ServiceFactory<
            actix_web::dev::ServiceRequest,
            Config = (),
            Response = actix_web::dev::ServiceResponse<impl actix_web::body::MessageBody>,
            Error = actix_web::Error,
            InitError = ()
        >
    > {
        App::new()
            .app_data(web::Data::new(self.pool.clone()))
            .app_data(web::Data::new(self.cache.clone()))
            .app_data(web::Data::new(self.contract.clone()))
            .app_data(web::Data::new(self.rate_limiter.clone()))
            .wrap(RateLimiterMiddleware::new(self.rate_limiter.clone()))
            .configure(configure_routes)
    }
}

pub async fn setup_mock_contract_calls(mock_server: &MockServer) {
    // Mock totalSupply call
    Mock::given(method("POST"))
        .and(path("/"))
        .respond_with(
            ResponseTemplate::new(200)
                .set_body_json(serde_json::json!({
                    "jsonrpc": "2.0",
                    "id": 1,
                    "result": "0x0000000000000000000000000000000000000000000000000de0b6b3a7640000" // 1e18
                }))
        )
        .mount(mock_server)
        .await;
}

pub async fn insert_test_user(pool: &PgPool, address: &str, username: Option<&str>) {
    sqlx::query(
        "INSERT INTO users (address, username) VALUES ($1, $2)"
    )
    .bind(address)
    .bind(username)
    .execute(pool)
    .await
    .expect("Failed to insert test user");
}

pub async fn insert_test_transaction(
    pool: &PgPool,
    tx_hash: &str,
    user_address: &str,
    transaction_type: &str,
    nbgn_amount: &str,
) {
    sqlx::query(
        r#"
        INSERT INTO transactions 
        (tx_hash, block_number, timestamp, user_address, transaction_type, nbgn_amount)
        VALUES ($1, $2, NOW(), $3, $4, $5)
        "#
    )
    .bind(tx_hash)
    .bind(1i64)
    .bind(user_address)
    .bind(transaction_type)
    .bind(nbgn_amount)
    .execute(pool)
    .await
    .expect("Failed to insert test transaction");
}