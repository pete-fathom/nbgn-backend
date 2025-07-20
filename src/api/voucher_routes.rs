use actix_web::{web, HttpResponse, Result, HttpRequest};
use sqlx::PgPool;
use serde_json::json;
use crate::db::voucher_models::*;
use crate::services::voucher::VoucherService;
use crate::middleware::rate_limiter::RedisRateLimiter;
use tracing::{info, warn};
use ethers::prelude::*;
use std::sync::Arc;

// POST /api/vouchers/link - Generate shareable link for on-chain voucher
pub async fn create_voucher_link(
    _pool: web::Data<PgPool>,
    service: web::Data<VoucherService>,
    req: web::Json<CreateLinkRequest>,
) -> Result<HttpResponse> {
    // TODO: In production, verify voucher exists on-chain by calling getVoucher(voucher_id)
    
    match service.create_voucher_link(
        &req.voucher_id, 
        req.password.as_deref(),
        req.creator_address.as_deref(),
        req.amount.as_deref()
    ).await {
        Ok(code) => {
            info!("Created voucher link with code {} for voucher_id {}", code, req.voucher_id);
            Ok(HttpResponse::Ok().json(json!({
                "success": true,
                "code": code,
                "shareable_code": code.clone(),
                "shareable_link": format!("/claim/{}", code),
                "link": format!("/claim/{}", code) // Keep for backward compatibility
            })))
        }
        Err(e) => {
            warn!("Failed to create voucher link: {}", e);
            Ok(HttpResponse::BadRequest().json(json!({
                "error": "Failed to create voucher link",
                "message": e.to_string()
            })))
        }
    }
}

// POST /api/vouchers/verify - Check if voucher is valid and claimable
pub async fn verify_voucher(
    pool: web::Data<PgPool>,
    service: web::Data<VoucherService>,
    limiter: web::Data<RedisRateLimiter>,
    req: web::Json<VerifyRequest>,
    http_req: HttpRequest,
) -> Result<HttpResponse> {
    // Extract IP for rate limiting
    let ip = extract_ip(&http_req);
    
    // Skip rate limiting for localhost in development
    if !ip.starts_with("127.0.0.1") && !ip.starts_with("::1") && !ip.starts_with("localhost") {
        let rate_limit_key = format!("verify:{}:{}", req.code, ip);
        
        // Rate limit: 100 attempts per code per IP per hour (increased for dev)
        match limiter.check_rate_limit(&rate_limit_key, 100, 3600).await {
            Ok(result) if !result.allowed => {
                return Ok(HttpResponse::TooManyRequests().json(json!({
                    "error": "Too many verification attempts",
                    "retry_after": result.retry_after
                })));
            }
            _ => {}
        }
    }

    // Log attempt
    sqlx::query(
        "INSERT INTO claim_attempts (voucher_code, ip_address, success) VALUES ($1, $2, false)"
    )
    .bind(&req.code)
    .bind(&ip)
    .execute(pool.get_ref())
    .await
    .ok();

    // Get voucher from DB
    let voucher = match service.get_voucher_by_code(&req.code).await {
        Ok(Some(v)) => v,
        Ok(None) => {
            return Ok(HttpResponse::NotFound().json(json!({
                "error": "Voucher not found"
            })));
        }
        Err(_e) => {
            return Ok(HttpResponse::InternalServerError().json(json!({
                "error": "Database error"
            })));
        }
    };

    // Check password if set
    if let Some(hash) = &voucher.password_hash {
        if req.password.is_none() {
            return Ok(HttpResponse::Unauthorized().json(json!({
                "error": "Password required"
            })));
        }
        
        if !VoucherService::verify_password(req.password.as_ref().unwrap(), hash) {
            return Ok(HttpResponse::Unauthorized().json(json!({
                "error": "Invalid password"
            })));
        }
    }

    // Check if already claimed
    if voucher.claimed {
        return Ok(HttpResponse::BadRequest().json(json!({
            "error": "Voucher already claimed",
            "claimed_by": voucher.claimed_by,
            "claimed_at": voucher.claimed_at
        })));
    }

    // Update successful attempt
    sqlx::query(
        "UPDATE claim_attempts SET success = true WHERE voucher_code = $1 AND ip_address = $2 AND attempted_at = (SELECT MAX(attempted_at) FROM claim_attempts WHERE voucher_code = $1 AND ip_address = $2)"
    )
    .bind(&req.code)
    .bind(&ip)
    .execute(pool.get_ref())
    .await
    .ok();

    // For now, just return the voucher info without signature generation
    Ok(HttpResponse::Ok().json(json!({
        "valid": true,
        "voucher": {
            "voucher_id": voucher.voucher_id,
            "amount": voucher.amount.unwrap_or_else(|| "0".to_string()),
            "creator_address": voucher.creator_address,
            "claimed": voucher.claimed,
            "cancelled": voucher.cancelled,
        },
        "hasPassword": voucher.password_hash.is_some()
    })))
}

