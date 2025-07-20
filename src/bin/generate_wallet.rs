use ethers::prelude::*;
use rand::thread_rng;

fn main() {
    let wallet = LocalWallet::new(&mut thread_rng());
    let address = format!("{:?}", wallet.address());
    let private_key = format!("0x{}", hex::encode(wallet.signer().to_bytes()));
    
    println!("🔐 NBGN Backend Wallet Generated\n");
    println!("PUBLIC Address (share this): {}", address);
    println!("=====================================\n");
    
    println!("⚠️  CRITICAL SECURITY WARNINGS:");
    println!("1. Save the private key below in a password manager IMMEDIATELY");
    println!("2. Add to .env file (already in .gitignore)");
    println!("3. NEVER commit, log, or share the private key");
    println!("4. Use secret management service in production\n");
    
    println!("PRIVATE Key (KEEP SECRET):");
    println!("{}\n", private_key);
    
    println!("📋 Next Steps:");
    println!("1. Add to .env: BACKEND_PRIVATE_KEY={}", private_key);
    println!("2. Send PUBLIC address to Jenny: {}", address);
    println!("3. Jenny calls updateBackendSigner(\"{}\")", address);
    println!("4. Set up monitoring alerts for this wallet");
    println!("5. Review SECURITY.md for best practices");
}