use anyhow::Result;
use std::time::{Duration, Instant};
use tracing::{info, error};

/// Safe QA Test Runner - Simple version without potential loops
#[tokio::main]
async fn main() -> Result<()> {
    // Initialize logging
    tracing_subscriber::fmt()
        .with_env_filter("info,sniperforge=debug")
        .init();

    info!("🧪 Starting SniperForge ArbitrageBot QA Test Suite (Safe Mode)");
    info!("{}", "=".repeat(70));

    let start_time = Instant::now();
    let mut total_tests = 0;
    let mut total_passed = 0;
    let mut total_failed = 0;

    // Test 1: Basic Module Loading
    info!("🔍 Test 1: Module Loading...");
    match test_module_loading().await {
        Ok(_) => {
            info!("✅ Module loading test passed");
            total_tests += 1;
            total_passed += 1;
        },
        Err(e) => {
            error!("❌ Module loading test failed: {}", e);
            total_tests += 1;
            total_failed += 1;
        }
    }

    // Test 2: Configuration Loading
    info!("🔍 Test 2: Configuration Loading...");
    match test_config_loading().await {
        Ok(_) => {
            info!("✅ Configuration loading test passed");
            total_tests += 1;
            total_passed += 1;
        },
        Err(e) => {
            error!("❌ Configuration loading test failed: {}", e);
            total_tests += 1;
            total_failed += 1;
        }
    }

    // Test 3: Basic ArbitrageBot Creation
    info!("🔍 Test 3: ArbitrageBot Creation...");
    match test_arbitrage_bot_creation().await {
        Ok(_) => {
            info!("✅ ArbitrageBot creation test passed");
            total_tests += 1;
            total_passed += 1;
        },
        Err(e) => {
            error!("❌ ArbitrageBot creation test failed: {}", e);
            total_tests += 1;
            total_failed += 1;
        }
    }

    let total_duration = start_time.elapsed();

    // Print summary
    info!("\n🎯 QA TEST SUMMARY (Safe Mode)");
    info!("{}", "=".repeat(70));
    info!("📊 Test Results:");
    info!("   Total Tests: {}", total_tests);
    info!("   ✅ Passed: {}", total_passed);
    info!("   ❌ Failed: {}", total_failed);

    let success_rate = if total_tests > 0 {
        (total_passed as f64 / total_tests as f64) * 100.0
    } else {
        0.0
    };

    info!("   📈 Success Rate: {:.1}%", success_rate);
    info!("   ⏱️ Total Duration: {:.1}s", total_duration.as_secs_f64());

    if total_failed > 0 {
        info!("\n❌ Some tests failed - check configuration and dependencies");
        std::process::exit(1);
    } else {
        info!("\n✅ All basic tests passed!");
        std::process::exit(0);
    }
}

async fn test_module_loading() -> Result<()> {
    // Test that we can load the main modules
    use sniperforge::config::Config;
    use sniperforge::bots::arbitrage_bot::ArbitrageBot;
    use sniperforge::shared::SharedServices;

    info!("   📦 Modules loaded successfully");
    Ok(())
}

async fn test_config_loading() -> Result<()> {
    use sniperforge::config::Config;

    // Try to load config
    let config_result = tokio::time::timeout(
        Duration::from_secs(10),
        async { Config::load("config/devnet.toml") }
    ).await;

    match config_result {
        Ok(Ok(_config)) => {
            info!("   ⚙️ DevNet configuration loaded successfully");
            Ok(())
        },
        Ok(Err(e)) => {
            error!("   ❌ Config loading failed: {}", e);
            Err(e)
        },
        Err(_) => {
            error!("   ⏰ Config loading timed out");
            Err(anyhow::anyhow!("Config loading timeout"))
        }
    }
}

async fn test_arbitrage_bot_creation() -> Result<()> {
    use sniperforge::config::Config;
    use sniperforge::shared::SharedServices;
    use sniperforge::bots::arbitrage_bot::ArbitrageBot;
    use std::sync::Arc;

    // Try to create ArbitrageBot with timeout
    let creation_result = tokio::time::timeout(
        Duration::from_secs(30),
        async {
            let config = Config::load("config/devnet.toml")?;
            let shared_services = Arc::new(SharedServices::new(&config).await?);

            // Use test wallet address and minimal capital
            let wallet_address = "11111111111111111111111111111112".to_string();
            let initial_capital = 10.0; // $10 for DevNet

            let _bot = ArbitrageBot::new(
                wallet_address,
                initial_capital,
                &config.network,
                shared_services.clone(),
            ).await?;
            Ok::<(), anyhow::Error>(())
        }
    ).await;

    match creation_result {
        Ok(Ok(_)) => {
            info!("   🤖 ArbitrageBot created successfully");
            Ok(())
        },
        Ok(Err(e)) => {
            error!("   ❌ ArbitrageBot creation failed: {}", e);
            Err(e)
        },
        Err(_) => {
            error!("   ⏰ ArbitrageBot creation timed out");
            Err(anyhow::anyhow!("ArbitrageBot creation timeout"))
        }
    }
}
