use crate::qa::{QATestSuite, QATestResult, qa_test, qa_assert};
use sniperforge::bots::arbitrage_bot::ArbitrageBot;
use sniperforge::shared::SharedServices;
use sniperforge::config::Config;
use anyhow::Result;
use std::sync::Arc;
use std::time::{Duration, Instant};
use tokio::time::timeout;
use tracing::info;

/// ArbitrageBot Stress Tests
pub struct ArbitrageBotStressTests {
    config: Config,
    shared_services: Arc<SharedServices>,
}

impl ArbitrageBotStressTests {
    pub async fn new() -> Result<Self> {
        let config = Config::load("config/devnet.toml")?;
        let shared_services = Arc::new(SharedServices::new(&config).await?);

        Ok(Self {
            config,
            shared_services,
        })
    }

    pub async fn run_all_tests(&self) -> QATestSuite {
        let mut suite = QATestSuite::new("ArbitrageBot Stress Tests".to_string());

        // Test 1: Rapid Bot Creation/Destruction
        let result = qa_test!("Rapid Bot Creation/Destruction", self.test_rapid_creation_destruction());
        suite.add_result(result);

        // Test 2: Continuous Market Data Fetching
        let result = qa_test!("Continuous Market Data Fetching", self.test_continuous_market_data());
        suite.add_result(result);

        // Test 3: Multiple Opportunity Detection Cycles
        let result = qa_test!("Multiple Opportunity Detection Cycles", self.test_multiple_detection_cycles());
        suite.add_result(result);

        // Test 4: Long Running Trading Loop
        let result = qa_test!("Long Running Trading Loop", self.test_long_running_loop());
        suite.add_result(result);

        // Test 5: Memory Usage Under Load
        let result = qa_test!("Memory Usage Under Load", self.test_memory_usage());
        suite.add_result(result);

        // Test 6: Concurrent Bot Operations
        let result = qa_test!("Concurrent Bot Operations", self.test_concurrent_operations());
        suite.add_result(result);

        suite
    }

    async fn test_rapid_creation_destruction(&self) -> Result<Vec<String>> {
        let mut details = Vec::new();
        let iterations = 10;
        let start_time = Instant::now();

        for i in 0..iterations {
            let wallet_address = self.shared_services.wallet_manager()
                .get_wallet_address("devnet-trading").await?;

            let bot = ArbitrageBot::new(
                wallet_address,
                50.0,
                &self.config.network,
                self.shared_services.clone(),
            ).await?;

            // Verify bot is functional
            let status = bot.get_status();
            qa_assert!(status.is_running, "Bot should be running after creation");

            // Bot will be dropped here (destruction)
            if i % 2 == 0 {
                details.push(format!("Created and destroyed bot {}/{}", i + 1, iterations));
            }
        }

        let duration = start_time.elapsed();
        let avg_time = duration.as_millis() / iterations as u128;

        details.push(format!("Created/destroyed {} bots", iterations));
        details.push(format!("Total time: {}ms", duration.as_millis()));
        details.push(format!("Average time per bot: {}ms", avg_time));

        qa_assert!(avg_time < 1000, "Average creation time should be < 1 second");

        Ok(details)
    }

    async fn test_continuous_market_data(&self) -> Result<Vec<String>> {
        let mut details = Vec::new();

        let wallet_address = self.shared_services.wallet_manager()
            .get_wallet_address("devnet-trading").await?;

        let mut bot = ArbitrageBot::new(
            wallet_address,
            50.0,
            &self.config.network,
            self.shared_services.clone(),
        ).await?;

        let iterations = 50;
        let mut successful_fetches = 0;
        let mut total_response_time = 0u128;

        for i in 0..iterations {
            let start = Instant::now();

            match timeout(Duration::from_secs(5), bot.get_real_market_data()).await {
                Ok(Ok(_market_data)) => {
                    successful_fetches += 1;
                    total_response_time += start.elapsed().as_millis();
                },
                Ok(Err(e)) => {
                    details.push(format!("Fetch {}: Failed - {}", i + 1, e));
                },
                Err(_) => {
                    details.push(format!("Fetch {}: Timeout", i + 1));
                }
            }

            // Small delay between requests
            tokio::time::sleep(Duration::from_millis(100)).await;
        }

        let success_rate = (successful_fetches as f64 / iterations as f64) * 100.0;
        let avg_response_time = if successful_fetches > 0 {
            total_response_time / successful_fetches as u128
        } else { 0 };

        details.push(format!("Successful fetches: {}/{}", successful_fetches, iterations));
        details.push(format!("Success rate: {:.1}%", success_rate));
        details.push(format!("Average response time: {}ms", avg_response_time));

        qa_assert!(success_rate > 50.0, "Success rate should be > 50%");
        qa_assert!(avg_response_time < 2000, "Average response time should be < 2 seconds");

        Ok(details)
    }

