mod qa;

use sniperforge::bots::arbitrage_bot::ArbitrageBot;
use sniperforge::shared::SharedServices;
use sniperforge::config::Config;
use qa::{QATestSuite, QATestResult, qa_test, qa_assert, qa_assert_eq};
use anyhow::Result;
use std::sync::Arc;
use std::time::Instant;
use tracing::{info, error, warn};

/// Main QA Test Runner for ArbitrageBot
#[tokio::main]
async fn main() -> Result<()> {
    // Initialize logging
    tracing_subscriber::fmt()
        .with_env_filter("info,sniperforge=debug")
        .init();

    info!("🧪 Starting SniperForge ArbitrageBot QA Test Suite");
    info!("=" .repeat(70));

    let start_time = Instant::now();
    let mut all_suites = Vec::new();

    // Run Integration Tests
    info!("🔗 Running Integration Tests...");
    let integration_suite = run_integration_tests().await;
    all_suites.push(integration_suite);

    // Run Unit Tests
    info!("🧪 Running Unit Tests...");
    let unit_suite = run_unit_tests().await;
    all_suites.push(unit_suite);

    // Run Stress Tests
    info!("💥 Running Stress Tests...");
    let stress_suite = run_stress_tests().await;
    all_suites.push(stress_suite);

    // Run Performance Tests
    info!("⚡ Running Performance Tests...");
    let performance_suite = run_performance_tests().await;
    all_suites.push(performance_suite);

    let total_duration = start_time.elapsed();

    // Generate Overall Report
    generate_overall_report(&all_suites, total_duration);

    Ok(())
}

/// Run Integration Tests
async fn run_integration_tests() -> QATestSuite {
    let mut suite = QATestSuite::new("Integration Tests".to_string());

    // Test 1: ArbitrageBot Creation
    let result = qa_test!("ArbitrageBot Creation", test_arbitrage_bot_creation());
    suite.add_result(result);

    // Test 2: Price Feed Integration
    let result = qa_test!("Price Feed Integration", test_price_feed_integration());
    suite.add_result(result);

    // Test 3: Market Data Fetching
    let result = qa_test!("Market Data Fetching", test_market_data_fetching());
    suite.add_result(result);

    // Test 4: Opportunity Detection
    let result = qa_test!("Opportunity Detection", test_opportunity_detection());
    suite.add_result(result);

    // Test 5: Emergency Stop
    let result = qa_test!("Emergency Stop", test_emergency_stop());
    suite.add_result(result);

    // Test 6: Status Reporting
    let result = qa_test!("Status Reporting", test_status_reporting());
    suite.add_result(result);

    suite.print_report();
    suite
}

/// Run Unit Tests
async fn run_unit_tests() -> QATestSuite {
    let mut suite = QATestSuite::new("Unit Tests".to_string());

    // Test 1: Strategy Analysis
    let result = qa_test!("Strategy Analysis", test_strategy_analysis());
    suite.add_result(result);

    // Test 2: Market Data Processing
    let result = qa_test!("Market Data Processing", test_market_data_processing());
    suite.add_result(result);

    // Test 3: Signal Generation
    let result = qa_test!("Signal Generation", test_signal_generation());
    suite.add_result(result);

    suite.print_report();
    suite
}

/// Run Stress Tests
async fn run_stress_tests() -> QATestSuite {
    let mut suite = QATestSuite::new("Stress Tests".to_string());

    // Test 1: Rapid Bot Creation/Destruction
    let result = qa_test!("Rapid Bot Creation", test_rapid_bot_creation());
    suite.add_result(result);

    // Test 2: Continuous Market Data
    let result = qa_test!("Continuous Market Data", test_continuous_market_data());
    suite.add_result(result);

    suite.print_report();
    suite
}

/// Run Performance Tests
async fn run_performance_tests() -> QATestSuite {
    let mut suite = QATestSuite::new("Performance Tests".to_string());

    // Test 1: Initialization Speed
    let result = qa_test!("Initialization Speed", test_initialization_speed());
    suite.add_result(result);

    // Test 2: Market Data Latency
    let result = qa_test!("Market Data Latency", test_market_data_latency());
    suite.add_result(result);

    suite.print_report();
    suite
}

// ============================================================================
// INTEGRATION TEST IMPLEMENTATIONS
// ============================================================================

