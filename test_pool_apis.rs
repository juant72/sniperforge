use anyhow::Result;
use reqwest;
use serde_json::{json, Value};
use std::collections::HashMap;
use tokio;

#[tokio::main]
async fn main() -> Result<()> {
    println!("🔍 TESTING SOLANA POOL APIs FOR MAINNET DATA");
    println!("===========================================");

    // Test multiple API sources for pool data
    test_jupiter_api().await?;
    test_raydium_api().await?;
    test_orca_api().await?;
    test_dexscreener_api().await?;
    test_coingecko_api().await?;
    test_birdeye_api().await?;
    test_solscan_api().await?;

    Ok(())
}

async fn test_jupiter_api() -> Result<()> {
    println!("\n🪐 TESTING JUPITER API");
    println!("----------------------");

    let client = reqwest::Client::new();

    // Jupiter Token List - Contains all tokens with their mints
    println!("📊 Testing Jupiter Token List...");
    let token_url = "https://token.jup.ag/all";

    match client.get(token_url).send().await {
        Ok(response) => {
            if response.status().is_success() {
                let tokens: Value = response.json().await?;
                if let Some(token_array) = tokens.as_array() {
                    println!(
                        "   ✅ Jupiter Tokens: {} tokens available",
                        token_array.len()
                    );

                    // Show first few tokens
                    for (i, token) in token_array.iter().take(5).enumerate() {
                        if let (Some(symbol), Some(address)) =
                            (token.get("symbol"), token.get("address"))
                        {
                            println!("   {} - {}: {}", i + 1, symbol, address);
                        }
                    }
                }
            } else {
                println!("   ❌ Jupiter Token List failed: {}", response.status());
            }
        }
        Err(e) => println!("   ❌ Jupiter Token List error: {}", e),
    }

    // Jupiter Quote API - Can give us route information
    println!("\n📊 Testing Jupiter Quote API...");
    let quote_url = "https://quote-api.jup.ag/v6/quote";
    let params = [
        ("inputMint", "So11111111111111111111111111111111111111112"), // SOL
        ("outputMint", "EPjFWdd5AufqSSqeM2qN1xzybapC8G4wEGGkZwyTDt1v"), // USDC
        ("amount", "1000000000"),                                     // 1 SOL
        ("slippageBps", "50"),
    ];

    match client.get(quote_url).query(&params).send().await {
        Ok(response) => {
            if response.status().is_success() {
                let quote: Value = response.json().await?;
                println!("   ✅ Jupiter Quote successful");

                // Extract route information
                if let Some(route_plan) = quote.get("routePlan") {
                    if let Some(route_array) = route_plan.as_array() {
                        println!("   📍 Route has {} steps:", route_array.len());
                        for (i, step) in route_array.iter().enumerate() {
                            if let (Some(swap_info), Some(percent)) =
                                (step.get("swapInfo"), step.get("percent"))
                            {
                                if let Some(amm_key) = swap_info.get("ammKey") {
                                    println!("   {} - Pool: {} ({}%)", i + 1, amm_key, percent);
                                }
                            }
                        }
                    }
                }
            } else {
                println!("   ❌ Jupiter Quote failed: {}", response.status());
            }
        }
        Err(e) => println!("   ❌ Jupiter Quote error: {}", e),
    }

    Ok(())
}

async fn test_raydium_api() -> Result<()> {
    println!("\n🌊 TESTING RAYDIUM API");
    println!("----------------------");

    let client = reqwest::Client::new();

    // Raydium SDK Liquidity Data
    println!("📊 Testing Raydium SDK Liquidity...");
    let raydium_url = "https://api.raydium.io/v2/sdk/liquidity/mainnet.json";

    match client.get(raydium_url).send().await {
        Ok(response) => {
            if response.status().is_success() {
                let data: Value = response.json().await?;
                println!("   ✅ Raydium API successful");

                // Check official pools
                if let Some(official) = data.get("official") {
                    if let Some(pools) = official.as_array() {
                        println!("   📍 Official Raydium Pools: {}", pools.len());

                        // Show first few pools
                        for (i, pool) in pools.iter().take(5).enumerate() {
                            if let (Some(id), Some(base_mint), Some(quote_mint)) =
                                (pool.get("id"), pool.get("baseMint"), pool.get("quoteMint"))
                            {
                                println!("   {} - {}: {} / {}", i + 1, id, base_mint, quote_mint);
                            }
                        }
                    }
                }

                // Check unofficial pools
                if let Some(unofficial) = data.get("unOfficial") {
                    if let Some(pools) = unofficial.as_array() {
                        println!("   📍 Unofficial Raydium Pools: {}", pools.len());
                    }
                }
            } else {
                println!("   ❌ Raydium API failed: {}", response.status());
            }
        }
        Err(e) => println!("   ❌ Raydium API error: {}", e),
    }

    // Raydium Pairs API
    println!("\n📊 Testing Raydium Pairs API...");
    let pairs_url = "https://api.raydium.io/v2/sdk/liquidity/mainnet.json";

    match client.get(pairs_url).send().await {
        Ok(response) => {
            if response.status().is_success() {
                println!("   ✅ Raydium Pairs API accessible");
            } else {
                println!("   ❌ Raydium Pairs API failed: {}", response.status());
            }
        }
        Err(e) => println!("   ❌ Raydium Pairs API error: {}", e),
    }

    Ok(())
}

