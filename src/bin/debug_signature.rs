use ethers::prelude::*;
use ethers::utils::keccak256;
use std::str::FromStr;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Test data
    let voucher_id = "0x1582f594f01e816170c2c05bd2546539999ca1bd5313bdcf093379a473cdc8e1";
    let recipient = "0x742d35Cc6634C0532925a3b844Bc9e7595f8fA8e";
    let amount = "1000000000000000000"; // 1 ARB in wei
    let deadline = 1827329883u64;
    let contract_address = "0x66Eb0Aa46827e5F3fFcb6Dea23C309CB401690B6";
    let chain_id = 42161u64;
    
    // Parse values
    let voucher_id = H256::from_str(voucher_id)?;
    let recipient = Address::from_str(recipient)?;
    let amount = U256::from_dec_str(amount)?;
    let deadline = U256::from(deadline);
    let contract_address = Address::from_str(contract_address)?;
    let chain_id = U256::from(chain_id);
    
    println!("=== Debugging Signature Generation ===");
    println!("Voucher ID: 0x{}", hex::encode(voucher_id));
    println!("Recipient: 0x{}", hex::encode(recipient));
    println!("Amount: {}", amount);
    println!("Deadline: {}", deadline);
    println!("Contract: 0x{}", hex::encode(contract_address));
    println!("Chain ID: {}", chain_id);
    
    // Test abi.encodePacked - WITHOUT amount (as per contract)
    let mut encoded = Vec::new();
    encoded.extend_from_slice(voucher_id.as_bytes());
    println!("\nAfter voucher_id: {} bytes", encoded.len());
    
    encoded.extend_from_slice(recipient.as_bytes());
    println!("After recipient: {} bytes", encoded.len());
    
    let deadline_bytes: [u8; 32] = deadline.into();
    encoded.extend_from_slice(&deadline_bytes);
    println!("After deadline: {} bytes", encoded.len());
    println!("Deadline bytes: 0x{}", hex::encode(&deadline_bytes));
    
    encoded.extend_from_slice(contract_address.as_bytes());
    println!("After contract: {} bytes", encoded.len());
    
    let chain_id_bytes: [u8; 32] = chain_id.into();
    encoded.extend_from_slice(&chain_id_bytes);
    println!("After chain_id: {} bytes", encoded.len());
    println!("Chain ID bytes: 0x{}", hex::encode(&chain_id_bytes));
    
    println!("\nTotal encoded length: {} bytes", encoded.len());
    println!("Encoded data: 0x{}", hex::encode(&encoded));
    
    let message_hash = keccak256(&encoded);
    println!("\nMessage hash: 0x{}", hex::encode(message_hash));
    
    // Ethereum signed message
    let eth_message = format!("\x19Ethereum Signed Message:\n32");
    let prefixed_hash = keccak256([eth_message.as_bytes(), &message_hash].concat());
    println!("\nPrefixed hash for signing: 0x{}", hex::encode(prefixed_hash));
    
    // Also test with ethers abi encoding
    println!("\n=== Testing with ethers ABI encoding ===");
    let encoded_ethers = ethers::abi::encode(&[
        ethers::abi::Token::FixedBytes(voucher_id.as_bytes().to_vec()),
        ethers::abi::Token::Address(recipient),
        ethers::abi::Token::Uint(deadline),
        ethers::abi::Token::Address(contract_address),
        ethers::abi::Token::Uint(chain_id),
    ]);
    println!("Ethers encoded length: {} bytes", encoded_ethers.len());
    println!("Ethers encoded: 0x{}", hex::encode(&encoded_ethers));
    
    Ok(())
}