// POST /api/vouchers/claim - Generate signature for claiming
pub async fn claim_voucher(
    pool: web::Data<PgPool>,
    service: web::Data<VoucherService>,
    limiter: web::Data<RedisRateLimiter>,
    req: web::Json<ClaimRequest>,
    http_req: HttpRequest,
) -> Result<HttpResponse> {
    // Validate recipient address
    if !is_valid_ethereum_address(&req.recipient_address) {
        return Ok(HttpResponse::BadRequest().json(json!({
            "error": "Invalid recipient address"
        })));
    }

    // Rate limit by IP
    let ip = extract_ip(&http_req);
    
    // Skip rate limiting for localhost in development
    if !ip.starts_with("127.0.0.1") && !ip.starts_with("::1") && !ip.starts_with("localhost") {
        let rate_limit_key = format!("claim:{}", ip);
        
        // Rate limit: 50 attempts per IP per hour (increased for dev)
        match limiter.check_rate_limit(&rate_limit_key, 50, 3600).await {
            Ok(result) if !result.allowed => {
                return Ok(HttpResponse::TooManyRequests().json(json!({
                    "error": "Too many claim attempts",
                    "retry_after": result.retry_after
                })));
            }
            _ => {}
        }
    }

    // Log claim attempt
    sqlx::query(
        "INSERT INTO claim_attempts (voucher_code, ip_address, recipient_address, success) VALUES ($1, $2, $3, false)"
    )
    .bind(&req.code)
    .bind(&ip)
    .bind(&req.recipient_address)
    .execute(pool.get_ref())
    .await
    .ok();

    // Generate claim authorization
    match service.create_claim_authorization(
        &req.code,
        &req.recipient_address,
        req.password.as_deref()
    ).await {
        Ok(authorization) => {
            info!("Generated claim authorization for voucher {} to recipient {}", 
                  req.code, req.recipient_address);
            Ok(HttpResponse::Ok().json(authorization))
        }
        Err(e) => {
            warn!("Failed to generate claim authorization: {}", e);
            Ok(HttpResponse::BadRequest().json(json!({
                "error": "Failed to generate claim authorization",
                "message": e.to_string()
            })))
        }
    }
}

