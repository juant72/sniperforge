/// Ultimate RPC Performance Comparison
/// 
/// Compara diferentes proveedores de RPC para encontrar el más rápido:
/// 1. Current HTTP Jupiter API
/// 2. Syndica WebSocket
/// 3. Helius WebSocket  
/// 4. Standard Solana RPC WebSocket

use anyhow::Result;
use std::time::{Duration, Instant};
use tracing::{info, warn, debug};
use tokio::time::timeout;

use crate::config::Config;
use crate::shared::jupiter::client::JupiterClient;
use crate::shared::jupiter::JupiterConfig;
use crate::shared::syndica_websocket::{SyndicaWebSocketClient, SyndicaConfig};

pub struct UltimateRpcComparison {
    jupiter_client: JupiterClient,
    syndica_client: Option<SyndicaWebSocketClient>,
}

impl UltimateRpcComparison {
    pub async fn new() -> Result<Self> {
        info!("🚀 Initializing Ultimate RPC Performance Comparison");
        
        // Initialize Jupiter HTTP client (our current best)
        let jupiter_config = JupiterConfig::default();
        let jupiter_client = JupiterClient::new(&jupiter_config).await?;
        
        // Try to initialize Syndica WebSocket (if token available)
        let syndica_client = if std::env::var("SYNDICA_TOKEN").is_ok() {
            let config = SyndicaConfig::default();
            match SyndicaWebSocketClient::new(config).await {
                Ok(mut client) => {
                    info!("🔗 Attempting Syndica connection...");
                    match client.connect().await {
                        Ok(_) => {
                            info!("✅ Syndica WebSocket connected successfully");
                            Some(client)
                        }
                        Err(e) => {
                            warn!("⚠️ Syndica connection failed: {}", e);
                            None
                        }
                    }
                }
                Err(e) => {
                    warn!("⚠️ Syndica client creation failed: {}", e);
                    None
                }
            }
        } else {
            info!("ℹ️ SYNDICA_TOKEN not found, skipping Syndica test");
            None
        };
        
        Ok(Self {
            jupiter_client,
            syndica_client,
        })
    }

    /// Run comprehensive performance comparison
    pub async fn run_ultimate_comparison(&self) -> Result<()> {
        info!("🏁 Starting Ultimate RPC Performance Battle");
        println!("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━");
        println!("🎯 ULTIMATE RPC PERFORMANCE COMPARISON");
        println!("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━");
        
        let test_token = "So11111111111111111111111111111111111111112"; // SOL
        let iterations = 10;
        
        // Test 1: Current Jupiter HTTP (our baseline)
        println!("\n🔥 TEST 1: Jupiter HTTP API (Current Champion)");
        println!("─────────────────────────────────────────────────");
        let http_results = self.test_jupiter_http(test_token, iterations).await?;
        
        // Test 2: Syndica WebSocket (if available)
        if let Some(syndica) = &self.syndica_client {
            println!("\n⚡ TEST 2: Syndica Ultra-Fast WebSocket");
            println!("─────────────────────────────────────────────────");
            let syndica_results = self.test_syndica_websocket(syndica, test_token, iterations).await?;
            self.compare_results("Jupiter HTTP", &http_results, "Syndica WebSocket", &syndica_results);
        } else {
            println!("\n⚠️ TEST 2: Syndica WebSocket - SKIPPED (no token)");
            println!("   💡 Set SYNDICA_TOKEN=your_token to test");
        }
        
        // Test 3: Hybrid Strategy (best of both worlds)
        println!("\n🚀 TEST 3: Hybrid Strategy (WebSocket + HTTP Fallback)");
        println!("─────────────────────────────────────────────────");
        let hybrid_results = self.test_hybrid_strategy(test_token, iterations).await?;
        self.compare_results("Jupiter HTTP", &http_results, "Hybrid Strategy", &hybrid_results);
        
        // Final recommendations
        self.show_recommendations(&http_results, &hybrid_results);
        
        Ok(())
    }

    /// Test Jupiter HTTP performance (our current champion)
    async fn test_jupiter_http(&self, token: &str, iterations: usize) -> Result<Vec<Duration>> {
        let mut results = Vec::new();
        
        for i in 1..=iterations {
            let start = Instant::now();
            match self.jupiter_client.get_price(token).await {
                Ok(Some(price)) => {
                    let duration = start.elapsed();
                    results.push(duration);
                    debug!("HTTP {}: ${:.4} in {}ms", i, price, duration.as_millis());
                }
                Ok(None) => {
                    warn!("HTTP {}: No price returned", i);
                    results.push(Duration::from_millis(999)); // Penalty for failure
                }
                Err(e) => {
                    warn!("HTTP {}: Error - {}", i, e);
                    results.push(Duration::from_millis(999)); // Penalty for failure
                }
            }
            
            // Small delay between tests
            tokio::time::sleep(Duration::from_millis(100)).await;
        }
        
        let avg_ms = results.iter().map(|d| d.as_millis()).sum::<u128>() / results.len() as u128;
        println!("📊 Jupiter HTTP Average: {}ms", avg_ms);
        
        Ok(results)
    }

