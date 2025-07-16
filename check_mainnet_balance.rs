use anyhow::Result;
use solana_client::rpc_client::RpcClient;
use solana_sdk::{
    commitment_config::CommitmentConfig,
    signature::Keypair,
    signer::Signer,
};
use std::fs;
use tracing::{info, warn, error};

const MAINNET_RPC: &str = "https://api.mainnet-beta.solana.com";

#[tokio::main]
async fn main() -> Result<()> {
    tracing_subscriber::fmt::init();
    
    info!("🔍 === QUICK MAINNET BALANCE CHECK ===");
    
    let wallet_path = "mainnet-arbitrage-wallet.json";
    
    if !std::path::Path::new(wallet_path).exists() {
        error!("❌ MainNet wallet not found: {}", wallet_path);
        return Ok(());
    }
    
    // Load wallet
    let wallet_data = fs::read_to_string(wallet_path)?;
    let secret_key: Vec<u8> = serde_json::from_str(&wallet_data)?;
    let wallet = Keypair::from_bytes(&secret_key)?;
    let wallet_address = wallet.pubkey();
    
    info!("🔑 Wallet: {}", wallet_address);
    
    // Check balance with timeout
    let client = RpcClient::new_with_commitment(MAINNET_RPC.to_string(), CommitmentConfig::confirmed());
    
    info!("🌐 Connecting to MainNet...");
    
    match client.get_balance(&wallet_address) {
        Ok(balance_lamports) => {
            let balance_sol = balance_lamports as f64 / 1_000_000_000.0;
            info!("💰 MainNet Balance: {:.9} SOL", balance_sol);
            
            if balance_sol >= 0.001 {
                info!("✅ Ready for ultra-conservative arbitrage");
                info!("   🎯 Execute: cargo run --bin phase3_mainnet_simple");
            } else {
                warn!("⚠️  Low balance: {:.9} SOL", balance_sol);
            }
        }
        Err(e) => {
            error!("❌ RPC Error: {}", e);
            error!("   🌐 Possible issues:");
            error!("      - RPC rate limits");
            error!("      - Network connectivity");
            error!("      - Wallet not funded yet");
        }
    }
    
    Ok(())
}