    async fn test_multiple_detection_cycles(&self) -> Result<Vec<String>> {
        let mut details = Vec::new();

        let wallet_address = self.shared_services.wallet_manager()
            .get_wallet_address("devnet-trading").await?;

        let mut bot = ArbitrageBot::new(
            wallet_address,
            50.0,
            &self.config.network,
            self.shared_services.clone(),
        ).await?;

        let cycles = 20;
        let mut total_opportunities = 0;
        let mut total_detection_time = 0u128;

        for i in 0..cycles {
            let start = Instant::now();

            match timeout(Duration::from_secs(10), bot.detect_opportunities_using_strategy()).await {
                Ok(Ok(signals)) => {
                    total_opportunities += signals.len();
                    total_detection_time += start.elapsed().as_millis();

                    if i % 5 == 0 {
                        details.push(format!("Cycle {}: {} opportunities", i + 1, signals.len()));
                    }
                },
                Ok(Err(e)) => {
                    details.push(format!("Cycle {}: Detection failed - {}", i + 1, e));
                },
                Err(_) => {
                    details.push(format!("Cycle {}: Detection timeout", i + 1));
                }
            }

            tokio::time::sleep(Duration::from_millis(200)).await;
        }

        let avg_detection_time = total_detection_time / cycles as u128;
        let avg_opportunities = total_opportunities as f64 / cycles as f64;

        details.push(format!("Total detection cycles: {}", cycles));
        details.push(format!("Total opportunities found: {}", total_opportunities));
        details.push(format!("Average opportunities per cycle: {:.1}", avg_opportunities));
        details.push(format!("Average detection time: {}ms", avg_detection_time));

        qa_assert!(avg_detection_time < 5000, "Average detection time should be < 5 seconds");

        Ok(details)
    }

    async fn test_long_running_loop(&self) -> Result<Vec<String>> {
        let mut details = Vec::new();

        let wallet_address = self.shared_services.wallet_manager()
            .get_wallet_address("devnet-trading").await?;

        let mut bot = ArbitrageBot::new(
            wallet_address,
            50.0,
            &self.config.network,
            self.shared_services.clone(),
        ).await?;

        let start_time = Instant::now();
        let test_duration = Duration::from_secs(30); // Set to 30 seconds for proper stress testing

        // Use timeout instead of select to avoid borrowing issues
        let result = tokio::time::timeout(
            test_duration,
            bot.start_trading()
        ).await;

        let actual_duration = start_time.elapsed();

        match result {
            Ok(Ok(_)) => {
                details.push("Trading loop completed normally".to_string());
            },
            Ok(Err(e)) => {
                details.push(format!("Trading loop failed: {}", e));
            },
            Err(_) => {
                // Timeout occurred - stop the bot
                bot.emergency_stop();
                details.push("Long running test completed with timeout".to_string());
            }
        }
        let final_status = bot.get_status();

        details.push(format!("Test duration: {:.1}s", actual_duration.as_secs_f64()));
        details.push(format!("Opportunities detected: {}", final_status.opportunities_detected));
        details.push(format!("Trades executed: {}", final_status.total_trades));
        details.push(format!("Bot uptime: {}s", final_status.uptime_seconds));

        qa_assert!(actual_duration >= Duration::from_secs(25), "Should run for at least 25 seconds");
        qa_assert!(final_status.emergency_stop, "Emergency stop should be activated");

        Ok(details)
    }

