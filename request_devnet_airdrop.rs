//! Request DevNet airdrop using Solana RPC
use solana_client::rpc_client::RpcClient;
use solana_sdk::{
    commitment_config::CommitmentConfig,
    pubkey::Pubkey,
    signature::{Keypair, Signer},
};
use std::str::FromStr;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    dotenv::dotenv().ok();

    // Load wallet from environment
    let private_key_str = std::env::var("SOLANA_PRIVATE_KEY")
        .expect("SOLANA_PRIVATE_KEY environment variable not set");

    let private_key_bytes = bs58::decode(&private_key_str)
        .into_vec()
        .expect("Failed to decode private key");

    let keypair =
        Keypair::from_bytes(&private_key_bytes).expect("Failed to create keypair from bytes");

    println!("🔑 Wallet: {}", keypair.pubkey());

    // Connect to DevNet
    let rpc_client = RpcClient::new_with_commitment(
        "https://api.devnet.solana.com".to_string(),
        CommitmentConfig::confirmed(),
    );

    // Check current balance
    let balance = rpc_client.get_balance(&keypair.pubkey())?;
    println!(
        "💰 Current balance: {} lamports ({:.6} SOL)",
        balance,
        balance as f64 / 1_000_000_000.0
    );

    if balance < 100_000_000 {
        // Less than 0.1 SOL
        println!("🪂 Requesting DevNet airdrop...");

        // Request airdrop (2 SOL = 2_000_000_000 lamports)
        let airdrop_amount = 2_000_000_000;
        match rpc_client.request_airdrop(&keypair.pubkey(), airdrop_amount) {
            Ok(signature) => {
                println!("✅ Airdrop requested: {}", signature);
                println!("⏳ Waiting for confirmation...");

                // Wait for confirmation
                let mut retries = 0;
                loop {
                    std::thread::sleep(std::time::Duration::from_secs(2));
                    let new_balance = rpc_client.get_balance(&keypair.pubkey())?;

                    if new_balance > balance {
                        println!("✅ Airdrop confirmed!");
                        println!(
                            "💰 New balance: {} lamports ({:.6} SOL)",
                            new_balance,
                            new_balance as f64 / 1_000_000_000.0
                        );
                        break;
                    }

                    retries += 1;
                    if retries > 30 {
                        println!("⏰ Timeout waiting for airdrop confirmation");
                        break;
                    }
                }
            }
            Err(e) => {
                println!("❌ Airdrop failed: {}", e);
                println!("💡 You may have reached the daily airdrop limit");
            }
        }
    } else {
        println!("✅ Wallet has sufficient balance for testing");
    }

    Ok(())
}