// POST /api/vouchers/execute-claim - Execute gasless claim transaction
pub async fn execute_claim(
    _pool: web::Data<PgPool>,
    service: web::Data<VoucherService>,
    limiter: web::Data<RedisRateLimiter>,
    req: web::Json<ClaimRequest>,
    http_req: HttpRequest,
) -> Result<HttpResponse> {
    // Validate recipient address
    if !is_valid_ethereum_address(&req.recipient_address) {
        return Ok(HttpResponse::BadRequest().json(json!({
            "error": "Invalid recipient address"
        })));
    }

    // Rate limit by IP
    let ip = extract_ip(&http_req);
    
    // Skip rate limiting for localhost in development
    if !ip.starts_with("127.0.0.1") && !ip.starts_with("::1") && !ip.starts_with("localhost") {
        let rate_limit_key = format!("execute_claim:{}", ip);
        
        // Rate limit: 10 gasless claims per IP per hour
        match limiter.check_rate_limit(&rate_limit_key, 10, 3600).await {
            Ok(result) if !result.allowed => {
                return Ok(HttpResponse::TooManyRequests().json(json!({
                    "error": "Too many gasless claim attempts",
                    "retry_after": result.retry_after
                })));
            }
            _ => {}
        }
    }

    // Execute the gasless claim
    match service.execute_claim(
        &req.code,
        &req.recipient_address,
        req.password.as_deref()
    ).await {
        Ok(tx_hash) => {
            info!("Executed gasless claim transaction {} for voucher {} to recipient {}", 
                  tx_hash, req.code, req.recipient_address);
            Ok(HttpResponse::Ok().json(json!({
                "success": true,
                "tx_hash": tx_hash,
                "status": "pending",
                "message": "Gasless claim transaction submitted"
            })))
        }
        Err(e) => {
            warn!("Failed to execute gasless claim: {}", e);
            Ok(HttpResponse::BadRequest().json(json!({
                "error": "Failed to execute gasless claim",
                "message": e.to_string()
            })))
        }
    }
}

// POST /api/vouchers/claim-status - Update claim status after transaction
pub async fn update_claim_status(
    pool: web::Data<PgPool>,
    service: web::Data<VoucherService>,
    req: web::Json<ClaimStatusRequest>,
) -> Result<HttpResponse> {
    // Validate transaction hash
    if !is_valid_tx_hash(&req.tx_hash) {
        return Ok(HttpResponse::BadRequest().json(json!({
            "error": "Invalid transaction hash"
        })));
    }

    // Get voucher to find recipient
    let voucher = match service.get_voucher_by_code(&req.code).await {
        Ok(Some(v)) => v,
        _ => {
            return Ok(HttpResponse::NotFound().json(json!({
                "error": "Voucher not found"
            })));
        }
    };

    // Get recipient from last claim attempt
    let recipient: Option<(String,)> = sqlx::query_as(
        r#"
        SELECT recipient_address 
        FROM claim_attempts 
        WHERE voucher_code = $1 
        ORDER BY attempted_at DESC 
        LIMIT 1
        "#
    )
    .bind(&req.code)
    .fetch_optional(pool.get_ref())
    .await
    .unwrap_or(None);

    let claimed_by = recipient.map(|(addr,)| addr).unwrap_or_default();

    // Update claim status
    match service.update_claim_status(&req.code, &req.tx_hash, req.success, &claimed_by).await {
        Ok(_) => {
            Ok(HttpResponse::Ok().json(json!({
                "success": true,
                "message": if req.success { "Voucher claimed successfully" } else { "Claim failed" }
            })))
        }
        Err(e) => {
            Ok(HttpResponse::InternalServerError().json(json!({
                "error": "Failed to update claim status",
                "message": e.to_string()
            })))
        }
    }
}

// GET /api/vouchers/user/{address} - List vouchers for a user
pub async fn list_user_vouchers(
    _pool: web::Data<PgPool>,
    service: web::Data<VoucherService>,
    address: web::Path<String>,
    query: web::Query<ListQuery>,
) -> Result<HttpResponse> {
    let address = address.into_inner();
    
    // Validate address
    if !is_valid_ethereum_address(&address) {
        return Ok(HttpResponse::BadRequest().json(json!({
            "error": "Invalid Ethereum address"
        })));
    }

    let query_type = query.query_type.as_deref().unwrap_or("created");
    let page = query.page.unwrap_or(0).max(0);
    let limit = query.limit.unwrap_or(20).min(100);

    match service.list_user_vouchers(&address, query_type, page, limit).await {
        Ok(vouchers) => {
            Ok(HttpResponse::Ok().json(json!({
                "vouchers": vouchers,
                "page": page,
                "limit": limit,
                "type": query_type
            })))
        }
        Err(e) => {
            Ok(HttpResponse::BadRequest().json(json!({
                "error": "Failed to list vouchers",
                "message": e.to_string()
            })))
        }
    }
}

