use ethers::prelude::*;
use ethers::utils::keccak256;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Jenny's exact test parameters
    let voucher_id = H256::from_slice(&hex::decode("adeea4c8e0c60f95c97fe102e11d8b1c5d1ddd9d58bbd63f65e45abbc0e3f98b")?);
    let recipient = Address::from_slice(&hex::decode("742d35cc6634c0532925a3b844bc9e7595f8fa8e")?);
    let deadline = U256::from(1753038976u64);
    let contract_address = Address::from_slice(&hex::decode("66Eb0Aa46827e5F3fFcb6Dea23C309CB401690B6")?);
    let chain_id = U256::from(42161u64);
    
    println!("=== Manual Encoding (Backend Method) ===");
    
    // Manual encoding as our backend does
    let mut encoded = Vec::new();
    encoded.extend_from_slice(voucher_id.as_bytes()); // bytes32
    encoded.extend_from_slice(recipient.as_bytes()); // address (20 bytes)
    let deadline_bytes: [u8; 32] = deadline.into();
    encoded.extend_from_slice(&deadline_bytes); // uint256 (full 32 bytes)
    encoded.extend_from_slice(contract_address.as_bytes()); // address (20 bytes)
    let chain_id_bytes: [u8; 32] = chain_id.into();
    encoded.extend_from_slice(&chain_id_bytes); // uint256 (full 32 bytes)
    
    println!("Encoded data: 0x{}", hex::encode(&encoded));
    println!("Encoded length: {} bytes", encoded.len());
    
    let message_hash = keccak256(&encoded);
    let message_hash_hex = hex::encode(message_hash);
    
    println!("\nMessage hash: 0x{}", message_hash_hex);
    println!("Expected hash: 0x21dd818649439786256b1f46b86215086542b3f5dadb9fd3a4dd1eb0dd5543ca");
    
    if message_hash_hex == "21dd818649439786256b1f46b86215086542b3f5dadb9fd3a4dd1eb0dd5543ca" {
        println!("✅ Hash matches!");
        
        // Generate signature
        let private_key = std::env::var("BACKEND_PRIVATE_KEY")
            .expect("BACKEND_PRIVATE_KEY not set in environment");
        let wallet = private_key.parse::<LocalWallet>()?;
        
        println!("\nBackend signer address: 0x{}", hex::encode(wallet.address()));
        
        // Create Ethereum Signed Message format
        let eth_message = format!("\x19Ethereum Signed Message:\n32");
        let prefixed_hash = keccak256([eth_message.as_bytes(), &message_hash].concat());
        
        // Sign the prefixed hash
        let signature = wallet.sign_hash(H256::from(prefixed_hash))?;
        
        println!("\n=== SIGNATURE FOR JENNY ===");
        println!("0x{}", hex::encode(signature.to_vec()));
        
        // Verify
        let recovered = signature.recover(H256::from(prefixed_hash))?;
        if recovered == wallet.address() {
            println!("\n✅ Signature verification successful!");
        }
    } else {
        println!("❌ Hash DOES NOT match!");
    }
    
    Ok(())
}