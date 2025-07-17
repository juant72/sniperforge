// ===== MILITARY INTELLIGENCE: ENHANCED POOL DISCOVERY VIA REAL APIs =====

use reqwest;
use serde_json::Value;
use std::collections::HashMap;
use anyhow::Result;

impl MilitaryArbitrageSystem {
    /// MILITARY STRATEGY: Use real API sources to discover operational pools
    pub async fn discover_pools_via_apis(&mut self) -> Result<Vec<String>> {
        info!("🔍 MILITARY RECONNAISSANCE: Discovering pools via real APIs...");
        
        let mut all_pools = Vec::new();
        
        // 1. Jupiter API - Route discovery (most reliable)
        if let Ok(jupiter_pools) = self.fetch_jupiter_pools().await {
            info!("✅ Jupiter API: {} pools discovered", jupiter_pools.len());
            all_pools.extend(jupiter_pools);
        }
        
        // 2. Raydium API - Official pools (highly reliable)
        if let Ok(raydium_pools) = self.fetch_raydium_pools_enhanced().await {
            info!("✅ Raydium API: {} pools discovered", raydium_pools.len());
            all_pools.extend(raydium_pools);
        }
        
        // 3. Orca API - Whirlpool data (excellent coverage)
        if let Ok(orca_pools) = self.fetch_orca_pools_enhanced().await {
            info!("✅ Orca API: {} pools discovered", orca_pools.len());
            all_pools.extend(orca_pools);
        }
        
        // 4. DexScreener API - Cross-DEX validation (good for verification)
        if let Ok(dexscreener_pools) = self.fetch_dexscreener_pools().await {
            info!("✅ DexScreener API: {} pools discovered", dexscreener_pools.len());
            all_pools.extend(dexscreener_pools);
        }
        
        // Remove duplicates and filter for major pairs
        let mut unique_pools = HashMap::new();
        for pool in all_pools {
            if self.is_major_token_pair(&pool.token_a, &pool.token_b) {
                unique_pools.insert(pool.address.clone(), pool);
            }
        }
        
        info!("🎯 MILITARY INTELLIGENCE: {} unique major pools identified", unique_pools.len());
        
        // Validate pools before adding to monitoring
        let mut validated_pools = Vec::new();
        for (address, pool_info) in unique_pools {
            match self.validate_pool_from_api(&pool_info).await {
                Ok(validated_pool) => {
                    self.pools.insert(address.clone(), validated_pool);
                    validated_pools.push(address);
                }
                Err(e) => {
                    warn!("❌ Pool validation failed for {}: {}", address, e);
                }
            }
        }
        
        info!("✅ MILITARY OBJECTIVE COMPLETE: {} operational pools ready", validated_pools.len());
        Ok(validated_pools)
    }
    
    /// Fetch pools from Jupiter API (most reliable for routing)
    async fn fetch_jupiter_pools(&self) -> Result<Vec<PoolInfo>> {
        let client = reqwest::Client::new();
        let mut pools = Vec::new();
        
        // Get major token pairs from Jupiter routing
        let major_pairs = vec![
            ("So11111111111111111111111111111111111111112", "EPjFWdd5AufqSSqeM2qN1xzybapC8G4wEGGkZwyTDt1v"), // SOL/USDC
            ("So11111111111111111111111111111111111111112", "Es9vMFrzaCERmJfrF4H2FYD4KCoNkY11McCe8BenwNYB"), // SOL/USDT
            ("4k3Dyjzvzp8eMZWUXbBCjEvwSkkk59S5iCNLY3QrkX6R", "EPjFWdd5AufqSSqeM2qN1xzybapC8G4wEGGkZwyTDt1v"), // RAY/USDC
            ("4k3Dyjzvzp8eMZWUXbBCjEvwSkkk59S5iCNLY3QrkX6R", "So11111111111111111111111111111111111111112"), // RAY/SOL
            ("mSoLzYCxHdYgdzU16g5QSh3i5K3z3KZK7ytfqcJm7So", "EPjFWdd5AufqSSqeM2qN1xzybapC8G4wEGGkZwyTDt1v"), // mSOL/USDC
        ];
        
        for (input_mint, output_mint) in major_pairs {
            let url = "https://quote-api.jup.ag/v6/quote";
            let params = [
                ("inputMint", input_mint),
                ("outputMint", output_mint),
                ("amount", "1000000000"), // 1 SOL equivalent
                ("slippageBps", "50"),
            ];
            
            match client.get(url).query(&params).send().await {
                Ok(response) => {
                    if response.status().is_success() {
                        if let Ok(data) = response.json::<Value>().await {
                            if let Some(route_plan) = data.get("routePlan") {
                                if let Some(routes) = route_plan.as_array() {
                                    for route in routes {
                                        if let Some(swap_info) = route.get("swapInfo") {
                                            if let Some(amm_key) = swap_info.get("ammKey") {
                                                if let Some(label) = swap_info.get("label") {
                                                    pools.push(PoolInfo {
                                                        address: amm_key.as_str().unwrap_or("").to_string(),
                                                        token_a: input_mint.to_string(),
                                                        token_b: output_mint.to_string(),
                                                        dex_type: self.detect_dex_from_label(label.as_str().unwrap_or("")),
                                                        source: "Jupiter".to_string(),
                                                    });
                                                }
                                            }
                                        }
                                    }
                                }
                            }
                        }
                    }
                }
                Err(e) => warn!("Jupiter API error for {}/{}: {}", input_mint, output_mint, e),
            }
        }
        
        Ok(pools)
    }
    
