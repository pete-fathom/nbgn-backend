use ethers::prelude::*;
use ethers::abi::{encode_packed, Token};
use ethers::utils::keccak256;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Jenny's exact test parameters
    let voucher_id = "0xadeea4c8e0c60f95c97fe102e11d8b1c5d1ddd9d58bbd63f65e45abbc0e3f98b";
    let recipient = "0x742d35cc6634c0532925a3b844bc9e7595f8fa8e";
    let deadline = 1753038976u64;
    let contract_address = "0x66Eb0Aa46827e5F3fFcb6Dea23C309CB401690B6";
    let chain_id = 42161u64;
    
    println!("=== Testing Jenny's Voucher Parameters ===");
    println!("VoucherId: {}", voucher_id);
    println!("Recipient: {}", recipient);
    println!("Deadline: {}", deadline);
    println!("Contract: {}", contract_address);
    println!("ChainId: {}", chain_id);
    
    // Step 1: Pack the parameters using ethers encode_packed
    let packed = encode_packed(&[
        Token::FixedBytes(hex::decode(&voucher_id[2..])?), // Remove 0x prefix
        Token::Address(recipient.parse()?),
        Token::Uint(U256::from(deadline)),
        Token::Address(contract_address.parse()?),
        Token::Uint(U256::from(chain_id)),
    ])?;
    
    println!("\nPacked data: 0x{}", hex::encode(&packed));
    println!("Packed length: {} bytes", packed.len());
    
    // Step 2: Hash it
    let message_hash = keccak256(&packed);
    let message_hash_hex = hex::encode(message_hash);
    
    println!("\nMessage hash: 0x{}", message_hash_hex);
    println!("Expected hash: 0x21dd818649439786256b1f46b86215086542b3f5dadb9fd3a4dd1eb0dd5543ca");
    
    // Step 3: Verify it matches Jenny's hash
    if message_hash_hex == "21dd818649439786256b1f46b86215086542b3f5dadb9fd3a4dd1eb0dd5543ca" {
        println!("✅ Hash matches Jenny's expected hash!");
    } else {
        println!("❌ Hash DOES NOT match!");
        return Err("Hash mismatch".into());
    }
    
    // Step 4: Sign with wallet
    let private_key = std::env::var("BACKEND_PRIVATE_KEY")
        .expect("BACKEND_PRIVATE_KEY not set in environment");
    let wallet = private_key.parse::<LocalWallet>()?;
    
    println!("\nBackend signer address: 0x{}", hex::encode(wallet.address()));
    
    // Sign the message (this adds the Ethereum prefix automatically)
    let signature = wallet.sign_message(&message_hash).await?;
    
    println!("\n=== SIGNATURE ===");
    println!("0x{}", hex::encode(signature.to_vec()));
    
    // Verify the signature
    let eth_message = format!("\x19Ethereum Signed Message:\n32");
    let prefixed_hash = keccak256([eth_message.as_bytes(), &message_hash].concat());
    let recovered = signature.recover(H256::from(prefixed_hash))?;
    println!("\nRecovered address: 0x{}", hex::encode(recovered));
    
    if recovered == wallet.address() {
        println!("✅ Signature verification successful!");
    } else {
        println!("❌ Signature verification failed!");
    }
    
    Ok(())
}