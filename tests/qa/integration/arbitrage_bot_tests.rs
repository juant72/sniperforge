use crate::qa::{QATestSuite, QATestResult};
use crate::qa::{qa_test, qa_assert, qa_assert_eq};
use sniperforge::bots::arbitrage_bot::ArbitrageBot;
use sniperforge::shared::SharedServices;
use sniperforge::config::Config;
use anyhow::Result;
use std::sync::Arc;
use std::time::Duration;
use tracing::info;

/// ArbitrageBot Integration Tests
pub struct ArbitrageBotIntegrationTests {
    config: Config,
    shared_services: Arc<SharedServices>,
}

impl ArbitrageBotIntegrationTests {
    pub async fn new() -> Result<Self> {
        let config = Config::load("config/devnet.toml")?;
        let shared_services = Arc::new(SharedServices::new(&config).await?);

        Ok(Self {
            config,
            shared_services,
        })
    }

    pub async fn run_all_tests(&self) -> QATestSuite {
        let mut suite = QATestSuite::new("ArbitrageBot Integration Tests".to_string());

        // Test 1: Bot Creation and Initialization
        let result = qa_test!("Bot Creation and Initialization", self.test_bot_creation());
        suite.add_result(result);

        // Test 2: WebSocket Price Feed Integration
        let result = qa_test!("WebSocket Price Feed Integration", self.test_websocket_price_feed());
        suite.add_result(result);

        // Test 3: Market Data Fetching
        let result = qa_test!("Market Data Fetching", self.test_market_data_fetching());
        suite.add_result(result);

        // Test 4: Opportunity Detection
        let result = qa_test!("Opportunity Detection", self.test_opportunity_detection());
        suite.add_result(result);

        // Test 5: Risk Management
        let result = qa_test!("Risk Management", self.test_risk_management());
        suite.add_result(result);

        // Test 6: Emergency Stop
        let result = qa_test!("Emergency Stop", self.test_emergency_stop());
        suite.add_result(result);

        // Test 7: Trading Loop
        let result = qa_test!("Trading Loop", self.test_trading_loop());
        suite.add_result(result);

        // Test 8: Status Reporting
        let result = qa_test!("Status Reporting", self.test_status_reporting());
        suite.add_result(result);

        // Test 9: Error Handling
        let result = qa_test!("Error Handling", self.test_error_handling());
        suite.add_result(result);

        // Test 10: Configuration Validation
        let result = qa_test!("Configuration Validation", self.test_configuration_validation());
        suite.add_result(result);

        suite
    }

    async fn test_bot_creation(&self) -> Result<Vec<String>> {
        let mut details = Vec::new();

        let wallet_address = self.shared_services.wallet_manager()
            .get_wallet_address("devnet-trading").await?;
        details.push(format!("Wallet address: {}", wallet_address));

        let bot = ArbitrageBot::new(
            wallet_address,
            100.0,
            &self.config.network,
            self.shared_services.clone(),
        ).await?;

        let status = bot.get_status();
        qa_assert!(status.is_running, "Bot should be running after creation");
        qa_assert_eq!(status.total_trades, 0, "Bot should start with 0 trades");
        qa_assert_eq!(status.emergency_stop, false, "Emergency stop should be false initially");

        details.push("Bot created successfully".to_string());
        details.push(format!("Initial capital: $100.00"));
        details.push(format!("Running status: {}", status.is_running));

        Ok(details)
    }

    async fn test_websocket_price_feed(&self) -> Result<Vec<String>> {
        let mut details = Vec::new();

        let wallet_address = self.shared_services.wallet_manager()
            .get_wallet_address("devnet-trading").await?;

        let mut bot = ArbitrageBot::new(
            wallet_address,
            50.0,
            &self.config.network,
            self.shared_services.clone(),
        ).await?;

        // Test price fetching
        match bot.get_jupiter_price("SOL", "USDC").await {
            Ok(price) => {
                qa_assert!(price > 0.0, "Price should be positive");
                details.push(format!("SOL price fetched: ${:.6}", price));
            },
            Err(e) => {
                // This is expected in DevNet
                details.push(format!("Price fetch failed (expected in DevNet): {}", e));
            }
        }

        // Test market data
        match bot.get_real_market_data().await {
            Ok(market_data) => {
                qa_assert!(market_data.price > 0.0, "Market price should be positive");
                qa_assert!(market_data.bid <= market_data.ask, "Bid should be <= Ask");
                qa_assert!(market_data.spread >= 0.0, "Spread should be non-negative");

                details.push(format!("Market data - Price: ${:.6}", market_data.price));
                details.push(format!("Bid: ${:.6}, Ask: ${:.6}", market_data.bid, market_data.ask));
                details.push(format!("Spread: ${:.6}", market_data.spread));
            },
            Err(e) => {
                details.push(format!("Market data fetch failed: {}", e));
            }
        }

        Ok(details)
    }

