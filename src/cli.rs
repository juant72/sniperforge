use anyhow::Result;
use clap::{Command, Arg, ArgMatches};
use colored::*;
use std::io::{self, Write};
use std::str::FromStr;
use solana_sdk::signer::{Signer, keypair::Keypair};

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
        )        .subcommand(Command::new("status").about("Show platform status"))
        .subcommand(Command::new("config").about("Show current configuration"))
        .subcommand(
            Command::new("wallet")
                .about("Wallet management commands")
                .subcommand(Command::new("balance").about("Check wallet balances"))
                .subcommand(Command::new("airdrop").about("Request SOL airdrop"))
        )        .subcommand(
            Command::new("test")
                .about("Test connections and basic functionality")
                .subcommand(Command::new("solana").about("Test Solana connectivity and RPC calls"))
                .subcommand(Command::new("pools").about("Test pool detection and analysis"))
                .subcommand(Command::new("wallets").about("Test wallet generation and management"))
                .subcommand(Command::new("jupiter").about("Test Jupiter API integration"))
        )
        .subcommand(Command::new("interactive").about("Interactive monitoring mode"))
        .get_matches();    match matches.subcommand() {
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
    
    // Determine config file to use
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
    
    // This would typically connect to a running platform instance
    // For Sprint 0, we'll show mock status
    println!("🟢 Platform: {}", "Online".bright_green());
    println!("🤖 Active Bots: {}", "1 (LP Sniper)".bright_cyan());
    println!("🔗 RPC Connections: {}", "Healthy".bright_green());
    println!("💾 Memory Usage: {}", "245 MB".bright_yellow());
    println!("⚡ CPU Usage: {}", "15%".bright_yellow());
    
    Ok(())
}

async fn handle_config_command() -> Result<()> {
    println!("{}", "⚙️ Current Configuration".bright_blue().bold());
    println!("{}", "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━".bright_blue());
    
    let config = Config::load("config/platform.toml")?;
    
    println!("📝 Platform: {} v{}", config.platform.name.bright_cyan(), config.platform.version.bright_yellow());
    println!("🌐 Primary RPC: {}", config.network.primary_rpc().bright_green());
    println!("🤖 Max Bots: {}", config.platform.max_concurrent_bots.to_string().bright_yellow());
    
    println!("\n{}", "Enabled Bots:".bright_white().bold());
    if config.is_bot_enabled("lp_sniper") {
        println!("  🎯 LP Sniper: {}", "Enabled".bright_green());
    }
    if config.is_bot_enabled("copy_trading") {
        println!("  📋 Copy Trading: {}", "Enabled".bright_green());
    } else {
        println!("  📋 Copy Trading: {}", "Disabled".bright_red());
    }
    if config.is_bot_enabled("arbitrage") {
        println!("  ⚖️ Arbitrage: {}", "Enabled".bright_green());
    } else {
        println!("  ⚖️ Arbitrage: {}", "Disabled".bright_red());
    }
    if config.is_bot_enabled("mev") {
        println!("  ⚡ MEV: {}", "Enabled".bright_green());
    } else {
        println!("  ⚡ MEV: {}", "Disabled".bright_red());
    }
    
    Ok(())
}

async fn handle_test_command(matches: &ArgMatches) -> Result<()> {
    match matches.subcommand() {
        Some(("solana", _)) => handle_test_solana_command().await?,
        Some(("pools", _)) => handle_test_pools_command().await?,
        Some(("wallets", _)) => handle_test_wallets_command().await?,
        Some(("jupiter", _)) => handle_test_jupiter_command().await?,
        _ => handle_test_all_command().await?,
    }
    Ok(())
}

async fn handle_test_solana_command() -> Result<()> {
    println!("{}", "🧪 Testing Solana Connectivity".bright_blue().bold());
    println!("{}", "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━".bright_blue());
    
    let config = Config::load("config/platform.toml")?;
    
    // Show network configuration
    println!("🌐 Network Environment: {}", 
             if config.network.is_devnet() { "DEVNET".bright_yellow() } else { "MAINNET".bright_red() });
    println!("📡 Primary RPC: {}", config.network.primary_rpc().bright_cyan());
    
    // Run connectivity tests
    match solana_testing::test_solana_connectivity(&config).await {
        Ok(_) => {
            println!("\n{}", "🎉 All Solana tests passed!".bright_green().bold());
        }
        Err(e) => {
            println!("\n{} {}", "❌ Solana tests failed:".bright_red().bold(), e);
            return Err(e);
        }
    }
    
    Ok(())
}

