use anyhow::Result;
use solana_client::rpc_client::RpcClient;
use solana_sdk::{commitment_config::CommitmentConfig, signature::Keypair, signer::Signer};
use std::fs;
use std::time::Duration;
use tracing::{info, error};

#[tokio::main]
async fn main() -> Result<()> {
    tracing_subscriber::fmt::init();
    
    info!("🌐 === MAINNET CONNECTIVITY TEST ===");
    
    // Test wallet loading
    let wallet_path = "mainnet-arbitrage-wallet.json";
    if !std::path::Path::new(wallet_path).exists() {
        error!("❌ Wallet not found: {}", wallet_path);
        return Ok(());
    }
    
    let wallet_data = fs::read_to_string(wallet_path)?;
    let secret_key: Vec<u8> = serde_json::from_str(&wallet_data)?;
    let wallet = Keypair::from_bytes(&secret_key)?;
    
    info!("✅ Wallet loaded: {}", wallet.pubkey());
    
    // Test RPC connectivity with timeout
    let rpcs = vec![
        "https://api.mainnet-beta.solana.com",
        "https://solana-api.projectserum.com", 
        "https://rpc.ankr.com/solana",
    ];
    
    for (i, rpc_url) in rpcs.iter().enumerate() {
        info!("🔌 Testing RPC {}: {}", i + 1, rpc_url);
        
        let client = RpcClient::new_with_timeout_and_commitment(
            rpc_url.to_string(),
            Duration::from_secs(10), // 10 second timeout
            CommitmentConfig::confirmed(),
        );
        
        // Test basic connectivity
        match client.get_health() {
            Ok(_) => {
                info!("  ✅ Health check: OK");
                
                // Test balance check
                match client.get_balance(&wallet.pubkey()) {
                    Ok(balance) => {
                        let sol_balance = balance as f64 / 1_000_000_000.0;
                        info!("  💰 Balance: {:.9} SOL", sol_balance);
                        
                        if sol_balance >= 0.001 {
                            info!("  🎯 Ready for arbitrage with this RPC!");
                            break;
                        } else {
                            error!("  ❌ Insufficient balance: {:.9} SOL", sol_balance);
                        }
                    }
                    Err(e) => {
                        error!("  ❌ Balance check failed: {}", e);
                    }
                }
            }
            Err(e) => {
                error!("  ❌ Health check failed: {}", e);
            }
        }
    }
    
    info!("🏁 Connectivity test completed");
    Ok(())
}