async fn test_orca_api() -> Result<()> {
    println!("\n🐋 TESTING ORCA API");
    println!("-------------------");

    let client = reqwest::Client::new();

    // Orca Whirlpool List
    println!("📊 Testing Orca Whirlpool List...");
    let whirlpool_url = "https://api.orca.so/v1/whirlpool/list";

    match client.get(whirlpool_url).send().await {
        Ok(response) => {
            if response.status().is_success() {
                let data: Value = response.json().await?;
                println!("   ✅ Orca Whirlpool API successful");

                if let Some(whirlpools) = data.get("whirlpools") {
                    if let Some(pools) = whirlpools.as_array() {
                        println!("   📍 Whirlpool Pools: {}", pools.len());

                        // Show first few pools
                        for (i, pool) in pools.iter().take(5).enumerate() {
                            if let (Some(address), Some(token_a), Some(token_b)) =
                                (pool.get("address"), pool.get("tokenA"), pool.get("tokenB"))
                            {
                                println!(
                                    "   {} - {}: {} / {}",
                                    i + 1,
                                    address,
                                    token_a.get("mint").unwrap_or(&json!("N/A")),
                                    token_b.get("mint").unwrap_or(&json!("N/A"))
                                );
                            }
                        }
                    }
                }
            } else {
                println!("   ❌ Orca Whirlpool API failed: {}", response.status());
            }
        }
        Err(e) => println!("   ❌ Orca Whirlpool API error: {}", e),
    }

    // Try alternative Orca endpoints
    println!("\n📊 Testing Orca Pools endpoint...");
    let orca_pools_url = "https://api.orca.so/v1/pools";

    match client.get(orca_pools_url).send().await {
        Ok(response) => {
            if response.status().is_success() {
                let data: Value = response.json().await?;
                println!("   ✅ Orca Pools API successful");

                if let Some(pools) = data.as_array() {
                    println!("   📍 Orca Classic Pools: {}", pools.len());
                } else if let Some(pools) = data.as_object() {
                    println!("   📍 Orca Pools Object: {} entries", pools.len());
                }
            } else {
                println!("   ❌ Orca Pools API failed: {}", response.status());
            }
        }
        Err(e) => println!("   ❌ Orca Pools API error: {}", e),
    }

    Ok(())
}

async fn test_dexscreener_api() -> Result<()> {
    println!("\n📊 TESTING DEXSCREENER API");
    println!("--------------------------");

    let client = reqwest::Client::new();

    // DexScreener Solana Pairs
    println!("📊 Testing DexScreener Solana pairs...");
    let dexscreener_url =
        "https://api.dexscreener.com/latest/dex/tokens/So11111111111111111111111111111111111111112";

    match client.get(dexscreener_url).send().await {
        Ok(response) => {
            if response.status().is_success() {
                let data: Value = response.json().await?;
                println!("   ✅ DexScreener API successful");

                if let Some(pairs) = data.get("pairs") {
                    if let Some(pairs_array) = pairs.as_array() {
                        println!("   📍 SOL Trading Pairs: {}", pairs_array.len());

                        // Show first few pairs
                        for (i, pair) in pairs_array.iter().take(5).enumerate() {
                            if let (
                                Some(dex_id),
                                Some(pair_address),
                                Some(base_token),
                                Some(quote_token),
                            ) = (
                                pair.get("dexId"),
                                pair.get("pairAddress"),
                                pair.get("baseToken"),
                                pair.get("quoteToken"),
                            ) {
                                println!(
                                    "   {} - {} ({}): {} / {}",
                                    i + 1,
                                    dex_id,
                                    pair_address,
                                    base_token.get("symbol").unwrap_or(&json!("N/A")),
                                    quote_token.get("symbol").unwrap_or(&json!("N/A"))
                                );
                            }
                        }
                    }
                }
            } else {
                println!("   ❌ DexScreener API failed: {}", response.status());
            }
        }
        Err(e) => println!("   ❌ DexScreener API error: {}", e),
    }

    Ok(())
}