    /// Fetch pools from Raydium API (official and reliable)
    async fn fetch_raydium_pools_enhanced(&self) -> Result<Vec<PoolInfo>> {
        let client = reqwest::Client::new();
        let url = "https://api.raydium.io/v2/sdk/liquidity/mainnet.json";
        
        match client.get(url).send().await {
            Ok(response) => {
                if response.status().is_success() {
                    let data: Value = response.json().await?;
                    let mut pools = Vec::new();
                    
                    // Official pools (most reliable)
                    if let Some(official) = data.get("official") {
                        if let Some(official_pools) = official.as_array() {
                            for pool in official_pools {
                                if let (Some(id), Some(base_mint), Some(quote_mint)) = 
                                    (pool.get("id"), pool.get("baseMint"), pool.get("quoteMint")) {
                                    
                                    // Only include pools with significant liquidity
                                    if let Some(liquidity) = pool.get("liquidity") {
                                        if liquidity.as_f64().unwrap_or(0.0) > 10000.0 {
                                            pools.push(PoolInfo {
                                                address: id.as_str().unwrap_or("").to_string(),
                                                token_a: base_mint.as_str().unwrap_or("").to_string(),
                                                token_b: quote_mint.as_str().unwrap_or("").to_string(),
                                                dex_type: PoolType::Raydium,
                                                source: "Raydium Official".to_string(),
                                            });
                                        }
                                    }
                                }
                            }
                        }
                    }
                    
                    // Limit to top 50 pools by liquidity
                    pools.truncate(50);
                    Ok(pools)
                } else {
                    Err(anyhow!("Raydium API failed: {}", response.status()))
                }
            }
            Err(e) => Err(anyhow!("Raydium API error: {}", e)),
        }
    }
    
    /// Fetch pools from Orca API (excellent Whirlpool coverage)
    async fn fetch_orca_pools_enhanced(&self) -> Result<Vec<PoolInfo>> {
        let client = reqwest::Client::new();
        let url = "https://api.orca.so/v1/whirlpool/list";
        
        match client.get(url).send().await {
            Ok(response) => {
                if response.status().is_success() {
                    let data: Value = response.json().await?;
                    let mut pools = Vec::new();
                    
                    if let Some(whirlpools) = data.get("whirlpools") {
                        if let Some(whirlpool_array) = whirlpools.as_array() {
                            for pool in whirlpool_array {
                                if let (Some(address), Some(token_a), Some(token_b)) = 
                                    (pool.get("address"), pool.get("tokenA"), pool.get("tokenB")) {
                                    
                                    if let (Some(mint_a), Some(mint_b)) = 
                                        (token_a.get("mint"), token_b.get("mint")) {
                                        
                                        // Check if pool has reasonable liquidity
                                        if let Some(liquidity) = pool.get("liquidity") {
                                            if liquidity.as_f64().unwrap_or(0.0) > 1000.0 {
                                                pools.push(PoolInfo {
                                                    address: address.as_str().unwrap_or("").to_string(),
                                                    token_a: mint_a.as_str().unwrap_or("").to_string(),
                                                    token_b: mint_b.as_str().unwrap_or("").to_string(),
                                                    dex_type: PoolType::OrcaWhirlpool,
                                                    source: "Orca Whirlpool".to_string(),
                                                });
                                            }
                                        }
                                    }
                                }
                            }
                        }
                    }
                    
                    // Limit to top 50 pools by liquidity
                    pools.truncate(50);
                    Ok(pools)
                } else {
                    Err(anyhow!("Orca API failed: {}", response.status()))
                }
            }
            Err(e) => Err(anyhow!("Orca API error: {}", e)),
        }
    }
    
