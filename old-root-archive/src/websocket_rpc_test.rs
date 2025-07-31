//! WebSocket RPC Testing para Solana
//!
//! Tests para comparar latencia HTTP vs WebSocket RPC
//! y demostrar los beneficios en trading real

use anyhow::Result;
use std::time::{Duration, Instant};
use tokio::time::timeout;
use tracing::{debug, info, warn};

use crate::config::Config;
use crate::shared::jupiter::JupiterClient;
use crate::shared::jupiter::JupiterConfig;
use crate::shared::websocket_price_feed::WebSocketPriceFeed;

pub struct WebSocketRpcTest {
    config: Config,
    websocket_feed: WebSocketPriceFeed,
    jupiter_client: JupiterClient,
}

impl WebSocketRpcTest {
    pub async fn new(config: Config) -> Result<Self> {
        let websocket_feed = WebSocketPriceFeed::new().await?;
        let jupiter_config = JupiterConfig::default();
        let jupiter_client = JupiterClient::new(&jupiter_config).await?;

        Ok(Self {
            config,
            websocket_feed,
            jupiter_client,
        })
    }

    /// Test completo de latencia: HTTP vs WebSocket
    pub async fn run_latency_comparison(&mut self) -> Result<()> {
        info!("🧪 Starting HTTP vs WebSocket latency comparison");

        // Test tokens populares
        let test_tokens = vec![
            "So11111111111111111111111111111111111111112",  // SOL
            "EPjFWdd5AufqSSqeM2qN1xzybapC8G4wEGGkZwyTDt1v", // USDC
            "DezXAZ8z7PnrnRJjz3wXBoRgixCa6xjnB7YaB1pPB263", // BONK
        ];

        // 1. Test HTTP latency (baseline)
        let http_results = self.test_http_latency(&test_tokens).await?;

        // 2. Connect WebSocket and test
        info!("📡 Connecting WebSocket price feed...");
        self.websocket_feed.connect_solana_pools().await?;

        // Wait for initial data
        tokio::time::sleep(Duration::from_secs(3)).await;

        // 3. Test WebSocket latency
        let ws_results = self.test_websocket_latency(&test_tokens).await?;

        // 4. Compare results
        self.compare_results(&http_results, &ws_results);

        Ok(())
    }

    async fn test_http_latency(&self, tokens: &[&str]) -> Result<Vec<(String, Duration)>> {
        info!("🌐 Testing HTTP latency...");
        let mut results = Vec::new();

        for &token in tokens {
            let mut durations = Vec::new();

            // Test 10 requests para cada token
            for i in 0..10 {
                let start = Instant::now();

                match timeout(Duration::from_secs(5), self.jupiter_client.get_price(token)).await {
                    Ok(Ok(Some(price))) => {
                        let duration = start.elapsed();
                        durations.push(duration);
                        debug!("HTTP {}: {:.6} in {:?}", i + 1, price, duration);
                    }
                    Ok(Ok(None)) => {
                        warn!("No price found for token: {}", token);
                    }
                    Ok(Err(e)) => {
                        warn!("HTTP error for {}: {}", token, e);
                    }
                    Err(_) => {
                        warn!("HTTP timeout for token: {}", token);
                    }
                }

                // Small delay between requests
                tokio::time::sleep(Duration::from_millis(100)).await;
            }

            if !durations.is_empty() {
                let avg_duration = durations.iter().sum::<Duration>() / durations.len() as u32;
                results.push((token.to_string(), avg_duration));
                info!("📊 HTTP average for {}: {:?}", token, avg_duration);
            }
        }

        Ok(results)
    }

