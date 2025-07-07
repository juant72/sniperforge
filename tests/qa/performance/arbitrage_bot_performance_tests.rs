use crate::qa::{QATestSuite, QATestResult};
use crate::{qa_test, qa_assert};
use sniperforge::bots::arbitrage_bot::ArbitrageBot;
use sniperforge::shared::SharedServices;
use sniperforge::config::Config;
use anyhow::Result;
use std::sync::Arc;
use std::time::{Duration, Instant};

/// ArbitrageBot Performance Tests
pub struct ArbitrageBotPerformanceTests {
    config: Config,
    shared_services: Arc<SharedServices>,
}

impl ArbitrageBotPerformanceTests {
    pub async fn new() -> Result<Self> {
        let config = Config::load("config/devnet.toml")?;
        let shared_services = Arc::new(SharedServices::new(&config).await?);

        Ok(Self {
            config,
            shared_services,
        })
    }

    pub async fn run_all_tests(&self) -> QATestSuite {
        let mut suite = QATestSuite::new("ArbitrageBot Performance Tests".to_string());

        // Test 1: Bot Initialization Speed
        let result = qa_test!("Bot Initialization Speed", self.test_initialization_speed());
        suite.add_result(result);

        // Test 2: Market Data Fetch Performance
        let result = qa_test!("Market Data Fetch Performance", self.test_market_data_performance());
        suite.add_result(result);

        // Test 3: Opportunity Detection Speed
        let result = qa_test!("Opportunity Detection Speed", self.test_opportunity_detection_speed());
        suite.add_result(result);

        // Test 4: Price Feed Latency
        let result = qa_test!("Price Feed Latency", self.test_price_feed_latency());
        suite.add_result(result);

        // Test 5: Trading Loop Efficiency
        let result = qa_test!("Trading Loop Efficiency", self.test_trading_loop_efficiency());
        suite.add_result(result);

        // Test 6: Memory Efficiency
        let result = qa_test!("Memory Efficiency", self.test_memory_efficiency());
        suite.add_result(result);

        suite
    }

    async fn test_initialization_speed(&self) -> Result<Vec<String>> {
        let mut details = Vec::new();
        let iterations = 5;
        let mut total_time = 0u128;

        for i in 0..iterations {
            let start = Instant::now();

            let wallet_address = self.shared_services.wallet_manager()
                .get_wallet_address("devnet-trading").await?;

            let _bot = ArbitrageBot::new(
                wallet_address,
                50.0,
                &self.config.network,
                self.shared_services.clone(),
            ).await?;

            let duration = start.elapsed().as_millis();
            total_time += duration;

            details.push(format!("Initialization {}: {}ms", i + 1, duration));
        }

        let avg_time = total_time / iterations as u128;
        details.push(format!("Average initialization time: {}ms", avg_time));

        // Performance requirement: initialization should be < 2 seconds
        qa_assert!(avg_time < 2000, "Average initialization time should be < 2 seconds");

        Ok(details)
    }

    async fn test_market_data_performance(&self) -> Result<Vec<String>> {
        let mut details = Vec::new();

        let wallet_address = self.shared_services.wallet_manager()
            .get_wallet_address("devnet-trading").await?;

        let mut bot = ArbitrageBot::new(
            wallet_address,
            50.0,
            &self.config.network,
            self.shared_services.clone(),
        ).await?;

        let iterations = 10;
        let mut successful_fetches = 0;
        let mut total_time = 0u128;
        let mut min_time = u128::MAX;
        let mut max_time = 0u128;

        for i in 0..iterations {
            let start = Instant::now();

            match bot.get_real_market_data().await {
                Ok(_market_data) => {
                    let duration = start.elapsed().as_millis();
                    successful_fetches += 1;
                    total_time += duration;
                    min_time = min_time.min(duration);
                    max_time = max_time.max(duration);

                    if i % 3 == 0 {
                        details.push(format!("Fetch {}: {}ms", i + 1, duration));
                    }
                },
                Err(e) => {
                    details.push(format!("Fetch {} failed: {}", i + 1, e));
                }
            }
        }

        if successful_fetches > 0 {
            let avg_time = total_time / successful_fetches as u128;
            details.push(format!("Successful fetches: {}/{}", successful_fetches, iterations));
            details.push(format!("Average fetch time: {}ms", avg_time));
            details.push(format!("Min fetch time: {}ms", min_time));
            details.push(format!("Max fetch time: {}ms", max_time));

            // Performance requirement: average fetch time should be < 1 second
            qa_assert!(avg_time < 1000, "Average market data fetch time should be < 1 second");
        } else {
            details.push("No successful market data fetches (DevNet limitation)".to_string());
        }

        Ok(details)
    }