    /// Fetch pools from DexScreener API (good for validation)
    async fn fetch_dexscreener_pools(&self) -> Result<Vec<PoolInfo>> {
        let client = reqwest::Client::new();
        let mut pools = Vec::new();
        
        // Query major tokens
        let major_tokens = vec![
            "So11111111111111111111111111111111111111112", // SOL
            "EPjFWdd5AufqSSqeM2qN1xzybapC8G4wEGGkZwyTDt1v", // USDC
            "Es9vMFrzaCERmJfrF4H2FYD4KCoNkY11McCe8BenwNYB", // USDT
            "4k3Dyjzvzp8eMZWUXbBCjEvwSkkk59S5iCNLY3QrkX6R", // RAY
        ];
        
        for token in major_tokens {
            let url = format!("https://api.dexscreener.com/latest/dex/tokens/{}", token);
            
            match client.get(&url).send().await {
                Ok(response) => {
                    if response.status().is_success() {
                        if let Ok(data) = response.json::<Value>().await {
                            if let Some(pairs) = data.get("pairs") {
                                if let Some(pairs_array) = pairs.as_array() {
                                    for pair in pairs_array {
                                        if let (Some(dex_id), Some(pair_address), Some(base_token), Some(quote_token)) = 
                                            (pair.get("dexId"), pair.get("pairAddress"), pair.get("baseToken"), pair.get("quoteToken")) {
                                            
                                            if let (Some(base_address), Some(quote_address)) = 
                                                (base_token.get("address"), quote_token.get("address")) {
                                                
                                                pools.push(PoolInfo {
                                                    address: pair_address.as_str().unwrap_or("").to_string(),
                                                    token_a: base_address.as_str().unwrap_or("").to_string(),
                                                    token_b: quote_address.as_str().unwrap_or("").to_string(),
                                                    dex_type: self.detect_dex_from_id(dex_id.as_str().unwrap_or("")),
                                                    source: "DexScreener".to_string(),
                                                });
                                            }
                                        }
                                    }
                                }
                            }
                        }
                    }
                }
                Err(e) => warn!("DexScreener API error for {}: {}", token, e),
            }
        }
        
        Ok(pools)
    }
    
    /// Detect DEX type from Jupiter label
    fn detect_dex_from_label(&self, label: &str) -> PoolType {
        match label.to_lowercase().as_str() {
            l if l.contains("raydium") => PoolType::Raydium,
            l if l.contains("orca") => PoolType::OrcaWhirlpool,
            l if l.contains("whirlpool") => PoolType::OrcaWhirlpool,
            l if l.contains("serum") => PoolType::Serum,
            _ => PoolType::Raydium, // Default fallback
        }
    }
    
    /// Detect DEX type from DexScreener ID
    fn detect_dex_from_id(&self, dex_id: &str) -> PoolType {
        match dex_id.to_lowercase().as_str() {
            "raydium" => PoolType::Raydium,
            "orca" => PoolType::OrcaWhirlpool,
            "serum" => PoolType::Serum,
            _ => PoolType::Raydium,
        }
    }
    
    /// Validate pool from API data
    async fn validate_pool_from_api(&self, pool_info: &PoolInfo) -> Result<PoolData> {
        let pool_pubkey = Pubkey::from_str(&pool_info.address)?;
        
        // Try to get the account from blockchain
        let account = self.client.get_account(&pool_pubkey).await
            .map_err(|e| anyhow!("Pool account not found: {}", e))?;
        
        // Parse based on detected DEX type
        let pool_data = match pool_info.dex_type {
            PoolType::Raydium => {
                self.intelligent_raydium_parsing(pool_pubkey, &account).await?
            }
            PoolType::OrcaWhirlpool => {
                self.intelligent_whirlpool_parsing(pool_pubkey, &account).await?
            }
            PoolType::Orca => {
                self.intelligent_orca_parsing(pool_pubkey, &account).await?
            }
            PoolType::Serum => {
                return Err(anyhow!("Serum parsing not implemented"));
            }
        };
        
        // Additional validation - ensure liquidity is reasonable
        if pool_data.token_a_amount < 1000 || pool_data.token_b_amount < 1000 {
            return Err(anyhow!("Insufficient liquidity"));
        }
        
        info!("✅ API Pool validated: {} ({}) - {:.2}K + {:.2}K liquidity", 
            &pool_info.address[..8], 
            pool_info.source,
            pool_data.token_a_amount as f64 / 1000.0,
            pool_data.token_b_amount as f64 / 1000.0
        );
        
        Ok(pool_data)
    }
}

/// Structure to hold pool information from APIs
#[derive(Debug, Clone)]
struct PoolInfo {
    address: String,
    token_a: String,
    token_b: String,
    dex_type: PoolType,
    source: String,
}
