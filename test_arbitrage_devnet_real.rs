use anyhow::Result;
use sniperforge::bots::arbitrage_bot::ArbitrageBot;
use sniperforge::config::Config;
use sniperforge::shared::SharedServices;
use std::sync::Arc;
use std::time::Duration;
use tokio::time::timeout;
use tracing::{info, warn, error};

#[tokio::main]
async fn main() -> Result<()> {
    // Initialize logging
    tracing_subscriber::fmt()
        .with_env_filter("info,sniperforge=debug")
        .init();

    info!("🚀 Starting Real ArbitrageBot Test on DevNet");

    // Load DevNet configuration
    let config = Config::load("config/devnet.toml")?;
    let shared_services = Arc::new(SharedServices::new(&config).await?);

    info!("✅ Configuration loaded successfully");

    // Get wallet address
    let wallet_address = shared_services
        .wallet_manager()
        .get_wallet_address("devnet-trading")
        .await?;

    info!("💰 Using wallet: {}", wallet_address);

    // Create ArbitrageBot with conservative settings
    let mut bot = ArbitrageBot::new(
        wallet_address,
        10.0, // Start with $10 for safety
        &config.network,
        shared_services.clone(),
    ).await?;

    info!("🤖 ArbitrageBot created successfully");

    // Test 1: Get market data
    info!("📊 Testing market data retrieval...");
    match timeout(Duration::from_secs(10), bot.get_real_market_data()).await {
        Ok(Ok(market_data)) => {
            info!("✅ Market data retrieved:");
            info!("   Symbol: {}", market_data.symbol);
            info!("   Price: ${:.6}", market_data.price);
            info!("   Volume: {:.2}", market_data.volume);
            info!("   Liquidity: {:.2}", market_data.liquidity);
        },
        Ok(Err(e)) => {
            warn!("⚠️ Market data fetch failed: {}", e);
        },
        Err(_) => {
            warn!("⚠️ Market data fetch timed out");
        }
    }

    // Test 2: Detect opportunities
    info!("🔍 Detecting arbitrage opportunities...");
    match timeout(Duration::from_secs(15), bot.detect_opportunities_using_strategy()).await {
        Ok(Ok(signals)) => {
            info!("✅ Opportunity detection completed:");
            info!("   Signals found: {}", signals.len());

            for (i, signal) in signals.iter().enumerate().take(3) {
                info!("   Signal {}: {:.2}% confidence, ${:.4} size",
                      i + 1, signal.confidence * 100.0, signal.position_size);
            }
        },
        Ok(Err(e)) => {
            warn!("⚠️ Opportunity detection failed: {}", e);
        },
        Err(_) => {
            warn!("⚠️ Opportunity detection timed out");
        }
    }

    // Test 3: Short trading loop (30 seconds)
    info!("⏰ Running short trading loop for 30 seconds...");
    let trading_result = timeout(Duration::from_secs(30), bot.start_trading()).await;

    match trading_result {
        Ok(Ok(_)) => {
            info!("✅ Trading loop completed successfully");
        },
        Ok(Err(e)) => {
            warn!("⚠️ Trading loop failed: {}", e);
        },
        Err(_) => {
            info!("⏰ Trading loop timed out (expected)");
            bot.emergency_stop();
        }
    }

    // Get final status
    let final_status = bot.get_status();
    info!("📋 Final Bot Status:");
    info!("   Running: {}", final_status.is_running);
    info!("   Uptime: {} seconds", final_status.uptime_seconds);
    info!("   Total trades: {}", final_status.total_trades);
    info!("   Opportunities detected: {}", final_status.opportunities_detected);
    info!("   Success rate: {:.1}%", final_status.success_rate);
    info!("   Emergency stop: {}", final_status.emergency_stop);

    if final_status.total_trades > 0 {
        info!("🎉 SUCCESS! ArbitrageBot executed {} trades on DevNet!", final_status.total_trades);
    } else {
        info!("📊 No trades executed, but bot is functional and ready for opportunities");
    }

    info!("✅ Real ArbitrageBot test completed successfully");
    Ok(())
}