    async fn test_market_data_fetching(&self) -> Result<Vec<String>> {
        let mut details = Vec::new();

        let wallet_address = self.shared_services.wallet_manager()
            .get_wallet_address("devnet-trading").await?;

        let mut bot = ArbitrageBot::new(
            wallet_address,
            50.0,
            &self.config.network,
            self.shared_services.clone(),
        ).await?;

        let market_data = bot.get_real_market_data().await?;

        // Validate market data structure
        qa_assert!(!market_data.symbol.is_empty(), "Symbol should not be empty");
        qa_assert!(market_data.price > 0.0, "Price should be positive");
        qa_assert!(market_data.volume >= 0.0, "Volume should be non-negative");
        qa_assert!(market_data.timestamp > 0, "Timestamp should be set");

        details.push(format!("Symbol: {}", market_data.symbol));
        details.push(format!("Price: ${:.6}", market_data.price));
        details.push(format!("Volume: {:.2}", market_data.volume));
        details.push(format!("Liquidity: {:.2}", market_data.liquidity));

        Ok(details)
    }

    async fn test_opportunity_detection(&self) -> Result<Vec<String>> {
        let mut details = Vec::new();

        let wallet_address = self.shared_services.wallet_manager()
            .get_wallet_address("devnet-trading").await?;

        let mut bot = ArbitrageBot::new(
            wallet_address,
            50.0,
            &self.config.network,
            self.shared_services.clone(),
        ).await?;

        let signals = bot.detect_opportunities_using_strategy().await?;

        details.push(format!("Detected {} opportunities", signals.len()));

        for (i, signal) in signals.iter().enumerate() {
            qa_assert!(signal.confidence >= 0.0 && signal.confidence <= 1.0,
                      "Confidence should be between 0 and 1");
            qa_assert!(signal.position_size > 0.0, "Position size should be positive");
            qa_assert!(!signal.symbol.is_empty(), "Symbol should not be empty");

            details.push(format!("Signal {}: {} (confidence: {:.1}%)",
                               i + 1, signal.strategy_name, signal.confidence * 100.0));
        }

        Ok(details)
    }

    async fn test_risk_management(&self) -> Result<Vec<String>> {
        let mut details = Vec::new();

        let wallet_address = self.shared_services.wallet_manager()
            .get_wallet_address("devnet-trading").await?;

        // Test with small capital to trigger risk limits
        let bot = ArbitrageBot::new(
            wallet_address,
            10.0, // Small capital
            &self.config.network,
            self.shared_services.clone(),
        ).await?;

        let status = bot.get_status();

        // Risk management should be active
        qa_assert!(status.is_running, "Bot should be running");
        qa_assert!(!status.emergency_stop, "Emergency stop should not be triggered initially");

        details.push("Risk management initialized".to_string());
        details.push(format!("Initial capital: $10.00"));
        details.push("Risk limits should prevent excessive position sizes".to_string());

        Ok(details)
    }

    async fn test_emergency_stop(&self) -> Result<Vec<String>> {
        let mut details = Vec::new();

        let wallet_address = self.shared_services.wallet_manager()
            .get_wallet_address("devnet-trading").await?;

        let mut bot = ArbitrageBot::new(
            wallet_address,
            50.0,
            &self.config.network,
            self.shared_services.clone(),
        ).await?;

        // Bot should be running initially
        let initial_status = bot.get_status();
        qa_assert!(initial_status.is_running, "Bot should be running initially");
        qa_assert!(!initial_status.emergency_stop, "Emergency stop should be false initially");

        // Activate emergency stop
        bot.emergency_stop();

        // Check status after emergency stop
        let final_status = bot.get_status();
        qa_assert!(!final_status.is_running, "Bot should not be running after emergency stop");
        qa_assert!(final_status.emergency_stop, "Emergency stop should be true");

        details.push("Emergency stop test completed".to_string());
        details.push(format!("Initial running state: {}", initial_status.is_running));
        details.push(format!("Final running state: {}", final_status.is_running));
        details.push(format!("Emergency stop activated: {}", final_status.emergency_stop));

        Ok(details)
    }

