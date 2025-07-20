use actix_web::{web, HttpResponse};
use crate::api::{handlers, voucher_routes};

pub fn configure_routes(cfg: &mut web::ServiceConfig) {
    cfg
        // API Documentation
        .route("/", web::get().to(|| async { 
            HttpResponse::Found()
                .append_header(("Location", "/docs"))
                .finish() 
        }))
        .route("/docs", web::get().to(|| async { 
            HttpResponse::Ok()
                .content_type("text/html")
                .body(include_str!("../../static/swagger.html"))
        }))
        .route("/openapi.yaml", web::get().to(|| async { 
            HttpResponse::Ok()
                .content_type("application/x-yaml")
                .body(include_str!("../../openapi.yaml"))
        }))
        
        // Health check
        .route("/health", web::get().to(|| async { 
            HttpResponse::Ok().json(serde_json::json!({
                "status": "healthy",
                "service": "nbgn-voucher-backend"
            }))
        }))
        
        // User endpoints
        .route("/api/users/{address}", web::get().to(handlers::get_user_profile))
        .route("/api/users/username", web::post().to(handlers::set_username))
        
        // Transaction endpoints
        .route("/api/transactions/{address}", web::get().to(handlers::get_user_transactions))
        .route("/api/transactions/recent", web::get().to(handlers::get_recent_transactions))
        
        // Analytics endpoints
        .route("/api/analytics/overview", web::get().to(handlers::get_analytics))
        
        // Contract data (cached)
        .route("/api/contract/reserve-ratio", web::get().to(handlers::get_reserve_ratio));
    
    // Configure voucher routes
    voucher_routes::configure_voucher_routes(cfg);
}