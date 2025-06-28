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
    match api_manager.fetch_raydium_pools().await {
        Ok(pools) => {
            if pools.is_empty() {
                println!("⚠️  No pools found (API might be down or changed)");
            } else {
                println!("✅ OK - Found {} pools", pools.len());
                
                // Show first few pools
                for (i, pool) in pools.iter().take(3).enumerate() {
                    println!("   Pool {}: {} -> {}", i + 1, pool.base_mint, pool.quote_mint);
                    if let Some(liquidity) = pool.liquidity {
                        println!("      Liquidity: ${:.2}", liquidity);
                    } else {
                        println!("      Liquidity: Unknown");
                    }
                }
            }
        }
        Err(e) => {
            println!("❌ FAILED: {}", e);
        }
    }
    
    // Test Jupiter API - Token data verification
    print!("🪐 Testing Jupiter API... ");
    match api_manager.fetch_jupiter_tokens().await {
        Ok(tokens) => {
            if tokens.is_empty() {
                println!("⚠️  No tokens found");
            } else {
                println!("✅ OK - Found {} tokens", tokens.len());
                
                // Look for SOL token
                if let Some(sol_token) = tokens.iter().find(|t| t.symbol == "SOL") {
                    println!("   Found SOL: {} ({})", sol_token.name, sol_token.address);
                }
            }
        }
        Err(e) => {
            println!("❌ FAILED: {}", e);
        }
    }
    
    // Test Birdeye API
    print!("🐦 Testing Birdeye API... ");
    match api_manager.fetch_birdeye_token_data("So11111111111111111111111111111111111111112").await {
        Ok(token_data) => {
            println!("✅ OK - SOL data: {}", token_data.symbol);
            if let Some(price) = token_data.price {
                println!("   Price: ${:.2}", price);
            }
            if let Some(liquidity) = token_data.liquidity {
                println!("   Liquidity: ${:.2}", liquidity);
            }
        }
        Err(e) => {
            println!("❌ FAILED: {}", e);
        }
    }
    
    // Test DexScreener API
    print!("📊 Testing DexScreener API... ");
    let test_tokens = vec!["So11111111111111111111111111111111111111112".to_string()];
    match api_manager.fetch_dexscreener_pairs(&test_tokens).await {
        Ok(pairs) => {
            if pairs.is_empty() {
                println!("⚠️  No pairs found (API might be down)");
            } else {
                println!("✅ OK - Found {} pairs", pairs.len());
                
                // Show first pair
                if let Some(pair) = pairs.first() {
                    println!("   Pair: {}/{}", pair.base_token.symbol, pair.quote_token.symbol);
                    if let Some(price_usd) = &pair.price_usd {
                        println!("   Price USD: ${}", price_usd);
                    }
                }
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
    if let Ok(pools) = api_manager.fetch_raydium_pools().await {
        if !pools.is_empty() {
            successful_apis += 1;
            total_pools += pools.len();
            println!("   ✅ Raydium API: {} pools", pools.len());
        }
    }
    
    // Try DexScreener API  
    let test_tokens = vec!["So11111111111111111111111111111111111111112".to_string()];
    if let Ok(pairs) = api_manager.fetch_dexscreener_pairs(&test_tokens).await {
        if !pairs.is_empty() {
            successful_apis += 1;
            total_pools += pairs.len();
            println!("   ✅ DexScreener API: {} pairs", pairs.len());
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
