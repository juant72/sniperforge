//! Test for Cache-Free Trading Engine
//! 
//! This test verifies that the cache-free trader works correctly

use anyhow::Result;
use sniperforge::shared::cache_free_trader_simple::{CacheFreeTraderSimple, TradingSafetyConfig, test_cache_free_trading};

#[tokio::main]
async fn main() -> Result<()> {
    println!("🛡️ Testing Cache-Free Trading Engine");
    println!("=====================================");
    
    // Initialize tracing
    tracing_subscriber::fmt()
        .with_env_filter("info")
        .init();
    
    // Run the test
    test_cache_free_trading().await?;
    
    println!("\n🎉 Cache-Free Trading Engine Test Completed!");
    
    Ok(())
}