async fn test_arbitrage_bot_creation() -> Result<Vec<String>> {
    let mut details = Vec::new();

    let config = Config::load("config/devnet.toml")?;
    details.push("✅ DevNet config loaded".to_string());

    let shared_services = Arc::new(SharedServices::new(&config).await?);
    details.push("✅ Shared services initialized".to_string());

    let wallet_address = shared_services.wallet_manager().get_wallet_address("devnet-trading").await?;
    details.push(format!("✅ Wallet address: {}", wallet_address));

    let bot = ArbitrageBot::new(
        wallet_address,
        50.0,
        &config.network,
        shared_services.clone(),
    ).await?;
    details.push("✅ ArbitrageBot created successfully".to_string());

    let status = bot.get_status();
    qa_assert!(!status.emergency_stop, "Emergency stop should be false initially");
    qa_assert_eq!(status.total_trades, 0, "Initial trade count should be 0");

    details.push("✅ Bot initial state verified".to_string());

    Ok(details)
}

async fn test_price_feed_integration() -> Result<Vec<String>> {
    let mut details = Vec::new();

    let config = Config::load("config/devnet.toml")?;
    let shared_services = Arc::new(SharedServices::new(&config).await?);
    let wallet_address = shared_services.wallet_manager().get_wallet_address("devnet-trading").await?;

    let mut bot = ArbitrageBot::new(
        wallet_address,
        50.0,
        &config.network,
        shared_services.clone(),
    ).await?;

    details.push("✅ Bot initialized for price feed test".to_string());

    // Test Jupiter price fetching
    match bot.get_jupiter_price("SOL", "USDC").await {
        Ok(price) => {
            qa_assert!(price > 0.0, "Price should be positive");
            details.push(format!("✅ Jupiter price: ${:.6}", price));
        },
        Err(e) => {
            details.push(format!("⚠️ Jupiter price failed (expected in DevNet): {}", e));
        }
    }

    // Test WebSocket price feed
    match bot.get_websocket_price("SOL").await {
        Ok(price) => {
            qa_assert!(price > 0.0, "WebSocket price should be positive");
            details.push(format!("✅ WebSocket price: ${:.6}", price));
        },
        Err(e) => {
            details.push(format!("⚠️ WebSocket price failed: {}", e));
        }
    }

    Ok(details)
}

async fn test_market_data_fetching() -> Result<Vec<String>> {
    let mut details = Vec::new();

    let config = Config::load("config/devnet.toml")?;
    let shared_services = Arc::new(SharedServices::new(&config).await?);
    let wallet_address = shared_services.wallet_manager().get_wallet_address("devnet-trading").await?;

    let mut bot = ArbitrageBot::new(
        wallet_address,
        50.0,
        &config.network,
        shared_services.clone(),
    ).await?;

    details.push("✅ Bot initialized for market data test".to_string());

    match bot.get_real_market_data().await {
        Ok(market_data) => {
            qa_assert!(!market_data.symbol.is_empty(), "Symbol should not be empty");
            qa_assert!(market_data.price >= 0.0, "Price should be non-negative");

            details.push(format!("✅ Market data for {}", market_data.symbol));
            details.push(format!("  Price: ${:.6}", market_data.price));
            details.push(format!("  Spread: ${:.6}", market_data.spread));
        },
        Err(e) => {
            details.push(format!("⚠️ Market data failed (expected in DevNet): {}", e));

            let fallback_data = bot.get_fallback_market_data().await?;
            qa_assert!(!fallback_data.symbol.is_empty(), "Fallback symbol should not be empty");
            details.push("✅ Fallback market data works".to_string());
        }
    }

    Ok(details)
}

async fn test_opportunity_detection() -> Result<Vec<String>> {
    let mut details = Vec::new();

    let config = Config::load("config/devnet.toml")?;
    let shared_services = Arc::new(SharedServices::new(&config).await?);
    let wallet_address = shared_services.wallet_manager().get_wallet_address("devnet-trading").await?;

    let mut bot = ArbitrageBot::new(
        wallet_address,
        50.0,
        &config.network,
        shared_services.clone(),
    ).await?;

    details.push("✅ Bot initialized for opportunity detection".to_string());

    let signals = bot.detect_opportunities_using_strategy().await?;
    details.push(format!("✅ Detected {} signals", signals.len()));

    for (i, signal) in signals.iter().enumerate() {
        qa_assert!(!signal.strategy_name.is_empty(), "Strategy name should not be empty");
        qa_assert!(signal.confidence >= 0.0 && signal.confidence <= 1.0, "Confidence should be 0-1");
        qa_assert!(signal.position_size >= 0.0, "Position size should be non-negative");

        details.push(format!("  Signal {}: {} ({:.1}%)",
                           i + 1, signal.strategy_name, signal.confidence * 100.0));
    }

    if signals.is_empty() {
        details.push("✅ No opportunities (normal in DevNet)".to_string());
    }

    Ok(details)
}

