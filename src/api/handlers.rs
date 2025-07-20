use actix_web::{web, HttpResponse, Result};
use sqlx::PgPool;
use serde_json::json;
use ethers::types::Signature;
use ethers::utils::hash_message;
use crate::db::models::{UserProfile, SetUsernameRequest, Transaction};
use crate::services::cache::CacheService;
use std::time::Duration;
use std::str::FromStr;

pub async fn get_user_profile(
    pool: web::Data<PgPool>,
    cache: web::Data<CacheService>,
    path: web::Path<String>,
) -> Result<HttpResponse> {
    let address = path.into_inner().to_lowercase();

    // Try cache first
    if let Ok(Some(profile)) = cache.get_user_profile(&address).await {
        return Ok(HttpResponse::Ok().json(profile));
    }

    // Query database using runtime query
    let profile_result = sqlx::query_as::<_, UserProfile>(
        r#"
        SELECT 
            u.address,
            u.username,
            u.created_at,
            COALESCE(SUM(CASE WHEN t.transaction_type = 'mint' THEN t.nbgn_amount::numeric ELSE 0 END), 0)::text as total_minted,
            COALESCE(SUM(CASE WHEN t.transaction_type = 'redeem' THEN t.nbgn_amount::numeric ELSE 0 END), 0)::text as total_redeemed,
            COALESCE(SUM(CASE WHEN t.transaction_type = 'burn' THEN t.nbgn_amount::numeric ELSE 0 END), 0)::text as total_burned,
            COUNT(t.id) as transaction_count
        FROM users u
        LEFT JOIN transactions t ON u.address = t.user_address
        WHERE LOWER(u.address) = $1
        GROUP BY u.address, u.username, u.created_at
        "#
    )
    .bind(&address)
    .fetch_optional(pool.get_ref())
    .await;

    match profile_result {
        Ok(Some(p)) => {
            // Cache for 5 minutes
            let _ = cache.set_user_profile(&p, Duration::from_secs(300)).await;
            Ok(HttpResponse::Ok().json(p))
        }
        Ok(None) => Ok(HttpResponse::NotFound().json(json!({ "error": "User not found" }))),
        Err(_) => Ok(HttpResponse::InternalServerError().json(json!({ "error": "Database error" }))),
    }
}

pub async fn set_username(
    pool: web::Data<PgPool>,
    cache: web::Data<CacheService>,
    req: web::Json<SetUsernameRequest>,
) -> Result<HttpResponse> {
    // Verify signature
    let message = hash_message(&req.message);
    let signature = Signature::from_str(&req.signature)
        .map_err(|_| actix_web::error::ErrorBadRequest("Invalid signature format"))?;
    
    let recovered_address = signature.recover(message)
        .map_err(|_| actix_web::error::ErrorBadRequest("Failed to recover address from signature"))?;

    if format!("0x{:x}", recovered_address).to_lowercase() != req.address.to_lowercase() {
        return Ok(HttpResponse::Unauthorized().json(json!({ "error": "Invalid signature" })));
    }

    // Check username availability
    let existing: Option<(String,)> = sqlx::query_as(
        "SELECT address FROM users WHERE LOWER(username) = LOWER($1)"
    )
    .bind(&req.username)
    .fetch_optional(pool.get_ref())
    .await
    .map_err(|_| actix_web::error::ErrorInternalServerError("Database error"))?;

    if existing.is_some() {
        return Ok(HttpResponse::BadRequest().json(json!({ "error": "Username already taken" })));
    }

    // Update username
    sqlx::query(
        r#"
        INSERT INTO users (address, username) 
        VALUES ($1, $2)
        ON CONFLICT (address) 
        DO UPDATE SET username = $2, updated_at = NOW()
        "#
    )
    .bind(&req.address.to_lowercase())
    .bind(&req.username)
    .execute(pool.get_ref())
    .await
    .map_err(|_| actix_web::error::ErrorInternalServerError("Database error"))?;

    // Clear cache
    let _ = cache.delete(&format!("user_profile:{}", req.address.to_lowercase())).await;

    Ok(HttpResponse::Ok().json(json!({ "success": true })))
}

