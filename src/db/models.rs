use chrono::{DateTime, Utc, NaiveDate};
use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize, sqlx::FromRow)]
pub struct User {
    pub address: String,
    pub username: Option<String>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Serialize, Deserialize, sqlx::FromRow)]
pub struct Transaction {
    pub id: i64,
    pub tx_hash: String,
    pub block_number: i64,
    pub timestamp: DateTime<Utc>,
    pub user_address: String,
    pub transaction_type: String,
    pub eure_amount: Option<String>,
    pub nbgn_amount: String,
    pub gas_used: Option<String>,
    pub gas_price: Option<String>,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Serialize, Deserialize, sqlx::FromRow)]
pub struct DailyStat {
    pub date: NaiveDate,
    pub total_volume: Option<String>,
    pub unique_users: Option<i32>,
    pub transaction_count: Option<i32>,
    pub average_tx_size: Option<String>,
    pub ending_supply: Option<String>,
    pub ending_reserves: Option<String>,
    pub reserve_ratio: Option<String>,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Serialize, Deserialize, sqlx::FromRow)]
pub struct SyncStatus {
    pub id: i32,
    pub last_indexed_block: i64,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Serialize, Deserialize, sqlx::FromRow)]
pub struct UserProfile {
    pub address: String,
    pub username: Option<String>,
    pub created_at: DateTime<Utc>,
    pub total_minted: String,
    pub total_redeemed: String,
    pub total_burned: String,
    pub transaction_count: i64,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct SetUsernameRequest {
    pub address: String,
    pub username: String,
    pub message: String,
    pub signature: String,
}