// Configure all voucher routes
pub async fn sync_voucher_status(
    pool: web::Data<PgPool>,
    voucher_id: web::Path<String>,
    provider: web::Data<Arc<Provider<Http>>>,
) -> Result<HttpResponse> {
    let voucher_id = voucher_id.into_inner();
    
    // Validate voucher_id format (should be 0x + 64 hex chars)
    if !voucher_id.starts_with("0x") || voucher_id.len() != 66 {
        return Ok(HttpResponse::BadRequest().json(json!({
            "error": "Invalid voucher ID format"
        })));
    }
    
    // Check if voucher exists in our database
    let voucher: Option<VoucherCode> = sqlx::query_as(
        "SELECT * FROM voucher_codes WHERE voucher_id = $1"
    )
    .bind(&voucher_id)
    .fetch_optional(pool.get_ref())
    .await
    .map_err(|e| actix_web::error::ErrorInternalServerError(e))?;
    
    if let Some(mut voucher) = voucher {
        // Get the voucher contract address from env
        let voucher_contract_address: Address = "0x66Eb0Aa46827e5F3fFcb6Dea23C309CB401690B6"
            .parse()
            .map_err(|e| actix_web::error::ErrorInternalServerError(e))?;
        
        // Check on-chain status by calling getVoucher(voucherId)
        let voucher_id_bytes = H256::from_slice(&hex::decode(&voucher_id[2..])
            .map_err(|e| actix_web::error::ErrorInternalServerError(e))?);
        
        // Create contract instance
        let abi = ethers::abi::parse_abi(&[
            "function vouchers(bytes32) view returns (address creator, uint256 amount, bool claimed)"
        ]).map_err(|e| actix_web::error::ErrorInternalServerError(e))?;
        
        let contract = Contract::new(voucher_contract_address, abi, provider.as_ref().clone());
        
        // Call vouchers mapping
        let result: (Address, U256, bool) = contract
            .method::<_, (Address, U256, bool)>("vouchers", voucher_id_bytes)
            .map_err(|e| actix_web::error::ErrorInternalServerError(e))?
            .call()
            .await
            .map_err(|e| actix_web::error::ErrorInternalServerError(e))?;
        
        let (creator, _amount, on_chain_claimed) = result;
        
        // Determine if voucher was cancelled:
        // - If creator is zero address AND claimed, it was cancelled
        // - If creator exists but claimed = true, it was claimed by someone
        let on_chain_cancelled = creator == Address::zero() && on_chain_claimed;
        let on_chain_claimed_by_user = creator != Address::zero() && on_chain_claimed;
        
        // Update database if status differs
        let mut updated = false;
        
        if on_chain_cancelled && !voucher.cancelled {
            sqlx::query(
                "UPDATE voucher_codes SET cancelled = true, cancelled_at = NOW() WHERE voucher_id = $1"
            )
            .bind(&voucher_id)
            .execute(pool.get_ref())
            .await
            .map_err(|e| actix_web::error::ErrorInternalServerError(e))?;
            
            voucher.cancelled = true;
            voucher.cancelled_at = Some(chrono::Utc::now());
            updated = true;
        }
        
        if on_chain_claimed_by_user && !voucher.claimed {
            sqlx::query(
                "UPDATE voucher_codes SET claimed = true, claimed_at = NOW() WHERE voucher_id = $1"
            )
            .bind(&voucher_id)
            .execute(pool.get_ref())
            .await
            .map_err(|e| actix_web::error::ErrorInternalServerError(e))?;
            
            voucher.claimed = true;
            voucher.claimed_at = Some(chrono::Utc::now());
            updated = true;
        }
        
        Ok(HttpResponse::Ok().json(json!({
            "voucher_id": voucher_id,
            "code": voucher.code,
            "claimed": voucher.claimed,
            "cancelled": voucher.cancelled,
            "on_chain_claimed": on_chain_claimed,
            "on_chain_cancelled": on_chain_cancelled,
            "synced": updated,
            "message": if updated { "Database updated with on-chain status" } else { "Database already in sync" }
        })))
    } else {
        Ok(HttpResponse::NotFound().json(json!({
            "error": "Voucher not found in database"
        })))
    }
}

