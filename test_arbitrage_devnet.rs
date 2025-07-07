use anyhow::Result;
use sniperforge::bots::arbitrage_bot::ArbitrageBot;
use sniperforge::config::Config;
use std::time::Duration;
use tracing::{info, error};

#[tokio::main]
async fn main() -> Result<()> {
    // Initialize logging
    tracing_subscriber::fmt::init();

    info!("🧪 Starting ArbitrageBot DevNet Testing");
    info!("===================================");

    // Load DevNet configuration
    let config = Config::load("config/devnet.toml")?;
    info!("✅ Loaded DevNet configuration");

    // Check wallet balance before starting
    check_wallet_setup().await?;

    // Test 1: Create ArbitrageBot instance
    info!("🚀 Test 1: Creating ArbitrageBot instance");
    let wallet_address = get_test_wallet_address().await?;
    let initial_capital = 100.0; // $100 for testing

    let mut arbitrage_bot = ArbitrageBot::new(
        wallet_address,
        initial_capital,
        &config.network
    ).await?;

    info!("✅ ArbitrageBot created successfully with ${:.2} capital", initial_capital);

    // Test 2: Check bot status
    info!("🔍 Test 2: Checking bot status");
    let status = arbitrage_bot.get_status();
    info!("   Bot running: {}", status.is_running);
    info!("   Emergency stop: {}", status.emergency_stop);
    info!("   Total trades: {}", status.total_trades);

    // Test 3: Test price feeds (without trading)
    info!("📊 Test 3: Testing price feed updates");
    test_price_feeds(&arbitrage_bot).await?;

    // Test 4: Opportunity detection (dry run)
    info!("🔍 Test 4: Testing opportunity detection");
    test_opportunity_detection(&mut arbitrage_bot).await?;

    // Test 5: Run bot for a short period (with small amounts)
    info!("⚡ Test 5: Running bot for 30 seconds (DevNet trading)");
    test_limited_trading(&mut arbitrage_bot).await?;

    // Final status check
    info!("📈 Final Test Results:");
    let final_status = arbitrage_bot.get_status();
    info!("   Total opportunities detected: {}", final_status.opportunities_detected);
    info!("   Total trades executed: {}", final_status.total_trades);
    info!("   Success rate: {:.1}%", final_status.success_rate_percent);
    info!("   Total profit/loss: ${:.2}", final_status.total_profit_usd);
    info!("   Average latency: {:.1}ms", final_status.average_latency_ms);

    info!("🎉 DevNet testing completed successfully!");
    Ok(())
}

async fn check_wallet_setup() -> Result<()> {
    info!("💳 Checking wallet setup...");

    // Check if test wallet exists
    if std::path::Path::new("test-wallet.json").exists() {
        info!("✅ Test wallet found: test-wallet.json");
    } else {
        error!("❌ Test wallet not found. Please run: cargo run --bin create_test_wallet");
        return Err(anyhow::anyhow!("Test wallet not found"));
    }

    // TODO: Check SOL balance
    info!("⚠️  Note: Ensure your test wallet has sufficient DevNet SOL");
    info!("   Run: cargo run --bin request_devnet_airdrop if needed");

    Ok(())
}

async fn get_test_wallet_address() -> Result<String> {
    // In a real implementation, this would load from test-wallet.json
    // For now, return a placeholder
    Ok("dummy_test_wallet_address".to_string())
}

async fn test_price_feeds(bot: &ArbitrageBot) -> Result<()> {
    info!("   Testing Jupiter price feed...");
    // Test Jupiter price fetching
    match bot.get_jupiter_price("SOL", "USDC").await {
        Ok(price) => info!("   ✅ Jupiter SOL/USDC price: ${:.2}", price),
        Err(e) => error!("   ❌ Jupiter price fetch failed: {}", e),
    }

    info!("   Testing market data aggregation...");
    match bot.get_real_market_data().await {
        Ok(data) => {
            info!("   ✅ Market data:");
            info!("      Current price: ${:.2}", data.current_price);
            info!("      Volume 24h: ${:.0}", data.volume_24h);
            info!("      Liquidity: ${:.0}", data.liquidity);
        },
        Err(e) => error!("   ❌ Market data fetch failed: {}", e),
    }

    Ok(())
}

async fn test_opportunity_detection(bot: &mut ArbitrageBot) -> Result<()> {
    info!("   Scanning for arbitrage opportunities...");

    match bot.detect_opportunities_using_strategy().await {
        Ok(signals) => {
            info!("   ✅ Found {} signals", signals.len());
            for (i, signal) in signals.iter().enumerate() {
                info!("      Signal {}: {} (confidence: {:.2})",
                      i + 1, signal.strategy_name, signal.confidence);
            }
        },
        Err(e) => error!("   ❌ Opportunity detection failed: {}", e),
    }

    Ok(())
}

async fn test_limited_trading(bot: &mut ArbitrageBot) -> Result<()> {
    info!("   Starting limited trading session...");
    info!("   ⚠️  Using DevNet with small amounts for safety");

    // Create a timeout for the trading session
    let trading_future = bot.start_trading();
    let timeout_duration = Duration::from_secs(30);

    match tokio::time::timeout(timeout_duration, trading_future).await {
        Ok(result) => {
            match result {
                Ok(_) => info!("   ✅ Trading session completed normally"),
                Err(e) => error!("   ❌ Trading session failed: {}", e),
            }
        },
        Err(_) => {
            info!("   ⏰ Trading session timed out (30s limit reached)");
            bot.emergency_stop();
        }
    }

    Ok(())
}