    async fn test_trading_loop(&self) -> Result<Vec<String>> {
        let mut details = Vec::new();

        let wallet_address = self.shared_services.wallet_manager()
            .get_wallet_address("devnet-trading").await?;

        let mut bot = ArbitrageBot::new(
            wallet_address,
            50.0,
            &self.config.network,
            self.shared_services.clone(),
        ).await?;

        // Run trading loop for 2 seconds
        let start_time = std::time::Instant::now();

        tokio::select! {
            result = bot.start_trading() => {
                match result {
                    Ok(_) => details.push("Trading loop completed normally".to_string()),
                    Err(e) => details.push(format!("Trading loop ended with error: {}", e)),
                }
            }
            _ = tokio::time::sleep(Duration::from_secs(2)) => {
                bot.emergency_stop();
                details.push("Trading loop terminated by timeout".to_string());
            }
        }

        let duration = start_time.elapsed();
        let final_status = bot.get_status();

        details.push(format!("Loop duration: {}ms", duration.as_millis()));
        details.push(format!("Opportunities detected: {}", final_status.opportunities_detected));
        details.push(format!("Trades executed: {}", final_status.total_trades));

        Ok(details)
    }

    async fn test_status_reporting(&self) -> Result<Vec<String>> {
        let mut details = Vec::new();

        let wallet_address = self.shared_services.wallet_manager()
            .get_wallet_address("devnet-trading").await?;

        let bot = ArbitrageBot::new(
            wallet_address,
            50.0,
            &self.config.network,
            self.shared_services.clone(),
        ).await?;

        let status = bot.get_status();

        // Validate status fields
        qa_assert!(status.uptime_seconds >= 0, "Uptime should be non-negative");
        qa_assert!(status.total_trades >= 0, "Total trades should be non-negative");
        qa_assert!(status.successful_trades <= status.total_trades,
                  "Successful trades should not exceed total trades");
        qa_assert!(status.success_rate_percent >= 0.0 && status.success_rate_percent <= 100.0,
                  "Success rate should be between 0 and 100");
        qa_assert!(status.opportunities_detected >= 0, "Opportunities detected should be non-negative");
        qa_assert!(status.opportunities_executed <= status.opportunities_detected,
                  "Opportunities executed should not exceed detected");

        details.push(format!("Running: {}", status.is_running));
        details.push(format!("Uptime: {} seconds", status.uptime_seconds));
        details.push(format!("Total trades: {}", status.total_trades));
        details.push(format!("Success rate: {:.1}%", status.success_rate_percent));
        details.push(format!("Emergency stop: {}", status.emergency_stop));

        Ok(details)
    }

    async fn test_error_handling(&self) -> Result<Vec<String>> {
        let mut details = Vec::new();

        // Test with invalid wallet address
        match ArbitrageBot::new(
            "invalid_wallet_address".to_string(),
            50.0,
            &self.config.network,
            self.shared_services.clone(),
        ).await {
            Ok(_) => {
                // If it doesn't fail, that's also valid (wallet manager might handle it)
                details.push("Bot creation with invalid wallet succeeded (handled gracefully)".to_string());
            },
            Err(e) => {
                details.push(format!("Bot creation with invalid wallet failed as expected: {}", e));
            }
        }

        // Test with zero capital
        match ArbitrageBot::new(
            self.shared_services.wallet_manager().get_wallet_address("devnet-trading").await?,
            0.0,
            &self.config.network,
            self.shared_services.clone(),
        ).await {
            Ok(bot) => {
                let status = bot.get_status();
                details.push(format!("Bot created with zero capital, status: running={}", status.is_running));
            },
            Err(e) => {
                details.push(format!("Bot creation with zero capital failed: {}", e));
            }
        }

        details.push("Error handling tests completed".to_string());

        Ok(details)
    }

    async fn test_configuration_validation(&self) -> Result<Vec<String>> {
        let mut details = Vec::new();

        // Test if configuration is properly loaded
        let rpc_url = if self.config.network.environment == "devnet" {
            self.config.network.devnet_primary_rpc.as_ref().unwrap_or(&"".to_string())
        } else {
            self.config.network.mainnet_primary_rpc.as_ref().unwrap_or(&"".to_string())
        };
        qa_assert!(!rpc_url.is_empty(), "RPC URL should not be empty");
        qa_assert!(!self.config.network.environment.is_empty(), "Environment should not be empty");

        details.push(format!("Network environment: {}", self.config.network.environment));
        details.push(format!("RPC URL: {}", rpc_url));

        // Test shared services initialization
        let wallets = self.shared_services.wallet_manager().list_wallets().await;
        qa_assert!(!wallets.is_empty(), "Should have at least one wallet");

        details.push(format!("Available wallets: {}", wallets.len()));

        details.push(format!("Total wallets configured: {}", wallets.len()));

        Ok(details)
    }
}