// GET /api/vouchers/claim-tx/{tx_hash} - Get claim transaction status
pub async fn get_claim_tx_status(
    pool: web::Data<PgPool>,
    tx_hash: web::Path<String>,
) -> Result<HttpResponse> {
    let tx_hash = tx_hash.into_inner();
    
    // Validate tx hash format
    if !is_valid_tx_hash(&tx_hash) {
        return Ok(HttpResponse::BadRequest().json(json!({
            "error": "Invalid transaction hash format"
        })));
    }
    
    let voucher: Option<VoucherCode> = sqlx::query_as(
        "SELECT * FROM voucher_codes WHERE claim_tx_hash = $1"
    )
    .bind(&tx_hash)
    .fetch_optional(pool.get_ref())
    .await
    .map_err(|e| actix_web::error::ErrorInternalServerError(e))?;
    
    if let Some(voucher) = voucher {
        Ok(HttpResponse::Ok().json(json!({
            "tx_hash": tx_hash,
            "status": voucher.claim_tx_status.unwrap_or_else(|| "unknown".to_string()),
            "voucher_code": voucher.code,
            "recipient": voucher.claimed_by,
            "submitted_at": voucher.claim_tx_submitted_at,
            "claimed_at": voucher.claimed_at,
            "success": voucher.claimed
        })))
    } else {
        Ok(HttpResponse::NotFound().json(json!({
            "error": "Transaction not found"
        })))
    }
}

// GET /api/debug/wallet - Get backend wallet address for debugging
pub async fn get_wallet_address(
    service: web::Data<VoucherService>,
) -> Result<HttpResponse> {
    let wallet_address = service.get_wallet_address();
    Ok(HttpResponse::Ok().json(json!({
        "wallet_address": wallet_address,
        "message": "This is the backend wallet address used for signing"
    })))
}

// GET /api/vouchers/details/{voucher_id} - Get voucher details including creator
pub async fn get_voucher_details(
    pool: web::Data<PgPool>,
    voucher_id: web::Path<String>,
) -> Result<HttpResponse> {
    let voucher_id = voucher_id.into_inner();
    
    // Validate voucher_id format
    if !voucher_id.starts_with("0x") || voucher_id.len() != 66 {
        return Ok(HttpResponse::BadRequest().json(json!({
            "error": "Invalid voucher ID format"
        })));
    }
    
    let voucher: Option<VoucherCode> = sqlx::query_as(
        "SELECT * FROM voucher_codes WHERE voucher_id = $1"
    )
    .bind(&voucher_id)
    .fetch_optional(pool.get_ref())
    .await
    .map_err(|e| actix_web::error::ErrorInternalServerError(e))?;
    
    if let Some(voucher) = voucher {
        Ok(HttpResponse::Ok().json(json!({
            "voucher_id": voucher.voucher_id,
            "code": voucher.code,
            "creator_address": voucher.creator_address,
            "amount": voucher.amount,
            "claimed": voucher.claimed,
            "claimed_by": voucher.claimed_by,
            "cancelled": voucher.cancelled,
            "cancelled_at": voucher.cancelled_at,
            "created_at": voucher.created_at,
            "on_chain_created_at": voucher.on_chain_created_at,
        })))
    } else {
        Ok(HttpResponse::NotFound().json(json!({
            "error": "Voucher not found"
        })))
    }
}