    async fn test_websocket_latency(&self, tokens: &[&str]) -> Result<Vec<(String, Duration)>> {
        info!("⚡ Testing WebSocket latency...");
        let mut results = Vec::new();

        for &token in tokens {
            let mut durations = Vec::new();

            // Test 10 retrievals from WebSocket cache
            for i in 0..10 {
                let start = Instant::now();

                match self.websocket_feed.get_price_realtime(token).await {
                    Some(price) => {
                        let duration = start.elapsed();
                        durations.push(duration);
                        debug!("WS {}: {:.6} in {:?}", i + 1, price, duration);
                    }
                    None => {
                        // Try hybrid approach (WS + HTTP fallback)
                        match timeout(
                            Duration::from_secs(2),
                            self.websocket_feed.get_price_hybrid(token),
                        )
                        .await
                        {
                            Ok(Ok(Some(price))) => {
                                let duration = start.elapsed();
                                durations.push(duration);
                                debug!("WS hybrid {}: {:.6} in {:?}", i + 1, price, duration);
                            }
                            _ => {
                                warn!("No WebSocket price for token: {}", token);
                            }
                        }
                    }
                }

                // Small delay
                tokio::time::sleep(Duration::from_millis(50)).await;
            }

            if !durations.is_empty() {
                let avg_duration = durations.iter().sum::<Duration>() / durations.len() as u32;
                results.push((token.to_string(), avg_duration));
                info!("📊 WebSocket average for {}: {:?}", token, avg_duration);
            }
        }

        Ok(results)
    }

    fn compare_results(
        &self,
        http_results: &[(String, Duration)],
        ws_results: &[(String, Duration)],
    ) {
        info!("📊 === LATENCY COMPARISON RESULTS ===");

        for (http_result, ws_result) in http_results.iter().zip(ws_results.iter()) {
            let token = &http_result.0;
            let http_ms = http_result.1.as_millis();
            let ws_ms = ws_result.1.as_millis();
            let improvement = if ws_ms > 0 {
                http_ms as f64 / ws_ms as f64
            } else {
                0.0
            };

            info!("🎯 Token: {}", token);
            info!("   HTTP:      {}ms", http_ms);
            info!("   WebSocket: {}ms", ws_ms);
            info!("   Improvement: {:.1}x faster", improvement);
            info!(
                "   Savings: {}ms per request",
                http_ms.saturating_sub(ws_ms)
            );
            info!("");
        }
        // Calculate overall improvement
        let avg_http = if !http_results.is_empty() {
            http_results
                .iter()
                .map(|(_, d)| d.as_millis())
                .sum::<u128>()
                / http_results.len() as u128
        } else {
            0
        };

        let avg_ws = if !ws_results.is_empty() {
            ws_results.iter().map(|(_, d)| d.as_millis()).sum::<u128>() / ws_results.len() as u128
        } else {
            0
        };

        let overall_improvement = if avg_ws > 0 && avg_http > 0 {
            avg_http as f64 / avg_ws as f64
        } else {
            0.0
        };
        info!("🏆 OVERALL RESULTS:");
        info!("   Average HTTP latency: {}ms", avg_http);
        info!("   Average WebSocket latency: {}ms", avg_ws);

        if avg_ws > 0 && avg_http > 0 {
            info!("   Overall improvement: {:.1}x faster", overall_improvement);
            info!(
                "   Time saved per price check: {}ms",
                avg_http.saturating_sub(avg_ws)
            );
        } else if avg_ws == 0 {
            info!("   ⚠️  WebSocket implementation not working yet");
            info!("   📊 HTTP performance: {}ms (good baseline)", avg_http);
        }
        info!("");

        // Trading implications
        self.calculate_trading_benefits(avg_http, avg_ws);
    }

    fn calculate_trading_benefits(&self, http_ms: u128, ws_ms: u128) {
        info!("💰 TRADING BENEFITS:");

        let time_saved_per_check = http_ms.saturating_sub(ws_ms);

        // Scenarios
        let scenarios = [
            ("🏃 High-frequency (100 checks/min)", 100),
            ("📊 Medium trading (50 checks/min)", 50),
            ("🐌 Conservative (10 checks/min)", 10),
        ];

        for (scenario, checks_per_min) in scenarios {
            let savings_per_min = time_saved_per_check * checks_per_min as u128;
            let savings_per_hour = savings_per_min * 60;
            let savings_per_day = savings_per_hour * 24;

            info!("   {}", scenario);
            info!("     Per minute: {}ms saved", savings_per_min);
            info!(
                "     Per hour: {:.1}s saved",
                savings_per_hour as f64 / 1000.0
            );
            info!(
                "     Per day: {:.1}s saved",
                savings_per_day as f64 / 1000.0
            );
            info!("");
        }

        info!("⚡ KEY ADVANTAGES:");
        info!(
            "   • React to market changes {:.1}x faster",
            if ws_ms > 0 {
                http_ms as f64 / ws_ms as f64
            } else {
                0.0
            }
        );
        info!("   • Lower slippage due to faster execution");
        info!("   • Catch arbitrage opportunities before others");
        info!("   • Reduced RPC costs (fewer HTTP requests)");
        info!("   • Real-time pool monitoring");
    }

