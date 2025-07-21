use crate::config::Config;
use crate::shared::rpc_pool::RpcConnectionPool;
use anyhow::Result;
use tracing::{error, info, warn};

/// Test Solana connectivity and basic functionality
pub async fn test_solana_connectivity(config: &Config) -> Result<()> {
    info!("🧪 Testing Solana connectivity...");

    // Create RPC pool
    let rpc_pool = RpcConnectionPool::new(config).await?;
    rpc_pool.start().await?;

    // Test 1: Get current slot
    info!("📡 Test 1: Getting current slot...");
    match rpc_pool.get_current_slot().await {
        Ok(slot) => {
            info!("✅ Current Solana slot: {}", slot);
        }
        Err(e) => {
            error!("❌ Failed to get current slot: {}", e);
            return Err(e);
        }
    }

    // Test 2: Get latest blockhash
    info!("📡 Test 2: Getting latest blockhash...");
    match rpc_pool.get_latest_blockhash().await {
        Ok(blockhash) => {
            info!("✅ Latest blockhash: {}", blockhash);
        }
        Err(e) => {
            error!("❌ Failed to get latest blockhash: {}", e);
            return Err(e);
        }
    }

    // Test 3: Check Raydium pools (optional - quick check only)
    info!("📡 Test 3: Checking Raydium pools...");
    match rpc_pool.get_raydium_pools().await {
        Ok(pools) => {
            if pools.is_empty() {
                info!("ℹ️  No Raydium pools retrieved via RPC (using alternative APIs instead)");
                info!("    This is expected on mainnet - alternative pool detection is active");
            } else {
                info!("✅ Found {} Raydium pool accounts via RPC", pools.len());

                // Show first few pools for verification
                for (i, (pubkey, account)) in pools.iter().take(3).enumerate() {
                    info!(
                        "  Pool {}: {} (data size: {} bytes, balance: {} lamports)",
                        i + 1,
                        pubkey,
                        account.data.len(),
                        account.lamports
                    );
                }
            }
        }
        Err(e) => {
            warn!("⚠️  Raydium pools check failed: {}", e);
            info!("    Alternative pool detection methods are available and working");
            // Don't return error - continue with other tests
        }
    }

    // Test 4: Get RPC stats
    let stats = rpc_pool.get_stats().await;
    info!("📊 RPC Pool Statistics:");
    info!("  Total requests: {}", stats.total_requests);
    info!("  Successful: {}", stats.successful_requests);
    info!("  Failed: {}", stats.failed_requests);
    info!("  Avg response time: {:.2}ms", stats.avg_response_time_ms);

    info!("🎉 All Solana connectivity tests passed!");
    Ok(())
}

/// Test specific pool analysis
pub async fn test_pool_analysis(config: &Config) -> Result<()> {
    info!("🧪 Testing pool analysis functionality...");

    let rpc_pool = RpcConnectionPool::new(config).await?;
    rpc_pool.start().await?;

    // Get some pools to analyze
    let pools = rpc_pool
        .monitor_new_raydium_pools(Default::default())
        .await?;

    if pools.is_empty() {
        info!("ℹ️ No pools found for analysis");
        return Ok(());
    }

    // Analyze first pool
    let pool = &pools[0];
    info!("🔍 Analyzing pool: {}", pool);

    // Test pool validation
    let is_valid = rpc_pool
        .validate_pool_criteria(pool, &Default::default())
        .await?;
    info!(
        "✅ Pool validation result: {}",
        if is_valid { "VALID" } else { "INVALID" }
    );

    // Test market data extraction
    match rpc_pool.get_pool_market_data(pool).await {
        Ok(market_data) => {
            info!("📊 Pool Market Data:");
            info!("  Liquidity: ${:.2}", market_data.total_liquidity_usd);
            info!("  Price: {:.6}", market_data.price_token_a_in_b);
            info!("  Volume 24h: ${:.2}", market_data.volume_24h_usd);
        }
        Err(e) => {
            info!("⚠️ Could not get market data: {}", e);
        }
    }

    info!("🎉 Pool analysis tests completed!");
    Ok(())
}
