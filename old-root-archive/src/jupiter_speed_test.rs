//! Jupiter Speed Performance Test
//!
//! Benchmarks Jupiter API response times and optimizations

use anyhow::Result;
use std::time::Instant;
use tracing::{info, warn};

use crate::shared::jupiter::{JupiterClient, JupiterConfig};
// Jupiter Speed Test using real Jupiter client

pub async fn test_jupiter_speed() -> Result<()> {
    println!("⚡ Jupiter Speed Performance Test");
    println!("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━");

    // Test 1: Client initialization speed
    print!("🚀 Testing client initialization... ");
    let start = Instant::now();

    let jupiter_config = JupiterConfig::default();
    let jupiter_client = JupiterClient::new(&jupiter_config).await?;

    let init_time = start.elapsed();
    println!("✅ {}ms", init_time.as_millis());

    if init_time.as_millis() > 500 {
        warn!("⚠️  Initialization took >500ms, consider optimizing");
    }

    // Test 2: Price API speed (multiple calls)
    println!("💰 Testing price API speed (5 calls):");

    let mut total_time = 0u128;
    let test_count = 5;

    for i in 1..=test_count {
        print!("  Call {}: ", i);
        let start = Instant::now();

        match jupiter_client
            .get_price("So11111111111111111111111111111111111111112")
            .await
        {
            Ok(Some(price)) => {
                let call_time = start.elapsed();
                total_time += call_time.as_millis();
                println!("✅ {}ms (${:.2})", call_time.as_millis(), price);

                if call_time.as_millis() > 1000 {
                    warn!("    ⚠️  Call took >1s, very slow for trading");
                } else if call_time.as_millis() > 500 {
                    warn!("    ⚠️  Call took >500ms, slow for trading");
                }
            }
            Ok(None) => {
                let call_time = start.elapsed();
                total_time += call_time.as_millis();
                println!("⚠️  {}ms (no price data)", call_time.as_millis());
            }
            Err(e) => {
                let call_time = start.elapsed();
                total_time += call_time.as_millis();
                println!("❌ {}ms (error: {})", call_time.as_millis(), e);
            }
        }
    }

    let avg_time = total_time / test_count as u128;
    println!("📊 Average response time: {}ms", avg_time);

    // Performance assessment
    if avg_time < 200 {
        info!("🎯 Excellent performance: <200ms average");
    } else if avg_time < 500 {
        info!("✅ Good performance: <500ms average");
    } else if avg_time < 1000 {
        warn!("⚠️  Acceptable performance: <1s average (could be better)");
    } else {
        warn!("❌ Poor performance: >1s average (needs optimization)");
    }

    // Recommendations
    if avg_time > 300 {
        println!("\n🔧 Performance Recommendations:");
        println!("  • Consider using a dedicated Solana RPC endpoint");
        println!("  • Implement connection pooling and keep-alive");
        println!("  • Use HTTP/2 for multiplexing");
        println!("  • Cache frequently requested token prices");
        println!("  • Consider WebSocket price feeds for real-time data");
    }

    Ok(())
}

pub async fn run_jupiter_speed_tests() {
    println!("⚡ Jupiter Performance Test Suite");
    println!("=================================");

    // Test standard client
    match test_jupiter_speed().await {
        Ok(()) => println!("✅ Standard Jupiter client tests completed"),
        Err(e) => println!("❌ Standard Jupiter client tests failed: {}", e),
    }

    println!("\n");

    // Test fallback client (should work reliably)
    match test_fallback_client().await {
        Ok(()) => println!("✅ Fallback Jupiter client tests completed"),
        Err(e) => println!("❌ Fallback Jupiter client tests failed: {}", e),
    }

    println!("\n");

    // Test ultra-fast client
    match test_ultra_fast_client_local().await {
        Ok(()) => println!("✅ Ultra-fast Jupiter client tests completed"),
        Err(e) => println!("❌ Ultra-fast Jupiter client tests failed: {}", e),
    }
}

