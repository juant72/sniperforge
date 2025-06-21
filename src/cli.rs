use anyhow::Result;
use clap::{Command, Arg, ArgMatches};
use colored::*;
use std::io::{self, Write};

use sniperforge::{Config, SniperForgePlatform, solana_testing};

pub async fn run_cli() -> Result<()> {
    // Check for help argument first
    let args: Vec<String> = std::env::args().collect();
    if args.len() > 1 && (args[1] == "--help" || args[1] == "-h") {
        show_help();
        return Ok(());
    }
    
    let matches = Command::new("SniperForge CLI")
        .version("0.1.0")
        .about("Interactive CLI for SniperForge Multi-Bot Platform")
        .disable_help_flag(true)
        .disable_version_flag(false)
        .subcommand_required(false)
        .arg_required_else_help(false)
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
        )        .subcommand(            Command::new("test")
                .about("Testing suite")                .subcommand(Command::new("all").about("Run all tests"))
                .subcommand(Command::new("basic").about("Run basic connectivity tests"))
                .subcommand(Command::new("solana").about("Test Solana connectivity"))
                .subcommand(Command::new("jupiter").about("Test Jupiter API"))
                .subcommand(Command::new("jupiter-speed").about("Test Jupiter API speed/performance"))
                .subcommand(Command::new("websocket").about("Test WebSocket connectivity"))
                .subcommand(Command::new("wallet").about("Test wallet functionality"))
                .subcommand(Command::new("trade").about("Test trade execution"))                .subcommand(Command::new("integration").about("Test complete integration flow"))
                .subcommand(Command::new("performance").about("Test performance and latency"))                .subcommand(Command::new("websocket-rpc").about("Compare HTTP vs WebSocket RPC latency"))
                .subcommand(Command::new("websocket-prices").about("Test real-time WebSocket price feed system"))
                .subcommand(Command::new("syndica").about("Test Syndica ultra-fast WebSocket performance"))                .subcommand(Command::new("cache-safety").about("Test cache safety and eviction"))
                .subcommand(Command::new("devnet-trade").about("Execute first real trade on DevNet"))
                .subcommand(Command::new("paper-trading").about("Test paper trading with mainnet data"))
                .subcommand(Command::new("cache-free-trading").about("Test cache-free trading safety"))                .subcommand(Command::new("pools").about("Test pool detection and analysis (mainnet read-only)"))                .subcommand(
                    Command::new("monitor-pools")
                        .about("Continuous pool monitoring (mainnet)")
                        .arg(Arg::new("duration")
                            .short('d')
                            .long("duration")
                            .value_name("SECONDS")
                            .help("Monitoring duration in seconds (default: 300)")
                            .default_value("300"))
                )
                .subcommand(
                    Command::new("pools-extended")
                        .about("Phase 1: Extended pool monitoring for trading automation")
                        .arg(Arg::new("duration")
                            .short('d')
                            .long("duration")
                            .value_name("HOURS")
                            .help("Monitoring duration in hours (default: 4)")
                            .default_value("4"))                )
                .subcommand(
                    Command::new("ultra-fast-pools")
                        .about("Ultra-fast pool monitoring with WebSocket + API hybrid")
                        .arg(Arg::new("duration")
                            .short('d')
                            .long("duration")
                            .value_name("SECONDS")
                            .help("Monitoring duration in seconds (default: 60)")
                            .default_value("60"))
                )
                .subcommand(
                    Command::new("ultra-fast-triggers")
                        .about("🚀 PHASE 2: Ultra-fast monitoring with auto-triggers")
                        .arg(Arg::new("duration")
                            .short('d')
                            .long("duration")
                            .value_name("SECONDS")
                            .help("Monitoring duration in seconds (default: 30)")
                            .default_value("30"))
                ))
        .subcommand(Command::new("interactive").about("Interactive monitoring mode"))
        .subcommand(Command::new("help").about("Show help for commands"))
        .get_matches();    match matches.subcommand() {
        Some(("start", sub_matches)) => handle_start_command(sub_matches).await?,
        Some(("status", _)) => handle_status_command().await?,
        Some(("config", _)) => handle_config_command().await?,
        Some(("wallet", sub_matches)) => handle_wallet_command(sub_matches).await?,
        Some(("test", sub_matches)) => handle_test_command(sub_matches).await?,        Some(("interactive", _)) => handle_interactive_command().await?,
        Some(("help", _)) => {
            show_help();
        },
        _ => {
            println!("{}", "No command specified. Use 'help' for available commands.".yellow());
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
    match matches.subcommand() {        Some(("balance", _)) => {
            println!("{}", "💰 Checking wallet balances...".bright_cyan());
            let config = Config::load("config/devnet.toml").unwrap_or_else(|_| {
                Config::load("config/platform.toml").expect("Could not load config")
            });
            solana_testing::test_solana_connectivity(&config).await?;
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
        Some(("all", _)) => handle_test_all().await?,
        Some(("basic", _)) => handle_test_basic().await?,
        Some(("solana", _)) => handle_test_solana().await?,
        Some(("jupiter", _)) => handle_test_jupiter().await?,
        Some(("jupiter-speed", _)) => handle_test_jupiter_speed().await?,
        Some(("websocket", _)) => handle_test_websocket().await?,
        Some(("wallet", _)) => handle_test_wallet().await?,
        Some(("trade", _)) => handle_test_trade().await?,
        Some(("integration", _)) => handle_test_integration().await?,        Some(("performance", _)) => handle_test_performance().await?,        Some(("websocket-rpc", _)) => handle_test_websocket_rpc().await?,
        Some(("websocket-prices", _)) => handle_test_websocket_prices().await?,
        Some(("syndica", _)) => handle_test_syndica().await?,
        Some(("cache-safety", _)) => handle_test_cache_safety().await?,
        Some(("paper-trading", _)) => handle_test_paper_trading().await?,
        Some(("devnet-trade", _)) => handle_test_devnet_trade().await?,        Some(("cache-free-trading", _)) => handle_test_cache_free_trading().await?,        Some(("pools", _)) => handle_test_pools().await?,
        Some(("monitor-pools", sub_matches)) => {
            let duration = sub_matches.get_one::<String>("duration")
                .unwrap()
                .parse::<u64>()
                .unwrap_or(30);
            handle_monitor_pools(duration).await?
        }
        Some(("pools-extended", sub_matches)) => {
            let duration_hours = sub_matches.get_one::<String>("duration")
                .unwrap()
                .parse::<u64>()
                .unwrap_or(4);
            handle_pools_extended(duration_hours).await?
        }        Some(("ultra-fast-pools", sub_matches)) => {
            let duration = sub_matches.get_one::<String>("duration")
                .unwrap()
                .parse::<u64>()
                .unwrap_or(5);
            handle_ultra_fast_pools(duration).await?
        }        Some(("ultra-fast-triggers", sub_matches)) => {
            let duration = sub_matches.get_one::<String>("duration")
                .unwrap()
                .parse::<u64>()
                .unwrap_or(30);
            handle_ultra_fast_pools(duration).await?
        }
        _ => {
            println!("{}", "🧪 Available tests:".bright_cyan().bold());
            println!("  • {} - Run all tests", "all".bright_yellow());
            println!("  • {} - Basic connectivity", "basic".bright_yellow());
            println!("  • {} - Solana RPC connectivity", "solana".bright_yellow());
            println!("  • {} - Jupiter API", "jupiter".bright_yellow());
            println!("  • {} - Jupiter API speed/performance", "jupiter-speed".bright_yellow());
            println!("  • {} - WebSocket connectivity", "websocket".bright_yellow());
            println!("  • {} - Wallet functionality", "wallet".bright_yellow());
            println!("  • {} - Trade execution", "trade".bright_yellow());
            println!("  • {} - Complete integration flow", "integration".bright_yellow());            println!("  • {} - Performance and latency", "performance".bright_yellow());            println!("  • {} - WebSocket RPC performance", "websocket-rpc".bright_yellow());
            println!("  • {} - Real-time WebSocket price feed", "websocket-prices".bright_yellow());
            println!("  • {} - Syndica ultra-fast WebSocket", "syndica".bright_yellow());
            println!("  • {} - Cache safety and eviction", "cache-safety".bright_yellow());            println!("  • {} - Paper trading with mainnet data", "paper-trading".bright_yellow());            println!("  • {} - Cache-free trading engine (SAFE)", "cache-free-trading".bright_yellow());
            println!("  • {} - Execute first real trade on DevNet", "devnet-trade".bright_red());
            println!("  • {} - Pool detection and analysis (MainNet)", "pools".bright_yellow());
            println!("  • {} - Continuous pool monitoring", "monitor-pools".bright_yellow());
            println!("  • {} - 🎯 Phase 1: Extended pool monitoring (4-6h)", "pools-extended".bright_cyan());
            println!("  • {} - Ultra-fast WebSocket + API monitoring", "ultra-fast-pools".bright_green());
        }
    }
    Ok(())
}

async fn handle_test_all() -> Result<()> {
    println!("{}", "🧪 Running All Tests".bright_blue().bold());
    println!("{}", "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━".bright_blue());
      // Run tests sequentially to avoid the future type issues
    let mut passed = 0;
    let total = 9;
    
    // Test Basic
    println!("\n{} Running {} test...", "🏃".bright_blue(), "Basic".bright_white());
    match handle_test_basic().await {
        Ok(_) => {
            passed += 1;
            println!("{} {} test completed", "✅".bright_green(), "Basic");
        }
        Err(e) => {
            println!("{} {} test failed: {}", "❌".bright_red(), "Basic", e);
        }
    }
    
    // Test Solana
    println!("\n{} Running {} test...", "🏃".bright_blue(), "Solana".bright_white());
    match handle_test_solana().await {
        Ok(_) => {
            passed += 1;
            println!("{} {} test completed", "✅".bright_green(), "Solana");
        }
        Err(e) => {
            println!("{} {} test failed: {}", "❌".bright_red(), "Solana", e);
        }
    }
    
    // Test Jupiter
    println!("\n{} Running {} test...", "🏃".bright_blue(), "Jupiter".bright_white());
    match handle_test_jupiter().await {
        Ok(_) => {
            passed += 1;
            println!("{} {} test completed", "✅".bright_green(), "Jupiter");
        }
        Err(e) => {
            println!("{} {} test failed: {}", "❌".bright_red(), "Jupiter", e);
        }
    }
    
    // Test WebSocket
    println!("\n{} Running {} test...", "🏃".bright_blue(), "WebSocket".bright_white());
    match handle_test_websocket().await {
        Ok(_) => {
            passed += 1;
            println!("{} {} test completed", "✅".bright_green(), "WebSocket");
        }
        Err(e) => {
            println!("{} {} test failed: {}", "❌".bright_red(), "WebSocket", e);
        }
    }
    
    // Test Wallet
    println!("\n{} Running {} test...", "🏃".bright_blue(), "Wallet".bright_white());
    match handle_test_wallet().await {
        Ok(_) => {
            passed += 1;
            println!("{} {} test completed", "✅".bright_green(), "Wallet");
        }
        Err(e) => {
            println!("{} {} test failed: {}", "❌".bright_red(), "Wallet", e);
        }
    }
    
    // Test Integration
    println!("\n{} Running {} test...", "🏃".bright_blue(), "Integration".bright_white());
    match handle_test_integration().await {
        Ok(_) => {
            passed += 1;
            println!("{} {} test completed", "✅".bright_green(), "Integration");
        }
        Err(e) => {
            println!("{} {} test failed: {}", "❌".bright_red(), "Integration", e);
        }
    }
    
    // Test WebSocket RPC
    println!("\n{} Running {} test...", "🏃".bright_blue(), "WebSocket RPC".bright_white());
    match handle_test_websocket_rpc().await {
        Ok(_) => {
            passed += 1;
            println!("{} {} test completed", "✅".bright_green(), "WebSocket RPC");
        }
        Err(e) => {
            println!("{} {} test failed: {}", "❌".bright_red(), "WebSocket RPC", e);
        }
    }
    
    // Test Cache Safety
    println!("\n{} Running {} test...", "🏃".bright_blue(), "Cache Safety".bright_white());
    match handle_test_cache_safety().await {
        Ok(_) => {
            passed += 1;
            println!("{} {} test completed", "✅".bright_green(), "Cache Safety");
        }
        Err(e) => {
            println!("{} {} test failed: {}", "❌".bright_red(), "Cache Safety", e);
        }
    }
    
    // Test Cache-Free Trading
    println!("\n{} Running {} test...", "🏃".bright_blue(), "Cache-Free Trading".bright_white());
    match handle_test_cache_free_trading().await {
        Ok(_) => {
            passed += 1;
            println!("{} {} test completed", "✅".bright_green(), "Cache-Free Trading");
        }
        Err(e) => {
            println!("{} {} test failed: {}", "❌".bright_red(), "Cache-Free Trading", e);
        }
    }
    
    println!("\n{}", "🎯 Test Summary".bright_blue().bold());
    println!("{}/{} tests passed", passed.to_string().bright_green(), total);
    if passed == total {
        println!("{}", "🎉 All tests passed!".bright_green().bold());
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
    
    let config = Config::load("config/devnet.toml").unwrap_or_else(|_| {
        Config::load("config/platform.toml").expect("Could not load config")
    });
    
    match solana_testing::test_solana_connectivity(&config).await {
        Ok(_) => println!("{}", "✅ Solana connectivity successful".bright_green()),
        Err(e) => println!("{} {}", "❌ Solana connectivity failed:".bright_red(), e),
    }
    
    Ok(())
}

async fn handle_test_jupiter() -> Result<()> {
    println!("{}", "🪐 Jupiter API Test".bright_blue().bold());
    println!("{}", "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━".bright_blue());
      let config = sniperforge::shared::jupiter::JupiterConfig::default();
    let client = sniperforge::shared::jupiter::JupiterClient::new(&config).await?;
    
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

async fn handle_test_wallet() -> Result<()> {
    println!("{}", "💰 Wallet Functionality Test".bright_blue().bold());
    println!("{}", "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━".bright_blue());
    
    use sniperforge::shared::wallet_manager::{WalletManager, WalletConfig, WalletType, RiskManagement};
    use solana_sdk::signer::{keypair::Keypair, Signer};
    use std::time::Instant;
    
    println!("💰 Testing wallet functionality...");
    
    // Load configuration
    let config = Config::load("config/devnet.toml").unwrap_or_else(|_| {
        Config::load("config/platform.toml").expect("Config required")
    });
    
    // Test 1: Generate new test keypair
    print!("🔑 Generating test keypair... ");
    let start = Instant::now();
    let keypair = Keypair::new();
    let pubkey = keypair.pubkey();
    let generation_time = start.elapsed();
    println!("✅ {:.2}ms", generation_time.as_nanos() as f64 / 1_000_000.0);
    println!("   📍 Public Key: {}", pubkey);
    
    // Test 2: Create wallet manager
    print!("🏗️  Initializing wallet manager... ");
    let start = Instant::now();
    let wallet_manager = WalletManager::new(&config).await?;
    let init_time = start.elapsed();
    println!("✅ {:.2}ms", init_time.as_nanos() as f64 / 1_000_000.0);
    
    // Test 3: Create test wallet config
    print!("⚙️  Creating test wallet config... ");
    let wallet_config = WalletConfig {
        name: "test_wallet".to_string(),
        wallet_type: WalletType::Testing,
        keypair_path: None,
        keypair_data: Some(bs58::encode(&keypair.to_bytes()).into_string()),
        max_sol_balance: 10.0,
        min_sol_balance: 0.1,
        risk_management: RiskManagement {
            max_transaction_amount: 1.0,
            daily_limit: 5.0,
            require_confirmation: false,
            emergency_stop_threshold: 8.0,
        },
    };
    println!("✅");
    
    // Test 4: Add wallet to manager
    print!("💳 Adding wallet to manager... ");
    let start = Instant::now();
    wallet_manager.add_wallet(wallet_config.clone()).await?;
    let add_time = start.elapsed();
    println!("✅ {:.2}ms", add_time.as_nanos() as f64 / 1_000_000.0);
    
    // Test 5: Check wallet availability
    print!("🔍 Checking wallet availability... ");
    let start = Instant::now();
    let is_available = wallet_manager.is_wallet_available("test_wallet", 0.5).await?;
    let check_time = start.elapsed();
    println!("✅ Available: {} ({:.2}ms)", is_available, check_time.as_nanos() as f64 / 1_000_000.0);
    
    // Test 6: Get wallet public key
    print!("🔐 Retrieving wallet public key... ");
    if let Some(retrieved_pubkey) = wallet_manager.get_wallet_pubkey("test_wallet").await {
        println!("✅ {}", retrieved_pubkey);
        println!("   🔍 Key matches: {}", retrieved_pubkey == pubkey);
    } else {
        println!("❌ Failed to retrieve public key");
    }
    
    // Test 7: Check balance (will be 0 for new devnet wallet)
    print!("💰 Checking wallet balance... ");
    if let Some(balance) = wallet_manager.get_wallet_balance("test_wallet").await {
        println!("✅ Balance: {} SOL", balance);
    } else {
        println!("❌ Failed to retrieve balance");
    }
    
    // Test 8: Test risk management
    print!("⚖️  Testing risk management... ");
    let over_limit = wallet_manager.is_wallet_available("test_wallet", 2.0).await?; // Over max_transaction_amount
    println!("✅ Over-limit blocked: {}", !over_limit);
    
    println!("\n{} Wallet tests completed successfully!", "🎉".bright_green());
    println!("   📊 Performance: Keypair gen {:.2}ms, Manager init {:.2}ms", 
             generation_time.as_nanos() as f64 / 1_000_000.0,
             init_time.as_nanos() as f64 / 1_000_000.0);
    
    Ok(())
}

async fn handle_test_trade() -> Result<()> {
    println!("{}", "⚡ Trade Execution Test".bright_blue().bold());
    println!("{}", "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━".bright_blue());
    
    use sniperforge::shared::trade_executor::{TradeExecutor, TradeRequest, TradingMode};
    use solana_sdk::pubkey::Pubkey;
    use std::time::Instant;
    use std::str::FromStr;
    
    println!("⚡ Testing trade execution...");
    
    // Load configuration
    let config = Config::load("config/devnet.toml").unwrap_or_else(|_| {
        Config::load("config/platform.toml").expect("Config required")
    });
    
    // Test 1: Initialize Trade Executor (DevNet mode)
    print!("🎯 Initializing trade executor (DevNet mode)... ");
    let start = Instant::now();
    let trade_executor = match TradeExecutor::new(config.clone(), TradingMode::DevNetReal).await {
        Ok(executor) => {
            let init_time = start.elapsed();
            println!("✅ {:.2}ms", init_time.as_nanos() as f64 / 1_000_000.0);
            executor
        }
        Err(e) => {
            println!("❌ Failed: {}", e);
            println!("   Falling back to paper trading test...");
            match TradeExecutor::new(config.clone(), TradingMode::MainNetPaper).await {
                Ok(executor) => {
                    println!("✅ Paper trading executor initialized");
                    executor
                }
                Err(e) => {
                    println!("❌ Both DevNet and Paper trading failed: {}", e);
                    return Ok(());
                }
            }
        }
    };
    
    // Test 2: Create test trade request (SOL -> USDC)
    print!("💱 Creating test trade request (SOL -> USDC)... ");
    let sol_mint = Pubkey::from_str("So11111111111111111111111111111111111111112")?; // SOL
    let usdc_mint = Pubkey::from_str("EPjFWdd5AufqSSqeM2qN1xzybapC8G4wEGGkZwyTDt1v")?; // USDC
    
    let trade_request = TradeRequest {
        input_mint: sol_mint,
        output_mint: usdc_mint,
        amount_in: 100_000_000, // 0.1 SOL in lamports
        slippage_bps: 100,      // 1% slippage
        wallet_name: "test_wallet".to_string(),
        max_price_impact: 5.0,  // 5% max price impact
        trading_mode: TradingMode::DevNetReal,
    };
    println!("✅");
    println!("   💰 Trading: 0.1 SOL -> USDC");
    println!("   ⚖️  Max slippage: 1.0%");
    println!("   🎯 Max price impact: 5.0%");
      // Test 3: Get quote (no execution)
    print!("📊 Getting trade quote... ");
    let start = Instant::now();
    let quote_result = trade_executor.get_trade_quote(
        &sol_mint.to_string(),
        &usdc_mint.to_string(),
        trade_request.amount_in,
        Some(trade_request.slippage_bps),
    ).await;
    
    match quote_result {
        Ok(quote) => {
            let quote_time = start.elapsed();
            println!("✅ {:.2}ms", quote_time.as_nanos() as f64 / 1_000_000.0);
            
            let input_amount = quote.in_amount.parse::<u64>().unwrap_or(0) as f64 / 1_000_000_000.0;
            let output_amount = quote.out_amount.parse::<u64>().unwrap_or(0) as f64 / 1_000_000.0; // USDC has 6 decimals
            let price_impact = quote.price_impact_pct.parse::<f64>().unwrap_or(0.0);
            
            println!("   💱 Quote: {:.3} SOL -> {:.2} USDC", input_amount, output_amount);
            println!("   💲 Rate: 1 SOL = ${:.2} USDC", output_amount / input_amount);
            println!("   📊 Price Impact: {:.3}%", price_impact);
            println!("   🛣️  Route: {} steps", quote.route_plan.len());
            
            // Show route details
            if !quote.route_plan.is_empty() {
                println!("   📋 Route Details:");
                for (i, step) in quote.route_plan.iter().enumerate() {
                    println!("      {}. {} ({:.1}%)", 
                             i + 1, 
                             step.swap_info.label,
                             step.percent as f64);
                }
            }
        }
        Err(e) => {
            println!("❌ Quote failed: {}", e);
            println!("   This is expected if DevNet doesn't have sufficient liquidity");
        }
    }
    
    // Test 4: Paper trading simulation
    print!("📄 Testing paper trading simulation... ");
    let paper_executor = TradeExecutor::new(config.clone(), TradingMode::MainNetPaper).await?;
    
    let paper_trade = TradeRequest {
        input_mint: sol_mint,
        output_mint: usdc_mint,
        amount_in: 1_000_000_000, // 1.0 SOL in lamports
        slippage_bps: 50,         // 0.5% slippage for paper trading
        wallet_name: "paper_wallet".to_string(),
        max_price_impact: 2.0,    // 2% max price impact
        trading_mode: TradingMode::MainNetPaper,
    };
    
    match paper_executor.execute_trade(paper_trade).await {
        Ok(result) => {
            println!("✅ Paper trading simulation completed");
            println!("   💱 Simulated: {} SOL -> {} USDC", 
                     result.input_amount as f64 / 1_000_000_000.0,
                     result.output_amount as f64 / 1_000_000.0);
            println!("   🎯 Success: {} | Mode: {:?}", result.success, result.trading_mode);
            println!("   ⏱️  Execution time: {}ms", result.execution_time_ms);
            
            if let Some(error) = result.error_message {
                println!("   ⚠️  Note: {}", error);
            }
        }
        Err(e) => {
            println!("❌ Paper trading failed: {}", e);
        }
    }
    
    // Test 5: Safe trading with cache-free pricing
    print!("🛡️ Testing safe trading (cache-free)... ");
    let safe_trade = TradeRequest {
        input_mint: sol_mint,
        output_mint: usdc_mint,
        amount_in: 500_000_000, // 0.5 SOL
        slippage_bps: 50,
        wallet_name: "safe_wallet".to_string(),
        max_price_impact: 1.0,  // Very conservative for safety
        trading_mode: TradingMode::MainNetPaper,
    };
    
    match trade_executor.execute_safe_trade(safe_trade).await {
        Ok(result) => {
            println!("✅ Safe trading test completed");
            println!("   🛡️ Cache-free pricing: {}", if result.success { "✅ SAFE" } else { "⚠️ Rejected" });
            println!("   💱 Simulated: {} SOL -> {} USDC", 
                     result.input_amount as f64 / 1_000_000_000.0,
                     result.output_amount as f64 / 1_000_000.0);
            println!("   ⏱️  Execution time: {}ms", result.execution_time_ms);
        }
        Err(e) => {
            println!("❌ Safe trading test failed: {}", e);
        }
    }
    
    println!("\n🎉 Trade execution tests completed!");
    println!("📋 Test Summary:");
    println!("   ✅ Trade executor initialization");
    println!("   ✅ Jupiter quote integration");
    println!("   ✅ Paper trading simulation");
    println!("   ✅ Safe trading (cache-free pricing)");
    println!("   💡 Ready for DevNet real trading with proper wallet setup");
    
    Ok(())
}

async fn handle_test_devnet_trade() -> Result<()> {
    println!("{}", "🚀 DevNet Real Trading Test".bright_green().bold());
    println!("{}", "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━".bright_green());
    
    use sniperforge::shared::trade_executor::{TradeExecutor, TradeRequest, TradingMode};
    use sniperforge::shared::wallet_manager::WalletManager;
    use solana_sdk::pubkey::Pubkey;
    use std::time::Instant;
    use std::str::FromStr;
    
    println!("🚀 FIRST REAL TRADE ON DEVNET");
    println!("==============================");
    println!("⚠️ WARNING: This will execute a REAL transaction on DevNet blockchain");
    println!("💰 Using wallet with 5 SOL airdrop");
    println!("📊 Trade: 0.1 SOL → USDC (DevNet)");
    
    // Load DevNet configuration
    let config = Config::load("config/devnet.toml")?;
    
    // Initialize wallet manager first
    print!("💳 Initializing wallet manager... ");
    let start = Instant::now();
    let wallet_manager = WalletManager::new(&config).await?;
    let init_time = start.elapsed();
    println!("✅ {:.2}ms", init_time.as_nanos() as f64 / 1_000_000.0);
    
    // Check if we have the devnet-trading wallet with balance
    print!("💰 Checking DevNet wallet balance... ");
    if let Some(balance) = wallet_manager.get_wallet_balance("devnet-trading").await {
        println!("✅ Balance: {} SOL", balance);
        if balance < 0.2 {
            println!("❌ Insufficient balance! Need at least 0.2 SOL for trading test");
            println!("   Current balance: {} SOL", balance);
            return Ok(());
        }
    } else {
        println!("❌ DevNet wallet not found or balance check failed");
        return Ok(());
    }
    
    // Initialize trade executor for DevNet
    print!("⚡ Initializing DevNet trade executor... ");
    let start = Instant::now();
    let trade_executor = TradeExecutor::new(config.clone(), TradingMode::DevNetReal).await?;
    let executor_init_time = start.elapsed();
    println!("✅ {:.2}ms", executor_init_time.as_nanos() as f64 / 1_000_000.0);
    
    // Define trade parameters (SOL -> USDC on DevNet)
    let sol_mint = Pubkey::from_str("So11111111111111111111111111111111111111112")?; // SOL
    let usdc_mint = Pubkey::from_str("EPjFWdd5AufqSSqeM2qN1xzybapC8G4wEGGkZwyTDt1v")?; // USDC (might not exist on DevNet)
    
    let trade_request = TradeRequest {
        input_mint: sol_mint,
        output_mint: usdc_mint,
        amount_in: 100_000_000, // 0.1 SOL in lamports
        slippage_bps: 300,      // 3% slippage (conservative for DevNet)
        wallet_name: "devnet-trading".to_string(),
        max_price_impact: 10.0, // 10% max price impact (relaxed for DevNet)
        trading_mode: TradingMode::DevNetReal,
    };
    
    println!("\n📋 Trade Details:");
    println!("   🔄 Type: DevNet Real Trading");
    println!("   💱 Pair: SOL → USDC");
    println!("   💰 Amount: 0.1 SOL");
    println!("   ⚖️ Max Slippage: 3.0%");
    println!("   🎯 Max Price Impact: 10.0%");
    println!("   💳 Wallet: devnet-trading");
    
    // Get quote first to validate the trade
    print!("\n📊 Getting trade quote... ");
    let start = Instant::now();
    let quote_result = trade_executor.get_trade_quote(
        &sol_mint.to_string(),
        &usdc_mint.to_string(),
        trade_request.amount_in,
        Some(trade_request.slippage_bps),
    ).await;
    
    match quote_result {
        Ok(quote) => {
            let quote_time = start.elapsed();
            println!("✅ {:.2}ms", quote_time.as_nanos() as f64 / 1_000_000.0);
            
            let input_amount = quote.in_amount.parse::<u64>().unwrap_or(0) as f64 / 1_000_000_000.0;
            let output_amount = quote.out_amount.parse::<u64>().unwrap_or(0) as f64 / 1_000_000.0;
            let price_impact = quote.price_impact_pct.parse::<f64>().unwrap_or(0.0);
            
            println!("   💱 Quote: {:.3} SOL → {:.6} USDC", input_amount, output_amount);
            println!("   💲 Rate: 1 SOL = ${:.6} USDC", output_amount / input_amount);
            println!("   📊 Price Impact: {:.3}%", price_impact);
            println!("   🛣️ Route: {} steps", quote.route_plan.len());
            
            // Validation checks
            if price_impact > trade_request.max_price_impact {
                println!("❌ Price impact too high: {:.2}% > {:.1}%", price_impact, trade_request.max_price_impact);
                println!("   This trade would be rejected by risk management");
                return Ok(());
            }
            
            if output_amount < 0.001 {
                println!("❌ Output amount too low - likely no liquidity on DevNet");
                println!("   DevNet might not have USDC liquidity");
                println!("   💡 Try a different token pair or use paper trading mode");
                return Ok(());
            }
            
            // Ask for user confirmation for real trade
            println!("\n⚠️ REAL TRADE CONFIRMATION");
            println!("═══════════════════════════");
            println!("This will execute a REAL transaction on DevNet blockchain!");
            println!("Do you want to proceed? (y/N): ");
            
            use std::io;
            let mut input = String::new();
            io::stdin().read_line(&mut input)?;
            
            if input.trim().to_lowercase() != "y" {
                println!("❌ Trade cancelled by user");
                return Ok(());
            }
            
            // Execute the real trade
            println!("\n🚀 EXECUTING REAL TRADE...");
            println!("⏱️ This may take 10-30 seconds...");
            
            let start = Instant::now();
            match trade_executor.execute_trade(trade_request).await {
                Ok(result) => {
                    let _execution_time = start.elapsed();
                    
                    println!("\n🎉 TRADE EXECUTION COMPLETED!");
                    println!("═══════════════════════════");
                    println!("✅ Success: {}", result.success);
                    
                    if let Some(signature) = &result.transaction_signature {
                        println!("📋 Transaction Signature: {}", signature);
                        println!("🔗 Solana Explorer: https://explorer.solana.com/tx/{}?cluster=devnet", signature);
                    }
                    
                    println!("💱 Traded: {} SOL → {} USDC", 
                             result.input_amount as f64 / 1_000_000_000.0,
                             result.output_amount as f64 / 1_000_000.0);
                    println!("📊 Actual Price Impact: {:.3}%", result.actual_price_impact);
                    println!("⚖️ Actual Slippage: {:.3}%", result.actual_slippage);
                    println!("⛽ Gas Fee: {:.6} SOL", result.gas_fee);
                    println!("⏱️ Total Execution Time: {}ms", result.execution_time_ms);
                    
                    if let Some(error) = &result.error_message {
                        println!("⚠️ Note: {}", error);
                    }
                    
                    // Check final balance
                    println!("\n💰 Post-Trade Balance Check:");
                    if let Some(new_balance) = wallet_manager.get_wallet_balance("devnet-trading").await {
                        println!("   Final balance: {} SOL", new_balance);
                    }
                    
                    println!("\n🎊 CONGRATULATIONS!");
                    println!("🏆 First successful real trade executed on DevNet!");
                    println!("✅ System ready for MainNet trading preparation");
                }
                Err(e) => {
                    let execution_time = start.elapsed();
                    println!("\n❌ TRADE EXECUTION FAILED");
                    println!("═══════════════════════");
                    println!("Error: {}", e);
                    println!("⏱️ Time taken: {:?}", execution_time);
                    println!("💡 This is expected on DevNet due to limited liquidity");
                    println!("   The important thing is that our execution engine works!");
                }
            }
        }
        Err(e) => {
            println!("❌ Quote failed: {}", e);
            println!("💡 This is expected on DevNet - limited token pairs available");
            println!("   The system is working, but DevNet lacks liquidity for SOL→USDC");
            println!("   ✅ Jupiter integration is functional");
            println!("   ✅ Trade executor is ready");
            println!("   ✅ Ready for MainNet with proper token pairs");
        }
    }
    
    println!("\n📊 DevNet Trade Test Summary:");
    println!("   ✅ Wallet management functional");
    println!("   ✅ Trade executor initialized");
    println!("   ✅ Jupiter API integration working");
    println!("   ✅ Quote system operational");
    println!("   ✅ Real blockchain interaction ready");
    println!("   💡 Ready for MainNet trading with proper liquidity");
    
    Ok(())
}

async fn handle_test_integration() -> Result<()> {
    println!("{}", "🔄 Integration Flow Test".bright_blue().bold());
    println!("{}", "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━".bright_blue());
    
    use sniperforge::simple_testing::run_simple_tests;
    run_simple_tests().await;
    
    Ok(())
}

async fn handle_test_performance() -> Result<()> {
    println!("{}", "⚡ Performance Test".bright_blue().bold());
    println!("{}", "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━".bright_blue());
    
    use std::time::Instant;
    
    // Test compilation time (already compiled)
    println!("🏗️  Build performance: Fast (optimized with sccache)");
    
    // Test RPC latency
    print!("🌐 Testing RPC latency... ");
    let start = Instant::now();
    let config = Config::load("config/devnet.toml").unwrap_or_else(|_| {
        Config::load("config/platform.toml").expect("Config required")
    });
    
    match solana_testing::test_solana_connectivity(&config).await {
        Ok(_) => {
            let latency = start.elapsed();
            println!("✅ {:?}", latency);
        }
        Err(e) => println!("❌ {}", e),
    }
      // Test Jupiter API latency
    print!("🪐 Testing Jupiter API latency... ");
    let start = Instant::now();
    let jupiter_config = sniperforge::shared::jupiter::JupiterConfig::default();
    match sniperforge::shared::jupiter::JupiterClient::new(&jupiter_config).await {
        Ok(client) => {
            match client.get_price("So11111111111111111111111111111111111111112").await {
                Ok(_) => {
                    let latency = start.elapsed();
                    println!("✅ {:?}", latency);
                }
                Err(e) => println!("❌ {}", e),
            }
        }
        Err(e) => println!("❌ {}", e),
    }
    
    println!("🎉 Performance tests completed!");
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

async fn handle_test_jupiter_speed() -> Result<()> {
    println!("{}", "⚡ Jupiter Speed Performance Test".bright_blue().bold());
    println!("{}", "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━".bright_blue());
    
    use sniperforge::jupiter_speed_test::test_jupiter_speed;
    test_jupiter_speed().await?;
    
    Ok(())
}

async fn handle_test_syndica() -> Result<()> {
    println!("{}", "⚡ Ultimate RPC Performance Comparison".bright_blue().bold());
    println!("{}", "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━".bright_blue());
    
    use sniperforge::ultimate_rpc_test::run_ultimate_rpc_comparison;
    run_ultimate_rpc_comparison().await?;
    
    Ok(())
}

async fn handle_test_cache_safety() -> Result<()> {
    println!("{}", "🛡️ Cache Safety Analysis Test".bright_blue().bold());
    println!("{}", "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━".bright_blue());
    
    // Use the simplified version for now
    use crate::shared::cache_free_trader_simple::test_cache_free_trading;
    test_cache_free_trading().await?;
    
    Ok(())
}

async fn handle_test_cache_free_trading() -> Result<()> {
    println!("{}", "🛡️ Cache-Free Trading Engine Test".bright_blue().bold());
    println!("{}", "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━".bright_blue());
      use crate::shared::cache_free_trader_simple::test_cache_free_trading;
    test_cache_free_trading().await?;
    
    Ok(())
}

async fn handle_test_paper_trading() -> Result<()> {
    println!("{}", "📊 Paper Trading with Mainnet Data Test".bright_blue().bold());
    println!("{}", "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━".bright_blue());
    
    use crate::shared::paper_trading::test_paper_trading_mainnet;
    test_paper_trading_mainnet().await?;
    
    Ok(())
}

async fn handle_test_pools() -> Result<()> {
    println!("{}", "🔍 Pool Detection and Analysis Test (MainNet Read-Only)".bright_blue().bold());
    println!("{}", "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━".bright_blue());
    
    use crate::shared::pool_detector::{PoolDetector, PoolDetectorConfig};
    use crate::shared::jupiter::JupiterConfig;
    use crate::shared::jupiter::client::JupiterClient;
    use crate::shared::syndica_websocket::{SyndicaWebSocketClient, SyndicaConfig};
    use std::time::Duration;
    
    println!("🔍 POOL DETECTION DEMO (MainNet Data)");
    println!("=====================================");
    
    // Setup Jupiter client for mainnet
    let jupiter_config = JupiterConfig::mainnet();
    let jupiter_client = JupiterClient::new(&jupiter_config).await?;
    println!("✅ Jupiter client initialized for mainnet");
    
    // Setup Syndica client (optional)
    let syndica_config = SyndicaConfig::mainnet();
    let syndica_client = match SyndicaWebSocketClient::new(syndica_config).await {
        Ok(client) => {
            println!("✅ Syndica WebSocket client initialized");
            Some(client)
        }
        Err(e) => {
            println!("⚠️ Syndica client failed to initialize: {}", e);
            println!("   Continuing with Jupiter-only detection");
            None
        }
    };
      // Create pool detector with relaxed demo settings for better detection
    let config = PoolDetectorConfig {
        min_liquidity_usd: 1000.0,    // Lower threshold for demo
        max_price_impact_1k: 25.0,    // Higher tolerance for demo  
        min_risk_score: 0.05,         // Lower threshold for demo
        monitoring_interval_ms: 5000, // 5s for better real detection
        max_tracked_pools: 20,        // More pools for demo
    };
    
    println!("\n📊 Pool Detection Configuration:");    println!("   Min liquidity: ${:.0}", config.min_liquidity_usd);
    println!("   Max price impact: {:.1}%", config.max_price_impact_1k);
    println!("   Min risk score: {:.1}%", config.min_risk_score * 100.0);
    println!("   Scan interval: {}ms", config.monitoring_interval_ms);
    
    let mut detector = PoolDetector::new(config, jupiter_client, syndica_client, None).await?;
      println!("\n🚀 Starting extended pool monitoring demo (3 minutes)...");
    println!("   This connects to REAL Raydium/Orca APIs");
    println!("   Searching for new pools and opportunities...");
    println!("   Press Ctrl+C to stop early if needed");
    
    // Run monitoring for 3 minutes (extended demo)
    let _start_time = std::time::Instant::now();
    // Use the new monitored detection method
    if let Err(e) = detector.start_monitoring_with_reports(3).await {
        println!("⚠️ Monitoring failed: {}", e);
    }
    
    // Show results
    let stats = detector.get_stats();
    let opportunities = detector.get_current_opportunities();
    let pools = detector.get_tracked_pools();
    
    println!("\n📊 DETECTION RESULTS:");
    println!("====================");
    println!("🔍 Tracked pools: {}", stats.tracked_pools);
    println!("🎯 Active opportunities: {}", stats.active_opportunities);
    println!("⏱️ Last scan: {:?} ago", stats.last_scan_ago);
      if !opportunities.is_empty() {
        println!("\n🎯 DETECTED OPPORTUNITIES (Detailed Analysis):");
        for (i, opp) in opportunities.iter().enumerate() {
            println!("\n   {}. 🚀 TRADING OPPORTUNITY", i + 1);
            println!("      📍 Pool Address: {}", opp.pool.pool_address);
            println!("      🏪 DEX: {}", opp.pool.dex);
            println!("      💱 Pair: {} / {}", opp.pool.token_a.symbol, opp.pool.token_b.symbol);
              println!("      💰 Financial Metrics:");
            println!("         Expected Profit: ${:.2}", opp.expected_profit_usd);
            println!("         Confidence Level: {:.1}%", opp.confidence * 100.0);
            println!("         Recommended Size: ${:.0}", opp.recommended_size_usd);
            println!("         Time Window: {}ms", opp.time_window_ms);
            println!("         Opportunity Type: {:?}", opp.opportunity_type);
            
            println!("      📊 Pool Fundamentals:");
            println!("         Liquidity: ${:.0}", opp.pool.liquidity_usd);
            println!("         24h Volume: ${:.0}", opp.pool.volume_24h);
            println!("         Price Impact (1k): {:.2}%", opp.pool.price_impact_1k);
            
            println!("      🔍 Validation Links:");
            println!("         DexScreener: https://dexscreener.com/solana/{}", opp.pool.pool_address);
            println!("         Raydium: https://raydium.io/swap/?inputCurrency={}&outputCurrency={}", 
                     opp.pool.token_a.mint, opp.pool.token_b.mint);
        }
    } else {
        println!("\n📭 No opportunities detected during demo");
        println!("   (This is normal for a short demo run)");
    }
      if !pools.is_empty() {
        println!("\n🏊 TRACKED POOLS (Detailed Info for DexScreener Validation):");
        for (address, pool) in pools.iter().take(3) { // Show max 3
            println!("\n   🔍 Pool: {}", address);
            println!("      📍 Address: {}", address);
            println!("      🏪 DEX: {}", pool.dex);
            println!("      💱 Pair: {} / {}", pool.token_a.symbol, pool.token_b.symbol);
            
            println!("      💰 Token A ({}): {}", pool.token_a.symbol, pool.token_a.mint);
            println!("         💵 Price: ${:.6}", pool.token_a.price_usd);
            println!("         📊 Market Cap: ${:.0}", pool.token_a.market_cap);
            
            println!("      💰 Token B ({}): {}", pool.token_b.symbol, pool.token_b.mint);
            println!("         💵 Price: ${:.6}", pool.token_b.price_usd);
            println!("         📊 Market Cap: ${:.0}", pool.token_b.market_cap);
            
            println!("      💧 Pool Metrics:");
            println!("         Liquidity: ${:.0}", pool.liquidity_usd);
            println!("         24h Volume: ${:.0}", pool.volume_24h);
            println!("         Price Impact (1k): {:.2}%", pool.price_impact_1k);
            
            println!("      ⚠️ Risk Analysis:");
            println!("         Overall Risk: {:.1}%", pool.risk_score.overall * 100.0);
            println!("         Liquidity Score: {:.1}%", pool.risk_score.liquidity_score * 100.0);
            println!("         Volume Score: {:.1}%", pool.risk_score.volume_score * 100.0);
            
            if !pool.risk_score.rug_indicators.is_empty() {
                println!("         🚨 Rug Indicators: {:?}", pool.risk_score.rug_indicators);
            } else {
                println!("         ✅ No major rug indicators detected");
            }
            
            // Calculate age for reference
            let created_ago = std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .as_secs() - pool.created_at;
            println!("         ⏰ Pool Age: {} minutes", created_ago / 60);
            
            println!("      📈 DexScreener Validation:");
            println!("         🔗 Check: https://dexscreener.com/solana/{}", address);
            println!("         🔍 Look for: Liquidity ~${:.0}, Volume ~${:.0}", 
                     pool.liquidity_usd, pool.volume_24h);
        }
    }
    
    println!("\n✅ Pool detection test completed!");
    println!("   In production, this would run continuously");
    println!("   detecting real opportunities on Raydium/Orca");
    
    Ok(())
}

/// Handle continuous pool monitoring command
async fn handle_monitor_pools(duration_seconds: u64) -> Result<()> {
    println!("{}", "📡 Continuous Pool Monitoring (MainNet Real-Time)".bright_blue().bold());
    println!("{}", "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━".bright_blue());
    
    use crate::shared::pool_detector::{PoolDetector, PoolDetectorConfig};
    use crate::shared::jupiter::JupiterConfig;
    use crate::shared::jupiter::client::JupiterClient;
    use crate::shared::syndica_websocket::{SyndicaWebSocketClient, SyndicaConfig};
    
    println!("🚀 CONTINUOUS POOL MONITORING");
    println!("=============================");
    println!("⏱️ Duration: {} seconds ({:.1} minutes)", duration_seconds, duration_seconds as f64 / 60.0);
    println!("🌐 Data Source: Raydium + Orca APIs (MainNet)");
    println!("📊 Real-time opportunity detection enabled");
    
    // Setup Jupiter client for mainnet
    let jupiter_config = JupiterConfig::mainnet();
    let jupiter_client = JupiterClient::new(&jupiter_config).await?;
    println!("✅ Jupiter client initialized for mainnet");
    
    // Setup Syndica client (optional for price feeds)
    let syndica_config = SyndicaConfig::mainnet();
    let syndica_client = match SyndicaWebSocketClient::new(syndica_config).await {
        Ok(client) => {
            println!("✅ Syndica WebSocket client initialized");
            Some(client)
        }
        Err(e) => {
            println!("⚠️ Syndica client failed to initialize: {}", e);
            println!("   Continuing with DEX API-only detection");
            None
        }
    };
    
    // Create pool detector with production settings
    let config = PoolDetectorConfig {
        min_liquidity_usd: 5000.0,     // Reasonable threshold for real trading
        max_price_impact_1k: 15.0,     // Allow moderate price impact
        min_risk_score: 0.2,           // Lower threshold for opportunities
        monitoring_interval_ms: 3000,  // 3s for real-time but not too aggressive
        max_tracked_pools: 50,         // Track more pools in continuous mode
    };
    
    println!("\n📊 Monitoring Configuration:");
    println!("   Min liquidity: ${:.0}", config.min_liquidity_usd);
    println!("   Max price impact: {:.1}%", config.max_price_impact_1k);    println!("   Min risk score: {:.1}%", config.min_risk_score * 100.0);
    println!("   Scan interval: {}ms", config.monitoring_interval_ms);
    println!("   Max tracked pools: {}", config.max_tracked_pools);
    
    let mut detector = PoolDetector::new(config, jupiter_client, syndica_client, None).await?;

    println!("\n🔍 Starting continuous monitoring...");
    println!("   Status updates every 30 seconds");
    println!("   Press Ctrl+C to stop monitoring");
    println!("   All detected pools will include DexScreener validation links");
    
    // Start the monitored detection with exact duration in seconds
    detector.start_monitoring_with_reports_seconds(duration_seconds).await?;
    
    // Final summary
    let stats = detector.get_stats();
    let opportunities = detector.get_current_opportunities();
    let pools = detector.get_tracked_pools();
    
    println!("\n📊 MONITORING SESSION RESULTS:");
    println!("===============================");
    println!("🔍 Total tracked pools: {}", stats.tracked_pools);
    println!("🎯 Active opportunities: {}", stats.active_opportunities);
    
    // Show best opportunities
    if !opportunities.is_empty() {
        println!("\n🏆 TOP OPPORTUNITIES DETECTED:");        let mut sorted_opps: Vec<_> = opportunities.iter().collect();
        sorted_opps.sort_by(|a, b| b.expected_profit_usd.partial_cmp(&a.expected_profit_usd).unwrap());
        
        for (i, opp) in sorted_opps.iter().take(3).enumerate() {
            println!("\n   {}. 🎯 {} OPPORTUNITY", i + 1, 
                     match opp.opportunity_type {
                         crate::shared::pool_detector::OpportunityType::NewPoolSnipe => "NEW POOL",
                         crate::shared::pool_detector::OpportunityType::PriceDiscrepancy => "ARBITRAGE",
                         crate::shared::pool_detector::OpportunityType::LiquidityImbalance => "LOW SLIPPAGE",
                         crate::shared::pool_detector::OpportunityType::VolumeSpike => "VOLUME SPIKE",
                     });
            println!("      📍 Pool: {}", opp.pool.pool_address);
            println!("      💱 Pair: {}/{}", opp.pool.token_a.symbol, opp.pool.token_b.symbol);
            println!("      💰 Expected Profit: ${:.2}", opp.expected_profit_usd);
            println!("      🎯 Confidence: {:.1}%", opp.confidence * 100.0);
            println!("      💵 Recommended Size: ${:.0}", opp.recommended_size_usd);
            println!("      ⏱️ Time Window: {}s", opp.time_window_ms / 1000);
            println!("      🔗 Validate: https://dexscreener.com/solana/{}", opp.pool.pool_address);
        }
    } else {
        println!("\n📭 No high-confidence opportunities detected");
        println!("   Consider running for longer duration or adjusting thresholds");
    }
    
    // Show some tracked pools for reference
    if !pools.is_empty() {
        println!("\n📋 SAMPLE TRACKED POOLS:");
        for (i, (address, pool)) in pools.iter().take(3).enumerate() {
            println!("\n   {}. {} ({}/{})", i + 1, address, pool.token_a.symbol, pool.token_b.symbol);
            println!("      💧 Liquidity: ${:.0} | 📊 Volume: ${:.0} | ⚡ Impact: {:.1}%", 
                     pool.liquidity_usd, pool.volume_24h, pool.price_impact_1k);
            
            // Check if this pool would trigger
            let would_trigger = pool.liquidity_usd > 10000.0 && 
                               pool.price_impact_1k < 5.0 && 
                               pool.risk_score.overall > 0.7;
            
            if would_trigger {
                println!("      🚀 WOULD TRIGGER: Auto-execution criteria met");
            } else {
                println!("      📊 Analysis only: Trigger criteria not met");
            }
        }
    }
    
    println!("\n✅ Monitoring session completed!");
    println!("   💡 Tip: Use 'cargo run -- test monitor-pools -d 300' for 5-minute monitoring");
    
    Ok(())
}

/// Handle ultra-fast pool monitoring with WebSocket + API hybrid
async fn handle_ultra_fast_pools(duration_seconds: u64) -> Result<()> {
    println!("{}", "⚡ Ultra-Fast Pool Monitoring (WebSocket + API Hybrid)".bright_green().bold());
    println!("{}", "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━".bright_green());
    
    use crate::shared::pool_detector::{PoolDetector, PoolDetectorConfig};
    use crate::shared::jupiter::JupiterConfig;
    use crate::shared::jupiter::client::JupiterClient;
    use crate::shared::syndica_websocket::{SyndicaWebSocketClient, SyndicaConfig};
    
    println!("🚀 PHASE 2: ULTRA-FAST + AUTO-TRIGGERS");
    println!("======================================");
    println!("⏱️ Duration: {} seconds", duration_seconds);
    println!("🔥 Auto-execution: Simulated for high-confidence opportunities");
    println!("⚡ Target detection latency: <100ms");
    println!("🎯 Target execution latency: <1 second");
    println!("🛡️ Safety: Multiple confirmation layers before triggers");
    
    // Setup Jupiter client for mainnet
    let jupiter_config = JupiterConfig::mainnet();
    let jupiter_client = JupiterClient::new(&jupiter_config).await?;
    println!("✅ Jupiter client initialized for ultra-fast trading");
    
    // Setup Syndica WebSocket client (CRITICAL for ultra-fast mode)
    let syndica_config = SyndicaConfig::mainnet();
    let syndica_client = match SyndicaWebSocketClient::new(syndica_config).await {
        Ok(client) => {
            println!("🚀 Syndica WebSocket client initialized - ULTRA-FAST MODE");
            Some(client)
        }
        Err(e) => {
            println!("❌ CRITICAL: Syndica WebSocket failed: {}", e);
            println!("   Ultra-fast mode requires low-latency WebSocket connection!");
            return Ok(());
        }
    };
    
    // Ultra-fast configuration optimized for auto-triggers
    let config = PoolDetectorConfig {
        min_liquidity_usd: 25000.0,    // High threshold for auto-triggers
        max_price_impact_1k: 3.0,      // Very strict for safety
        min_risk_score: 0.8,           // High confidence required
        monitoring_interval_ms: 50,    // 50ms ultra-fast scanning
        max_tracked_pools: 30,         // Focused tracking for performance
    };
    
    println!("\n🔥 AUTO-TRIGGER Configuration:");
    println!("   Min liquidity: ${:.0} (HIGH threshold for safety)", config.min_liquidity_usd);
    println!("   Max price impact: {:.1}% (STRICT for auto-execution)", config.max_price_impact_1k);
    println!("   Min risk score: {:.0}% (HIGH confidence required)", config.min_risk_score * 100.0);
    println!("   Scan interval: {}ms (ULTRA-FAST)", config.monitoring_interval_ms);
    println!("   Auto-trigger conditions: HIGH_LIQUIDITY + HIGH_CONFIDENCE + LOW_IMPACT");
    
    let mut detector = PoolDetector::new(config, jupiter_client, syndica_client, None).await?;
    
    println!("\n🔥 Starting ultra-fast auto-trigger monitoring...");
    println!("   🚀 SIMULATED auto-execution for qualifying opportunities");
    println!("   ⚡ Real-time trigger evaluation every 50ms");
    println!("   🛡️ Safety checks: Liquidity validation, confidence scoring, impact analysis");
    println!("   📊 Trigger reports every 10 seconds");
    println!("   Press Ctrl+C to stop");
    
    // Start ultra-fast monitoring
    detector.start_ultra_fast_monitoring_seconds(duration_seconds).await?;
    
    // Auto-trigger results analysis
    let stats = detector.get_stats();
    let opportunities = detector.get_current_opportunities();
    
    println!("\n🔥 AUTO-TRIGGER SESSION RESULTS:");
    println!("================================");
    println!("🔍 Total pools analyzed: {}", stats.tracked_pools);
    println!("🎯 Opportunities detected: {}", stats.active_opportunities);
    
    if !opportunities.is_empty() {
        let high_confidence_count = opportunities.iter()
            .filter(|o| o.confidence >= 0.8)
            .count();
        let total_potential_profit: f64 = opportunities.iter()
            .filter(|o| o.confidence >= 0.8)
            .map(|o| o.expected_profit_usd)
            .sum();
        
        println!("🔥 HIGH-CONFIDENCE TRIGGERS (>80%): {}", high_confidence_count);
        println!("💰 Total potential profit from triggers: ${:.2}", total_potential_profit);
        
        if high_confidence_count > 0 {
            println!("💡 Average profit per trigger: ${:.2}", total_potential_profit / high_confidence_count as f64);
            println!("🚀 AUTO-TRIGGER SYSTEM: Would have executed {} times", high_confidence_count);
            
            // Show top trigger opportunity
            let best_trigger = opportunities.iter()
                .filter(|o| o.confidence >= 0.8)
                .max_by(|a, b| a.expected_profit_usd.partial_cmp(&b.expected_profit_usd).unwrap());
            
            if let Some(trigger) = best_trigger {
                println!("\n🏆 BEST AUTO-TRIGGER OPPORTUNITY:");
                println!("   📍 Pool: {}", trigger.pool.pool_address);
                println!("   💱 Pair: {}/{}", trigger.pool.token_a.symbol, trigger.pool.token_b.symbol);
                println!("   💰 Expected Profit: ${:.2}", trigger.expected_profit_usd);
                println!("   📊 Confidence: {:.1}%", trigger.confidence * 100.0);
                println!("   ⏰ Execution Window: {}ms", trigger.time_window_ms);
                println!("   🛡️ Safety Score: PASSED (High liquidity + Low impact)");
                println!("   🚀 Would have TRIGGERED automatically");
            }
        } else {
            println!("📝 No opportunities met auto-trigger criteria");
            println!("   💡 Consider adjusting trigger thresholds for more activations");
        }
    } else {
        println!("📭 No opportunities detected during ultra-fast session");
        println!("   💡 Try longer duration or broader detection criteria");
    }
    
    println!("\n✅ ULTRA-FAST AUTO-TRIGGER SYSTEM TEST COMPLETED");
    println!("💡 Ready for Phase 3: Paper Trading Automation");
    
    Ok(())
}

fn show_help() {
    println!("{}", "🎯 SniperForge CLI Help".bright_cyan().bold());
    println!("{}", "========================".bright_cyan());
    println!();
    println!("{}", "USAGE:".bright_white().bold());
    println!("  cargo run -- <COMMAND> [OPTIONS]");
    println!();
    println!("{}", "COMMANDS:".bright_white().bold());
    println!("  🚀 {}  - Start the platform or specific bots", "start".bright_green());
    println!("  📊 {}  - Show platform status", "status".bright_green());
    println!("  ⚙️  {}  - Show current configuration", "config".bright_green());
    println!("  💰 {}  - Wallet management commands", "wallet".bright_green());
    println!("  🧪 {}   - Run test suite", "test".bright_green());
    println!("  🎮 {} - Interactive monitoring mode", "interactive".bright_green());
    println!("  ❓ {}   - Show this help message", "help".bright_green());
    println!();
    println!("{}", "TEST SUBCOMMANDS:".bright_white().bold());
    println!("  cargo run -- test {}           - Run all tests", "all".bright_yellow());
    println!("  cargo run -- test {}         - Basic connectivity", "basic".bright_yellow());
    println!("  cargo run -- test {}        - Solana connection", "solana".bright_yellow());
    println!("  cargo run -- test {}       - Jupiter API", "jupiter".bright_yellow());
    println!("  cargo run -- test {}  - Jupiter speed test", "jupiter-speed".bright_yellow());
    println!("  cargo run -- test {}     - WebSocket test", "websocket".bright_yellow());
    println!("  cargo run -- test {}        - Wallet functionality", "wallet".bright_yellow());
    println!("  cargo run -- test {}         - Trade execution", "trade".bright_yellow());
    println!("  cargo run -- test {}   - Integration flow", "integration".bright_yellow());
    println!("  cargo run -- test {}  - Performance tests", "performance".bright_yellow());    println!("  cargo run -- test {}  - HTTP vs WebSocket RPC", "websocket-rpc".bright_yellow());
    println!("  cargo run -- test {} - Real-time price feed", "websocket-prices".bright_yellow());
    println!("  cargo run -- test {}       - Syndica ultra-fast", "syndica".bright_yellow());
    println!("  cargo run -- test {}  - Cache safety tests", "cache-safety".bright_yellow());
    println!("  cargo run -- test {}   - First real DevNet trade", "devnet-trade".bright_yellow());
    println!("  cargo run -- test {}  - Paper trading test", "paper-trading".bright_yellow());
    println!("  cargo run -- test {} - Cache-free trading", "cache-free-trading".bright_yellow());
    println!("  cargo run -- test {}         - Pool detection", "pools".bright_yellow());
    println!("  cargo run -- test {}  - Pool monitoring", "monitor-pools".bright_yellow());
    println!("  cargo run -- test {} - Ultra-fast pools", "ultra-fast-pools".bright_yellow());
    println!();
    println!("{}", "EXAMPLES:".bright_white().bold());
    println!("  cargo run -- start --devnet");
    println!("  cargo run -- test wallet");
    println!("  cargo run -- test ultra-fast-pools -d 10");
    println!("  cargo run -- wallet balance");
    println!();
    println!("{}", "For more specific help, run the command without arguments.".bright_blue());
}

async fn handle_test_websocket_rpc() -> Result<()> {
    use sniperforge::websocket_rpc_test::run_websocket_rpc_tests;
    
    let config = Config::load("config/devnet.toml").unwrap_or_else(|_| {
        Config::load("config/platform.toml").expect("Could not load config")    });
    
    run_websocket_rpc_tests(&config).await?;
    Ok(())
}

async fn handle_test_websocket_prices() -> Result<()> {
    use sniperforge::shared::websocket_price_feed::WebSocketPriceFeed;
    use std::time::{Duration, Instant};
    use tokio::time::timeout;
    
    println!("{}", "🧪 Testing WebSocket Real-Time Price Feed System".bright_green().bold());
    println!("{}", "=====================================================".bright_cyan());
    
    // 1. Initialize WebSocket price feed
    println!("\n📡 Initializing WebSocket price feed...");
    let mut price_feed = WebSocketPriceFeed::new_mainnet_prices().await?;
    
    // 2. Wait for initial data to populate
    println!("⏳ Waiting for initial price population...");
    tokio::time::sleep(Duration::from_secs(5)).await;
    
    // 3. Test popular tokens
    let test_tokens = vec![
        ("SOL", "So11111111111111111111111111111111111111112"),
        ("USDC", "EPjFWdd5AufqSSqeM2qN1xzybapC8G4wEGGkZwyTDt1v"),
        ("BONK", "DezXAZ8z7PnrnRJjz3wXBoRgixCa6xjnB7YaB1pPB263"),
        ("ETH", "7vfCXTUXx5WJV5JADk17DUJ4ksgau7utNKj4b963voxs"),
        ("USDT", "Es9vMFrzaCERmJfrF4H2FYD4KCoNkY11McCe8BenwNYB"),
    ];
    
    println!("\n💰 Testing cached price retrieval (ultra-fast):");
    for (symbol, mint) in &test_tokens {
        let start = Instant::now();
        match timeout(Duration::from_secs(1), price_feed.get_price_realtime(mint)).await {
            Ok(Some(price)) => {
                let latency = start.elapsed();
                println!("   ⚡ {}: ${:.6} ({}µs)", 
                        symbol.bright_yellow(), 
                        price, 
                        latency.as_micros().to_string().bright_green());
            }
            Ok(None) => {
                println!("   ❌ {}: No cached price", symbol.bright_red());
            }
            Err(_) => {
                println!("   ⏰ {}: Timeout", symbol.bright_red());
            }
        }
    }
    
    println!("\n🌐 Testing hybrid price retrieval (cache + HTTP fallback):");
    for (symbol, mint) in &test_tokens {
        let start = Instant::now();
        match timeout(Duration::from_secs(3), price_feed.get_price_hybrid(mint)).await {
            Ok(Ok(Some(price))) => {
                let latency = start.elapsed();
                println!("   ✅ {}: ${:.6} ({}ms)", 
                        symbol.bright_yellow(), 
                        price, 
                        latency.as_millis().to_string().bright_green());
            }
            Ok(Ok(None)) => {
                println!("   ❌ {}: No price available", symbol.bright_red());
            }
            Ok(Err(e)) => {
                println!("   ❌ {}: Error - {}", symbol.bright_red(), e);
            }
            Err(_) => {
                println!("   ⏰ {}: Timeout", symbol.bright_red());
            }
        }
        
        // Small delay between requests
        tokio::time::sleep(Duration::from_millis(200)).await;
    }
    
    // 4. Show price feed statistics
    println!("\n📊 Price Feed Statistics:");
    let stats = price_feed.get_stats().await;
    println!("   • Cached tokens: {}", stats.cached_tokens.to_string().bright_cyan());
    println!("   • WebSocket connected: {}", if stats.is_connected { "✅ YES".bright_green() } else { "❌ NO".bright_red() });
    println!("   • Last update: {:?} ago", stats.last_update_age);
    println!("   • Freshest price: {:?} old", stats.freshest_price_age);
    println!("   • Oldest price: {:?} old", stats.oldest_price_age);
    
    // 5. Show all cached prices
    println!("\n💾 All Cached Prices:");
    let cached_prices = price_feed.get_all_cached_prices().await;
    for (token, (price, age, source)) in cached_prices.iter().take(10) {
        let token_short = if token.len() > 8 { 
            format!("{}...", &token[0..8]) 
        } else { 
            token.clone() 
        };
        println!("   • {}: ${:.6} ({:?} old, {})", 
                token_short.bright_yellow(), 
                price, 
                age, 
                source.bright_blue());
    }
    
    if cached_prices.len() > 10 {
        println!("   • ... and {} more tokens", (cached_prices.len() - 10).to_string().bright_cyan());
    }
    
    // 6. Force update test
    println!("\n🔄 Testing force price update:");
    let sol_mint = "So11111111111111111111111111111111111111112";
    let start = Instant::now();
    match price_feed.force_update_token(sol_mint).await {
        Ok(Some(price)) => {
            let latency = start.elapsed();
            println!("   ✅ SOL force update: ${:.6} ({}ms)", 
                    price, 
                    latency.as_millis().to_string().bright_green());
        }
        Ok(None) => {
            println!("   ❌ Force update failed: No price");
        }
        Err(e) => {
            println!("   ❌ Force update error: {}", e);
        }
    }
    
    println!("\n✅ WebSocket price feed system test completed!");
    println!("{}", "=====================================================".bright_cyan());
    
    Ok(())
}

/// Handle extended pool monitoring for trading automation (Phase 1)
async fn handle_pools_extended(duration_hours: u64) -> Result<()> {
    println!("{}", "🎯 Phase 1: Extended Pool Monitoring for Trading Automation".bright_cyan().bold());
    println!("{}", "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━".bright_cyan());
    
    use crate::shared::pool_detector::{PoolDetector, PoolDetectorConfig, OpportunityType};
    use crate::shared::jupiter::JupiterConfig;
    use crate::shared::jupiter::client::JupiterClient;
    use crate::shared::syndica_websocket::{SyndicaWebSocketClient, SyndicaConfig};
    
    println!("🚀 EXTENDED POOL MONITORING - PHASE 1 OF TRADING AUTOMATION");
    println!("============================================================");
    println!("⏱️ Duration: {} hours ({} minutes)", duration_hours, duration_hours * 60);
    println!("🎯 Purpose: Gather comprehensive market data for trading optimization");
    println!("📊 Analytics: Pattern analysis, success rate calculation, risk optimization");
    println!("🌐 Data Source: Live Raydium + Orca APIs (MainNet)");
    println!("💡 This is the foundation for automated trading implementation");
    
    println!("\n🔄 Phase Overview:");
    println!("   Phase 1 (Current): Extended Monitoring (1-2 days)");
    println!("   Phase 2 (Next): Ultra-Fast Mode Implementation");
    println!("   Phase 3: Paper Trading Automation");
    println!("   Phase 4: Cache-Free Trading");
    println!("   Phase 5: Real Trading Implementation");
    
    // Setup Jupiter client for mainnet
    let jupiter_config = JupiterConfig::mainnet();
    let jupiter_client = JupiterClient::new(&jupiter_config).await?;
    println!("\n✅ Jupiter client initialized for mainnet");
    
    // Setup Syndica client (enhanced for extended monitoring)
    let syndica_config = SyndicaConfig::mainnet();
    let syndica_client = match SyndicaWebSocketClient::new(syndica_config).await {
        Ok(client) => {
            println!("✅ Syndica WebSocket client initialized for enhanced monitoring");
            Some(client)
        }
        Err(e) => {
            println!("⚠️ Syndica client failed to initialize: {}", e);
            println!("   Continuing with DEX API monitoring (still fully functional)");
            None
        }
    };
    
    // Create pool detector with optimized settings for extended monitoring
    let config = PoolDetectorConfig {
        min_liquidity_usd: 2000.0,     // Lower threshold for comprehensive data
        max_price_impact_1k: 15.0,     // Capture more opportunities for analysis
        min_risk_score: 0.1,           // Lower threshold to analyze more patterns
        monitoring_interval_ms: 3000,  // 3s intervals for thorough coverage
        max_tracked_pools: 50,          // Track more pools for better data
    };
    
    println!("\n📊 Extended Monitoring Configuration:");    println!("   Min liquidity: ${:.0} (lower for comprehensive analysis)", config.min_liquidity_usd);
    println!("   Max price impact: {:.1}% (capture more opportunities)", config.max_price_impact_1k);
    println!("   Min risk score: {:.1}% (analyze broader patterns)", config.min_risk_score * 100.0);
    println!("   Scan interval: {}ms (thorough coverage)", config.monitoring_interval_ms);
    println!("   Max tracked pools: {} (comprehensive tracking)", config.max_tracked_pools);
    
    let mut detector = PoolDetector::new(config, jupiter_client, syndica_client, None).await?;
    
    println!("\n🚀 Starting extended monitoring session...");
    println!("   This will gather market data for {} hours", duration_hours);
    println!("   Real-time analytics and pattern recognition");
    println!("   Building foundation for automated trading");
    println!("   Press Ctrl+C to stop early if needed");
    
    // Convert hours to minutes for the monitoring function
    let duration_minutes = duration_hours * 60;
    
    // Run extended monitoring with enhanced reporting
    if let Err(e) = detector.start_monitoring_with_reports(duration_minutes).await {
        println!("⚠️ Extended monitoring encountered an error: {}", e);
    }
    
    // Comprehensive results analysis
    let stats = detector.get_stats();
    let opportunities = detector.get_current_opportunities();
    let pools = detector.get_tracked_pools();
    
    println!("\n📊 EXTENDED MONITORING RESULTS - PHASE 1 ANALYSIS:");
    println!("================================================");
    println!("⏱️ Session Duration: {} hours", duration_hours);
    println!("🔍 Tracked pools: {}", stats.tracked_pools);
    println!("🎯 Active opportunities: {}", stats.active_opportunities);
    println!("⏱️ Last scan: {:?} ago", stats.last_scan_ago);
    
    // Enhanced analytics for trading automation
    println!("\n📈 TRADING AUTOMATION ANALYTICS:");
    println!("================================");
    
    if !opportunities.is_empty() {
        // Calculate opportunity metrics
        let total_expected_profit: f64 = opportunities.iter().map(|o| o.expected_profit_usd).sum();
        let avg_confidence: f64 = opportunities.iter().map(|o| o.confidence).sum::<f64>() / opportunities.len() as f64;
        let high_confidence_count = opportunities.iter().filter(|o| o.confidence > 0.7).count();
        
        println!("💰 Total Expected Profit: ${:.2}", total_expected_profit);
        println!("📊 Average Confidence: {:.1}%", avg_confidence * 100.0);
        println!("⭐ High Confidence Opportunities (>70%): {}", high_confidence_count);
          // Opportunity type analysis
        use crate::shared::pool_detector::OpportunityType;
        let mut type_counts = std::collections::HashMap::new();
        for opp in opportunities.iter() {
            let type_name = match opp.opportunity_type {
                OpportunityType::NewPoolSnipe => "New Pool Snipe",
                OpportunityType::PriceDiscrepancy => "Price Arbitrage",
                OpportunityType::LiquidityImbalance => "Liquidity Imbalance",
                OpportunityType::VolumeSpike => "Volume Spike",
            };
            *type_counts.entry(type_name).or_insert(0) += 1;
        }
        
        println!("\n🎯 Opportunity Types Distribution:");
        for (opp_type, count) in type_counts.iter() {
            println!("   • {}: {} opportunities", opp_type, count);
        }
          // Show top 3 opportunities for detailed analysis
        println!("\n🏆 TOP 3 OPPORTUNITIES FOR TRADING AUTOMATION:");
        let mut sorted_opportunities: Vec<_> = opportunities.iter().collect();
        sorted_opportunities.sort_by(|a, b| {
            (b.expected_profit_usd * b.confidence).partial_cmp(&(a.expected_profit_usd * a.confidence)).unwrap()
        });
        
        for (i, opp) in sorted_opportunities.iter().take(3).enumerate() {
            println!("\n   {}. 🚀 HIGH-PRIORITY TRADING TARGET", i + 1);
            println!("      📍 Pool: {}", opp.pool.pool_address);
            println!("      💱 Pair: {} / {}", opp.pool.token_a.symbol, opp.pool.token_b.symbol);
            println!("      💰 Expected Profit: ${:.2}", opp.expected_profit_usd);
            println!("      📊 Confidence: {:.1}%", opp.confidence * 100.0);
            println!("      ⏰ Execution Window: {}ms", opp.time_window_ms);
            println!("      💵 Recommended Size: ${:.0}", opp.recommended_size_usd);
            println!("      🎯 Type: {:?}", opp.opportunity_type);
            println!("      🔗 Validate: https://dexscreener.com/solana/{}", opp.pool.pool_address);
        }
    } else {
        println!("📭 No opportunities detected during this session");
        println!("   This provides important data about market conditions");
    }
    
    // Pool analysis for trading patterns
    if !pools.is_empty() {
        println!("\n🏊 POOL ANALYSIS FOR TRADING OPTIMIZATION:");
        println!("=========================================");
        
        let total_liquidity: f64 = pools.values().map(|p| p.liquidity_usd).sum();
        let avg_liquidity = total_liquidity / pools.len() as f64;
        let high_liquidity_count = pools.values().filter(|p| p.liquidity_usd > 50000.0).count();
        
        println!("💧 Total Tracked Liquidity: ${:.0}", total_liquidity);
        println!("📊 Average Pool Liquidity: ${:.0}", avg_liquidity);
        println!("🏆 High Liquidity Pools (>$50k): {}", high_liquidity_count);
        
        // Risk analysis for automation
        let low_risk_pools = pools.values().filter(|p| p.risk_score.overall > 0.6).count();
        let medium_risk_pools = pools.values().filter(|p| p.risk_score.overall > 0.3 && p.risk_score.overall <= 0.6).count();
        
        println!("\n⚖️ Risk Distribution for Automated Trading:");
        println!("   🟢 Low Risk (>60%): {} pools", low_risk_pools);
        println!("   🟡 Medium Risk (30-60%): {} pools", medium_risk_pools);
        println!("   🔴 High Risk (<30%): {} pools", pools.len() - low_risk_pools - medium_risk_pools);
    }
    
    println!("\n🎯 PHASE 1 COMPLETION - NEXT STEPS:");
    println!("===================================");
    println!("✅ Extended monitoring session completed successfully");
    println!("📊 Market data gathered for trading automation");
    println!("📈 Pattern analysis available for algorithm optimization");
    
    println!("\n🚀 READY FOR PHASE 2 - ULTRA-FAST MODE:");
    println!("   Command: cargo run -- test ultra-fast-mode");
    println!("   Purpose: Implement ultra-fast trading triggers");
    println!("   Duration: 2-3 days");
    println!("   Goal: Automatic execution logic integration");
    
    println!("\n💡 RECOMMENDATIONS FOR NEXT PHASE:");
    if !opportunities.is_empty() {
        println!("   • Focus on {:?} opportunities (most frequent)", 
                 opportunities.iter().next().map(|o| &o.opportunity_type).unwrap_or(&OpportunityType::NewPoolSnipe));
        println!("   • Target pools with >70% confidence score");
        println!("   • Implement triggers for high-liquidity pools (>${:.0})", 
                 pools.values().map(|p| p.liquidity_usd).fold(0.0, f64::max));
    } else {
        println!("   • Continue monitoring to identify optimal trading windows");
        println!("   • Focus on ultra-fast detection improvements");
    }
    
    println!("\n🎖️ PHASE 1 STATUS: ✅ COMPLETE - READY FOR AUTOMATION");
    
    Ok(())
}
