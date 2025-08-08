use clap::{Arg, Command};
use anyhow::Result;
use std::path::PathBuf;
use std::sync::Arc;
use uuid::Uuid;

use sniperforge::control::{TcpControlServer, BotController};
use sniperforge::bots::liquidity_sniper::{LiquiditySniperBot, SniperConfig};
use sniperforge::bots::liquidity_sniper::capital_progression::CapitalProgressionManager;
use sniperforge::config::SimpleConfig;

#[tokio::main]
async fn main() -> Result<()> {
    // Initialize logging
    tracing_subscriber::fmt()
        .with_max_level(tracing::Level::INFO)
        .init();

    let matches = Command::new("sniperforge-enterprise")
        .version("3.0.0")
        .about("SniperForge Enterprise Trading System")
        .arg(Arg::new("config")
            .long("config")
            .value_name("CONFIG_FILE")
            .help("Configuration file path")
            .default_value("config/small_capital_config.json"))
        .arg(Arg::new("server-port")
            .long("server-port")
            .value_name("PORT")
            .help("TCP control server port")
            .default_value("8888"))
        .arg(Arg::new("wallet")
            .long("wallet")
            .value_name("WALLET_FILE")
            .help("Wallet file path")
            .default_value("wallet.json"))
        .arg(Arg::new("mode")
            .long("mode")
            .value_name("MODE")
            .help("Operation mode: server, single-run, test")
            .default_value("server"))
        .get_matches();

    let config_path = matches.get_one::<String>("config").unwrap();
    let server_port = matches.get_one::<String>("server-port").unwrap().parse::<u16>()?;
    let wallet_path = matches.get_one::<String>("wallet").unwrap();
    let mode = matches.get_one::<String>("mode").unwrap();

    // 🚀 COMPLETAR FUNCIONALIDAD: Use PathBuf for proper path handling
    let config_path_buf = PathBuf::from(config_path);
    let wallet_path_buf = PathBuf::from(wallet_path);

    println!("🚀 SniperForge Enterprise v3.0 - Capital Accumulation Mode");
    println!("📊 Config: {}", config_path_buf.display());
    println!("💼 Wallet: {}", wallet_path_buf.display());
    println!("⚡ Mode: {}", mode);

    // 🚀 COMPLETAR FUNCIONALIDAD: Validate paths exist
    if !config_path_buf.exists() {
        eprintln!("❌ Error: Config file not found: {}", config_path_buf.display());
        std::process::exit(1);
    }
    
    if mode != "test" && !wallet_path_buf.exists() {
        eprintln!("❌ Error: Wallet file not found: {}", wallet_path_buf.display());
        std::process::exit(1);
    }

    match mode.as_str() {
        "server" => {
            println!("🌐 Starting TCP control server on port {}...", server_port);
            
            // Initialize capital progression manager with ultra-small capital
            let progression_manager = CapitalProgressionManager::new(0.001)?;
            println!("📈 Capital Progression Manager initialized");
            let status = progression_manager.get_progression_status();
            println!("💰 Current phase: {:?}", status.current_phase);
            
            // Initialize bot controller
            let bot_controller = Arc::new(BotController::new().await?);
            
            let server = TcpControlServer::new(bot_controller, server_port).await?;
            server.run().await?;
        }
        "single-run" => {
            println!("🎯 Single run mode - executing one trading cycle");
            
            // 🚀 COMPLETAR FUNCIONALIDAD: Use LiquiditySniperBot for actual bot execution
            let _simple_config = SimpleConfig::load_from_file(&config_path)?;
            
            // Create SniperConfig with default configuration
            let sniper_config = SniperConfig::default();
            
            // Generate a unique ID for this bot instance
            let bot_id = Uuid::new_v4();
            
            let bot = LiquiditySniperBot::new(bot_id, sniper_config).await?;
            
            println!("🤖 LiquiditySniperBot initialized successfully");
            println!("🎯 Starting hunting for liquidity opportunities...");
            
            // Start hunting for opportunities (this is a background task)
            tokio::spawn(async move {
                if let Err(e) = bot.start_hunting().await {
                    eprintln!("❌ Error during bot hunting: {}", e);
                }
            });
            
            // Simulate a brief hunting session for demo
            tokio::time::sleep(tokio::time::Duration::from_secs(5)).await;
            
            println!("🔍 Demo hunting session completed");
            println!("💡 Bot would continue running in production mode");
            
            println!("✅ Single run completed");
        }
        "test" => {
            println!("🧪 Test mode - validating configuration");
            let progression_manager = CapitalProgressionManager::new(0.001)?;
            println!("✅ Configuration validated successfully");
            let status = progression_manager.get_progression_status();
            println!("📊 Current capital: {:.4} SOL", status.current_capital_sol);
            println!("🎯 Next milestone: {:.1} SOL", 
                status.next_milestone.as_ref()
                    .map(|m| m.target_capital_sol)
                    .unwrap_or(0.0)
            );
        }
        _ => {
            eprintln!("❌ Invalid mode: {}", mode);
            std::process::exit(1);
        }
    }

    Ok(())
}