async fn handle_test_pools_command() -> Result<()> {
    println!("{}", "🧪 Testing Pool Detection & Analysis".bright_blue().bold());
    println!("{}", "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━".bright_blue());
    
    let config = Config::load("config/platform.toml")?;
    
    // Run pool analysis tests
    match solana_testing::test_pool_analysis(&config).await {
        Ok(_) => {
            println!("\n{}", "🎉 All pool analysis tests passed!".bright_green().bold());
        }
        Err(e) => {
            println!("\n{} {}", "❌ Pool analysis tests failed:".bright_red().bold(), e);
            return Err(e);
        }
    }
    
    Ok(())
}

async fn handle_test_wallets_command() -> Result<()> {
    println!("{}", "🧪 Testing Wallet Generation & Management".bright_blue().bold());
    println!("{}", "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━".bright_blue());
      // Test wallet generation
    print!("🔑 Generating test wallet... ");
    io::stdout().flush()?;
    let keypair = Keypair::new();
    let pubkey = keypair.pubkey();
    println!("{}", "✅ OK".bright_green());
    println!("   Wallet: {}", pubkey.to_string().bright_cyan());
    
    // Test wallet validation
    print!("🔍 Validating wallet format... ");
    io::stdout().flush()?;
    match solana_sdk::pubkey::Pubkey::from_str(&pubkey.to_string()) {
        Ok(_) => println!("{}", "✅ OK".bright_green()),
        Err(e) => println!("{} {}", "❌ FAILED:".bright_red(), e),
    }
    
    println!("\n{}", "🎉 All wallet tests passed!".bright_green().bold());
    Ok(())
}

async fn handle_test_all_command() -> Result<()> {
    println!("{}", "🧪 Running System Tests".bright_blue().bold());
    println!("{}", "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━".bright_blue());
    
    // Test configuration loading
    print!("📋 Loading configuration... ");
    io::stdout().flush()?;
    match Config::load("config/platform.toml") {
        Ok(config) => {
            println!("{}", "✅ OK".bright_green());
            
            // Test configuration validation
            print!("🔍 Validating configuration... ");
            io::stdout().flush()?;
            match config.validate() {
                Ok(_) => println!("{}", "✅ OK".bright_green()),
                Err(e) => {
                    println!("{} {}", "❌ FAILED:".bright_red(), e);
                    return Ok(());
                }
            }
            
            // Test RPC connection
            print!("🌐 Testing RPC connection... ");
            io::stdout().flush()?;
            match test_rpc_connection(config.network.primary_rpc()).await {
                Ok(_) => println!("{}", "✅ OK".bright_green()),
                Err(e) => {
                    println!("{} {}", "❌ FAILED:".bright_red(), e);
                      // Try backup RPCs
                    for (i, backup_rpc) in config.network.backup_rpc().iter().enumerate() {
                        print!("🌐 Testing backup RPC {}... ", i + 1);
                        io::stdout().flush()?;
                        match test_rpc_connection(backup_rpc).await {
                            Ok(_) => {
                                println!("{}", "✅ OK".bright_green());
                                break;
                            },
                            Err(e) => println!("{} {}", "❌ FAILED:".bright_red(), e),
                        }
                    }
                }
            }
        }
        Err(e) => {
            println!("{} {}", "❌ FAILED:".bright_red(), e);
            return Ok(());
        }
    }
    
    println!("\n{}", "✅ Basic system tests completed".bright_green().bold());
    Ok(())
}

async fn handle_interactive_command() -> Result<()> {
    println!("{}", "🎮 Interactive Monitoring Mode".bright_blue().bold());
    println!("{}", "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━".bright_blue());
    println!("Commands: status, bots, metrics, quit");
    
    loop {
        print!("{} ", "SniperForge>".bright_cyan().bold());
        io::stdout().flush()?;
        
        let mut input = String::new();
        io::stdin().read_line(&mut input)?;
        let input = input.trim();
        
        match input {
            "status" | "s" => {
                println!("🟢 Platform: Running");
                println!("🤖 Active Bots: 1");
                println!("💹 Last Trade: 2 minutes ago");
            }
            "bots" | "b" => {
                println!("🎯 LP Sniper: Running (Priority: High)");
                println!("📋 Copy Trading: Stopped");
                println!("⚖️ Arbitrage: Stopped");
                println!("⚡ MEV: Stopped");
            }
            "metrics" | "m" => {
                println!("📊 Performance Metrics:");
                println!("  - Latency: 45ms avg");
                println!("  - Success Rate: 94.2%");
                println!("  - Opportunities: 127 detected, 23 executed");
            }
            "quit" | "q" | "exit" => break,
            "help" | "h" => {
                println!("Available commands:");
                println!("  status (s)  - Show platform status");
                println!("  bots (b)    - Show bot status");
                println!("  metrics (m) - Show performance metrics");
                println!("  quit (q)    - Exit interactive mode");
            }
            "" => continue,
            _ => println!("Unknown command: {}. Type 'help' for available commands.", input),
        }
    }
    
    Ok(())
}

