//! Ultimate RPC Performance Comparison
//! 
//! Compara diferentes proveedores de RPC para encontrar el más rápido:
//! 1. Current HTTP Jupiter API
//! 2. Syndica WebSocket
//! 3. Helius WebSocket  
//! 4. Standard Solana RPC WebSocket

use anyhow::Result;
use std::time::{Duration, Instant};
use tracing::{info, warn, debug};
use tokio::time::timeout;

use crate::config::Config;
use crate::shared::jupiter::JupiterClient;
use crate::shared::jupiter::JupiterConfig;
use crate::shared::syndica_websocket::{SyndicaWebSocketClient, SyndicaConfig};

pub struct UltimateRpcComparison {
    jupiter_client: JupiterClient,
}

impl UltimateRpcComparison {
    pub async fn new() -> Result<Self> {
        info!("🚀 Initializing Ultimate RPC Performance Comparison");
        
        // Initialize Jupiter HTTP client (our current best)
        debug!("🌐 Setting up Jupiter HTTP client");
        let jupiter_config = JupiterConfig::default();
        let jupiter_client = JupiterClient::new(&jupiter_config).await?;
        info!("✅ Jupiter client initialized");
        
        Ok(Self {
            jupiter_client,
        })
    }
    
    /// Create Syndica WebSocket client only when needed
    async fn create_syndica_client() -> Result<Option<SyndicaWebSocketClient>> {
        if std::env::var("SYNDICA_TOKEN").is_ok() {
            info!("🚀 Initializing Syndica Ultra-Fast WebSocket Client");
            info!("   Endpoint: wss://solana-devnet.api.syndica.io");
            
            let config = SyndicaConfig::default();
            match SyndicaWebSocketClient::new(config).await {
                Ok(mut client) => {
                    info!("🔗 Attempting Syndica connection...");
                    match client.connect().await {
                        Ok(_) => {
                            info!("✅ Syndica WebSocket connected successfully");
                            Ok(Some(client))
                        }
                        Err(e) => {
                            warn!("⚠️ Syndica connection failed: {}", e);
                            Ok(None)
                        }
                    }
                }
                Err(e) => {
                    warn!("⚠️ Syndica client creation failed: {}", e);
                    Ok(None)
                }
            }
        } else {
            info!("ℹ️ SYNDICA_TOKEN not found, skipping Syndica test");
            Ok(None)        }
    }