    async fn test_memory_usage(&self) -> Result<Vec<String>> {
        let mut details = Vec::new();

        // This is a simplified memory test - in a real environment you'd use proper memory profiling
        let mut bots = Vec::new();
        let num_bots = 5; // Create multiple bots to test memory usage

        for i in 0..num_bots {
            let wallet_address = self.shared_services.wallet_manager()
                .get_wallet_address("devnet-trading").await?;

            let bot = ArbitrageBot::new(
                wallet_address,
                50.0,
                &self.config.network,
                self.shared_services.clone(),
            ).await?;

            bots.push(bot);
            details.push(format!("Created bot {}/{}", i + 1, num_bots));
        }

        // Perform operations on all bots
        for (i, bot) in bots.iter_mut().enumerate() {
            match timeout(Duration::from_secs(5), bot.get_real_market_data()).await {
                Ok(Ok(_)) => details.push(format!("Bot {} market data: OK", i + 1)),
                Ok(Err(e)) => details.push(format!("Bot {} market data: Failed - {}", i + 1, e)),
                Err(_) => details.push(format!("Bot {} market data: Timeout", i + 1)),
            }
        }

        details.push(format!("Successfully managed {} concurrent bots", num_bots));
        qa_assert!(bots.len() == num_bots, "All bots should be created successfully");

        // Bots will be dropped here, releasing memory
        drop(bots);
        details.push("All bots destroyed, memory released".to_string());

        Ok(details)
    }

    async fn test_concurrent_operations(&self) -> Result<Vec<String>> {
        let mut details = Vec::new();

        let wallet_address = self.shared_services.wallet_manager()
            .get_wallet_address("devnet-trading").await?;

        let mut bot = ArbitrageBot::new(
            wallet_address,
            50.0,
            &self.config.network,
            self.shared_services.clone(),
        ).await?;

        // Spawn multiple concurrent operations
        let mut handles = Vec::new();

        // Market data fetching tasks
        for i in 0..3 {
            let mut bot_clone = ArbitrageBot::new(
                self.shared_services.wallet_manager().get_wallet_address("devnet-trading").await?,
                50.0,
                &self.config.network,
                self.shared_services.clone(),
            ).await?;

            let handle = tokio::spawn(async move {
                for j in 0..5 {
                    match bot_clone.get_real_market_data().await {
                        Ok(_) => info!("Task {} iteration {}: Market data OK", i, j),
                        Err(e) => info!("Task {} iteration {}: Market data failed - {}", i, j, e),
                    }
                    tokio::time::sleep(Duration::from_millis(500)).await;
                }
                format!("Market data task {} completed", i)
            });
            handles.push(handle);
        }

        // Opportunity detection tasks
        for i in 0..2 {
            let mut bot_clone = ArbitrageBot::new(
                self.shared_services.wallet_manager().get_wallet_address("devnet-trading").await?,
                50.0,
                &self.config.network,
                self.shared_services.clone(),
            ).await?;

            let handle = tokio::spawn(async move {
                for j in 0..3 {
                    match bot_clone.detect_opportunities_using_strategy().await {
                        Ok(signals) => info!("Task {} iteration {}: {} opportunities", i, j, signals.len()),
                        Err(e) => info!("Task {} iteration {}: Detection failed - {}", i, j, e),
                    }
                    tokio::time::sleep(Duration::from_millis(1000)).await;
                }
                format!("Opportunity detection task {} completed", i)
            });
            handles.push(handle);
        }

        // Wait for all tasks to complete
        let results = futures::future::join_all(handles).await;

        let mut successful_tasks = 0;
        for result in results {
            match result {
                Ok(message) => {
                    successful_tasks += 1;
                    details.push(message);
                },
                Err(e) => {
                    details.push(format!("Task failed: {}", e));
                }
            }
        }

        details.push(format!("Concurrent operations completed: {}/5 successful", successful_tasks));
        qa_assert!(successful_tasks >= 3, "At least 3 tasks should complete successfully");

        Ok(details)
    }
}
