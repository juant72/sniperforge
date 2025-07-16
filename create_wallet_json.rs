use solana_sdk::signature::{Keypair, Signer};
use std::str::FromStr;
use bs58;

fn main() {
    let private_key_base58 = "ovG9FrqxHYQ6hpPUFCKA9f1r4n8PEqBj19vqBoWagX9MdVzAj6Aa9jfQ9uY8YTARExp6n2WZMbGFrxH3iad5hE8";
    
    // Decode base58 private key
    let keypair_bytes = bs58::decode(private_key_base58).into_vec().unwrap();
    
    // Create keypair
    let keypair = Keypair::from_bytes(&keypair_bytes).unwrap();
    
    println!("Public key: {}", keypair.pubkey());
    println!("Private key bytes: {:?}", keypair_bytes);
    
    // Save to JSON file
    std::fs::write("test-arbitrage-wallet.json", format!("{:?}", keypair_bytes)).unwrap();
    println!("✅ Wallet saved to test-arbitrage-wallet.json");
}