    /// Run comprehensive performance comparison
    pub async fn run_ultimate_comparison(&self) -> Result<()> {
        info!("🏁 Starting Ultimate RPC Performance Battle");
        println!("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━");
        println!("🎯 ULTIMATE RPC PERFORMANCE COMPARISON");
        println!("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━");
        
        let test_token = "So11111111111111111111111111111111111111112"; // SOL
        let iterations = 10;
          // Test 1: PURE Jupiter HTTP (isolated, no WebSocket interference)
        println!("\n🔥 TEST 1: Pure Jupiter HTTP API (Isolated Test)");
        println!("─────────────────────────────────────────────────");
        let http_results = self.test_jupiter_http(test_token, iterations).await?;
          // Test 2: Syndica WebSocket (created fresh for this test only)
        if std::env::var("SYNDICA_TOKEN").is_ok() {
            println!("\n⚡ TEST 2: Syndica Ultra-Fast WebSocket (Fresh Connection)");
            println!("─────────────────────────────────────────────────");
            
            // Create fresh Syndica client for isolated test
            match Self::create_syndica_client().await? {
                Some(syndica) => {
                    // Run diagnostics first
                    println!("🔍 Running pre-test diagnostics...");
                    syndica.run_cache_diagnostics(test_token).await?;
                    
                    let syndica_results = self.test_syndica_websocket(&syndica, test_token, iterations).await?;
                    self.compare_results("Jupiter HTTP", &http_results, "Syndica WebSocket", &syndica_results);
                    
                    // Run post-test diagnostics
                    println!("\n🔍 Running post-test diagnostics...");
                    syndica.run_cache_diagnostics(test_token).await?;
                }
                None => {
                    println!("⚠️ Syndica WebSocket test skipped (connection failed)");
                }
            }
        } else {
            println!("\n⚠️ TEST 2: Syndica WebSocket - SKIPPED (no token)");
            println!("   💡 Set SYNDICA_TOKEN=your_token to test");
        };
        
        // Test 3: Hybrid Strategy (WebSocket + HTTP fallback)
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
    }    /// Test Syndica WebSocket performance
    async fn test_syndica_websocket(&self, syndica: &SyndicaWebSocketClient, token: &str, iterations: usize) -> Result<Vec<Duration>> {
        let mut results = Vec::new();
        let mut prices_seen = Vec::new();
        
        // Wait for WebSocket to populate cache
        println!("⏳ Waiting for Syndica WebSocket to warm up...");
        tokio::time::sleep(Duration::from_secs(3)).await;
        
        for i in 1..=iterations {
            let start = Instant::now();
            match syndica.get_price_ultra_fast(token).await {
                Ok(Some(price)) => {
                    let duration = start.elapsed();
                    results.push(duration);
                    prices_seen.push(price);
                      // Data validation flags
                    let mut flags = Vec::new();
                    if duration.as_micros() < 1 { flags.push("SUSPICIOUSLY_FAST"); }
                    if duration.as_micros() > 50000 { flags.push("SLOW"); }
                    if !(50.0..=500.0).contains(&price) { flags.push("PRICE_OUT_OF_RANGE"); }
                    
                    // Determine likely data source based on timing and cache status
                    let cache_health = syndica.get_cache_health().await;
                    let data_source = if cache_health.total_entries == 0 {
                        "HTTP_FALLBACK"
                    } else if duration.as_micros() < 5000 {
                        "WEBSOCKET_CACHE"
                    } else {
                        "HTTP_FALLBACK"
                    };
                    flags.push(data_source);
                    
                    let flag_str = if flags.is_empty() { "".to_string() } else { format!(" [{}]", flags.join(",")) };
                    
                    // Show both microseconds and milliseconds for clarity
                    println!("Syndica {}: ${:.4} in {}μs ({:.3}ms){}", 
                             i, price, duration.as_micros(), duration.as_secs_f64() * 1000.0, flag_str);
                }
                Ok(None) => {
                    println!("Syndica {}: No WebSocket data available", i);
                    results.push(Duration::from_millis(999)); // Penalty
                }
                Err(e) => {
                    println!("Syndica {}: Error - {}", i, e);
                    results.push(Duration::from_millis(999)); // Penalty
                }
            }
            
            tokio::time::sleep(Duration::from_millis(50)).await;
        }
        
        // Calculate and display comprehensive statistics
        let successful_results: Vec<_> = results.iter().filter(|&&d| d.as_millis() < 999).collect();
        let hit_rate = successful_results.len() as f64 / results.len() as f64 * 100.0;
        
        if !successful_results.is_empty() {
            let avg_ms = successful_results.iter().map(|d| d.as_millis()).sum::<u128>() / successful_results.len() as u128;
            let avg_micros = successful_results.iter().map(|d| d.as_micros()).sum::<u128>() / successful_results.len() as u128;
            let min_micros = successful_results.iter().map(|d| d.as_micros()).min().unwrap_or(0);
            let max_micros = successful_results.iter().map(|d| d.as_micros()).max().unwrap_or(0);
              let cache_health = syndica.get_cache_health().await;
            
            println!("📊 Syndica WebSocket Statistics:");
            println!("   Cache Hit Rate: {:.1}% ({}/{})", hit_rate, successful_results.len(), results.len());
            println!("   Average: {}ms ({}μs)", avg_ms, avg_micros);
            println!("   Range: {}μs - {}μs", min_micros, max_micros);
            
            // Data source analysis
            if cache_health.total_entries == 0 {
                println!("   ⚠️  Data Source: HTTP FALLBACK ONLY (no WebSocket price data)");
                println!("   ⚠️  Performance gains are from HTTP caching, not WebSocket");
            } else {
                println!("   ✅ Data Source: WebSocket cache ({} entries)", cache_health.total_entries);
            }
            
            // Price consistency check
            if prices_seen.len() >= 2 {
                let min_price = prices_seen.iter().fold(f64::INFINITY, |a, &b| a.min(b));
                let max_price = prices_seen.iter().fold(f64::NEG_INFINITY, |a, &b| a.max(b));
                let price_variation = (max_price - min_price) / min_price * 100.0;
                println!("   Price Range: ${:.4} - ${:.4} (±{:.2}%)", min_price, max_price, price_variation);
            }
        } else {
            println!("📊 Syndica WebSocket: NO SUCCESSFUL REQUESTS");
        }
        
        Ok(results)
    }

    /// Test hybrid strategy (WebSocket first, HTTP fallback)
    async fn test_hybrid_strategy(&self, token: &str, iterations: usize) -> Result<Vec<Duration>> {
        let mut results = Vec::new();
        
        // Create fresh Syndica client for hybrid test (isolated)
        let syndica_client = Self::create_syndica_client().await?;
        
        for i in 1..=iterations {
            let start = Instant::now();
            
            // Try WebSocket first (if available)
            let mut price_found = false;
            
            if let Some(ref syndica) = syndica_client {
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
          if avg2 == 0 && avg1 == 0 {
            println!("   🤝 Both systems have sub-millisecond performance");
        } else if avg2 == 0 {
            println!("   🚀 {} has sub-millisecond latency! ({}ms saved)", name2, avg1);
        } else if avg1 == 0 {
            println!("   🚀 {} has sub-millisecond latency! ({}ms saved)", name1, avg2);
        } else if avg2 < avg1 {
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
        
        if std::env::var("SYNDICA_TOKEN").is_ok() {
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
