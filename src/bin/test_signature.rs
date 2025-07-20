use ethers::prelude::*;
use ethers::utils::keccak256;
use hex;
use std::time::{SystemTime, UNIX_EPOCH};
use std::str::FromStr;

fn main() {
    // Test parameters from Jenny
    let voucher_id = H256::from_str("0x1234567890123456789012345678901234567890123456789012345678901234").unwrap();
    let recipient = "0x742d35Cc6634C0532925a3b844Bc9e7595f8fA8e".parse::<Address>().unwrap();
    let current_timestamp = SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_secs();
    let deadline = U256::from(current_timestamp + 3600); // Current + 1 hour
    let contract_address = "0x66Eb0Aa46827e5F3fFcb6Dea23C309CB401690B6".parse::<Address>().unwrap();
    let chain_id = U256::from(42161u64); // Arbitrum One
    
    // Create message exactly as contract expects
    // keccak256(abi.encodePacked(voucherId, recipient, deadline, address(this), block.chainid))
    let mut encoded = Vec::new();
    encoded.extend_from_slice(voucher_id.as_bytes()); // bytes32
    encoded.extend_from_slice(recipient.as_bytes()); // address (20 bytes)
    encoded.extend_from_slice(&[0u8; 12]); // padding for uint256
    let deadline_bytes: [u8; 32] = deadline.into();
    encoded.extend_from_slice(&deadline_bytes[12..]); // uint256 (last 20 bytes)
    encoded.extend_from_slice(contract_address.as_bytes()); // address (20 bytes)
    encoded.extend_from_slice(&[0u8; 12]); // padding for uint256
    let chain_id_bytes: [u8; 32] = chain_id.into();
    encoded.extend_from_slice(&chain_id_bytes[12..]); // uint256 (last 20 bytes)
    
    let message_hash = keccak256(&encoded);
    
    // Load private key from environment variable
    let private_key = std::env::var("BACKEND_PRIVATE_KEY")
        .expect("BACKEND_PRIVATE_KEY not set in environment");
    let wallet: LocalWallet = private_key.parse().unwrap();
    
    // Create Ethereum Signed Message format
    // This adds "\x19Ethereum Signed Message:\n32" prefix
    let eth_message = format!("\x19Ethereum Signed Message:\n32");
    let prefixed_hash = keccak256([eth_message.as_bytes(), &message_hash].concat());
    
    // Sign the prefixed hash
    let signature = wallet.sign_hash(H256::from(prefixed_hash)).unwrap();
    
    println!("🧪 Test Signature Generation for Jenny\n");
    println!("Input Parameters:");
    println!("  Voucher ID: 0x{}", hex::encode(voucher_id));
    println!("  Recipient: {:?}", recipient);
    println!("  Deadline: {}", deadline);
    println!("  Contract: {:?}", contract_address);
    println!("  Chain ID: {}", chain_id);
    println!("\nEncoded Data: 0x{}", hex::encode(&encoded));
    println!("Message Hash: 0x{}", hex::encode(message_hash));
    println!("Prefixed Hash (EthSignedMessage): 0x{}", hex::encode(prefixed_hash));
    println!("\n✅ Signature: 0x{}", hex::encode(signature.to_vec()));
    println!("\nSignature Components:");
    let r_bytes: [u8; 32] = signature.r.into();
    let s_bytes: [u8; 32] = signature.s.into();
    println!("  r: 0x{}", hex::encode(r_bytes));
    println!("  s: 0x{}", hex::encode(s_bytes));
    println!("  v: {}", signature.v);
    println!("\nThis signature can be used with claimVoucher()!");
}