pub async fn get_user_transactions(
    pool: web::Data<PgPool>,
    path: web::Path<String>,
    query: web::Query<PaginationQuery>,
) -> Result<HttpResponse> {
    let address = path.into_inner().to_lowercase();
    let limit = query.limit.unwrap_or(50).min(100);
    let offset = query.offset.unwrap_or(0);

    let transactions = sqlx::query_as::<_, Transaction>(
        r#"
        SELECT * FROM transactions
        WHERE LOWER(user_address) = $1
        ORDER BY timestamp DESC
        LIMIT $2 OFFSET $3
        "#
    )
    .bind(&address)
    .bind(limit as i64)
    .bind(offset as i64)
    .fetch_all(pool.get_ref())
    .await
    .map_err(|_| actix_web::error::ErrorInternalServerError("Database error"))?;

    let total: (i64,) = sqlx::query_as(
        "SELECT COUNT(*) FROM transactions WHERE LOWER(user_address) = $1"
    )
    .bind(&address)
    .fetch_one(pool.get_ref())
    .await
    .map_err(|_| actix_web::error::ErrorInternalServerError("Database error"))?;

    Ok(HttpResponse::Ok().json(json!({
        "transactions": transactions,
        "total": total.0,
        "limit": limit,
        "offset": offset
    })))
}

pub async fn get_recent_transactions(
    pool: web::Data<PgPool>,
    query: web::Query<PaginationQuery>,
) -> Result<HttpResponse> {
    let limit = query.limit.unwrap_or(50).min(100);
    let offset = query.offset.unwrap_or(0);

    let transactions = sqlx::query_as::<_, Transaction>(
        r#"
        SELECT * FROM transactions
        ORDER BY timestamp DESC
        LIMIT $1 OFFSET $2
        "#
    )
    .bind(limit as i64)
    .bind(offset as i64)
    .fetch_all(pool.get_ref())
    .await
    .map_err(|_| actix_web::error::ErrorInternalServerError("Database error"))?;

    Ok(HttpResponse::Ok().json(transactions))
}

#[derive(serde::Deserialize)]
pub struct PaginationQuery {
    pub limit: Option<i32>,
    pub offset: Option<i32>,
}

pub async fn get_reserve_ratio(
    cache: web::Data<CacheService>,
    contract: web::Data<crate::contracts::nbgn::NBGNContract>,
) -> Result<HttpResponse> {
    // Try cache first
    if let Ok(Some(ratio)) = cache.get_reserve_ratio().await {
        return Ok(HttpResponse::Ok().json(json!({ "reserve_ratio": ratio })));
    }

    // Query contract
    let total_supply_result = contract.method::<_, ethers::types::U256>("totalSupply", ())
        .map_err(|_| actix_web::error::ErrorInternalServerError("Failed to create contract call"))?
        .call().await
        .map_err(|_| actix_web::error::ErrorInternalServerError("Failed to query contract"))?;
    
    let reserves_result = contract.method::<_, ethers::types::U256>("reserves", ())
        .map_err(|_| actix_web::error::ErrorInternalServerError("Failed to create contract call"))?
        .call().await
        .map_err(|_| actix_web::error::ErrorInternalServerError("Failed to query contract"))?;

    let ratio = if !total_supply_result.is_zero() {
        format!("{:.4}", reserves_result.as_u128() as f64 / total_supply_result.as_u128() as f64)
    } else {
        "0".to_string()
    };

    // Cache for 1 minute
    let _ = cache.set_reserve_ratio(&ratio, Duration::from_secs(60)).await;

    Ok(HttpResponse::Ok().json(json!({ 
        "reserve_ratio": ratio,
        "total_supply": total_supply_result.to_string(),
        "reserves": reserves_result.to_string()
    })))
}

#[derive(serde::Serialize, sqlx::FromRow)]
struct AnalyticsResult {
    total_volume_24h: String,
    unique_users_24h: i64,
    total_transactions: i64,
    average_transaction_size: String,
}

pub async fn get_analytics(
    pool: web::Data<PgPool>,
) -> Result<HttpResponse> {
    let analytics = sqlx::query_as::<_, AnalyticsResult>(
        r#"
        SELECT 
            COALESCE(SUM(CASE 
                WHEN timestamp > NOW() - INTERVAL '24 hours' 
                THEN nbgn_amount::numeric 
                ELSE 0 
            END), 0)::text as total_volume_24h,
            COUNT(DISTINCT CASE 
                WHEN timestamp > NOW() - INTERVAL '24 hours' 
                THEN user_address 
            END) as unique_users_24h,
            COUNT(*) as total_transactions,
            COALESCE(AVG(nbgn_amount::numeric), 0)::text as average_transaction_size
        FROM transactions
        "#
    )
    .fetch_one(pool.get_ref())
    .await
    .map_err(|_| actix_web::error::ErrorInternalServerError("Database error"))?;

    Ok(HttpResponse::Ok().json(analytics))
}