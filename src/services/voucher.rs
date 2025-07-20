use crate::db::voucher_models::{VoucherCode, ClaimAuthorization};
use ethers::prelude::*;
use ethers::utils::keccak256;
use sqlx::PgPool;
use tracing::{info, error};
use argon2::{Argon2, PasswordHash, PasswordHasher, PasswordVerifier};
use argon2::password_hash::{rand_core::OsRng, SaltString};
use std::str::FromStr;
use std::sync::Arc;

const VOUCHER_CONTRACT: &str = "0x66Eb0Aa46827e5F3fFcb6Dea23C309CB401690B6";
const CHAIN_ID: u64 = 42161; // Arbitrum One
const DEFAULT_DEADLINE_SECONDS: u64 = 3600; // 1 hour

// Gas limits for frontend reference
const CREATE_VOUCHER_GAS: u64 = 150_000;
const CLAIM_VOUCHER_GAS: u64 = 200_000;

#[derive(Clone)]
pub struct VoucherService {
    pool: PgPool,
    wallet: LocalWallet,
    provider: Option<Arc<Provider<Http>>>,
}

impl VoucherService {
    pub fn new(pool: PgPool, private_key: &str) -> Result<Self, Box<dyn std::error::Error>> {
        let wallet = private_key.parse::<LocalWallet>()?
            .with_chain_id(CHAIN_ID);
        
        Ok(Self { pool, wallet, provider: None })
    }
    
    pub fn with_provider(mut self, provider: Arc<Provider<Http>>) -> Self {
        self.provider = Some(provider);
        self
    }
    
    // Get wallet address for debugging
    pub fn get_wallet_address(&self) -> String {
        format!("{:?}", self.wallet.address())
    }

    // Generate bytes32 voucher ID from user-friendly code
    pub fn code_to_voucher_id(code: &str) -> H256 {
        H256::from(keccak256(code.as_bytes()))
    }

    // Hash password for storage
    pub fn hash_password(password: &str) -> Result<String, Box<dyn std::error::Error>> {
        let salt = SaltString::generate(&mut OsRng);
        let argon2 = Argon2::default();
        let password_hash = argon2.hash_password(password.as_bytes(), &salt)
            .map_err(|e| format!("Password hashing failed: {}", e))?;
        Ok(password_hash.to_string())
    }

    // Verify password
    pub fn verify_password(password: &str, password_hash: &str) -> bool {
        if let Ok(parsed_hash) = PasswordHash::new(password_hash) {
            let argon2 = Argon2::default();
            argon2.verify_password(password.as_bytes(), &parsed_hash).is_ok()
        } else {
            false
        }
    }

    // Get voucher by code
    pub async fn get_voucher_by_code(&self, code: &str) -> Result<Option<VoucherCode>, sqlx::Error> {
        sqlx::query_as::<_, VoucherCode>(
            "SELECT * FROM voucher_codes WHERE code = $1"
        )
        .bind(code)
        .fetch_optional(&self.pool)
        .await
    }

    // Create claim authorization with signature
    pub async fn create_claim_authorization(
        &self,
        voucher_code: &str,
        recipient_address: &str,
        password: Option<&str>,
    ) -> Result<ClaimAuthorization, Box<dyn std::error::Error>> {
        // Get voucher from DB
        let voucher = self.get_voucher_by_code(voucher_code).await?
            .ok_or("Voucher not found")?;

        // Check if already claimed
        if voucher.claimed {
            return Err("Voucher already claimed".into());
        }
        
        // Check if cancelled
        if voucher.cancelled {
            return Err("Voucher has been cancelled".into());
        }

        // Verify password if set
        if let Some(hash) = &voucher.password_hash {
            if password.is_none() || !Self::verify_password(password.unwrap(), hash) {
                return Err("Invalid password".into());
            }
        }

        // Parse addresses and values
        let voucher_id = H256::from_str(&voucher.voucher_id)?;
        let recipient = Address::from_str(recipient_address)?;
        let deadline = U256::from(chrono::Utc::now().timestamp() as u64 + DEFAULT_DEADLINE_SECONDS);
        let contract_address = Address::from_str(VOUCHER_CONTRACT)?;
        let chain_id = U256::from(CHAIN_ID);

        // Create message hash exactly as contract expects
        // The contract expects: keccak256(abi.encodePacked(voucherId, recipient, deadline, address(this), block.chainid))
        let mut encoded = Vec::new();
        encoded.extend_from_slice(voucher_id.as_bytes()); // bytes32
        encoded.extend_from_slice(recipient.as_bytes()); // address (20 bytes)
        let deadline_bytes: [u8; 32] = deadline.into();
        encoded.extend_from_slice(&deadline_bytes); // uint256 (full 32 bytes)
        encoded.extend_from_slice(contract_address.as_bytes()); // address (20 bytes)
        let chain_id_bytes: [u8; 32] = chain_id.into();
        encoded.extend_from_slice(&chain_id_bytes); // uint256 (full 32 bytes)

        let message_hash = keccak256(&encoded);

        // Contract uses MessageHashUtils.toEthSignedMessageHash(message) and then recovers the signature
        // toEthSignedMessageHash adds the EIP-191 prefix: "\x19Ethereum Signed Message:\n32" + message
        // So we need to create the same prefixed hash and sign it
        let eth_message_prefix = "\x19Ethereum Signed Message:\n32";
        let prefixed_message = [eth_message_prefix.as_bytes(), &message_hash].concat();
        let eth_signed_message_hash = keccak256(&prefixed_message);
        
        // Sign the prefixed hash directly (no additional prefix)
        let signature = self.wallet.sign_hash(H256::from(eth_signed_message_hash))?;

        Ok(ClaimAuthorization {
            voucher_id: voucher.voucher_id,
            recipient: recipient_address.to_string(),
            amount: voucher.amount.unwrap_or_else(|| "0".to_string()),
            deadline: deadline.as_u64(),
            signature: format!("0x{}", hex::encode(signature.to_vec())),
            contract_address: VOUCHER_CONTRACT.to_string(),
        })
    }