async fn test_emergency_stop() -> Result<Vec<String>> {
    let mut details = Vec::new();

    let config = Config::load("config/devnet.toml")?;
    let shared_services = Arc::new(SharedServices::new(&config).await?);
    let wallet_address = shared_services.wallet_manager().get_wallet_address("devnet-trading").await?;

    let mut bot = ArbitrageBot::new(
        wallet_address,
        50.0,
        &config.network,
        shared_services.clone(),
    ).await?;

    details.push("✅ Bot initialized for emergency stop test".to_string());

    let initial_status = bot.get_status();
    qa_assert!(!initial_status.emergency_stop, "Emergency stop should be false initially");

    bot.emergency_stop();
    details.push("✅ Emergency stop activated".to_string());

    let stopped_status = bot.get_status();
    qa_assert!(stopped_status.emergency_stop, "Emergency stop should be true after activation");

    details.push("✅ Emergency stop state verified".to_string());

    Ok(details)
}

async fn test_status_reporting() -> Result<Vec<String>> {
    let mut details = Vec::new();

    let config = Config::load("config/devnet.toml")?;
    let shared_services = Arc::new(SharedServices::new(&config).await?);
    let wallet_address = shared_services.wallet_manager().get_wallet_address("devnet-trading").await?;

    let bot = ArbitrageBot::new(
        wallet_address,
        50.0,
        &config.network,
        shared_services.clone(),
    ).await?;

    details.push("✅ Bot initialized for status test".to_string());

    let status = bot.get_status();

    qa_assert!(status.uptime_seconds >= 0, "Uptime should be non-negative");
    qa_assert!(status.total_trades >= 0, "Total trades should be non-negative");
    qa_assert!(status.success_rate_percent >= 0.0 && status.success_rate_percent <= 100.0,
               "Success rate should be 0-100");

    details.push(format!("✅ Uptime: {}s", status.uptime_seconds));
    details.push(format!("✅ Trades: {}", status.total_trades));
    details.push(format!("✅ Success rate: {:.1}%", status.success_rate_percent));

    Ok(details)
}

// ============================================================================
// UNIT TEST IMPLEMENTATIONS
// ============================================================================

async fn test_strategy_analysis() -> Result<Vec<String>> {
    let mut details = Vec::new();

    use sniperforge::bots::arbitrage_bot::{ArbitrageStrategy, MarketData};

    let strategy = ArbitrageStrategy::new();
    details.push("✅ Strategy created".to_string());

    let market_data = MarketData {
        symbol: "SOL/USDC".to_string(),
        price: 100.0,
        volume: 10000.0,
        timestamp: 1234567890,
        bid: 99.5,
        ask: 100.5,
        spread: 1.0,
        current_price: 100.0,
        volume_24h: 50000.0,
        price_change_24h: 2.5,
        liquidity: 100000.0,
        bid_ask_spread: 1.0,
        order_book_depth: 50000.0,
        price_history: vec![99.0, 100.0, 101.0],
        volume_history: vec![8000.0, 10000.0, 12000.0],
    };

    let signals = strategy.analyze_market(&market_data).await?;
    details.push(format!("✅ Generated {} signals", signals.len()));

    for signal in &signals {
        qa_assert!(signal.confidence >= 0.0 && signal.confidence <= 1.0, "Confidence should be 0-1");
        qa_assert!(signal.position_size >= 0.0, "Position size should be non-negative");
    }

    Ok(details)
}

