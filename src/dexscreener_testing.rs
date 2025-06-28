use crate::shared::alternative_apis::{AlternativeApiManager, BasicConfig};
use anyhow::Result;
use tracing::{info, error};

pub async fn test_dexscreener_integration() -> Result<()> {
    println!("\n🧪 Testing DexScreener API Integration...");
    
    // Create basic config for API manager
    let config = BasicConfig {
        raydium_api_base: "https://api.raydium.io/v2".to_string(),
        jupiter_api_base: "https://quote-api.jup.ag/v6".to_string(),
        birdeye_api_base: "https://public-api.birdeye.so".to_string(),
        dexscreener_api_base: "https://api.dexscreener.com".to_string(),
    };
    
    let apis = AlternativeApiManager::new(&config);
    
    // Test 1: Search for SOL/USDC pairs
    println!("\n📍 Test 1: Searching for SOL/USDC pairs");
    match apis.search_dexscreener_pairs("SOL/USDC").await {
        Ok(pairs) => {
            println!("✅ Found {} SOL/USDC pairs", pairs.len());
            for (i, pair) in pairs.iter().take(3).enumerate() {
                // Safe price extraction
                let price = pair.price_usd.parse::<f64>().unwrap_or(0.0);
                
                let liquidity = pair.liquidity.as_ref()
                    .map(|l| l.usd)
                    .unwrap_or(0.0);
                
                println!("  {}. {}/{} on {} - Price: ${:.6} - Liquidity: ${:.0}",
                    i + 1,
                    pair.base_token.symbol,
                    pair.quote_token.symbol,
                    pair.dex_id,
                    price,
                    liquidity
                );
            }
        }
        Err(e) => {
            error!("❌ Failed to search SOL/USDC pairs: {}", e);
        }
    }
    
    // Test 2: Get pools for SOL token
    println!("\n📍 Test 2: Getting pools for SOL token");
    let sol_address = "So11111111111111111111111111111111111111112";
    match apis.get_token_pools_dexscreener(sol_address).await {
        Ok(pairs) => {
            println!("✅ Found {} pools for SOL", pairs.len());
            for (i, pair) in pairs.iter().take(5).enumerate() {
                let volume_24h = pair.volume.as_ref()
                    .and_then(|v| v.get("h24"))
                    .copied()
                    .unwrap_or(0.0);
                
                println!("  {}. {}/{} on {} - Volume 24h: ${:.0}",
                    i + 1,
                    pair.base_token.symbol,
                    pair.quote_token.symbol,
                    pair.dex_id,
                    volume_24h
                );
            }
        }
        Err(e) => {
            error!("❌ Failed to get SOL pools: {}", e);
        }
    }
    
    // Test 3: Batch fetch multiple tokens
    println!("\n📍 Test 3: Batch fetching multiple popular tokens");
    let tokens = vec![
        "So11111111111111111111111111111111111111112".to_string(), // SOL
        "EPjFWdd5AufqSSqeM2qN1xzybapC8G4wEGGkZwyTDt1v".to_string(), // USDC
        "Es9vMFrzaCERmJfrF4H2FYD4KCoNkY11McCe8BenwNYB".to_string(), // USDT
    ];
    
    match apis.fetch_dexscreener_pairs(&tokens).await {
        Ok(pairs) => {
            println!("✅ Batch fetched {} pairs for {} tokens", pairs.len(), tokens.len());
            
            // Group by token
            for token in &tokens {
                let token_pairs: Vec<_> = pairs.iter()
                    .filter(|p| p.base_token.address == *token || p.quote_token.address == *token)
                    .collect();
                
                if !token_pairs.is_empty() {
                    let symbol = if token_pairs[0].base_token.address == *token {
                        &token_pairs[0].base_token.symbol
                    } else {
                        &token_pairs[0].quote_token.symbol
                    };
                    println!("  📊 {} has {} pairs", symbol, token_pairs.len());
                }
            }
        }
        Err(e) => {
            error!("❌ Failed to batch fetch tokens: {}", e);
        }
    }
    
    // Test 4: Get specific pair info (if we found any pairs in previous tests)
    println!("\n📍 Test 4: Getting specific pair information");
    // Use a known Raydium SOL/USDC pair address
    let pair_address = "58oQChx4yWmvKdwLLZzBi4ChoCc2fqCUWBkwMihLYQo2"; // Raydium SOL/USDC
    match apis.get_pair_info_dexscreener(pair_address).await {
        Ok(Some(pair)) => {
            println!("✅ Found pair info for {}:", pair_address);
            println!("  📊 Pair: {}/{} on {}", 
                pair.base_token.symbol, 
                pair.quote_token.symbol,
                pair.dex_id
            );
            
            let price = pair.price_usd.parse::<f64>().unwrap_or(0.0);
            
            let liquidity = pair.liquidity.as_ref()
                .map(|l| l.usd)
                .unwrap_or(0.0);
            
            let volume_24h = pair.volume.as_ref()
                .and_then(|v| v.get("h24"))
                .copied()
                .unwrap_or(0.0);
            
            println!("  💰 Price: ${:.6}", price);
            println!("  🌊 Liquidity: ${:.0}", liquidity);
            println!("  📈 Volume 24h: ${:.0}", volume_24h);
            // Note: price_change is not currently captured in our DexscreenerPair struct
            // but could be added if needed for more detailed analysis
        }
        Ok(None) => {
            println!("⚠️ No pair found for address {}", pair_address);
        }
        Err(e) => {
            error!("❌ Failed to get pair info: {}", e);
        }
    }
    
    println!("\n✅ DexScreener API integration test completed!");
    Ok(())
}
