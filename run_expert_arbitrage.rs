// 🚀 EXPERT ARBITRAGE RUNNER
// Integrates all expert improvements for ultra-fast profitable arbitrage

use std::env;
use anyhow::Result;
use tracing::{info, warn};
use tracing_subscriber;

// Load the modules directly since they're in the same directory
#[path = "military_arbitrage_system.rs"]
mod military_arbitrage_system;

#[path = "expert_speed_engine.rs"]
mod expert_speed_engine;

#[path = "expert_price_feeds.rs"]
mod expert_price_feeds;

use military_arbitrage_system::MilitaryArbitrageSystem;

#[tokio::main]
async fn main() -> Result<()> {
    // Initialize logging
    tracing_subscriber::fmt()
        .with_env_filter("info")
        .with_target(false)
        .with_thread_ids(false)
        .with_file(false)
        .with_line_number(false)
        .compact()
        .init();

    println!();
    println!("╔═══════════════════════════════════════════════════════════════════════════════╗");
    println!("║                          🚀 EXPERT ARBITRAGE SYSTEM 🚀                       ║");
    println!("║                   Complete Expert Roadmap Implementation                     ║");
    println!("║                                                                               ║");
    println!("║  ✅ Expert Mathematical Foundation (AMM calculations)                        ║");
    println!("║  ✅ Mainnet Production Deployment (Real pools with $10M+ TVL)               ║");
    println!("║  ✅ Speed Optimization Module (<200ms execution target)                     ║");
    println!("║  ✅ Real-Time Price Feeds (<400ms data refresh)                             ║");
    println!("║  ✅ Ultra-Fast Parallel Processing (20 pools simultaneously)               ║");
    println!("║  ✅ Priority Fee Optimization (2M lamports for speed)                       ║");
    println!("║                                                                               ║");
    println!("║              🎯 TARGET: 25x faster execution for profitable trades           ║");
    println!("╚═══════════════════════════════════════════════════════════════════════════════╝");
    println!();

    // Validate environment
    info!("🔍 EXPERT SETUP: Validating environment...");
    
    // Check for premium RPC
    if let Ok(helius_key) = env::var("HELIUS_API_KEY") {
        info!("✅ HELIUS PREMIUM RPC: Active (Key: {}...)", &helius_key[..8]);
    } else {
        warn!("⚠️  HELIUS_API_KEY not found - using standard RPC (may impact speed)");
    }

    // Check wallet
    let wallet_path = "mainnet_wallet.json";
    if std::path::Path::new(wallet_path).exists() {
        info!("✅ MAINNET WALLET: Found and ready");
    } else {
        return Err(anyhow::anyhow!("❌ MAINNET WALLET: {} not found", wallet_path));
    }

    info!("🚀 EXPERT INITIALIZATION: Starting all systems...");

    // Initialize expert arbitrage system
    let mut system = MilitaryArbitrageSystem::new().await?;
    
    info!("✅ EXPERT SYSTEM: All modules loaded successfully");
    info!("🎯 EXPERT MODE: Starting ultra-fast arbitrage execution...");

    // Run expert arbitrage with all improvements
    system.run_expert_arbitrage().await?;

    Ok(())
}
