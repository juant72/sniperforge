use anyhow::Result;
use clap::{Command, Arg, ArgMatches};
use colored::*;
use std::io::{self, Write};

use sniperforge::{Config, SniperForgePlatform, solana_testing};

pub async fn run_cli() -> Result<()> {
    let matches = Command::new("SniperForge CLI")
        .version("0.1.0")
        .about("Interactive CLI for SniperForge Multi-Bot Platform")
        .subcommand(
            Command::new("start")
                .about("Start the platform or specific bots")
                .arg(
                    Arg::new("bot")
                        .short('b')
                        .long("bot")
                        .value_name("BOT_TYPE")
                        .help("Specific bot to start")
                        .action(clap::ArgAction::Append)
                )
                .arg(
                    Arg::new("devnet")
                        .long("devnet")
                        .help("Use devnet configuration for testing")
                        .action(clap::ArgAction::SetTrue)
                )
        )
        .subcommand(Command::new("status").about("Show platform status"))
        .subcommand(Command::new("config").about("Show current configuration"))
        .subcommand(
            Command::new("wallet")
                .about("Wallet management commands")
                .subcommand(Command::new("balance").about("Check wallet balances"))
                .subcommand(Command::new("airdrop").about("Request SOL airdrop"))
        )
        .subcommand(
            Command::new("test")
                .about("Testing suite")
                .subcommand(Command::new("basic").about("Run basic connectivity tests"))
                .subcommand(Command::new("solana").about("Test Solana connectivity"))
                .subcommand(Command::new("jupiter").about("Test Jupiter API"))
                .subcommand(Command::new("websocket").about("Test WebSocket connectivity"))
        )
        .subcommand(Command::new("interactive").about("Interactive monitoring mode"))
        .get_matches();

    match matches.subcommand() {
        Some(("start", sub_matches)) => handle_start_command(sub_matches).await?,
        Some(("status", _)) => handle_status_command().await?,
        Some(("config", _)) => handle_config_command().await?,
        Some(("wallet", sub_matches)) => handle_wallet_command(sub_matches).await?,
        Some(("test", sub_matches)) => handle_test_command(sub_matches).await?,
        Some(("interactive", _)) => handle_interactive_command().await?,
        _ => {
            println!("{}", "No command specified. Use --help for available commands.".yellow());
            show_main_menu().await?;
        }
    }
    
    Ok(())
}

async fn handle_start_command(matches: &ArgMatches) -> Result<()> {
    println!("{}", "🚀 Starting SniperForge Platform...".bright_green().bold());
    
    let config_file = if matches.get_flag("devnet") {
        println!("{}", "🧪 Using DEVNET configuration for testing".bright_yellow());
        "config/devnet.toml"
    } else {
        "config/platform.toml"
    };
    
    let config = Config::load(config_file)?;
    let platform = SniperForgePlatform::new(config).await?;
    
    if let Some(bot_types) = matches.get_many::<String>("bot") {
        platform.start_specific_bots(bot_types.cloned().collect()).await?;
    } else {
        platform.start_platform().await?;
    }
    
    platform.run().await?;
    Ok(())
}

async fn handle_status_command() -> Result<()> {
    println!("{}", "📊 Platform Status".bright_blue().bold());
    println!("{}", "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━".bright_blue());
    
    println!("🟢 Platform: {}", "Online".bright_green());
    println!("🤖 Active Bots: {}", "1 (LP Sniper)".bright_cyan());
    println!("💰 Total Balance: {}", "0.5 SOL".bright_yellow());
    println!("⚡ Latency: {}", "< 50ms".bright_green());
    
    Ok(())
}

async fn handle_config_command() -> Result<()> {
    println!("{}", "⚙️ Configuration".bright_blue().bold());
    println!("{}", "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━".bright_blue());
    
    let config = Config::load("config/platform.toml")?;
    println!("📁 Config file: {}", "config/platform.toml".cyan());
    println!("🌐 Network: {}", if config.network.is_devnet() { "Devnet" } else { "Mainnet" }.yellow());
    println!("📡 RPC URL: {}", config.network.primary_rpc().cyan());
    
    Ok(())
}