    // Create shareable link for existing on-chain voucher
    pub async fn create_voucher_link(
        &self,
        voucher_id: &str,
        password: Option<&str>,
        creator_address: Option<&str>,
        amount: Option<&str>,
    ) -> Result<String, Box<dyn std::error::Error>> {
        // Check if we already have a code for this voucher_id
        let existing: Option<(String,)> = sqlx::query_as(
            "SELECT code FROM voucher_codes WHERE voucher_id = $1"
        )
        .bind(voucher_id)
        .fetch_optional(&self.pool)
        .await?;

        if let Some((code,)) = existing {
            // Update password if provided
            if let Some(pwd) = password {
                let password_hash = Self::hash_password(pwd)?;
                sqlx::query(
                    "UPDATE voucher_codes SET password_hash = $1 WHERE code = $2"
                )
                .bind(&password_hash)
                .bind(&code)
                .execute(&self.pool)
                .await?;
            }
            return Ok(code);
        }

        // Generate new code
        let code = crate::services::event_indexer::generate_voucher_code();
        let password_hash = password.map(Self::hash_password).transpose()?;

        // If amount not provided, fetch from blockchain
        let final_amount = if amount.is_some() {
            amount.map(|s| s.to_string())
        } else if let Some(provider) = &self.provider {
            // Fetch voucher data from blockchain
            match self.fetch_voucher_from_blockchain(voucher_id, provider).await {
                Ok((creator, amount_u256, _claimed)) => {
                    Some(amount_u256.to_string())
                }
                Err(e) => {
                    info!("Failed to fetch amount from blockchain for {}: {}", voucher_id, e);
                    None
                }
            }
        } else {
            None
        };

        // If creator not provided, fetch from blockchain
        let final_creator = if creator_address.is_some() {
            creator_address.map(|s| s.to_string())
        } else if let Some(provider) = &self.provider {
            // Fetch voucher data from blockchain
            match self.fetch_voucher_from_blockchain(voucher_id, provider).await {
                Ok((creator, _amount_u256, _claimed)) => {
                    Some(format!("{:?}", creator))
                }
                Err(e) => {
                    info!("Failed to fetch creator from blockchain for {}: {}", voucher_id, e);
                    None
                }
            }
        } else {
            None
        };

        // Store mapping
        sqlx::query(
            r#"
            INSERT INTO voucher_codes (code, voucher_id, password_hash, creator_address, amount)
            VALUES ($1, $2, $3, $4, $5)
            "#
        )
        .bind(&code)
        .bind(voucher_id)
        .bind(&password_hash)
        .bind(&final_creator)
        .bind(&final_amount)
        .execute(&self.pool)
        .await?;

        Ok(code)
    }
    
    // Helper method to fetch voucher data from blockchain
    async fn fetch_voucher_from_blockchain(
        &self,
        voucher_id: &str,
        provider: &Provider<Http>,
    ) -> Result<(Address, U256, bool), Box<dyn std::error::Error>> {
        // Parse voucher ID
        let voucher_id_bytes = H256::from_str(voucher_id)?;
        
        // Get contract address
        let contract_address: Address = VOUCHER_CONTRACT.parse()?;
        
        // Create contract instance
        let abi = ethers::abi::parse_abi(&[
            "function vouchers(bytes32) view returns (address creator, uint256 amount, bool claimed)"
        ])?;
        
        let contract = Contract::new(contract_address, abi, Arc::new(provider.clone()));
        
        // Call vouchers mapping
        let result: (Address, U256, bool) = contract
            .method::<_, (Address, U256, bool)>("vouchers", voucher_id_bytes)?
            .call()
            .await?;
        
        Ok(result)
    }