// DELETE /api/vouchers/{voucher_id} - Soft delete a voucher
pub async fn delete_voucher(
    pool: web::Data<PgPool>,
    voucher_id: web::Path<String>,
    req: HttpRequest,
) -> Result<HttpResponse> {
    let voucher_id = voucher_id.into_inner();
    
    // Validate voucher ID format
    if !is_valid_voucher_id(&voucher_id) {
        return Ok(HttpResponse::BadRequest().json(json!({
            "error": "Invalid voucher ID format"
        })));
    }
    
    // Get the requester's address from header (you might want to add auth here)
    let creator_address = req.headers()
        .get("X-User-Address")
        .and_then(|h| h.to_str().ok())
        .unwrap_or("");
    
    // Check if voucher exists and belongs to the requester
    let voucher: Option<VoucherCode> = sqlx::query_as(
        "SELECT * FROM voucher_codes WHERE voucher_id = $1"
    )
    .bind(&voucher_id)
    .fetch_optional(pool.get_ref())
    .await
    .map_err(|e| actix_web::error::ErrorInternalServerError(e))?;
    
    match voucher {
        Some(v) => {
            // Verify ownership (optional - remove if you want anyone to delete)
            if !creator_address.is_empty() && 
               v.creator_address.as_ref().map(|c| c.to_lowercase()) != Some(creator_address.to_lowercase()) {
                return Ok(HttpResponse::Forbidden().json(json!({
                    "error": "You can only delete your own vouchers"
                })));
            }
            
            // Mark as cancelled (soft delete)
            sqlx::query(
                "UPDATE voucher_codes SET cancelled = true, cancelled_at = NOW() WHERE voucher_id = $1"
            )
            .bind(&voucher_id)
            .execute(pool.get_ref())
            .await
            .map_err(|e| actix_web::error::ErrorInternalServerError(e))?;
            
            info!("Voucher {} marked as deleted/cancelled", voucher_id);
            
            Ok(HttpResponse::Ok().json(json!({
                "success": true,
                "message": "Voucher deleted successfully"
            })))
        }
        None => {
            Ok(HttpResponse::NotFound().json(json!({
                "error": "Voucher not found"
            })))
        }
    }
}

pub fn configure_voucher_routes(cfg: &mut web::ServiceConfig) {
    cfg.service(
        web::scope("/api/vouchers")
            .route("/link", web::post().to(create_voucher_link))
            .route("/verify", web::post().to(verify_voucher))
            .route("/claim", web::post().to(claim_voucher))
            .route("/execute-claim", web::post().to(execute_claim))
            .route("/claim-status", web::post().to(update_claim_status))
            .route("/claim-tx/{tx_hash}", web::get().to(get_claim_tx_status))
            .route("/user/{address}", web::get().to(list_user_vouchers))
            .route("/sync/{voucher_id}", web::post().to(sync_voucher_status))
            .route("/details/{voucher_id}", web::get().to(get_voucher_details))
            .route("/{voucher_id}", web::delete().to(delete_voucher))
    );
    cfg.service(
        web::scope("/api/debug")
            .route("/wallet", web::get().to(get_wallet_address))
    );
}

// Helper functions
fn extract_ip(req: &HttpRequest) -> String {
    let connection_info = req.connection_info();
    connection_info
        .realip_remote_addr()
        .or_else(|| connection_info.peer_addr())
        .unwrap_or("unknown")
        .to_string()
}

fn is_valid_voucher_id(id: &str) -> bool {
    // Check if it's a valid hex string with 0x prefix and 64 hex chars (32 bytes)
    id.len() == 66 && id.starts_with("0x") && id[2..].chars().all(|c| c.is_ascii_hexdigit())
}

fn is_valid_ethereum_address(address: &str) -> bool {
    if !address.starts_with("0x") || address.len() != 42 {
        return false;
    }
    
    address[2..].chars().all(|c| c.is_ascii_hexdigit())
}

fn is_valid_tx_hash(hash: &str) -> bool {
    if !hash.starts_with("0x") || hash.len() != 66 {
        return false;
    }
    
    hash[2..].chars().all(|c| c.is_ascii_hexdigit())
}