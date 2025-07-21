use solana_sdk::signature::{Keypair, Signer};
use std::fs;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Read the CLI wallet JSON file
    let wallet_data = fs::read_to_string("test-cli-arbitrage.json")?;
    let wallet_bytes: Vec<u8> = serde_json::from_str(&wallet_data)?;

    // Create keypair from bytes
    let keypair = Keypair::from_bytes(&wallet_bytes)?;

    // Convert to base58 string (first 32 bytes are the private key)
    let private_key_base58 = bs58::encode(&wallet_bytes[0..32]).into_string();

    println!("✅ Wallet CLI convertida a formato base58:");
    println!("🔑 Public key: {}", keypair.pubkey());
    println!("🔐 Private key (base58): {}", private_key_base58);
    println!();
    println!("Para usar esta wallet con arbitraje real, exporta:");
    println!("set SOLANA_PRIVATE_KEY={}", private_key_base58);
    println!();
    println!("O en PowerShell:");
    println!("$env:SOLANA_PRIVATE_KEY=\"{}\"", private_key_base58);

    Ok(())
}
