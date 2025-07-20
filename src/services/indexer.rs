use crate::contracts::nbgn::NBGNContract;
use ethers::prelude::*;
use sqlx::PgPool;
use std::sync::Arc;
use tracing::{info, error, debug};

#[derive(Clone)]
pub struct Indexer {
    contract: NBGNContract,
    pool: PgPool,
    provider: Arc<Provider<Http>>,
}

impl Indexer {
    pub fn new(contract: NBGNContract, pool: PgPool, provider: Arc<Provider<Http>>) -> Self {
        Self {
            contract,
            pool,
            provider,
        }
    }

    pub async fn get_last_indexed_block(&self) -> Result<u64, Box<dyn std::error::Error>> {
        let result: Option<(i64,)> = sqlx::query_as(
            "SELECT last_indexed_block FROM sync_status WHERE id = 1"
        )
        .fetch_optional(&self.pool)
        .await?;

        match result {
            Some((block,)) => Ok(block as u64),
            None => {
                // Initialize sync status
                sqlx::query(
                    "INSERT INTO sync_status (id, last_indexed_block) VALUES (1, 0)"
                )
                .execute(&self.pool)
                .await?;
                Ok(0)
            }
        }
    }

    pub async fn update_last_indexed_block(&self, block_number: u64) -> Result<(), Box<dyn std::error::Error>> {
        sqlx::query(
            "UPDATE sync_status SET last_indexed_block = $1, updated_at = NOW() WHERE id = 1"
        )
        .bind(block_number as i64)
        .execute(&self.pool)
        .await?;
        Ok(())
    }

    pub async fn index_events(&self, from_block: u64, to_block: u64) -> Result<(), Box<dyn std::error::Error>> {
        info!("Indexing events from block {} to {}", from_block, to_block);

        // Note: In a real implementation, you would query the events here
        // For now, we'll just update the last indexed block
        self.update_last_indexed_block(to_block).await?;

        Ok(())
    }

    pub async fn run_indexer_loop(&self, poll_interval_secs: u64) -> Result<(), Box<dyn std::error::Error>> {
        let mut interval = tokio::time::interval(tokio::time::Duration::from_secs(poll_interval_secs));

        loop {
            interval.tick().await;
            
            match self.index_latest_events().await {
                Ok(_) => debug!("Indexer cycle completed successfully"),
                Err(e) => error!("Error in indexer cycle: {}", e),
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
                self.index_events(from_block, to_block).await?;
                from_block = to_block + 1;
            }
        }

        Ok(())
    }
}