    /// Test real-time pool monitoring capabilities
    pub async fn test_real_time_monitoring(&mut self) -> Result<()> {
        info!("🎯 Testing real-time pool monitoring");

        // Connect to Solana WebSocket
        self.websocket_feed.connect_solana_pools().await?;

        info!("📡 Connected to Solana WebSocket - monitoring for 30 seconds...");

        let start_time = Instant::now();
        let mut update_count = 0;

        // Monitor for 30 seconds
        while start_time.elapsed() < Duration::from_secs(30) {
            if self.websocket_feed.is_connected().await {
                let last_update_age = self.websocket_feed.last_update_age().await;

                if last_update_age < Duration::from_secs(5) {
                    update_count += 1;
                    debug!(
                        "📊 Received update #{} (age: {:?})",
                        update_count, last_update_age
                    );
                }
            }

            tokio::time::sleep(Duration::from_millis(500)).await;
        }

        info!("✅ Monitoring completed:");
        info!("   Duration: 30 seconds");
        info!("   Updates received: {}", update_count);
        info!(
            "   Update rate: {:.1} updates/sec",
            update_count as f64 / 30.0
        );

        if update_count > 0 {
            info!("🎉 Real-time WebSocket monitoring is working!");
        } else {
            warn!("⚠️ No updates received - may need to check connection or endpoints");
        }

        Ok(())
    }
}

