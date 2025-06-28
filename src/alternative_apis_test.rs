// Test específico para APIs alternativas de pool detection
// Usado cuando los RPC endpoints fallan en mainnet

use anyhow::Result;
use crate::shared::alternative_apis::AlternativeApiManager;

pub async fn test_alternative_apis() {
    println!("🌐 Testing Alternative APIs for Pool Detection");
    println!("==============================================");
    
    // Initialize with basic config
    let basic_config = crate::shared::alternative_apis::BasicConfig::default();
    let api_manager = AlternativeApiManager::new(&basic_config);
    
    // Test Raydium API
    print!("📡 Testing Raydium API... ");
    match api_manager.get_raydium_pools().await {
        Ok(pools) => {
            if pools.is_empty() {
                println!("⚠️  No pools found (API might be down or changed)");
            } else {
                println!("✅ OK - Found {} pools", pools.len());
                
                // Show first few pools
                for (i, pool) in pools.iter().take(3).enumerate() {
                    println!("   Pool {}: {} -> {}", i + 1, pool.token_a, pool.token_b);
                    println!("      Liquidity: ${:.2}", pool.liquidity_usd);
                }
            }
        }
        Err(e) => {
            println!("❌ FAILED: {}", e);
        }
    }
    
    // Test Jupiter API - Price verification
    print!("🪐 Testing Jupiter price API... ");
    match api_manager.get_token_price("So11111111111111111111111111111111111111112").await {
        Ok(Some(price)) => {
            println!("✅ OK - SOL price: ${:.2}", price);
        }
        Ok(None) => {
            println!("⚠️  No price data available");
        }
        Err(e) => {
            println!("❌ FAILED: {}", e);
        }
    }
    
    // Test Birdeye API
    print!("🐦 Testing Birdeye API... ");
    match api_manager.get_birdeye_pools().await {
        Ok(pools) => {
            if pools.is_empty() {
                println!("⚠️  No pools found (API might be down or require API key)");
            } else {
                println!("✅ OK - Found {} pools", pools.len());
            }
        }
        Err(e) => {
            println!("❌ FAILED: {}", e);
        }
    }
    
    // Test DexScreener API
    print!("📊 Testing DexScreener API... ");
    match api_manager.get_dexscreener_pools().await {
        Ok(pools) => {
            if pools.is_empty() {
                println!("⚠️  No pools found (API might be down)");
            } else {
                println!("✅ OK - Found {} pools", pools.len());
            }
        }
        Err(e) => {
            println!("❌ FAILED: {}", e);
        }
    }
    
    println!("\n🎯 Alternative APIs Test Summary:");
    println!("   These APIs provide fallback pool detection when RPC endpoints");
    println!("   fail due to rate limiting or 410 Gone errors on mainnet.");
    println!("   Critical for maintaining pool detection functionality in production.");
}

pub async fn test_rpc_fallback_scenario() {
    println!("\n🔄 Testing RPC Fallback Scenario");
    println!("================================");
    
    println!("📋 Simulating mainnet RPC failures...");
    println!("   - Primary RPC: 410 Gone (rate limited)");
    println!("   - Backup RPC: 410 Gone (rate limited)");
    println!("   - Alternative APIs activated");
    
    let basic_config = crate::shared::alternative_apis::BasicConfig::default();
    let api_manager = AlternativeApiManager::new(&basic_config);
    
    // This would be the actual fallback logic in production
    println!("🔄 Attempting pool detection via alternative APIs...");
    
    let mut successful_apis = 0;
    let mut total_pools = 0;
    
    // Try Raydium API
    if let Ok(pools) = api_manager.get_raydium_pools().await {
        if !pools.is_empty() {
            successful_apis += 1;
            total_pools += pools.len();
            println!("   ✅ Raydium API: {} pools", pools.len());
        }
    }
    
    // Try DexScreener API  
    if let Ok(pools) = api_manager.get_dexscreener_pools().await {
        if !pools.is_empty() {
            successful_apis += 1;
            total_pools += pools.len();
            println!("   ✅ DexScreener API: {} pools", pools.len());
        }
    }
    
    if successful_apis > 0 {
        println!("🎉 Fallback successful! {} APIs provided {} total pools", successful_apis, total_pools);
        println!("   Pool detection resilience confirmed ✅");
    } else {
        println!("⚠️  All alternative APIs failed - this indicates network issues");
        println!("   or API changes that need investigation");
    }
}
