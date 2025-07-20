use actix_web::{web, App, HttpServer, middleware as actix_middleware};
use actix_cors::Cors;
use dotenv::dotenv;
use ethers::prelude::*;
use std::sync::Arc;
use tracing::{info, error};
use tracing_subscriber::EnvFilter;

mod api;
mod config;
mod contracts;
mod db;
mod middleware;
mod services;

use config::Settings;
use middleware::rate_limiter::RedisRateLimiter;
use services::{
    cache::CacheService, 
    indexer::Indexer,
    event_indexer::EventIndexer,
    voucher::VoucherService,
};

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    // Load environment variables
    dotenv().ok();

    // Initialize logging
    tracing_subscriber::fmt()
        .with_env_filter(EnvFilter::from_default_env())
        .init();

    // Load configuration
    let settings = Settings::new()
        .expect("Failed to load configuration");

    info!("Starting NBGN backend server");

    // Initialize database pool
    let pool = db::create_pool(&settings.database.url)
        .await
        .expect("Failed to create database pool");

    // Run migrations
    db::run_migrations(&pool)
        .await
        .expect("Failed to run database migrations");

    // Initialize Redis cache
    let cache = CacheService::new(&settings.redis.url)
        .expect("Failed to initialize Redis connection");
    
    // Initialize rate limiter
    let rate_limiter = RedisRateLimiter::new(&settings.redis.url)
        .expect("Failed to initialize Redis rate limiter");

    // Initialize Ethereum provider and contract
    let provider = Provider::<Http>::try_from(&settings.ethereum.rpc_url)
        .expect("Failed to create Ethereum provider");
    let provider = Arc::new(provider);

    let contract_address = settings.ethereum.nbgn_contract_address
        .parse::<Address>()
        .expect("Invalid contract address");

    let contract = contracts::nbgn::get_contract(contract_address, provider.clone())
        .expect("Failed to initialize contract");

    // Initialize voucher contract address
    let voucher_contract_address = settings.ethereum.voucher_contract_address
        .parse::<Address>()
        .expect("Invalid voucher contract address");

    // Initialize voucher service with provider
    let voucher_service = VoucherService::new(pool.clone(), &settings.backend.private_key)
        .expect("Failed to initialize voucher service")
        .with_provider(provider.clone());

    // Start the indexer in the background
    let indexer = Indexer::new(contract.clone(), pool.clone(), provider.clone());
    let _indexer_handle = {
        let indexer = indexer.clone();
        let poll_interval = settings.indexer.poll_interval_secs;
        tokio::spawn(async move {
            if let Err(e) = indexer.run_indexer_loop(poll_interval).await {
                error!("Indexer error: {}", e);
            }
        })
    };

    // Start the voucher event indexer
    let event_indexer = EventIndexer::new(pool.clone(), provider.clone(), voucher_contract_address);
    let _event_indexer_handle = {
        let event_indexer = event_indexer.clone();
        let poll_interval = settings.indexer.poll_interval_secs;
        tokio::spawn(async move {
            if let Err(e) = event_indexer.run_indexer_loop(poll_interval).await {
                error!("Voucher event indexer error: {}", e);
            }
        })
    };

    // Start HTTP server
    let server_bind = format!("{}:{}", settings.server.host, settings.server.port);
    info!("Starting HTTP server on {}", server_bind);

    HttpServer::new(move || {
        App::new()
            .app_data(web::Data::new(pool.clone()))
            .app_data(web::Data::new(cache.clone()))
            .app_data(web::Data::new(contract.clone()))
            .app_data(web::Data::new(rate_limiter.clone()))
            .app_data(web::Data::new(voucher_service.clone()))
            .app_data(web::Data::new(provider.clone()))
            .wrap(
                Cors::default()
                    .allowed_origin("http://localhost:3000")
                    .allowed_origin("http://localhost:3001")
                    .allowed_origin("http://localhost:5173")
                    .allowed_origin("http://localhost:5174")
                    .allowed_methods(vec!["GET", "POST", "PUT", "DELETE", "OPTIONS"])
                    .allowed_headers(vec!["Content-Type", "Authorization", "X-API-Key"])
                    .supports_credentials()
                    .max_age(3600)
            )
            .wrap(actix_middleware::Logger::default())
            .configure(api::routes::configure_routes)
    })
    .bind(&server_bind)?
    .run()
    .await
}
