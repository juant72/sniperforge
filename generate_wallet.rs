use solana_sdk::signature::{Keypair, Signer};
use std::fs;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Generate a new keypair
    let keypair = Keypair::new();
    
    // Get the wallet data as bytes
    let wallet_bytes = keypair.to_bytes();
    
    // Convert to JSON format that solana CLI expects
    let wallet_json = serde_json::to_string_pretty(&wallet_bytes.to_vec())?;
    
    // Write to file
    fs::write("test-wallet.json", wallet_json)?;
    
    println!("✅ Generated test wallet: test-wallet.json");
    println!("📍 Public key: {}", keypair.pubkey());
    println!("💡 Use this wallet for DevNet testing");
    println!();
    println!("To request an airdrop:");
    println!("  solana airdrop 2 {} --url devnet", keypair.pubkey());
    
    Ok(())
}