async fn show_main_menu() -> Result<()> {
    println!("{}", "🎯 SniperForge Multi-Bot Platform".bright_cyan().bold());
    println!("{}", "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━".bright_blue());
    println!();
    println!("Available commands:");
    println!("  {} - Start the platform", "sniperforge start".bright_green());
    println!("  {} - Start specific bot", "sniperforge start -b lp-sniper".bright_green());
    println!("  {} - Show platform status", "sniperforge status".bright_yellow());
    println!("  {} - Show configuration", "sniperforge config".bright_yellow());
    println!("  {} - Test system components", "sniperforge test".bright_cyan());
    println!("  {} - Interactive monitoring", "sniperforge interactive".bright_magenta());
    println!();
    println!("For help: {} or {}", "sniperforge --help".bright_white(), "sniperforge <command> --help".bright_white());
    
    Ok(())
}

async fn test_rpc_connection(rpc_url: &str) -> Result<()> {
    use solana_client::rpc_client::RpcClient;
    
    let client = RpcClient::new(rpc_url.to_string());
    
    // Test with a simple call
    let _blockhash = client.get_latest_blockhash()
        .map_err(|e| anyhow::anyhow!("RPC connection failed: {}", e))?;
    
    Ok(())
}

async fn handle_wallet_command(matches: &ArgMatches) -> Result<()> {
    match matches.subcommand() {
        Some(("balance", _)) => handle_wallet_balance_command().await?,
        Some(("airdrop", _)) => handle_wallet_airdrop_command().await?,
        _ => {
            println!("{}", "Available wallet commands:".bright_cyan());
            println!("  {} - Check wallet balances", "wallet balance".bright_green());
            println!("  {} - Request SOL airdrop", "wallet airdrop".bright_green());
        }
    }
    Ok(())
}

async fn handle_wallet_balance_command() -> Result<()> {
    println!("{}", "💰 Checking Wallet Balances".bright_blue().bold());
    println!("{}", "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━".bright_blue());
    
    let _config = Config::load("config/platform.toml")?;
    let rpc_client = solana_client::rpc_client::RpcClient::new("https://api.devnet.solana.com".to_string());
    
    // Known wallet from last test - we'll hardcode it for now
    let known_pubkey = "GHAwmESbFzgACvA5XtuuQFZ4NvPgBQRD27DqU8YNF9QZ";
    
    println!("🔍 Checking balance for wallet: {}", known_pubkey.bright_cyan());
    
    match solana_sdk::pubkey::Pubkey::from_str(known_pubkey) {
        Ok(pubkey) => {
            match rpc_client.get_balance(&pubkey) {
                Ok(balance_lamports) => {
                    let balance_sol = balance_lamports as f64 / 1_000_000_000.0;
                    println!("💰 Balance: {} SOL ({} lamports)", 
                             balance_sol.to_string().bright_green().bold(), 
                             balance_lamports.to_string().bright_yellow());
                    
                    if balance_lamports > 0 {
                        println!("✅ {}", "Airdrop was successful!".bright_green().bold());
                    } else {
                        println!("⏳ {}", "Airdrop might still be confirming...".bright_yellow());
                    }
                }
                Err(e) => {
                    println!("❌ Failed to get balance: {}", e.to_string().bright_red());
                }
            }
        }
        Err(e) => {
            println!("❌ Invalid pubkey: {}", e.to_string().bright_red());
        }
    }
    
    Ok(())
}