    /// Test Syndica WebSocket performance
    async fn test_syndica_websocket(&self, syndica: &SyndicaWebSocketClient, token: &str, iterations: usize) -> Result<Vec<Duration>> {
        let mut results = Vec::new();
        
        // Wait for WebSocket to populate cache
        println!("⏳ Waiting for Syndica WebSocket to warm up...");
        tokio::time::sleep(Duration::from_secs(3)).await;
        
        for i in 1..=iterations {
            let start = Instant::now();
            match syndica.get_price_ultra_fast(token).await {
                Ok(Some(price)) => {
                    let duration = start.elapsed();
                    results.push(duration);
                    debug!("Syndica {}: ${:.4} in {}μs", i, price, duration.as_micros());
                }
                Ok(None) => {
                    warn!("Syndica {}: No WebSocket data available", i);
                    results.push(Duration::from_millis(999)); // Penalty
                }
                Err(e) => {
                    warn!("Syndica {}: Error - {}", i, e);
                    results.push(Duration::from_millis(999)); // Penalty
                }
            }
            
            tokio::time::sleep(Duration::from_millis(50)).await;
        }
        
        let avg_micros = results.iter().map(|d| d.as_micros()).sum::<u128>() / results.len() as u128;
        println!("📊 Syndica WebSocket Average: {}μs ({}ms)", avg_micros, avg_micros / 1000);
        
        Ok(results)
    }

    /// Test hybrid strategy (WebSocket first, HTTP fallback)
    async fn test_hybrid_strategy(&self, token: &str, iterations: usize) -> Result<Vec<Duration>> {
        let mut results = Vec::new();
        
        for i in 1..=iterations {
            let start = Instant::now();
            
            // Try WebSocket first (if available)
            let mut price_found = false;
            
            if let Some(syndica) = &self.syndica_client {
                if let Ok(Some(price)) = syndica.get_price_ultra_fast(token).await {
                    let duration = start.elapsed();
                    results.push(duration);
                    debug!("Hybrid {}: ${:.4} via WebSocket in {}μs", i, price, duration.as_micros());
                    price_found = true;
                }
            }
            
            // Fallback to HTTP if WebSocket failed
            if !price_found {
                match self.jupiter_client.get_price(token).await {
                    Ok(Some(price)) => {
                        let duration = start.elapsed();
                        results.push(duration);
                        debug!("Hybrid {}: ${:.4} via HTTP fallback in {}ms", i, price, duration.as_millis());
                    }
                    _ => {
                        results.push(Duration::from_millis(999)); // Penalty
                    }
                }
            }
            
            tokio::time::sleep(Duration::from_millis(75)).await;
        }
        
        let avg_ms = results.iter().map(|d| d.as_millis()).sum::<u128>() / results.len() as u128;
        println!("📊 Hybrid Strategy Average: {}ms", avg_ms);
        
        Ok(results)
    }

    /// Compare two sets of results
    fn compare_results(&self, name1: &str, results1: &[Duration], name2: &str, results2: &[Duration]) {
        let avg1 = results1.iter().map(|d| d.as_millis()).sum::<u128>() / results1.len() as u128;
        let avg2 = results2.iter().map(|d| d.as_millis()).sum::<u128>() / results2.len() as u128;
        
        println!("\n🏆 PERFORMANCE COMPARISON:");
        println!("   {}: {}ms", name1, avg1);
        println!("   {}: {}ms", name2, avg2);
        
        if avg2 < avg1 {
            let improvement = avg1 as f64 / avg2 as f64;
            println!("   🚀 {} is {:.1}x FASTER! ({} ms saved)", name2, improvement, avg1 - avg2);
        } else if avg1 < avg2 {
            let improvement = avg2 as f64 / avg1 as f64;
            println!("   🏆 {} is {:.1}x FASTER! ({} ms saved)", name1, improvement, avg2 - avg1);
        } else {
            println!("   🤝 Both systems have similar performance");
        }
    }

    /// Show final recommendations based on test results
    fn show_recommendations(&self, http_results: &[Duration], hybrid_results: &[Duration]) {
        println!("\n");
        println!("🎯 FINAL RECOMMENDATIONS");
        println!("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━");
        
        let http_avg = http_results.iter().map(|d| d.as_millis()).sum::<u128>() / http_results.len() as u128;
        let hybrid_avg = hybrid_results.iter().map(|d| d.as_millis()).sum::<u128>() / hybrid_results.len() as u128;
        
        if self.syndica_client.is_some() {
            println!("✅ Syndica WebSocket: AVAILABLE and tested");
            if hybrid_avg < http_avg {
                println!("🚀 RECOMMENDATION: Use Hybrid Strategy");
                println!("   • WebSocket for ultra-fast price checks");
                println!("   • HTTP fallback for reliability");
                println!("   • Best of both worlds approach");
            } else {
                println!("🎯 RECOMMENDATION: Continue with HTTP");
                println!("   • HTTP performance is already excellent");
                println!("   • WebSocket complexity not worth marginal gains");
            }
        } else {
            println!("⚠️ Syndica WebSocket: NOT TESTED (missing token)");
            println!("🎯 RECOMMENDATION: Current HTTP system is excellent");
            println!("   • {}ms average latency is very competitive", http_avg);
            println!("   • Consider getting Syndica token for future testing");
        }
        
        println!("\n💡 NEXT STEPS:");
        println!("   1. If keeping HTTP: Focus on trading logic implementation");
        println!("   2. If adding WebSocket: Implement gradual rollout");
        println!("   3. Monitor real trading performance under load");
    }
}

/// Main test function callable from CLI
pub async fn run_ultimate_rpc_comparison() -> Result<()> {
    let comparison = UltimateRpcComparison::new().await?;
    comparison.run_ultimate_comparison().await?;
    Ok(())
}
