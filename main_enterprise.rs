// 🏛️ ENTERPRISE ARBITRAGE SYSTEM - BINANCE LABS DEMO
// Sistema empresarial funcional de arbitraje para demostración

use std::env;
use std::io::{self, Write};
use anyhow::Result;
use tracing::{info, warn, error, Level};
use tracing_subscriber;

mod types;
mod price_feeds;
mod pool_validator;
mod jupiter_api;
mod calculations;
mod enterprise_pool_discovery;
mod enterprise_opportunity_engine;
mod enterprise_arbitrage_executor;
mod enterprise_arbitrage_system;

use enterprise_arbitrage_system::EnterpriseArbitrageSystem;

#[tokio::main]
async fn main() -> Result<()> {
    // Initialize enterprise logging
    tracing_subscriber::fmt()
        .with_max_level(Level::INFO)
        .with_target(false)
        .init();
    
    print_enterprise_banner();
    
    // Get configuration from environment or user input
    let config = get_enterprise_configuration().await?;
    
    // Initialize enterprise system
    let mut arbitrage_system = EnterpriseArbitrageSystem::new_binance_labs_demo(
        config.rpc_url,
        config.private_key,
        config.simulation_mode,
    ).await?;
    
    info!("🎯 ENTERPRISE SYSTEM READY FOR BINANCE LABS DEMONSTRATION");
    info!("💡 Press Ctrl+C to stop the system gracefully");
    
    // Setup graceful shutdown
    let running = std::sync::Arc::new(std::sync::atomic::AtomicBool::new(true));
    let r = running.clone();
    ctrlc::set_handler(move || {
        warn!("🛑 SHUTDOWN SIGNAL RECEIVED");
        r.store(false, std::sync::atomic::Ordering::SeqCst);
    })?;
    
    // Run enterprise arbitrage demonstration
    if let Err(e) = arbitrage_system.run_enterprise_arbitrage_demo().await {
        error!("🚨 ENTERPRISE SYSTEM ERROR: {}", e);
    }
    
    info!("✅ ENTERPRISE ARBITRAGE SYSTEM SHUTDOWN COMPLETE");
    Ok(())
}

struct EnterpriseConfig {
    rpc_url: String,
    private_key: String,
    simulation_mode: bool,
}

async fn get_enterprise_configuration() -> Result<EnterpriseConfig> {
    info!("🔧 ENTERPRISE CONFIGURATION SETUP");
    
    // Check for environment variables first
    if let (Ok(rpc_url), Ok(private_key)) = (env::var("SOLANA_RPC_URL"), env::var("SOLANA_PRIVATE_KEY")) {
        info!("✅ Using environment configuration");
        return Ok(EnterpriseConfig {
            rpc_url,
            private_key,
            simulation_mode: env::var("SIMULATION_MODE").unwrap_or_default() == "true",
        });
    }
    
    // Interactive configuration
    info!("🎯 INTERACTIVE ENTERPRISE SETUP");
    
    print!("🌐 Enter Solana RPC URL (or press Enter for devnet): ");
    io::stdout().flush()?;
    let mut rpc_input = String::new();
    io::stdin().read_line(&mut rpc_input)?;
    let rpc_url = if rpc_input.trim().is_empty() {
        "https://api.devnet.solana.com".to_string()
    } else {
        rpc_input.trim().to_string()
    };
    
    print!("🔑 Enter private key (or press Enter for simulation): ");
    io::stdout().flush()?;
    let mut key_input = String::new();
    io::stdin().read_line(&mut key_input)?;
    let private_key = key_input.trim().to_string();
    
    let simulation_mode = private_key.is_empty();
    if simulation_mode {
        warn!("⚠️ Running in SIMULATION MODE - No real transactions");
    }
    
    Ok(EnterpriseConfig {
        rpc_url,
        private_key: if simulation_mode { "demo_key".to_string() } else { private_key },
        simulation_mode,
    })
}

fn print_enterprise_banner() {
    println!();
    println!("🏛️ ═══════════════════════════════════════════════════════════════");
    println!("🎯                 ENTERPRISE ARBITRAGE SYSTEM                   ");
    println!("🚀              BINANCE LABS DEMONSTRATION READY                 ");
    println!("🏛️ ═══════════════════════════════════════════════════════════════");
    println!();
    println!("✅ ENTERPRISE FEATURES:");
    println!("   🔍 Multi-DEX Pool Discovery (Raydium, Orca, Whirlpool, Meteora)");
    println!("   💎 Advanced Opportunity Detection (Direct + Triangle Arbitrage)");
    println!("   🚀 Real Transaction Execution with Risk Management");
    println!("   📊 Enterprise-Grade Monitoring and Metrics");
    println!("   🛡️ Military-Precision Risk Controls");
    println!("   🏛️ Institutional Quality Architecture");
    println!();
    println!("🎯 DEMO CAPABILITIES:");
    println!("   ✅ Functional Arbitrage Detection");
    println!("   ✅ Real Pool Data Analysis");
    println!("   ✅ Live Opportunity Calculation");
    println!("   ✅ Transaction Simulation & Execution");
    println!("   ✅ Performance Metrics & Reporting");
    println!();
    println!("🏛️ ═══════════════════════════════════════════════════════════════");
    println!();
}
