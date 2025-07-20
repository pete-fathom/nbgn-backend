use actix_web::{
    dev::{forward_ready, Service, ServiceRequest, ServiceResponse, Transform},
    Error, HttpResponse,
    http::{header, StatusCode},
};
use futures_util::future::LocalBoxFuture;
use redis::{AsyncCommands, RedisError};
use serde_json::json;
use std::{
    future::{ready, Ready},
    rc::Rc,
    time::{SystemTime, UNIX_EPOCH},
};
use tracing::warn;

#[derive(Clone)]
pub struct RedisRateLimiter {
    client: redis::Client,
}

impl RedisRateLimiter {
    pub fn new(redis_url: &str) -> Result<Self, RedisError> {
        let client = redis::Client::open(redis_url)?;
        Ok(Self { client })
    }

    pub async fn check_rate_limit(
        &self,
        key: &str,
        limit: u64,
        window_seconds: u64,
    ) -> Result<RateLimitResult, RedisError> {
        let mut conn = self.client.get_async_connection().await?;
        
        // Use a sliding window approach with Redis
        let current_time = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_secs();
        
        // Create a unique key for this time window
        let window_key = format!("rate_limit:{}:{}", key, current_time / window_seconds);
        
        // Increment the counter
        let count: u64 = conn.incr(&window_key, 1).await?;
        
        // Set expiry on first request
        if count == 1 {
            conn.expire(&window_key, window_seconds as i64).await?;
        }
        
        // Get TTL for the key
        let ttl: i64 = conn.ttl(&window_key).await?;
        let ttl = if ttl < 0 { window_seconds as i64 } else { ttl };
        
        Ok(RateLimitResult {
            allowed: count <= limit,
            limit,
            remaining: limit.saturating_sub(count),
            reset_time: current_time + ttl as u64,
            retry_after: if count > limit { Some(ttl as u64) } else { None },
        })
    }
}

#[derive(Debug)]
pub struct RateLimitResult {
    pub allowed: bool,
    pub limit: u64,
    pub remaining: u64,
    pub reset_time: u64,
    pub retry_after: Option<u64>,
}

pub struct RateLimiterMiddleware {
    rate_limiter: Rc<RedisRateLimiter>,
}

impl RateLimiterMiddleware {
    pub fn new(rate_limiter: RedisRateLimiter) -> Self {
        Self {
            rate_limiter: Rc::new(rate_limiter),
        }
    }
}

impl<S> Transform<S, ServiceRequest> for RateLimiterMiddleware
where
    S: Service<ServiceRequest, Response = ServiceResponse, Error = Error> + 'static,
    S::Future: 'static,
{
    type Response = ServiceResponse;
    type Error = Error;
    type InitError = ();
    type Transform = RateLimiterMiddlewareService<S>;
    type Future = Ready<Result<Self::Transform, Self::InitError>>;

    fn new_transform(&self, service: S) -> Self::Future {
        ready(Ok(RateLimiterMiddlewareService {
            service: Rc::new(service),
            rate_limiter: self.rate_limiter.clone(),
        }))
    }
}

pub struct RateLimiterMiddlewareService<S> {
    service: Rc<S>,
    rate_limiter: Rc<RedisRateLimiter>,
}

impl<S> Service<ServiceRequest> for RateLimiterMiddlewareService<S>
where
    S: Service<ServiceRequest, Response = ServiceResponse, Error = Error> + 'static,
    S::Future: 'static,
{
    type Response = ServiceResponse;
    type Error = Error;
    type Future = LocalBoxFuture<'static, Result<Self::Response, Self::Error>>;

    forward_ready!(service);

    fn call(&self, req: ServiceRequest) -> Self::Future {
        let service = self.service.clone();
        let rate_limiter = self.rate_limiter.clone();

        Box::pin(async move {
            // Extract identifier (API key or IP address)
            let identifier = extract_identifier(&req);
            
            // Get rate limit configuration based on path
            let (limit, window) = get_rate_limit_config(req.path());
            
            // Check rate limit
            match rate_limiter.check_rate_limit(&identifier, limit, window).await {
                Ok(result) => {
                    if result.allowed {
                        // Add rate limit headers to the response
                        let fut = service.call(req);
                        let mut res = fut.await?;
                        
                        let headers = res.headers_mut();
                        headers.insert(
                            header::HeaderName::from_static("x-ratelimit-limit"),
                            header::HeaderValue::from_str(&result.limit.to_string()).unwrap(),
                        );
                        headers.insert(
                            header::HeaderName::from_static("x-ratelimit-remaining"),
                            header::HeaderValue::from_str(&result.remaining.to_string()).unwrap(),
                        );
                        headers.insert(
                            header::HeaderName::from_static("x-ratelimit-reset"),
                            header::HeaderValue::from_str(&result.reset_time.to_string()).unwrap(),
                        );
                        
                        Ok(res)
                    } else {
                        // Rate limit exceeded
                        let body = json!({
                            "error": "Too Many Requests",
                            "message": format!(
                                "Rate limit exceeded. Maximum {} requests per {} seconds allowed.",
                                limit, window
                            ),
                            "retry_after": result.retry_after,
                        });
                        
                        let err_response = HttpResponse::build(StatusCode::TOO_MANY_REQUESTS)
                            .insert_header((
                                header::HeaderName::from_static("x-ratelimit-limit"),
                                result.limit.to_string(),
                            ))
                            .insert_header((
                                header::HeaderName::from_static("x-ratelimit-remaining"),
                                "0",
                            ))
                            .insert_header((
                                header::HeaderName::from_static("x-ratelimit-reset"),
                                result.reset_time.to_string(),
                            ))
                            .insert_header((
                                header::HeaderName::from_static("retry-after"),
                                result.retry_after.unwrap_or(60).to_string(),
                            ))
                            .json(body);
                        
                        Ok(req.into_response(err_response))
                    }
                }
                Err(e) => {
                    // Redis error - fail open (allow the request)
                    warn!("Redis error in rate limiter, failing open: {}", e);
                    service.call(req).await
                }
            }
        })
    }
}

fn extract_identifier(req: &ServiceRequest) -> String {
    // First check for API key
    if let Some(api_key) = req.headers().get("x-api-key") {
        if let Ok(key) = api_key.to_str() {
            return format!("api_key:{}", key);
        }
    }
    
    // Fall back to IP address
    let connection_info = req.connection_info();
    let ip = connection_info
        .realip_remote_addr()
        .or_else(|| connection_info.peer_addr())
        .unwrap_or("unknown");
    
    format!("ip:{}", ip)
}

pub fn get_rate_limit_config(path: &str) -> (u64, u64) {
    match path {
        p if p.starts_with("/api/users/username") => (5, 3600),      // 5 per hour
        p if p.starts_with("/api/vouchers/verify") => (10, 3600),    // 10 per hour (handled per code+IP in handler)
        p if p.starts_with("/api/vouchers/claim") => (10, 3600),     // 10 per hour
        p if p.starts_with("/api/vouchers/link") => (20, 60),        // 20 per minute
        p if p.starts_with("/api/vouchers") => (50, 60),             // 50 per minute for other voucher endpoints
        p if p.starts_with("/api/transactions") => (100, 60),        // 100 per minute
        p if p.starts_with("/api/analytics") => (50, 60),            // 50 per minute
        _ => (200, 60),                                               // Default: 200 per minute
    }
}