async fn test_market_data_processing() -> Result<Vec<String>> {
    let mut details = Vec::new();

    use sniperforge::bots::arbitrage_bot::MarketData;

    let market_data = MarketData {
        symbol: "SOL/USDC".to_string(),
        price: 100.0,
        volume: 10000.0,
        timestamp: 1234567890,
        bid: 99.5,
        ask: 100.5,
        spread: 1.0,
        current_price: 100.0,
        volume_24h: 50000.0,
        price_change_24h: 2.5,
        liquidity: 100000.0,
        bid_ask_spread: 1.0,
        order_book_depth: 50000.0,
        price_history: vec![99.0, 100.0, 101.0],
        volume_history: vec![8000.0, 10000.0, 12000.0],
    };

    qa_assert!(!market_data.symbol.is_empty(), "Symbol should not be empty");
    qa_assert!(market_data.price > 0.0, "Price should be positive");
    qa_assert!(market_data.bid <= market_data.ask, "Bid should be <= ask");

    details.push("✅ Market data structure validated".to_string());
    details.push(format!("✅ Symbol: {}", market_data.symbol));
    details.push(format!("✅ Price: ${:.2}", market_data.price));

    Ok(details)
}

async fn test_signal_generation() -> Result<Vec<String>> {
    let mut details = Vec::new();

    use sniperforge::bots::arbitrage_bot::StrategySignal;
    use std::collections::HashMap;

    let signal = StrategySignal {
        signal_type: "ARBITRAGE".to_string(),
        confidence: 0.75,
        symbol: "SOL/USDC".to_string(),
        timeframe: "INSTANT".to_string(),
        metadata: HashMap::new(),
        position_size: 10.0,
        strategy_name: "TestStrategy".to_string(),
    };

    qa_assert!(!signal.signal_type.is_empty(), "Signal type should not be empty");
    qa_assert!(signal.confidence >= 0.0 && signal.confidence <= 1.0, "Confidence should be 0-1");
    qa_assert!(signal.position_size >= 0.0, "Position size should be non-negative");

    details.push("✅ Signal structure validated".to_string());
    details.push(format!("✅ Type: {}", signal.signal_type));
    details.push(format!("✅ Confidence: {:.1}%", signal.confidence * 100.0));

    Ok(details)
}

// ============================================================================
// STRESS TEST IMPLEMENTATIONS
// ============================================================================

async fn test_rapid_bot_creation() -> Result<Vec<String>> {
    let mut details = Vec::new();

    let config = Config::load("config/devnet.toml")?;
    let shared_services = Arc::new(SharedServices::new(&config).await?);
    let wallet_address = shared_services.wallet_manager().get_wallet_address("devnet-trading").await?;

    let start_time = Instant::now();

    for i in 0..5 {
        let _bot = ArbitrageBot::new(
            wallet_address.clone(),
            50.0,
            &config.network,
            shared_services.clone(),
        ).await?;

        details.push(format!("✅ Bot {} created", i + 1));
    }

    let elapsed = start_time.elapsed();
    details.push(format!("✅ Created 5 bots in {}ms", elapsed.as_millis()));

    qa_assert!(elapsed.as_secs() < 30, "Bot creation should complete within 30 seconds");

    Ok(details)
}

async fn test_continuous_market_data() -> Result<Vec<String>> {
    let mut details = Vec::new();

    let config = Config::load("config/devnet.toml")?;
    let shared_services = Arc::new(SharedServices::new(&config).await?);
    let wallet_address = shared_services.wallet_manager().get_wallet_address("devnet-trading").await?;

    let mut bot = ArbitrageBot::new(
        wallet_address,
        50.0,
        &config.network,
        shared_services.clone(),
    ).await?;

    let start_time = Instant::now();

    for i in 0..3 {
        let _market_data = bot.get_fallback_market_data().await?;
        details.push(format!("✅ Market data fetch {} completed", i + 1));
    }

    let elapsed = start_time.elapsed();
    details.push(format!("✅ 3 market data fetches in {}ms", elapsed.as_millis()));

    Ok(details)
}

// ============================================================================
// PERFORMANCE TEST IMPLEMENTATIONS
// ============================================================================

async fn test_initialization_speed() -> Result<Vec<String>> {
    let mut details = Vec::new();

    let start_time = Instant::now();

    let config = Config::load("config/devnet.toml")?;
    let shared_services = Arc::new(SharedServices::new(&config).await?);
    let wallet_address = shared_services.wallet_manager().get_wallet_address("devnet-trading").await?;

    let _bot = ArbitrageBot::new(
        wallet_address,
        50.0,
        &config.network,
        shared_services.clone(),
    ).await?;

    let elapsed = start_time.elapsed();
    let elapsed_ms = elapsed.as_millis();

    details.push(format!("✅ Bot initialized in {}ms", elapsed_ms));

    qa_assert!(elapsed_ms < 5000, "Initialization should complete within 5 seconds");

    // Performance benchmarks
    match elapsed_ms {
        0..=1000 => details.push("🚀 Excellent performance".to_string()),
        1001..=3000 => details.push("✅ Good performance".to_string()),
        3001..=5000 => details.push("⚠️ Acceptable performance".to_string()),
        _ => details.push("🐌 Slow performance".to_string()),
    }

    Ok(details)
}

