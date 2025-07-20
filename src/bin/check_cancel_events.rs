use ethers::prelude::*;
use ethers::abi::RawLog;
use ethers::contract::EthEvent;

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

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let provider = Provider::<Http>::try_from("https://arb1.arbitrum.io/rpc")?;
    let voucher_contract = "0x66Eb0Aa46827e5F3fFcb6Dea23C309CB401690B6".parse::<Address>()?;
    
    // Get current block
    let current_block = provider.get_block_number().await?.as_u64();
    println!("Current block: {}", current_block);
    
    // Look for events in the last 1000 blocks
    let from_block = current_block.saturating_sub(1000);
    
    println!("Searching for VoucherCancelled events from block {} to {}", from_block, current_block);
    
    // Create filter for VoucherCancelled events
    let event_signature = VoucherCancelled::signature();
    println!("VoucherCancelled event signature: {:?}", event_signature);
    
    let filter = Filter::new()
        .address(voucher_contract)
        .topic0(event_signature)
        .from_block(from_block)
        .to_block(current_block);
    
    let logs = provider.get_logs(&filter).await?;
    println!("\nFound {} VoucherCancelled events", logs.len());
    
    for log in logs.iter() {
        let raw_log = RawLog {
            topics: log.topics.clone(),
            data: log.data.to_vec(),
        };
        
        if let Ok(event) = <VoucherCancelled as EthEvent>::decode_log(&raw_log) {
            println!("\nVoucherCancelled event:");
            println!("  Voucher ID: 0x{}", hex::encode(event.voucher_id));
            println!("  Creator: {:?}", event.creator);
            println!("  Amount: {}", event.amount);
            println!("  Block: {:?}", log.block_number);
            println!("  Tx: {:?}", log.transaction_hash);
        }
    }
    
    // Also check all events from the contract
    println!("\n\nChecking ALL events from the contract in the last 100 blocks:");
    let all_filter = Filter::new()
        .address(voucher_contract)
        .from_block(current_block.saturating_sub(100))
        .to_block(current_block);
    
    let all_logs = provider.get_logs(&all_filter).await?;
    println!("Found {} total events", all_logs.len());
    
    for log in all_logs.iter() {
        if !log.topics.is_empty() {
            println!("\nEvent topic: {:?}", log.topics[0]);
            if log.topics[0] == event_signature {
                println!("  -> This is a VoucherCancelled event!");
            }
        }
    }
    
    Ok(())
}