#![allow(dead_code)]
#![allow(unused_imports)]

use anyhow::Result;
use dotenv::dotenv;
use tracing::info;

pub mod bots;
mod cache_safety_test;
mod cli;
pub mod config;
mod jupiter_speed_test;
pub mod platform;
pub mod shared;
pub mod types;

use config::Config;

#[tokio::main]
async fn main() -> Result<()> {
    // Check for help arguments before processing
    let args: Vec<String> = std::env::args().collect();
    if args.len() > 1 {
        match args[1].as_str() {
            "--help" | "-h" | "help" => {
                show_help();
                return Ok(());
            }
            "--version" | "-V" => {
                println!("SniperForge CLI 0.1.0");
                return Ok(());
            }
            _ => {}
        }

        // Check for subcommand help
        if args.len() > 2 && (args[2] == "--help" || args[2] == "-h") {
            show_subcommand_help(&args[1]);
            return Ok(());
        }

        // Check for sub-subcommand help (e.g., test swap-real --help)
        if args.len() > 3 && (args[3] == "--help" || args[3] == "-h") {
            show_subsubcommand_help(&args[1], &args[2]);
            return Ok(());
        }
    }

    // Load environment variables from .env file
    dotenv().ok();

    // Initialize rustls crypto provider first
    init_crypto_provider();

    // Initialize logging
    init_logging()?;

    info!("🚀 Starting SniperForge Multi-Bot Platform v0.1.0");

    // Always use the modern CLI system
    cli::run_cli().await
}

fn init_logging() -> Result<()> {
    use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};

    let file_appender = tracing_appender::rolling::daily("logs", "sniperforge.log");
    let (non_blocking, _guard) = tracing_appender::non_blocking(file_appender);

    tracing_subscriber::registry()
        .with(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| "sniperforge=info,warn".into()),
        )
        .with(tracing_subscriber::fmt::layer().with_target(false))
        .with(
            tracing_subscriber::fmt::layer()
                .with_writer(non_blocking)
                .with_ansi(false),
        )
        .init();

    // Keep the guard alive for the duration of the program
    std::mem::forget(_guard);
    Ok(())
}

fn init_crypto_provider() {
    // Initialize rustls default crypto provider to fix:
    // "no process-level CryptoProvider available"

    eprintln!("🔐 Setting up crypto provider for TLS connections...");

    // For rustls 0.23+, we need to explicitly install a crypto provider
    // This MUST be done once at program startup before any TLS operations

    // Try to install the ring crypto provider
    let result = rustls::crypto::ring::default_provider().install_default();

    match result {
        Ok(()) => {
            eprintln!("✅ Ring crypto provider installed successfully");
        }
        Err(_) => {
            // Provider was already installed, which is fine
            eprintln!("ℹ️  Crypto provider was already installed");
        }
    }

    eprintln!("✅ Crypto setup completed");
}

fn show_help() {
    println!("SniperForge CLI 0.1.0");
    println!("Interactive CLI for SniperForge Multi-Bot Platform");
    println!();
    println!("USAGE:");
    println!("    sniperforge [COMMAND] [OPTIONS]");
    println!();
    println!("COMMANDS:");
    println!("    start                 Start the platform or specific bots");
    println!("    status                Show platform status");
    println!("    config                Show current configuration");
    println!("    wallet                Wallet management commands");
    println!("    test                  Comprehensive testing suite");
    println!("    interactive           Interactive monitoring mode");
    println!("    multi-strategy-trading [PHASE 6A] Execute multiple trading strategies");
    println!("    strategy-backtest     [PHASE 6A] Backtest trading strategies");
    println!("    pattern-analysis      [PHASE 6A] Analyze market patterns");
    println!("    arbitrage-scan        [PHASE 6A] Scan for arbitrage opportunities");
    println!("    ml                    [PHASE 6B] Machine Learning and AI-powered trading");
    println!("    portfolio             [PHASE 6C] Portfolio management and analytics");
    println!();
    println!("OPTIONS:");
    println!("    -h, --help            Print help information");
    println!("    -V, --version         Print version information");
    println!();
    println!("EXAMPLES:");
    println!("    sniperforge start                              Start the platform");
    println!("    sniperforge wallet balance test-wallet.json   Check wallet balance");
    println!("    sniperforge test all                           Run all tests");
    println!("    sniperforge test swap-real --help             Get help for swap command");
    println!();
    println!("For more information on a specific command, use:");
    println!("    sniperforge [COMMAND] --help");
    println!();
    println!("🚀 Sprint 1 Status: Real data integration complete!");
    println!("   All trading operations use live data (Jupiter API + Solana RPC)");
    println!("   Ready for real transactions on DevNet and Mainnet");
}

