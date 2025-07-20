use ethers::prelude::*;
use ethers::abi::Abi;
use std::sync::Arc;

// Define the contract events
#[derive(Clone, Debug, EthEvent)]
pub struct Minted {
    #[ethevent(indexed)]
    pub user: Address,
    pub eure_amount: U256,
    pub nbgn_amount: U256,
}

#[derive(Clone, Debug, EthEvent)]
pub struct Redeemed {
    #[ethevent(indexed)]
    pub user: Address,
    pub nbgn_amount: U256,
    pub eure_amount: U256,
}

#[derive(Clone, Debug, EthEvent)]
pub struct Burned {
    #[ethevent(indexed)]
    pub user: Address,
    pub nbgn_amount: U256,
    pub refund_amount: U256,
}

// ABI for the NBGN contract (simplified for events)
pub const NBGN_ABI: &str = r#"[
    {
        "type": "event",
        "name": "Minted",
        "anonymous": false,
        "inputs": [
            {"name": "user", "type": "address", "indexed": true},
            {"name": "eureAmount", "type": "uint256", "indexed": false},
            {"name": "nbgnAmount", "type": "uint256", "indexed": false}
        ]
    },
    {
        "type": "event",
        "name": "Redeemed",
        "anonymous": false,
        "inputs": [
            {"name": "user", "type": "address", "indexed": true},
            {"name": "nbgnAmount", "type": "uint256", "indexed": false},
            {"name": "eureAmount", "type": "uint256", "indexed": false}
        ]
    },
    {
        "type": "event",
        "name": "Burned",
        "anonymous": false,
        "inputs": [
            {"name": "user", "type": "address", "indexed": true},
            {"name": "nbgnAmount", "type": "uint256", "indexed": false},
            {"name": "refundAmount", "type": "uint256", "indexed": false}
        ]
    },
    {
        "type": "function",
        "name": "totalSupply",
        "inputs": [],
        "outputs": [{"name": "", "type": "uint256"}],
        "stateMutability": "view"
    },
    {
        "type": "function",
        "name": "reserves",
        "inputs": [],
        "outputs": [{"name": "", "type": "uint256"}],
        "stateMutability": "view"
    }
]"#;

pub type NBGNContract = Contract<Provider<Http>>;

pub fn get_contract(
    address: Address,
    provider: Arc<Provider<Http>>,
) -> Result<NBGNContract, Box<dyn std::error::Error>> {
    let abi: Abi = serde_json::from_str(NBGN_ABI)?;
    Ok(Contract::new(address, abi, provider))
}