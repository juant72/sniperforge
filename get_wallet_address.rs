use solana_sdk::signer::{keypair::Keypair, Signer};
use std::fs;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Leer el archivo de wallet simple
    let wallet_content = fs::read_to_string("test-wallet-new.json")?;
    let keypair_data: Vec<u8> = serde_json::from_str(&wallet_content)?;

    // Crear keypair desde los datos
    let keypair = Keypair::from_bytes(&keypair_data)?;

    println!("🔍 Tu dirección pública de wallet:");
    println!("{}", keypair.pubkey());

    Ok(())
}
