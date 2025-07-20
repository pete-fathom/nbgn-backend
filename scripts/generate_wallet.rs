use ethers::prelude::*;

fn main() {
    let wallet = LocalWallet::new(&mut rand::thread_rng());
    let address = format!("{:?}", wallet.address());
    let private_key = format!("0x{}", hex::encode(wallet.signer().to_bytes()));
    
    println!("🔐 New Backend Wallet Generated:");
    println!("Address: {}", address);
    println!("Private Key: {}", private_key);
    println!("\n⚠️  IMPORTANT:");
    println!("1. Save the private key in your .env file as BACKEND_PRIVATE_KEY");
    println!("2. Send this address to Jenny: {}", address);
    println!("3. Jenny needs to call updateBackendSigner({}) on the contract", address);
    println!("4. NEVER share your private key with anyone!");
}