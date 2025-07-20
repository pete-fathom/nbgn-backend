use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize, sqlx::FromRow)]
pub struct VoucherCode {
    pub code: String,
    pub voucher_id: String,
    pub password_hash: Option<String>,
    pub created_at: DateTime<Utc>,
    pub creator_address: Option<String>,
    pub amount: Option<String>,
    pub on_chain_created_at: Option<DateTime<Utc>>,
    pub claimed: bool,
    pub claimed_by: Option<String>,
    pub claimed_at: Option<DateTime<Utc>>,
    pub claim_tx_hash: Option<String>,
    pub cancelled: bool,
    pub cancelled_at: Option<DateTime<Utc>>,
    pub cancel_tx_hash: Option<String>,
    pub claim_tx_status: Option<String>,
    pub claim_tx_submitted_at: Option<DateTime<Utc>>,
}

#[derive(Debug, Serialize, Deserialize, sqlx::FromRow)]
pub struct ClaimAttempt {
    pub id: i32,
    pub voucher_code: Option<String>,
    pub ip_address: Option<String>,
    pub attempted_at: DateTime<Utc>,
    pub success: Option<bool>,
    pub recipient_address: Option<String>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct CreateLinkRequest {
    pub voucher_id: String,
    pub password: Option<String>,
    pub creator_address: Option<String>,
    pub amount: Option<String>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct VerifyRequest {
    pub code: String,
    pub password: Option<String>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ClaimRequest {
    pub code: String,
    pub password: Option<String>,
    pub recipient_address: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ClaimStatusRequest {
    pub code: String,
    pub tx_hash: String,
    pub success: bool,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ClaimAuthorization {
    pub voucher_id: String,
    pub recipient: String,
    pub amount: String,
    pub deadline: u64,
    pub signature: String,
    pub contract_address: String,
}

#[derive(Debug, Deserialize)]
pub struct ListQuery {
    #[serde(rename = "type")]
    pub query_type: Option<String>, // "created" or "received"
    pub page: Option<i32>,
    pub limit: Option<i32>,
}