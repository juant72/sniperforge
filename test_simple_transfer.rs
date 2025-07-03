// Test de transferencia simple en DevNet para verificar que todo funciona
// Este script hace una transferencia básica de SOL sin usar Jupiter

use anyhow::Result;
use solana_client::rpc_client::RpcClient;
use solana_sdk::{
    commitment_config::CommitmentConfig,
    signature::{Keypair, Signer},
    system_instruction,
    transaction::Transaction,
    native_token::LAMPORTS_PER_SOL,
};
use tracing::{info, error, debug};

#[tokio::main]
async fn main() -> Result<()> {
    // Load environment variables
    dotenv::dotenv().ok();
    
    tracing_subscriber::fmt()
        .with_max_level(tracing::Level::INFO)
        .init();

    info!("🔧 Simple SOL Transfer Test for DevNet");
    info!("======================================");

    // Load wallet from environment variable
    let private_key = std::env::var("SOLANA_PRIVATE_KEY")
        .expect("SOLANA_PRIVATE_KEY environment variable not set");
    let sender_keypair = Keypair::from_base58_string(&private_key);
    
    info!("📋 Wallet loaded: {}", sender_keypair.pubkey());
    
    // Connect to DevNet
    let rpc_client = RpcClient::new_with_commitment(
        "https://api.devnet.solana.com".to_string(),
        CommitmentConfig::confirmed(),
    );
    
    // Check balance
    let balance = rpc_client.get_balance(&sender_keypair.pubkey())?;
    let balance_sol = balance as f64 / LAMPORTS_PER_SOL as f64;
    info!("💰 Current balance: {} SOL", balance_sol);
    
    if balance_sol < 0.01 {
        error!("❌ Insufficient balance for test");
        return Ok(());
    }
    
    // Create a simple transfer to ourselves (this should always work)
    let transfer_amount = 1_000; // 0.000001 SOL in lamports  
    let destination = sender_keypair.pubkey(); // Send to ourselves
    
    info!("🔄 Creating transfer instruction...");
    info!("   Amount: {} lamports ({} SOL)", transfer_amount, transfer_amount as f64 / LAMPORTS_PER_SOL as f64);
    
    // Create transfer instruction
    let transfer_instruction = system_instruction::transfer(
        &sender_keypair.pubkey(),
        &destination,
        transfer_amount,
    );
    
    // Get recent blockhash
    let recent_blockhash = rpc_client.get_latest_blockhash()?;
    info!("🔗 Recent blockhash obtained");
    
    // Create transaction
    let transaction = Transaction::new_signed_with_payer(
        &[transfer_instruction],
        Some(&sender_keypair.pubkey()),
        &[&sender_keypair],
        recent_blockhash,
    );
    
    info!("📝 Transaction created and signed");
    
    // Send the transaction quickly (no confirmation wait)
    info!("🚀 Sending transaction to DevNet...");
    match rpc_client.send_transaction(&transaction) {
        Ok(signature) => {
            info!("✅ TRANSACTION SENT!");
            info!("🎯 Signature: {}", signature);
            info!("🔍 View on explorer: https://explorer.solana.com/tx/{}?cluster=devnet", signature);
            info!("⏳ Transaction submitted - check explorer for confirmation");
        }
        Err(e) => {
            error!("❌ SEND FAILED: {}", e);
        }
    }
    
    Ok(())
}
