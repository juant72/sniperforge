use anyhow::Result;
use sniperforge::bots::arbitrage_bot::ArbitrageBot;
use sniperforge::config::Config;
use std::time::Duration;
use tracing::{info, warn, error};

#[tokio::main]
async fn main() -> Result<()> {
    // Initialize logging
    tracing_subscriber::fmt::init();

    info!("🧪 SIMPLE ArbitrageBot DevNet Test");
    info!("==================================");

    // Load DevNet configuration
    match Config::load("config/devnet.toml") {
        Ok(config) => {
            info!("✅ Loaded DevNet configuration");

            // Test 1: Create ArbitrageBot instance
            info!("🚀 Test 1: Creating ArbitrageBot instance");
            let wallet_address = "dummy_test_wallet".to_string(); // Temporary for this test
            let initial_capital = 10.0; // Very small amount for testing

            match ArbitrageBot::new(wallet_address, initial_capital, &config.network).await {
                Ok(arbitrage_bot) => {
                    info!("✅ ArbitrageBot created successfully with ${:.2} capital", initial_capital);

                    // Test 2: Check bot status
                    info!("🔍 Test 2: Checking bot status");
                    let status = arbitrage_bot.get_status();
                    info!("   Bot running: {}", status.is_running);
                    info!("   Emergency stop: {}", status.emergency_stop);
                    info!("   Total trades: {}", status.total_trades);
                    info!("   Success rate: {:.1}%", status.success_rate_percent);
                    info!("   Total profit: ${:.2}", status.total_profit_usd);
                    info!("   Opportunities detected: {}", status.opportunities_detected);
                    info!("   Average latency: {:.1}ms", status.average_latency_ms);

                    info!("✅ ArbitrageBot basic functionality test completed");
                    info!("📋 Next steps: Set up real wallet and run full trading test");
                }
                Err(e) => {
                    error!("❌ Failed to create ArbitrageBot: {}", e);
                    error!("💡 Make sure DevNet configuration is correct");
                }
            }
        }
        Err(e) => {
            error!("❌ Failed to load DevNet config: {}", e);
            warn!("💡 Make sure config/devnet.toml exists and is properly configured");

            // Show basic info about what's needed
            info!("📋 For DevNet testing you need:");
            info!("   1. config/devnet.toml with proper RPC endpoints");
            info!("   2. Test wallet with some DevNet SOL");
            info!("   3. Network connectivity to DevNet");
        }
    }

    Ok(())
}