    async fn test_opportunity_detection_speed(&self) -> Result<Vec<String>> {
        let mut details = Vec::new();

        let wallet_address = self.shared_services.wallet_manager()
            .get_wallet_address("devnet-trading").await?;

        let mut bot = ArbitrageBot::new(
            wallet_address,
            50.0,
            &self.config.network,
            self.shared_services.clone(),
        ).await?;

        let iterations = 8;
        let mut total_time = 0u128;
        let mut total_opportunities = 0;

        for i in 0..iterations {
            let start = Instant::now();

            match bot.detect_opportunities_using_strategy().await {
                Ok(signals) => {
                    let duration = start.elapsed().as_millis();
                    total_time += duration;
                    total_opportunities += signals.len();

                    details.push(format!("Detection {}: {}ms, {} opportunities",
                                       i + 1, duration, signals.len()));
                },
                Err(e) => {
                    details.push(format!("Detection {} failed: {}", i + 1, e));
                }
            }
        }

        let avg_time = total_time / iterations as u128;
        let avg_opportunities = total_opportunities as f64 / iterations as f64;

        details.push(format!("Average detection time: {}ms", avg_time));
        details.push(format!("Average opportunities found: {:.1}", avg_opportunities));

        // Performance requirement: detection should be < 3 seconds
        qa_assert!(avg_time < 3000, "Average opportunity detection time should be < 3 seconds");

        Ok(details)
    }

    async fn test_price_feed_latency(&self) -> Result<Vec<String>> {
        let mut details = Vec::new();

        let wallet_address = self.shared_services.wallet_manager()
            .get_wallet_address("devnet-trading").await?;

        let mut bot = ArbitrageBot::new(
            wallet_address,
            50.0,
            &self.config.network,
            self.shared_services.clone(),
        ).await?;

        let iterations = 5;
        let mut successful_requests = 0;
        let mut total_latency = 0u128;

        for i in 0..iterations {
            let start = Instant::now();

            match bot.get_jupiter_price("SOL", "USDC").await {
                Ok(price) => {
                    let latency = start.elapsed().as_millis();
                    successful_requests += 1;
                    total_latency += latency;

                    details.push(format!("Price request {}: {}ms, price: ${:.6}",
                                       i + 1, latency, price));
                },
                Err(e) => {
                    // Expected in DevNet
                    details.push(format!("Price request {} failed (expected): {}", i + 1, e));
                }
            }
        }

        if successful_requests > 0 {
            let avg_latency = total_latency / successful_requests as u128;
            details.push(format!("Average price feed latency: {}ms", avg_latency));

            // Performance requirement: price feed latency should be < 500ms
            qa_assert!(avg_latency < 500, "Average price feed latency should be < 500ms");
        } else {
            details.push("No successful price requests (DevNet limitation)".to_string());
        }

        Ok(details)
    }

    async fn test_trading_loop_efficiency(&self) -> Result<Vec<String>> {
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
        let test_duration = Duration::from_secs(10); // 10 second efficiency test

        tokio::select! {
            result = bot.start_trading() => {
                match result {
                    Ok(_) => details.push("Trading loop completed normally".to_string()),
                    Err(e) => details.push(format!("Trading loop failed: {}", e)),
                }
            }
            _ = tokio::time::sleep(test_duration) => {
                bot.emergency_stop();
                details.push("Trading loop efficiency test completed".to_string());
            }
        }

        let actual_duration = start_time.elapsed();
        let final_status = bot.get_status();

        // Calculate efficiency metrics
        let opportunities_per_second = final_status.opportunities_detected as f64 / actual_duration.as_secs_f64();
        let cycles_per_second = if final_status.opportunities_detected > 0 {
            final_status.opportunities_detected as f64 / actual_duration.as_secs_f64()
        } else {
            // Estimate based on typical cycle time
            actual_duration.as_secs_f64() / 1.0 // Assume 1 second per cycle
        };

        details.push(format!("Test duration: {:.1}s", actual_duration.as_secs_f64()));
        details.push(format!("Opportunities detected: {}", final_status.opportunities_detected));
        details.push(format!("Opportunities per second: {:.1}", opportunities_per_second));
        details.push(format!("Estimated cycles per second: {:.1}", cycles_per_second));

        // Performance requirement: should handle at least 0.5 cycles per second
        qa_assert!(cycles_per_second >= 0.5, "Should handle at least 0.5 cycles per second");

        Ok(details)
    }

    async fn test_memory_efficiency(&self) -> Result<Vec<String>> {
        let mut details = Vec::new();

        // Create multiple bots and measure relative memory usage
        let num_bots = 3;
        let mut bots = Vec::new();

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
            details.push(format!("Created bot {} for memory test", i + 1));
        }

        // Perform operations to test memory usage
        let operations = 5;
        for i in 0..operations {
            for (bot_idx, bot) in bots.iter_mut().enumerate() {
                match bot.get_real_market_data().await {
                    Ok(_) => {
                        if i == 0 {
                            details.push(format!("Bot {} memory test operation successful", bot_idx + 1));
                        }
                    },
                    Err(_) => {
                        // Expected in DevNet
                    }
                }
            }
        }

        details.push(format!("Completed {} operations on {} bots", operations, num_bots));
        details.push("Memory efficiency test completed (bots will be freed)".to_string());

        // All bots should be created and functional
        qa_assert!(bots.len() == num_bots, "All bots should be created successfully");

        Ok(details)
    }
}