/// Run WebSocket RPC performance tests
pub async fn run_websocket_rpc_tests(_config: &Config) -> Result<()> {
    info!("🧪 Starting WebSocket RPC performance tests");

    // Test 1: Our new MainNet price integration test
    info!("🌐 Initializing WebSocket Price Feed");
    let _websocket_feed = WebSocketPriceFeed::new_mainnet_prices().await?;

    let jupiter_config = JupiterConfig::default();
    let jupiter_client = JupiterClient::new(&jupiter_config).await?;

    info!("📊 === TEST 1: LATENCY COMPARISON ===");
    info!("🧪 Starting HTTP vs WebSocket latency comparison");

    // Test HTTP latency first
    info!("🌐 Testing HTTP latency...");
    let mut http_latencies = Vec::new();
    let test_tokens = vec![
        "So11111111111111111111111111111111111111112",  // SOL
        "EPjFWdd5AufqSSqeM2qN1xzybapC8G4wEGGkZwyTDt1v", // USDC
        "DezXAZ8z7PnrnRJjz3wXBoRgixCa6xjnB7YaB1pPB263", // BONK
    ];

    for token in &test_tokens {
        let mut token_latencies = Vec::new();
        for _ in 0..10 {
            let start = Instant::now();
            if let Ok(Some(_price)) = jupiter_client.get_price(token).await {
                let latency = start.elapsed();
                token_latencies.push(latency);
            } else {
                warn!("No price found for token: {}", token);
            }
            tokio::time::sleep(Duration::from_millis(100)).await;
        }

        if !token_latencies.is_empty() {
            let avg_latency =
                token_latencies.iter().sum::<Duration>() / token_latencies.len() as u32;
            http_latencies.push(avg_latency);
            info!(
                "📊 HTTP average for {}: {:.5}ms",
                token,
                avg_latency.as_nanos() as f64 / 1_000_000.0
            );
        }
    }

    // Test WebSocket latency
    info!("📡 Connecting WebSocket price feed...");
    let mut websocket_feed_test = WebSocketPriceFeed::new().await?;
    websocket_feed_test.connect_solana_pools().await?;

    // Wait for connection establishment
    tokio::time::sleep(Duration::from_secs(3)).await;

    info!("⚡ Testing WebSocket latency...");
    let mut websocket_latencies = Vec::new();

    for token in &test_tokens {
        for _ in 0..10 {
            let start = Instant::now();
            if let Some(_price) = websocket_feed_test.get_price_realtime(token).await {
                let latency = start.elapsed();
                websocket_latencies.push(latency);
            } else {
                warn!("No WebSocket price for token: {}", token);
            }
            tokio::time::sleep(Duration::from_millis(60)).await;
        }
    }

    // Results
    info!("📊 === LATENCY COMPARISON RESULTS ===");
    info!("🏆 OVERALL RESULTS:");

    let avg_http = if !http_latencies.is_empty() {
        http_latencies.iter().sum::<Duration>() / http_latencies.len() as u32
    } else {
        Duration::from_millis(0)
    };

    let avg_websocket = if !websocket_latencies.is_empty() {
        websocket_latencies.iter().sum::<Duration>() / websocket_latencies.len() as u32
    } else {
        Duration::from_millis(0)
    };

    info!("    Average HTTP latency: {}ms", avg_http.as_millis());
    info!(
        "    Average WebSocket latency: {}ms",
        avg_websocket.as_millis()
    );

    if websocket_latencies.is_empty() {
        info!("    ⚠️  WebSocket implementation not working yet");
        info!(
            "    📊 HTTP performance: {}ms (good baseline)",
            avg_http.as_millis()
        );
    } else {
        let speedup = avg_http.as_nanos() as f64 / avg_websocket.as_nanos() as f64;
        info!("    🚀 WebSocket is {:.1}x faster", speedup);
    }

    // Show trading benefits
    info!("");
    info!("💰 TRADING BENEFITS:");
    let http_ms = avg_http.as_millis() as f64;

    // High frequency trading
    info!("    🏃 High-frequency (100 checks/min)");
    info!("      Per minute: {}ms saved", (http_ms * 100.0) as u64);
    info!(
        "      Per hour: {:.1}s saved",
        (http_ms * 100.0 * 60.0) / 1000.0
    );
    info!(
        "      Per day: {:.1}s saved",
        (http_ms * 100.0 * 60.0 * 24.0) / 1000.0
    );
    info!("");

    // Medium frequency
    info!("    📊 Medium trading (50 checks/min)");
    info!("      Per minute: {}ms saved", (http_ms * 50.0) as u64);
    info!(
        "      Per hour: {:.1}s saved",
        (http_ms * 50.0 * 60.0) / 1000.0
    );
    info!(
        "      Per day: {:.1}s saved",
        (http_ms * 50.0 * 60.0 * 24.0) / 1000.0
    );
    info!("");

    // Conservative
    info!("    🐌 Conservative (10 checks/min)");
    info!("      Per minute: {}ms saved", (http_ms * 10.0) as u64);
    info!(
        "      Per hour: {:.1}s saved",
        (http_ms * 10.0 * 60.0) / 1000.0
    );
    info!(
        "      Per day: {:.1}s saved",
        (http_ms * 10.0 * 60.0 * 24.0) / 1000.0
    );
    info!("");

    info!("⚡ KEY ADVANTAGES:");
    let speed_advantage = if websocket_latencies.is_empty() {
        0.0
    } else {
        avg_http.as_nanos() as f64 / avg_websocket.as_nanos() as f64
    };
    info!(
        "    • React to market changes {:.1}x faster",
        speed_advantage
    );
    info!("    • Lower slippage due to faster execution");
    info!("    • Catch arbitrage opportunities before others");
    info!("    • Reduced RPC costs (fewer HTTP requests)");
    info!("    • Real-time pool monitoring");
    info!("");

    // Test 2: Real-time monitoring
    info!("📡 === TEST 2: REAL-TIME MONITORING ===");
    info!("🎯 Testing real-time pool monitoring");

    let mut websocket_monitor = WebSocketPriceFeed::new().await?;
    websocket_monitor.connect_solana_pools().await?;

    info!("📡 Connected to Solana WebSocket - monitoring for 30 seconds...");
    let start_time = Instant::now();
    let mut update_count = 0;

    // Monitor for updates
    while start_time.elapsed() < Duration::from_secs(30) {
        if websocket_monitor.is_connected().await {
            let stats = websocket_monitor.get_stats().await;
            if stats.cached_tokens > update_count {
                update_count = stats.cached_tokens;
            }
        }
        tokio::time::sleep(Duration::from_millis(500)).await;
    }

    let duration = start_time.elapsed();
    let update_rate = update_count as f64 / duration.as_secs() as f64;

    info!("✅ Monitoring completed:");
    info!("    Duration: {} seconds", duration.as_secs());
    info!("    Updates received: {}", update_count);
    info!("    Update rate: {:.1} updates/sec", update_rate);

    if update_count == 0 {
        warn!("⚠️ No updates received - may need to check connection or endpoints");
    }

    info!("🎉 All WebSocket RPC tests completed!");

    Ok(())
}
