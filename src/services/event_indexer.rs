use crate::db::voucher_models::VoucherCode;
use ethers::prelude::*;
use sqlx::PgPool;
use std::sync::Arc;
use tracing::{info, error, debug};
use rand::{thread_rng, Rng};
use rand::distributions::Alphanumeric;

// Define the VoucherCreated event structure
#[derive(Debug, Clone, EthEvent)]
#[ethevent(name = "VoucherCreated", abi = "VoucherCreated(bytes32,address,uint256)")]
pub struct VoucherCreated {
    #[ethevent(indexed)]
    pub voucher_id: H256,
    #[ethevent(indexed)]
    pub creator: Address,
    pub amount: U256,
}

// Define the VoucherCancelled event structure
#[derive(Debug, Clone, EthEvent)]
#[ethevent(name = "VoucherCancelled", abi = "VoucherCancelled(bytes32,address,uint256)")]
pub struct VoucherCancelled {
    #[ethevent(indexed)]
    pub voucher_id: H256,
    #[ethevent(indexed)]
    pub creator: Address,
    pub amount: U256,
}

#[derive(Clone)]
pub struct EventIndexer {
    pool: PgPool,
    provider: Arc<Provider<Http>>,
    voucher_contract: Address,
}

impl EventIndexer {
    pub fn new(pool: PgPool, provider: Arc<Provider<Http>>, voucher_contract: Address) -> Self {
        Self {
            pool,
            provider,
            voucher_contract,
        }
    }

    pub async fn get_last_indexed_block(&self) -> Result<u64, Box<dyn std::error::Error>> {
        let result: Option<(i64,)> = sqlx::query_as(
            "SELECT last_indexed_block FROM sync_status WHERE id = 2" // id=2 for voucher indexer
        )
        .fetch_optional(&self.pool)
        .await?;

        match result {
            Some((block,)) => Ok(block as u64),
            None => {
                // Initialize sync status for voucher indexer
                sqlx::query(
                    "INSERT INTO sync_status (id, last_indexed_block) VALUES (2, 0) ON CONFLICT (id) DO NOTHING"
                )
                .execute(&self.pool)
                .await?;
                Ok(0)
            }
        }
    }

    pub async fn update_last_indexed_block(&self, block_number: u64) -> Result<(), Box<dyn std::error::Error>> {
        sqlx::query(
            "UPDATE sync_status SET last_indexed_block = $1, updated_at = NOW() WHERE id = 2"
        )
        .bind(block_number as i64)
        .execute(&self.pool)
        .await?;
        Ok(())
    }

    pub async fn index_voucher_events(&self, from_block: u64, to_block: u64) -> Result<(), Box<dyn std::error::Error>> {
        info!("Indexing voucher events from block {} to {}", from_block, to_block);

        // Get both VoucherCreated and VoucherCancelled events
        let filter = Filter::new()
            .address(self.voucher_contract)
            .from_block(from_block)
            .to_block(to_block);

        let logs = self.provider.get_logs(&filter).await?;
        
        info!("Found {} logs in blocks {} to {}", logs.len(), from_block, to_block);

        for log in logs {
            // Debug: print raw log topic
            if !log.topics.is_empty() {
                debug!("Processing log with topic: {:?}", log.topics[0]);
            }
            
            if let Ok(event) = parse_log::<VoucherCreated>(log.clone()) {
                let voucher_id_hex = format!("0x{}", hex::encode(event.voucher_id.as_bytes()));
                
                // Check if we already have a code for this voucher_id
                let existing: Option<(String,)> = sqlx::query_as(
                    "SELECT code FROM voucher_codes WHERE voucher_id = $1"
                )
                .bind(&voucher_id_hex)
                .fetch_optional(&self.pool)
                .await?;

                if existing.is_none() {
                    // Generate new voucher code
                    let code = generate_voucher_code();
                    
                    // Get block timestamp
                    let block = self.provider.get_block(log.block_number.unwrap()).await?;
                    let timestamp = block.and_then(|b| Some(b.timestamp.as_u64())).unwrap_or(0);
                    let on_chain_created_at = chrono::DateTime::from_timestamp(timestamp as i64, 0);

                    // Store mapping
                    sqlx::query(
                        r#"
                        INSERT INTO voucher_codes 
                        (code, voucher_id, creator_address, amount, on_chain_created_at)
                        VALUES ($1, $2, $3, $4, $5)
                        "#
                    )
                    .bind(&code)
                    .bind(&voucher_id_hex)
                    .bind(format!("{:?}", event.creator))
                    .bind(event.amount.to_string())
                    .bind(on_chain_created_at)
                    .execute(&self.pool)
                    .await?;

                    info!("Created voucher code {} for voucher_id {} from creator {}", 
                          code, voucher_id_hex, event.creator);
                }
            } else if let Ok(event) = parse_log::<VoucherCancelled>(log.clone()) {
                let voucher_id_hex = format!("0x{}", hex::encode(event.voucher_id.as_bytes()));
                
                info!("Processing VoucherCancelled event for voucher {} by creator {}", 
                      voucher_id_hex, event.creator);
                
                // Get block timestamp
                let block = self.provider.get_block(log.block_number.unwrap()).await?;
                let timestamp = block.and_then(|b| Some(b.timestamp.as_u64())).unwrap_or(0);
                let cancelled_at = chrono::DateTime::from_timestamp(timestamp as i64, 0);
                
                // Update voucher as cancelled
                let result = sqlx::query(
                    r#"
                    UPDATE voucher_codes 
                    SET cancelled = true, 
                        cancelled_at = $1,
                        cancel_tx_hash = $2
                    WHERE voucher_id = $3
                    "#
                )
                .bind(cancelled_at)
                .bind(format!("{:?}", log.transaction_hash.unwrap()))
                .bind(&voucher_id_hex)
                .execute(&self.pool)
                .await?;
                
                if result.rows_affected() > 0 {
                    info!("Marked voucher {} as cancelled by creator {}", 
                          voucher_id_hex, event.creator);
                }
            }
        }

        Ok(())
    }

    pub async fn run_indexer_loop(&self, poll_interval_secs: u64) -> Result<(), Box<dyn std::error::Error>> {
        let mut interval = tokio::time::interval(tokio::time::Duration::from_secs(poll_interval_secs));

        loop {
            interval.tick().await;
            
            match self.index_latest_events().await {
                Ok(_) => debug!("Voucher indexer cycle completed successfully"),
                Err(e) => error!("Error in voucher indexer cycle: {}", e),
            }
        }
    }

    async fn index_latest_events(&self) -> Result<(), Box<dyn std::error::Error>> {
        let last_indexed = self.get_last_indexed_block().await?;
        let current_block = self.provider.get_block_number().await?.as_u64();

        if current_block > last_indexed {
            // Index in batches of 1000 blocks
            let batch_size = 1000u64;
            let mut from_block = last_indexed + 1;

            while from_block <= current_block {
                let to_block = (from_block + batch_size - 1).min(current_block);
                self.index_voucher_events(from_block, to_block).await?;
                self.update_last_indexed_block(to_block).await?;
                from_block = to_block + 1;
            }
        }

        Ok(())
    }
}

// Generate a 16-character alphanumeric voucher code
pub fn generate_voucher_code() -> String {
    thread_rng()
        .sample_iter(&Alphanumeric)
        .take(16)
        .map(char::from)
        .collect::<String>()
        .to_uppercase()
}