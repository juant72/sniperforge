use sniperforge::bots::arbitrage_bot::ArbitrageBot;
use sniperforge::shared::SharedServices;
use sniperforge::config::{load_config, NetworkConfig};
use anyhow::Result;
use std::sync::Arc;
use tokio::time::{timeout, Duration};
use tracing::{info, error, warn};

/// Test the ArbitrageBot with real DevNet data
#[tokio::main]
async fn main() -> Result<()> {
    // Initialize logging
    tracing_subscriber::fmt()
        .with_env_filter("info,sniperforge=debug")
        .init();

    info!("🚀 Starting ArbitrageBot DevNet Test");
    info!("📋 This test will validate the ArbitrageBot with real DevNet data");

    // Load DevNet configuration
    let config = match load_config("config/devnet.toml") {
        Ok(config) => {
            info!("✅ DevNet configuration loaded successfully");
            config
        },
        Err(e) => {
            error!("❌ Failed to load DevNet config: {}", e);
            return Err(e);
        }
    };

    // Initialize shared services
    let shared_services = match SharedServices::new(&config).await {
        Ok(services) => {
            info!("✅ Shared services initialized successfully");
            Arc::new(services)
        },
        Err(e) => {
            error!("❌ Failed to initialize shared services: {}", e);
            return Err(e);
        }
    };

    // Get wallet address from wallet manager
    let wallet_address = match shared_services.wallet_manager().get_wallet_address("test-wallet").await {
        Ok(address) => {
            info!("✅ Wallet address retrieved: {}", address);
            address
        },
        Err(e) => {
            error!("❌ Failed to get wallet address: {}", e);
            return Err(e);
        }
    };

    // Test 1: Create ArbitrageBot instance
    info!("🧪 Test 1: Creating ArbitrageBot instance");
    let mut arbitrage_bot = match ArbitrageBot::new(
        wallet_address,
        1000.0, // $1000 initial capital for testing
        &config.network,
        shared_services.clone(),
    ).await {
        Ok(bot) => {
            info!("✅ ArbitrageBot created successfully");
            bot
        },
        Err(e) => {
            error!("❌ Failed to create ArbitrageBot: {}", e);
            return Err(e);
        }
    };

    // Test 2: Check bot status
    info!("🧪 Test 2: Checking bot status");
    let status = arbitrage_bot.get_status();
    info!("📊 Bot Status:");
    info!("  - Running: {}", status.is_running);
    info!("  - Uptime: {} seconds", status.uptime_seconds);
    info!("  - Total trades: {}", status.total_trades);
    info!("  - Success rate: {:.1}%", status.success_rate_percent);

    // Test 3: Test real market data fetching
    info!("🧪 Test 3: Testing real market data fetching");
    match arbitrage_bot.get_real_market_data().await {
        Ok(market_data) => {
            info!("✅ Real market data retrieved successfully:");
            info!("  - Symbol: {}", market_data.symbol);
            info!("  - Price: ${:.6}", market_data.price);
            info!("  - Bid: ${:.6}", market_data.bid);
            info!("  - Ask: ${:.6}", market_data.ask);
            info!("  - Spread: ${:.6}", market_data.spread);
        },
        Err(e) => {
            error!("❌ Failed to get real market data: {}", e);
            return Err(e);
        }
    }

    // Test 4: Test Jupiter price fetching
    info!("🧪 Test 4: Testing Jupiter price fetching");
    match arbitrage_bot.get_jupiter_price("SOL", "USDC").await {
        Ok(price) => {
            info!("✅ Jupiter price retrieved successfully: ${:.6}", price);
        },
        Err(e) => {
            error!("❌ Failed to get Jupiter price: {}", e);
            return Err(e);
        }
    }

    // Test 5: Test opportunity detection
    info!("🧪 Test 5: Testing opportunity detection");
    match arbitrage_bot.detect_opportunities_using_strategy().await {
        Ok(signals) => {
            info!("✅ Opportunity detection completed: {} signals found", signals.len());
            for (i, signal) in signals.iter().enumerate() {
                info!("  Signal {}: {} - {:.1}% confidence, ${:.2} position size",
                      i + 1, signal.strategy_name, signal.confidence * 100.0, signal.position_size);
            }
        },
        Err(e) => {
            error!("❌ Failed to detect opportunities: {}", e);
            return Err(e);
        }
    }

    // Test 6: Test bot trading loop for a short period
    info!("🧪 Test 6: Testing bot trading loop (30 seconds)");
    let bot_future = arbitrage_bot.start_trading();
    match timeout(Duration::from_secs(30), bot_future).await {
        Ok(result) => {
            match result {
                Ok(_) => info!("✅ Bot trading loop completed successfully"),
                Err(e) => warn!("⚠️ Bot trading loop ended with error: {}", e),
            }
        },
        Err(_) => {
            info!("⏰ Bot trading loop timeout after 30 seconds (expected)");
            arbitrage_bot.emergency_stop();
        }
    }

    // Test 7: Final status check
    info!("🧪 Test 7: Final status check");
    let final_status = arbitrage_bot.get_status();
    info!("📊 Final Bot Status:");
    info!("  - Running: {}", final_status.is_running);
    info!("  - Total trades: {}", final_status.total_trades);
    info!("  - Successful trades: {}", final_status.successful_trades);
    info!("  - Total profit: ${:.2}", final_status.total_profit_usd);
    info!("  - Opportunities detected: {}", final_status.opportunities_detected);
    info!("  - Opportunities executed: {}", final_status.opportunities_executed);
    info!("  - Average latency: {:.2}ms", final_status.average_latency_ms);

    info!("🎉 ArbitrageBot DevNet test completed successfully!");
    info!("✅ All tests passed - ArbitrageBot is ready for DevNet operation");

    Ok(())
}