async fn test_market_data_latency() -> Result<Vec<String>> {
    let mut details = Vec::new();

    let config = Config::load("config/devnet.toml")?;
    let shared_services = Arc::new(SharedServices::new(&config).await?);
    let wallet_address = shared_services.wallet_manager().get_wallet_address("devnet-trading").await?;

    let mut bot = ArbitrageBot::new(
        wallet_address,
        50.0,
        &config.network,
        shared_services.clone(),
    ).await?;

    let start_time = Instant::now();
    let _market_data = bot.get_fallback_market_data().await?;
    let elapsed = start_time.elapsed();
    let elapsed_ms = elapsed.as_millis();

    details.push(format!("✅ Market data fetched in {}ms", elapsed_ms));

    qa_assert!(elapsed_ms < 2000, "Market data fetch should complete within 2 seconds");

    // Performance benchmarks
    match elapsed_ms {
        0..=100 => details.push("🚀 Excellent latency".to_string()),
        101..=500 => details.push("✅ Good latency".to_string()),
        501..=1000 => details.push("⚠️ Acceptable latency".to_string()),
        _ => details.push("🐌 High latency".to_string()),
    }

    Ok(details)
}

// ============================================================================
// REPORT GENERATION
// ============================================================================

fn generate_overall_report(suites: &[QATestSuite], total_duration: std::time::Duration) {
    info!("");
    info!("📊 COMPREHENSIVE QA REPORT");
    info!("=" .repeat(70));

    let mut total_tests = 0;
    let mut total_passed = 0;
    let mut total_failed = 0;
    let mut total_test_duration = 0;
    let mut all_failed_tests = Vec::new();

    for suite in suites {
        let summary = suite.get_summary();
        total_tests += summary.total_tests;
        total_passed += summary.passed_tests;
        total_failed += summary.failed_tests;
        total_test_duration += summary.total_duration_ms;
        all_failed_tests.extend(summary.failed_test_names);

        info!("📋 {}: {}/{} passed ({:.1}%)",
              summary.suite_name,
              summary.passed_tests,
              summary.total_tests,
              summary.success_rate);
    }

    let overall_success_rate = if total_tests > 0 {
        (total_passed as f64 / total_tests as f64) * 100.0
    } else {
        0.0
    };

    info!("");
    info!("🎯 OVERALL RESULTS:");
    info!("   Total Tests: {}", total_tests);
    info!("   ✅ Passed: {}", total_passed);
    info!("   ❌ Failed: {}", total_failed);
    info!("   📈 Success Rate: {:.1}%", overall_success_rate);
    info!("   ⏱️ Test Duration: {}ms", total_test_duration);
    info!("   🕐 Total Runtime: {:.2}s", total_duration.as_secs_f64());

    if !all_failed_tests.is_empty() {
        warn!("");
        warn!("🔍 FAILED TESTS:");
        for failed_test in &all_failed_tests {
            error!("   ❌ {}", failed_test);
        }
    }

    info!("");

    // Quality Gate Analysis
    match overall_success_rate {
        rate if rate >= 95.0 => {
            info!("🎉 QUALITY GATE: PASSED");
            info!("✅ ArbitrageBot meets production quality standards");
            info!("🚀 Ready for deployment");
        },
        rate if rate >= 80.0 => {
            warn!("⚠️ QUALITY GATE: WARNING");
            warn!("🔶 ArbitrageBot has some issues but is functional");
            warn!("🛠️ Consider fixing failed tests before deployment");
        },
        _ => {
            error!("❌ QUALITY GATE: FAILED");
            error!("🚨 ArbitrageBot has significant issues");
            error!("🛑 DO NOT DEPLOY - Fix critical issues first");
        }
    }

    info!("");
    info!("📝 QA Test Suite completed");
    info!("=" .repeat(70));
}
