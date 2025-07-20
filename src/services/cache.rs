use redis::{AsyncCommands, Client, RedisError};
use serde::{Deserialize, Serialize};
use std::time::Duration;

#[derive(Clone)]
pub struct CacheService {
    client: Client,
}

impl CacheService {
    pub fn new(redis_url: &str) -> Result<Self, RedisError> {
        let client = Client::open(redis_url)?;
        Ok(Self { client })
    }

    pub async fn get<T: for<'de> Deserialize<'de>>(&self, key: &str) -> Result<Option<T>, RedisError> {
        let mut conn = self.client.get_async_connection().await?;
        let value: Option<String> = conn.get(key).await?;
        
        match value {
            Some(json) => {
                let data = serde_json::from_str(&json)
                    .map_err(|e| RedisError::from((redis::ErrorKind::TypeError, "JSON deserialization failed", e.to_string())))?;
                Ok(Some(data))
            }
            None => Ok(None),
        }
    }

    pub async fn set<T: Serialize>(&self, key: &str, value: &T, ttl: Duration) -> Result<(), RedisError> {
        let mut conn = self.client.get_async_connection().await?;
        let json = serde_json::to_string(value)
            .map_err(|e| RedisError::from((redis::ErrorKind::TypeError, "JSON serialization failed", e.to_string())))?;
        
        conn.set_ex(key, json, ttl.as_secs()).await?;
        Ok(())
    }

    pub async fn delete(&self, key: &str) -> Result<(), RedisError> {
        let mut conn = self.client.get_async_connection().await?;
        conn.del(key).await?;
        Ok(())
    }

    // Specific cache methods for common data
    pub async fn get_reserve_ratio(&self) -> Result<Option<String>, RedisError> {
        self.get("reserve_ratio").await
    }

    pub async fn set_reserve_ratio(&self, ratio: &str, ttl: Duration) -> Result<(), RedisError> {
        self.set("reserve_ratio", &ratio, ttl).await
    }

    pub async fn get_total_supply(&self) -> Result<Option<String>, RedisError> {
        self.get("total_supply").await
    }

    pub async fn set_total_supply(&self, supply: &str, ttl: Duration) -> Result<(), RedisError> {
        self.set("total_supply", &supply, ttl).await
    }

    pub async fn get_user_profile(&self, address: &str) -> Result<Option<crate::db::models::UserProfile>, RedisError> {
        self.get(&format!("user_profile:{}", address)).await
    }

    pub async fn set_user_profile(&self, profile: &crate::db::models::UserProfile, ttl: Duration) -> Result<(), RedisError> {
        self.set(&format!("user_profile:{}", profile.address), profile, ttl).await
    }
}