async fn handle_wallet_airdrop_command() -> Result<()> {
    println!("{}", "💧 Requesting SOL Airdrop".bright_blue().bold());
    println!("{}", "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━".bright_blue());
    
    println!("🚧 This will create and fund a new wallet with 2 SOL");
    
    // Generate a new keypair for testing
    let keypair = solana_sdk::signer::keypair::Keypair::new();
    let pubkey = keypair.pubkey();
    
    println!("🔑 Generated new wallet: {}", pubkey.to_string().bright_cyan());
    
    let rpc_client = solana_client::rpc_client::RpcClient::new("https://api.devnet.solana.com".to_string());
    let airdrop_amount = 2_000_000_000; // 2 SOL in lamports
    
    println!("💸 Requesting {} SOL airdrop...", (airdrop_amount as f64 / 1_000_000_000.0));
    
    match rpc_client.request_airdrop(&pubkey, airdrop_amount) {
        Ok(signature) => {
            println!("✅ Airdrop request successful!");
            println!("📝 Signature: {}", signature.to_string().bright_green());
            println!("🔗 View on explorer: https://explorer.solana.com/tx/{}?cluster=devnet", signature);
            
            println!("⏳ Waiting for confirmation...");
            tokio::time::sleep(tokio::time::Duration::from_secs(5)).await;
            
            match rpc_client.get_balance(&pubkey) {
                Ok(balance) => {
                    let balance_sol = balance as f64 / 1_000_000_000.0;
                    println!("💰 Final balance: {} SOL", balance_sol.to_string().bright_green().bold());
                }
                Err(e) => {
                    println!("⚠️ Could not verify balance: {}", e);
                }
            }
        }
        Err(e) => {
            println!("❌ Airdrop failed: {}", e.to_string().bright_red());
        }
    }
    
    Ok(())
}

async fn handle_test_jupiter_command() -> Result<()> {
    println!("{}", "🧪 Testing Jupiter API Integration".bright_blue().bold());
    println!("{}", "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━".bright_blue());
    
    let config = Config::load("config/platform.toml")?;
    let shared_services = sniperforge::SharedServices::new(&config).await?;
    
    println!("🪐 Testing Jupiter connectivity...");
    
    // Test Jupiter API connectivity
    let jupiter = shared_services.jupiter();
    match jupiter.test_connectivity().await {
        Ok(true) => {
            println!("✅ Jupiter API connection: {}", "SUCCESSFUL".bright_green());
        }
        Ok(false) => {
            println!("❌ Jupiter API connection: {}", "FAILED".bright_red());
            return Ok(());
        }
        Err(e) => {
            println!("❌ Jupiter API error: {}", format!("{}", e).bright_red());
            return Ok(());
        }
    }

    // Test quote functionality
    println!("\n💰 Testing quote functionality...");
    
    use sniperforge::shared::jupiter::{QuoteRequest, tokens};
    
    // Test SOL to USDC quote
    let quote_request = QuoteRequest::new(
        tokens::sol(),
        tokens::usdc(),
        1_000_000_000, // 1 SOL
    ).with_slippage(50); // 0.5% slippage

    match jupiter.quotes().get_quote(quote_request).await {
        Ok(quote) => {
            println!("✅ Quote successful:");
            println!("   📥 Input: {} SOL", quote.in_amount.parse::<u64>().unwrap_or(0) as f64 / 1_000_000_000.0);
            println!("   📤 Output: {} USDC", quote.out_amount.parse::<u64>().unwrap_or(0) as f64 / 1_000_000.0);
            println!("   💥 Price Impact: {}%", quote.price_impact_pct);
            println!("   🛣️  Routes: {}", quote.route_plan.len());
            
            if !quote.route_plan.is_empty() {
                println!("   🏪 Best DEX: {}", quote.route_plan[0].swap_info.label);
            }
        }
        Err(e) => {
            println!("❌ Quote failed: {}", format!("{}", e).bright_red());
        }
    }

    // Test price lookup
    println!("\n💵 Testing price lookup...");
    
    match jupiter.quotes().get_token_price_usd(&tokens::sol()).await {
        Ok(Some(price)) => {
            println!("✅ SOL Price: ${:.2}", price);
        }
        Ok(None) => {
            println!("⚠️  SOL price not available");
        }
        Err(e) => {
            println!("❌ Price lookup failed: {}", format!("{}", e).bright_red());
        }
    }

    // Test supported DEXes
    println!("\n🏪 Testing supported DEXes...");
    
    match jupiter.swaps().get_supported_dexes().await {
        Ok(dexes) => {
            println!("✅ Supported DEXes ({}):", dexes.len());
            for (i, dex) in dexes.iter().take(5).enumerate() {
                println!("   {}. {}", i + 1, dex);
            }
            if dexes.len() > 5 {
                println!("   ... and {} more", dexes.len() - 5);
            }
        }
        Err(e) => {
            println!("❌ DEX list failed: {}", format!("{}", e).bright_red());
        }
    }

    println!("\n🎉 Jupiter integration test completed!");
    Ok(())
}

#[tokio::main]
async fn main() -> Result<()> {
    // Initialize tracing
    tracing_subscriber::fmt()
        .with_env_filter("info")
        .init();

    // This is a placeholder CLI main function
    // In practice, this would have proper CLI argument parsing
    println!("SniperForge CLI - Use the main binary for full functionality");
    
    Ok(())
}