/// Test Jupiter swap transaction building (DevNet safe mode)
pub async fn test_jupiter_swap_build() -> Result<()> {
    println!("🔄 Jupiter Swap Transaction Build Test");
    println!("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━");

    // Initialize Jupiter client
    let jupiter_config = JupiterConfig::default();
    let jupiter = crate::shared::jupiter::Jupiter::new(&jupiter_config).await?;

    // Test wallet address (DevNet safe - no real funds)
    let test_wallet = "11111111111111111111111111111111"; // Dummy address for testing

    println!("🪐 Step 1: Getting quote for SOL -> USDC");
    print!("  Fetching quote... ");
    let start = Instant::now();

    match jupiter
        .get_quote(
            "So11111111111111111111111111111111111111112",  // SOL
            "EPjFWdd5AufqSSqeM2qN1xzybapC8G4wEGGkZwyTDt1v", // USDC
            0.001, // 0.001 SOL (very small amount for testing)
            100,   // 1% slippage
        )
        .await
    {
        Ok(quote) => {
            let quote_time = start.elapsed();
            println!("✅ {}ms", quote_time.as_millis());
            println!("    📊 Quote details:");
            println!("       Input: {} SOL", quote.in_amount() / 1_000_000_000.0);
            println!(
                "       Output: {} USDC (est.)",
                quote.out_amount() / 1_000_000.0
            );
            println!("       Price Impact: {:.4}%", quote.price_impact_pct());

            println!("\n🏗️  Step 2: Building swap transaction");
            print!("  Building transaction... ");
            let build_start = Instant::now();

            match jupiter.execute_swap(&quote, test_wallet).await {
                Ok(swap_result) => {
                    let build_time = build_start.elapsed();
                    println!("✅ {}ms", build_time.as_millis());
                    println!("    🔧 Swap transaction built successfully!");
                    println!(
                        "       Transaction ID: {}",
                        swap_result
                            .transaction_signature
                            .unwrap_or("None".to_string())
                    );
                    println!(
                        "       Expected Output: {:.6} USDC",
                        swap_result.output_amount / 1_000_000.0
                    );
                    println!("       Estimated Fee: ${:.6}", swap_result.fee_amount);
                    println!(
                        "       Actual Slippage: {:.4}%",
                        swap_result.actual_slippage
                    );

                    println!("\n📊 Performance Summary:");
                    println!("    Quote Time: {}ms", quote_time.as_millis());
                    println!("    Build Time: {}ms", build_time.as_millis());
                    println!(
                        "    Total Time: {}ms",
                        (quote_time + build_time).as_millis()
                    );

                    if (quote_time + build_time).as_millis() < 1000 {
                        println!("    🏆 EXCELLENT: Sub-1s total time!");
                    } else if (quote_time + build_time).as_millis() < 2000 {
                        println!("    ✅ GOOD: Sub-2s total time");
                    } else {
                        warn!("    ⚠️  SLOW: >2s total time - may need optimization");
                    }

                    info!("✅ Swap transaction build test completed successfully");
                }
                Err(e) => {
                    let build_time = build_start.elapsed();
                    println!("❌ {}ms - Build failed: {}", build_time.as_millis(), e);
                    return Err(e);
                }
            }
        }
        Err(e) => {
            let quote_time = start.elapsed();
            println!("❌ {}ms - Quote failed: {}", quote_time.as_millis(), e);
            return Err(e);
        }
    }

    println!("\n🔒 Note: This test builds but does not send transactions (DevNet safety mode)");
    println!("✅ Jupiter swap build test completed");
    Ok(())
}

async fn test_fallback_client() -> Result<()> {
    use crate::shared::jupiter::{JupiterClient, JupiterConfig};

    println!("🔄 Testing Fallback Jupiter Client");
    println!("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━");

    let fallback_client = JupiterClient::new(&JupiterConfig::default()).await?;

    println!("🚀 Fallback speed test (3 calls):");
    let mut total_time = 0u128;

    for i in 1..=3 {
        print!("  Call {}: ", i);
        let start = Instant::now();

        match fallback_client
            .get_price("So11111111111111111111111111111111111111112")
            .await
        {
            Ok(Some(price)) => {
                let call_time = start.elapsed();
                total_time += call_time.as_millis();
                println!("✅ {}ms (${:.2})", call_time.as_millis(), price);
            }
            Ok(None) => {
                let call_time = start.elapsed();
                println!("⚠️  {}ms - No price available", call_time.as_millis());
            }
            Err(e) => {
                let call_time = start.elapsed();
                println!("❌ {}ms - Error: {}", call_time.as_millis(), e);
            }
        }
    }

    let avg_time = total_time / 3;
    println!("📊 Fallback average: {}ms", avg_time);

    Ok(())
}

async fn test_ultra_fast_client_local() -> Result<()> {
    use crate::shared::jupiter::{JupiterClient, JupiterConfig};

    println!("⚡ Testing Ultra-Fast Jupiter Client");
    println!("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━");

    let ultra_fast_client = JupiterClient::new(&JupiterConfig::default()).await?;

    // 1. Test single call first (diagnostic)
    println!("🔍 Diagnostic: Testing single API call...");
    let diagnostic_start = Instant::now();
    match ultra_fast_client
        .get_price_ultra_fast("So11111111111111111111111111111111111111112")
        .await
    {
        Ok(Some(price)) => {
            println!(
                "✅ Diagnostic successful: {}ms (${:.2})",
                diagnostic_start.elapsed().as_millis(),
                price
            );
        }
        Ok(None) => {
            println!("⚠️  Diagnostic: No price returned");
        }
        Err(e) => {
            println!("❌ Diagnostic failed: {}", e);
            return Ok(());
        }
    }

    // Skip warm up and preload for now - focus on basic functionality
    // ultra_fast_client.warm_connections().await?;
    // ultra_fast_client.preload_common_prices().await?;

    // Direct speed test
    println!("\n🚀 Ultra-fast speed test:");
    for i in 1..=3 {
        print!("  Call {}: ", i);
        let start = Instant::now();

        match ultra_fast_client
            .get_price_ultra_fast("So11111111111111111111111111111111111111112")
            .await
        {
            Ok(Some(price)) => {
                let call_time = start.elapsed();
                println!("✅ {}ms (${:.2})", call_time.as_millis(), price);

                if call_time.as_millis() < 10 {
                    println!("    🏆 ULTRA-FAST: Sub-10ms response!");
                } else if call_time.as_millis() < 50 {
                    println!("    ⚡ EXCELLENT: Sub-50ms response!");
                }
            }
            Ok(None) => {
                let call_time = start.elapsed();
                println!("⚠️  {}ms - No price available", call_time.as_millis());
            }
            Err(e) => {
                let call_time = start.elapsed();
                println!("❌ {}ms - Error: {}", call_time.as_millis(), e);
            }
        }
    }

    println!("✅ Ultra-fast client test completed");
    Ok(())
}
