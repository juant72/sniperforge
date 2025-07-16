use anyhow::Result;
use solana_client::rpc_client::RpcClient;
use solana_sdk::{
    commitment_config::CommitmentConfig,
    signature::{Keypair, read_keypair_file},
    signer::Signer,
};
use std::fs;
use tracing::{info, warn, error};

const MAINNET_RPC: &str = "https://api.mainnet-beta.solana.com";

#[tokio::main]
async fn main() -> Result<()> {
    tracing_subscriber::fmt::init();
    
    info!("🔑 === MAINNET WALLET SETUP ===");
    info!("   🎯 Objetivo: Crear/verificar wallet para MainNet");
    info!("   💰 Network: MAINNET-BETA (REAL MONEY)");
    
    let wallet_path = "mainnet-arbitrage-wallet.json";
    
    // 🔧 Crear nueva wallet o cargar existente
    let wallet = if std::path::Path::new(wallet_path).exists() {
        info!("📂 Loading existing MainNet wallet...");
        let wallet_data = fs::read_to_string(wallet_path)?;
        let secret_key: Vec<u8> = serde_json::from_str(&wallet_data)?;
        Keypair::from_bytes(&secret_key)?
    } else {
        info!("🆕 Creating new MainNet wallet...");
        let new_wallet = Keypair::new();
        let secret_key = new_wallet.to_bytes().to_vec();
        let wallet_json = serde_json::to_string_pretty(&secret_key)?;
        fs::write(wallet_path, wallet_json)?;
        info!("✅ New wallet saved to: {}", wallet_path);
        new_wallet
    };
    
    let wallet_address = wallet.pubkey();
    info!("🔑 Wallet Address: {}", wallet_address);
    
    // 📊 Verificar balance en MainNet
    let client = RpcClient::new_with_commitment(MAINNET_RPC.to_string(), CommitmentConfig::confirmed());
    
    match client.get_balance(&wallet_address) {
        Ok(balance_lamports) => {
            let balance_sol = balance_lamports as f64 / 1_000_000_000.0;
            info!("💰 Current MainNet Balance: {:.9} SOL", balance_sol);
            
            if balance_sol >= 0.01 {
                info!("✅ Balance sufficient for ultra-conservative testing");
                info!("   🎯 Ready for: cargo run --bin phase3_mainnet_ultra_conservative");
            } else if balance_sol > 0.0 {
                warn!("⚠️  Balance low but present: {:.9} SOL", balance_sol);
                warn!("   💰 Recommended minimum: 0.01 SOL");
                warn!("   🎯 Current allows testing with: {:.6} SOL", balance_sol * 0.8);
            } else {
                error!("❌ NO BALANCE DETECTED");
                error!("   📝 TRANSFER SOL TO MAINNET WALLET:");
                error!("   💳 Address: {}", wallet_address);
                error!("   💰 Recommended: 0.01-0.1 SOL for safe testing");
                error!("   🚨 REMEMBER: This is REAL MONEY on MainNet");
            }
        }
        Err(e) => {
            error!("❌ Failed to check balance: {}", e);
            error!("   🌐 This might be due to RPC limits or new account");
            error!("   💳 Wallet address: {}", wallet_address);
        }
    }
    
    // 📋 INSTRUCCIONES PARA USUARIO
    info!("📋 === NEXT STEPS ===");
    info!("   1. 💰 Transfer SOL to MainNet wallet:");
    info!("      Address: {}", wallet_address);
    info!("      Amount: 0.01-0.1 SOL (for safe testing)");
    info!("   ");
    info!("   2. 🚀 Run ultra-conservative arbitrage:");
    info!("      cargo run --bin phase3_mainnet_ultra_conservative");
    info!("   ");
    info!("   3. 🛡️ Safety features:");
    info!("      - Starts with 0.001 SOL (~$0.20)");
    info!("      - Max loss: 0.0005 SOL");
    info!("      - Uses proven 2C technique");
    
    warn!("⚠️  === IMPORTANT REMINDERS ===");
    warn!("   💸 MainNet uses REAL MONEY");
    warn!("   🔒 Keep your wallet file secure");
    warn!("   📊 Start with minimal amounts");
    warn!("   🎯 This is the same technique that made +0.012 SOL in DevNet");
    
    Ok(())
}