fn show_subcommand_help(command: &str) {
    match command {
        "start" => {
            println!("sniperforge-start");
            println!("Start the platform or specific bots");
            println!();
            println!("USAGE:");
            println!("    sniperforge start [OPTIONS]");
            println!();
            println!("OPTIONS:");
            println!(
                "    -b, --bot <BOT_TYPE>    Specific bot to start (can be used multiple times)"
            );
            println!("        --devnet            Use DevNet configuration for testing");
            println!("    -h, --help              Print help information");
            println!();
            println!("EXAMPLES:");
            println!("    sniperforge start                Start full platform");
            println!("    sniperforge start --devnet       Start with DevNet config");
            println!("    sniperforge start -b lp-sniper   Start LP sniper bot only");
        }
        "wallet" => {
            println!("sniperforge-wallet");
            println!("Wallet management commands");
            println!();
            println!("USAGE:");
            println!("    sniperforge wallet [SUBCOMMAND]");
            println!();
            println!("SUBCOMMANDS:");
            println!("    balance     Check wallet balances");
            println!("    airdrop     Request SOL airdrop on DevNet");
            println!("    generate    Generate a new DevNet wallet");
            println!("    help        Print this message or the help of the given subcommand(s)");
            println!();
            println!("EXAMPLES:");
            println!("    sniperforge wallet balance test-wallet.json");
            println!("    sniperforge wallet generate --output my-wallet.json");
            println!("    sniperforge wallet airdrop");
        }
        "test" => {
            println!("sniperforge-test");
            println!("Comprehensive testing suite");
            println!();
            println!("USAGE:");
            println!("    sniperforge test [SUBCOMMAND]");
            println!();
            println!("SUBCOMMANDS:");
            println!("    all         Run all tests");
            println!("    basic       Run basic connectivity tests");
            println!("    solana      Test Solana connectivity and RPC calls");
            println!("    jupiter     Test Jupiter API integration");
            println!("    wallet      Test wallet functionality");
            println!("    websocket   Test WebSocket connectivity and subscriptions");
            println!("    trade       Test trade execution (simulation)");
            println!("    swap-real   🚀 Test REAL swap execution (Sprint 1)");
            println!("    integration Test complete integration flow");
            println!("    performance Test performance and latency");
            println!();
            println!("EXAMPLES:");
            println!("    sniperforge test all");
            println!("    sniperforge test swap-real --wallet test-wallet.json --confirm");
            println!("    sniperforge test swap-real --wallet mainnet-validation-wallet.json --network mainnet --amount 0.001 --confirm");
        }
        "ml" => {
            println!("sniperforge-ml");
            println!("[ML] PHASE 6B: Machine Learning and AI-powered trading");
            println!();
            println!("USAGE:");
            println!("    sniperforge ml [SUBCOMMAND]");
            println!();
            println!("SUBCOMMANDS:");
            println!("    analyze-patterns       Analyze market patterns using ML models");
            println!("    predict-trend          Predict price trends using ML models");
            println!(
                "    optimize-strategy      Optimize trading strategy using genetic algorithms"
            );
            println!("    backtest-optimized     Backtest optimized strategy parameters");
            println!("    assess-risk            Assess market risk using ML models");
            println!(
                "    market-regime          Detect current market regime (bull/bear/sideways)"
            );
            println!("    predict-timing         Predict optimal trade execution timing");
            println!("    optimize-execution     Optimize trade execution for large orders");
            println!("    train-models           Train or retrain ML models");
            println!("    model-status           Show ML model status and performance");
            println!("    advanced-predict       Run advanced ML ensemble prediction");
            println!("    optimize-portfolio     Optimize portfolio allocation using ML");
            println!();
            println!("EXAMPLES:");
            println!("    sniperforge ml analyze-patterns --symbol SOL/USDC --confidence 0.8");
            println!("    sniperforge ml predict-trend --horizon 15");
            println!("    sniperforge ml train-models --model all --days 30");
        }
        _ => {
            println!(
                "Help not available for '{}'. Use 'sniperforge --help' for main help.",
                command
            );
            println!("Available commands: start, status, config, wallet, test, interactive, ml, portfolio");
        }
    }
}