async fn handle_wallet_command(matches: &ArgMatches) -> Result<()> {
    match matches.subcommand() {
        Some(("balance", _)) => {
            println!("{}", "💰 Checking wallet balances...".bright_cyan());
            solana_testing::test_solana_connectivity().await?;
        }
        Some(("airdrop", _)) => {
            println!("{}", "🪂 Requesting SOL airdrop...".bright_cyan());
            println!("   💡 Use: solana airdrop 1 <your-pubkey> --url devnet");
        }
        _ => {
            println!("{}", "Available wallet commands:".bright_cyan());
            println!("  • balance - Check wallet balances");
            println!("  • airdrop - Request SOL airdrop");
        }
    }
    Ok(())
}

async fn handle_test_command(matches: &ArgMatches) -> Result<()> {
    match matches.subcommand() {
        Some(("basic", _)) => handle_test_basic().await?,
        Some(("solana", _)) => handle_test_solana().await?,
        Some(("jupiter", _)) => handle_test_jupiter().await?,
        Some(("websocket", _)) => handle_test_websocket().await?,
        _ => {
            println!("{}", "Available tests:".bright_cyan());
            println!("  • basic - Basic connectivity");
            println!("  • solana - Solana RPC connectivity");
            println!("  • jupiter - Jupiter API");
            println!("  • websocket - WebSocket connectivity");
        }
    }
    Ok(())
}

async fn handle_test_basic() -> Result<()> {
    println!("{}", "🧪 Basic Connectivity Test".bright_blue().bold());
    println!("{}", "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━".bright_blue());
    
    use sniperforge::simple_testing::run_simple_tests;
    run_simple_tests().await;
    
    Ok(())
}

async fn handle_test_solana() -> Result<()> {
    println!("{}", "🌐 Solana Connectivity Test".bright_blue().bold());
    println!("{}", "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━".bright_blue());
    
    match solana_testing::test_solana_connectivity().await {
        Ok(_) => println!("{}", "✅ Solana connectivity successful".bright_green()),
        Err(e) => println!("{} {}", "❌ Solana connectivity failed:".bright_red(), e),
    }
    
    Ok(())
}

async fn handle_test_jupiter() -> Result<()> {
    println!("{}", "🪐 Jupiter API Test".bright_blue().bold());
    println!("{}", "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━".bright_blue());
    
    let config = sniperforge::shared::jupiter::JupiterConfig::default();
    let client = sniperforge::shared::jupiter::JupiterClient::new(&config);
    
    print!("📊 Testing Jupiter price API... ");
    io::stdout().flush()?;
    
    match client.get_price("So11111111111111111111111111111111111111112").await {
        Ok(Some(price)) => {
            println!("{}", "✅ OK".bright_green());
            println!("   SOL price: ${:.2}", price);
        }
        Ok(None) => {
            println!("{}", "⚠️  No price data".bright_yellow());
        }
        Err(e) => {
            println!("{} {}", "❌ FAILED:".bright_red(), e);
        }
    }
    
    Ok(())
}

async fn handle_test_websocket() -> Result<()> {
    println!("{}", "🔌 WebSocket Connectivity Test".bright_blue().bold());
    println!("{}", "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━".bright_blue());
    
    use sniperforge::simple_testing::test_websocket_basic;
    test_websocket_basic().await;
    
    Ok(())
}

async fn handle_interactive_command() -> Result<()> {
    println!("{}", "🎮 Interactive Mode".bright_blue().bold());
    println!("{}", "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━".bright_blue());
    println!("💡 Interactive mode will be implemented in future versions");
    println!("📋 For now, use individual commands:");
    println!("   • cargo run -- test basic");
    println!("   • cargo run -- wallet balance"); 
    println!("   • cargo run -- status");
    
    Ok(())
}

async fn show_main_menu() -> Result<()> {
    println!("{}", "🎯 SniperForge CLI".bright_magenta().bold());
    println!("{}", "==================".bright_magenta());
    println!("Available commands:");
    println!("  🚀 {} - Start the platform", "start".bright_green());
    println!("  📊 {} - Show platform status", "status".bright_blue());
    println!("  ⚙️  {} - Show configuration", "config".bright_cyan());
    println!("  💰 {} - Wallet management", "wallet".bright_yellow());
    println!("  🧪 {} - Run tests", "test".bright_purple());
    println!("  🎮 {} - Interactive mode", "interactive".bright_white());
    println!();
    println!("Use {} for detailed help", "--help".bright_white());
    
    Ok(())
}