async fn test_coingecko_api() -> Result<()> {
    println!("\n🦎 TESTING COINGECKO API");
    println!("-------------------------");

    let client = reqwest::Client::new();

    // CoinGecko DeFi Pools
    println!("📊 Testing CoinGecko DeFi pools...");
    let coingecko_url = "https://api.coingecko.com/api/v3/onchain/networks/solana/pools";

    match client.get(coingecko_url).send().await {
        Ok(response) => {
            if response.status().is_success() {
                let data: Value = response.json().await?;
                println!("   ✅ CoinGecko API successful");

                if let Some(pools) = data.get("data") {
                    if let Some(pools_array) = pools.as_array() {
                        println!("   📍 Solana DeFi Pools: {}", pools_array.len());

                        // Show first few pools
                        for (i, pool) in pools_array.iter().take(5).enumerate() {
                            if let (Some(id), Some(attributes)) =
                                (pool.get("id"), pool.get("attributes"))
                            {
                                if let Some(dex_id) = attributes.get("dex_id") {
                                    println!("   {} - {} ({})", i + 1, id, dex_id);
                                }
                            }
                        }
                    }
                }
            } else {
                println!("   ❌ CoinGecko API failed: {}", response.status());
            }
        }
        Err(e) => println!("   ❌ CoinGecko API error: {}", e),
    }

    Ok(())
}

async fn test_birdeye_api() -> Result<()> {
    println!("\n🦅 TESTING BIRDEYE API");
    println!("----------------------");

    let client = reqwest::Client::new();

    // Birdeye Solana Pools
    println!("📊 Testing Birdeye Solana pools...");
    let birdeye_url = "https://public-api.birdeye.so/public/tokenlist";

    match client.get(birdeye_url).send().await {
        Ok(response) => {
            if response.status().is_success() {
                let data: Value = response.json().await?;
                println!("   ✅ Birdeye API successful");

                if let Some(tokens) = data.get("data") {
                    if let Some(tokens_array) = tokens.as_array() {
                        println!("   📍 Birdeye Tokens: {}", tokens_array.len());

                        // Show first few tokens
                        for (i, token) in tokens_array.iter().take(5).enumerate() {
                            if let (Some(address), Some(symbol)) =
                                (token.get("address"), token.get("symbol"))
                            {
                                println!("   {} - {}: {}", i + 1, symbol, address);
                            }
                        }
                    }
                }
            } else {
                println!("   ❌ Birdeye API failed: {}", response.status());
            }
        }
        Err(e) => println!("   ❌ Birdeye API error: {}", e),
    }

    // Try Birdeye pools endpoint
    println!("\n📊 Testing Birdeye pools endpoint...");
    let birdeye_pools_url = "https://public-api.birdeye.so/public/pools";

    match client.get(birdeye_pools_url).send().await {
        Ok(response) => {
            if response.status().is_success() {
                let data: Value = response.json().await?;
                println!("   ✅ Birdeye Pools API successful");

                if let Some(pools) = data.get("data") {
                    if let Some(pools_array) = pools.as_array() {
                        println!("   📍 Birdeye Pools: {}", pools_array.len());
                    }
                }
            } else {
                println!("   ❌ Birdeye Pools API failed: {}", response.status());
            }
        }
        Err(e) => println!("   ❌ Birdeye Pools API error: {}", e),
    }

    Ok(())
}

async fn test_solscan_api() -> Result<()> {
    println!("\n🔍 TESTING SOLSCAN API");
    println!("----------------------");

    let client = reqwest::Client::new();

    // Solscan Market Data
    println!("📊 Testing Solscan market data...");
    let solscan_url = "https://api.solscan.io/market";

    match client.get(solscan_url).send().await {
        Ok(response) => {
            if response.status().is_success() {
                let data: Value = response.json().await?;
                println!("   ✅ Solscan API successful");

                if let Some(data_obj) = data.as_object() {
                    println!("   📍 Solscan Market Data: {} entries", data_obj.len());

                    // Show some market data
                    for (key, value) in data_obj.iter().take(3) {
                        println!("   - {}: {}", key, value);
                    }
                }
            } else {
                println!("   ❌ Solscan API failed: {}", response.status());
            }
        }
        Err(e) => println!("   ❌ Solscan API error: {}", e),
    }

    Ok(())
}