fn show_subsubcommand_help(command: &str, subcommand: &str) {
    match (command, subcommand) {
        ("test", "swap-real") => {
            println!("sniperforge-test-swap-real");
            println!("🚀 SPRINT 1: Test REAL swap execution");
            println!();
            println!("⚠️  WARNING: This command executes REAL transactions on blockchain!");
            println!("   - DevNet: Uses test SOL (safe for testing)");
            println!("   - Mainnet: Uses REAL SOL with monetary value");
            println!();
            println!("USAGE:");
            println!("    sniperforge test swap-real [OPTIONS]");
            println!();
            println!("OPTIONS:");
            println!("    -a, --amount <SOL>         Amount of SOL to swap (default: 0.00001)");
            println!("    -w, --wallet <FILE>        Path to wallet keypair JSON file");
            println!("        --network <NET>        Network: devnet or mainnet (default: devnet)");
            println!("        --confirm              Confirm REAL transaction execution");
            println!("    -h, --help                 Print help information");
            println!();
            println!("BEHAVIOR:");
            println!("    Without --confirm:   Simulation mode (safe, no real transaction)");
            println!("    With --confirm:      REAL transaction sent to blockchain");
            println!();
            println!("EXAMPLES:");
            println!("    # Simulation on DevNet (safe)");
            println!("    sniperforge test swap-real --wallet test-wallet.json");
            println!();
            println!("    # Real swap on DevNet (test SOL)");
            println!("    sniperforge test swap-real --wallet test-wallet.json --confirm");
            println!();
            println!("    # Real swap on Mainnet (REAL MONEY!)");
            println!("    sniperforge test swap-real --wallet mainnet-validation-wallet.json --network mainnet --amount 0.001 --confirm");
            println!();
            println!("NETWORKS:");
            println!("    devnet    Safe testing with test SOL");
            println!("    mainnet   REAL money - use with extreme caution!");
            println!();
            println!("SPRINT 1 STATUS: ✅ This command is production-ready");
            println!("   - All mock data removed");
            println!("   - Jupiter API integration complete");
            println!("   - Real transaction capability verified");
        }
        ("wallet", "balance") => {
            println!("sniperforge-wallet-balance");
            println!("Check wallet balances");
            println!();
            println!("USAGE:");
            println!("    sniperforge wallet balance [WALLET_FILE]");
            println!();
            println!("ARGUMENTS:");
            println!("    [WALLET_FILE]    Path to wallet keypair JSON file");
            println!();
            println!("EXAMPLES:");
            println!("    sniperforge wallet balance test-wallet.json");
            println!("    sniperforge wallet balance mainnet-validation-wallet.json");
            println!();
            println!("The command automatically detects the network based on the wallet filename.");
        }
        ("wallet", "generate") => {
            println!("sniperforge-wallet-generate");
            println!("Generate a new DevNet wallet");
            println!();
            println!("USAGE:");
            println!("    sniperforge wallet generate [OPTIONS]");
            println!();
            println!("OPTIONS:");
            println!("    -o, --output <FILE>    Output file for the wallet (default: test-wallet-new.json)");
            println!("    -h, --help             Print help information");
            println!();
            println!("EXAMPLES:");
            println!("    sniperforge wallet generate");
            println!("    sniperforge wallet generate --output my-custom-wallet.json");
        }
        _ => {
            println!("Help not available for '{} {}'. Use 'sniperforge {} --help' for available subcommands.", command, subcommand, command);
        }
    }
}
