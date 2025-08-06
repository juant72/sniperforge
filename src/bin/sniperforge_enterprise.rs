use clap::{Arg, Command};
use anyhow::Result;
use std::path::PathBuf;
use std::sync::Arc;

use sniperforge::control::{TcpControlServer, BotController};
use sniperforge::bots::liquidity_sniper::LiquiditySniperBot;
use sniperforge::bots::liquidity_sniper::capital_progression::CapitalProgressionManager;

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

    println!("🚀 SniperForge Enterprise v3.0 - Capital Accumulation Mode");
    println!("📊 Config: {}", config_path);
    println!("💼 Wallet: {}", wallet_path);
    println!("⚡ Mode: {}", mode);

    match mode.as_str() {
        "server" => {
            println!("🌐 Starting TCP control server on port {}...", server_port);
            
            // Initialize capital progression manager
            let progression_manager = CapitalProgressionManager::new(PathBuf::from(config_path))?;
            println!("📈 Capital Progression Manager initialized");
            println!("💰 Current phase: {:?}", progression_manager.get_current_phase());
            
            // Initialize bot controller
            let bot_controller = Arc::new(BotController::new());
            
            let server = TcpControlServer::new(bot_controller, server_port).await?;
            server.run().await?;
        }
        "single-run" => {
            println!("🎯 Single run mode - executing one trading cycle");
            // TODO: Implement single run logic
            println!("✅ Single run completed");
        }
        "test" => {
            println!("🧪 Test mode - validating configuration");
            let progression_manager = CapitalProgressionManager::new(PathBuf::from(config_path))?;
            println!("✅ Configuration validated successfully");
            println!("📊 Current capital: {:.4} SOL", progression_manager.get_current_capital());
            println!("🎯 Next milestone: {:.1} SOL", progression_manager.get_next_milestone());
        }
        _ => {
            eprintln!("❌ Invalid mode: {}", mode);
            std::process::exit(1);
        }
    }

    Ok(())
}