    // Update claim status after transaction
    pub async fn update_claim_status(
        &self,
        code: &str,
        tx_hash: &str,
        success: bool,
        claimed_by: &str,
    ) -> Result<(), Box<dyn std::error::Error>> {
        if success {
            sqlx::query(
                r#"
                UPDATE voucher_codes 
                SET claimed = true, 
                    claimed_by = $1, 
                    claimed_at = NOW(), 
                    claim_tx_hash = $2
                WHERE code = $3
                "#
            )
            .bind(claimed_by)
            .bind(tx_hash)
            .bind(code)
            .execute(&self.pool)
            .await?;

            info!("Voucher {} successfully claimed by {} in tx {}", code, claimed_by, tx_hash);
        }

        Ok(())
    }

    // List vouchers for a user
    pub async fn list_user_vouchers(
        &self,
        address: &str,
        query_type: &str,
        page: i32,
        limit: i32,
    ) -> Result<Vec<VoucherCode>, Box<dyn std::error::Error>> {
        let offset = page * limit;
        
        let vouchers = match query_type {
            "created" => {
                sqlx::query_as::<_, VoucherCode>(
                    r#"
                    SELECT * FROM voucher_codes 
                    WHERE LOWER(creator_address) = LOWER($1)
                    AND cancelled = FALSE
                    ORDER BY created_at DESC
                    LIMIT $2 OFFSET $3
                    "#
                )
                .bind(address)
                .bind(limit)
                .bind(offset)
                .fetch_all(&self.pool)
                .await?
            },
            "received" => {
                sqlx::query_as::<_, VoucherCode>(
                    r#"
                    SELECT * FROM voucher_codes 
                    WHERE LOWER(claimed_by) = LOWER($1)
                    ORDER BY claimed_at DESC
                    LIMIT $2 OFFSET $3
                    "#
                )
                .bind(address)
                .bind(limit)
                .bind(offset)
                .fetch_all(&self.pool)
                .await?
            },
            _ => {
                return Err("Invalid query type. Use 'created' or 'received'".into());
            }
        };

        Ok(vouchers)
    }
    
    // Execute claim transaction on-chain (gasless for user)
    pub async fn execute_claim(
        &self,
        voucher_code: &str,
        recipient_address: &str,
        password: Option<&str>,
    ) -> Result<String, Box<dyn std::error::Error>> {
        // First create the claim authorization to validate everything
        let auth = self.create_claim_authorization(voucher_code, recipient_address, password).await?;
        
        // Get provider
        let provider = self.provider.as_ref()
            .ok_or("Provider not configured for gasless claims")?;
        
        // Create signer
        let signer = SignerMiddleware::new(provider.clone(), self.wallet.clone());
        
        // Parse contract ABI
        let abi = ethers::abi::parse_abi(&[
            "function claimVoucher(bytes32 voucherId, address recipient, uint256 deadline, bytes memory signature) external"
        ])?;
        
        // Create contract instance
        let contract_address: Address = VOUCHER_CONTRACT.parse()?;
        let contract = Contract::new(contract_address, abi, Arc::new(signer));
        
        // Parse parameters
        let voucher_id = H256::from_str(&auth.voucher_id)?;
        let recipient = Address::from_str(&auth.recipient)?;
        let deadline = U256::from(auth.deadline);
        let signature_bytes = hex::decode(&auth.signature[2..])?; // Remove 0x prefix
        
        // Build and send transaction
        let tx_call = contract
            .method::<_, ()>("claimVoucher", (voucher_id, recipient, deadline, signature_bytes))?
            .gas(CLAIM_VOUCHER_GAS);
            
        let pending_tx = tx_call.send().await?;
        let tx_hash_bytes = pending_tx.tx_hash();
        let tx_hash = format!("0x{}", hex::encode(tx_hash_bytes));
        
        info!(
            "Submitted gasless claim transaction {} for voucher {} to recipient {}",
            tx_hash, voucher_code, recipient_address
        );
        
        // Update claim status to pending
        sqlx::query(
            r#"
            UPDATE voucher_codes 
            SET claim_tx_hash = $1,
                claim_tx_status = 'pending',
                claim_tx_submitted_at = NOW()
            WHERE code = $2
            "#
        )
        .bind(&tx_hash)
        .bind(voucher_code)
        .execute(&self.pool)
        .await?;
        
        // Return immediately - we'll monitor the transaction status via another endpoint
        // In production, you'd want a background worker to monitor pending transactions
        Ok(tx_hash)
    }
}