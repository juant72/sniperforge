//! Simple script to create a test wallet for DevNet
use solana_sdk::signature::{Keypair, Signer};

fn main() {
    // Generate a new keypair
    let keypair = Keypair::new();
    
    // Get the private key bytes and encode as base58
    let private_key_bytes = keypair.to_bytes();
    let private_key_base58 = bs58::encode(&private_key_bytes).into_string();
    
    println!("✅ Generated DevNet test wallet:");
    println!("📍 Public key: {}", keypair.pubkey());
    println!("🔑 Private key (base58): {}", private_key_base58);
    println!();
    println!("Add this to your .env file:");
    println!("SOLANA_PRIVATE_KEY={}", private_key_base58);
    println!();
    println!("To request DevNet airdrop (if you have solana CLI):");
    println!("  solana airdrop 2 {} --url devnet", keypair.pubkey());
}
