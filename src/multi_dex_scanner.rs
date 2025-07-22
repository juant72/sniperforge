// Multi-DEX Pool Discovery Module
// Modular implementation for PROPOSAL-002

use crate::types::*;
use std::collections::HashMap;
use std::time::Duration;
use tokio::time::Instant;
use reqwest::Client;
use serde::{Deserialize, Serialize};

#[derive(Debug)]
pub struct MultiDexPoolScanner {
    pub dex_integrations: HashMap<String, Box<dyn DexIntegration>>,
    pub pool_cache: PoolCache,
    pub health_monitor: PoolHealthMonitor,
    pub client: Client,
}

#[derive(Debug, Clone)]
pub struct DiscoveredPool {
    pub address: String,
    pub dex_type: DexType,
    pub token_a: String,
    pub token_b: String,
    pub tvl_usd: f64,
    pub volume_24h_usd: f64,
    pub fee_tier: f64,
    pub discovered_at: Instant,
    pub health_score: f64,
    pub liquidity_concentration: Option<f64>,
}

#[derive(Debug, Clone)]
pub struct PoolCache {
    pub pools: HashMap<String, DiscoveredPool>,
    pub last_update: Instant,
    pub cache_duration: Duration,
}

#[derive(Debug, Clone)]
pub struct PoolHealthMonitor {
    pub min_tvl_usd: f64,
    pub min_volume_24h_usd: f64,
    pub max_age_hours: u64,
    pub health_check_interval: Duration,
}

// Trait for DEX integrations
#[async_trait::async_trait]
pub trait DexIntegration: Send + Sync + std::fmt::Debug {
    async fn get_pools(&self) -> Result<Vec<DiscoveredPool>, Box<dyn std::error::Error>>;
    async fn get_pool_info(&self, address: &str) -> Result<DiscoveredPool, Box<dyn std::error::Error>>;
    async fn validate_pool(&self, pool: &DiscoveredPool) -> Result<bool, Box<dyn std::error::Error>>;
    fn get_dex_name(&self) -> &str;
}

impl MultiDexPoolScanner {
    pub fn new() -> Self {
        let mut scanner = Self {
            dex_integrations: HashMap::new(),
            pool_cache: PoolCache::new(),
            health_monitor: PoolHealthMonitor::new(),
            client: Client::new(),
        };
        
        // Initialize DEX integrations
        scanner.initialize_dex_integrations();
        scanner
    }
    
    fn initialize_dex_integrations(&mut self) {
        // Add Meteora integration
        self.dex_integrations.insert(
            "meteora".to_string(),
            Box::new(MeteoraIntegration::new())
        );
        
        // Add Lifinity integration
        self.dex_integrations.insert(
            "lifinity".to_string(),
            Box::new(LifinityIntegration::new())
        );
        
        // Add Phoenix integration
        self.dex_integrations.insert(
            "phoenix".to_string(),
            Box::new(PhoenixIntegration::new())
        );
        
        // Add Saber integration for stablecoins
        self.dex_integrations.insert(
            "saber".to_string(),
            Box::new(SaberIntegration::new())
        );
    }
    
    pub async fn discover_all_pools(&mut self) -> Result<Vec<DiscoveredPool>, Box<dyn std::error::Error>> {
        println!("🔍 [MULTI-DEX] Starting comprehensive pool discovery...");
        let start_time = Instant::now();
        let mut all_pools = Vec::new();
        
        // 1. Scan each DEX for pools
        for (dex_name, integration) in &self.dex_integrations {
            println!("📊 [{}] Scanning for pools...", dex_name.to_uppercase());
            
            match integration.get_pools().await {
                Ok(dex_pools) => {
                    println!("✅ [{}] Found {} pools", dex_name.to_uppercase(), dex_pools.len());
                    all_pools.extend(dex_pools);
                }
                Err(e) => {
                    println!("⚠️ [{}] Error scanning pools: {}", dex_name.to_uppercase(), e);
                }
            }
        }
        
        // 2. Filter by liquidity and volume thresholds
        let qualified_pools: Vec<DiscoveredPool> = all_pools.into_iter()
            .filter(|pool| {
                pool.tvl_usd > self.health_monitor.min_tvl_usd &&
                pool.volume_24h_usd > self.health_monitor.min_volume_24h_usd
            })
            .collect();
        
        println!("🏆 [MULTI-DEX] Qualified pools after filtering: {}", qualified_pools.len());
        
        // 3. Calculate health scores
        let scored_pools = self.calculate_health_scores(qualified_pools).await?;
        
        // 4. Update pool cache
        self.pool_cache.update_pools(scored_pools.clone()).await;
        
        let discovery_time = start_time.elapsed();
        println!("⚡ [MULTI-DEX] Pool discovery completed in {:?}", discovery_time);
        
        Ok(scored_pools)
    }
    
    async fn calculate_health_scores(&self, pools: Vec<DiscoveredPool>) -> Result<Vec<DiscoveredPool>, Box<dyn std::error::Error>> {
        let mut scored_pools = Vec::new();
        
        for mut pool in pools {
            // Calculate health score based on multiple factors
            let mut health_score = 0.0;
            
            // TVL score (0-40 points)
            let tvl_score = if pool.tvl_usd > 1_000_000.0 {
                40.0
            } else if pool.tvl_usd > 500_000.0 {
                30.0
            } else if pool.tvl_usd > 200_000.0 {
                20.0
            } else {
                10.0
            };
            health_score += tvl_score;
            
            // Volume score (0-30 points)
            let volume_score = if pool.volume_24h_usd > 500_000.0 {
                30.0
            } else if pool.volume_24h_usd > 200_000.0 {
                20.0
            } else if pool.volume_24h_usd > 100_000.0 {
                15.0
            } else {
                5.0
            };
            health_score += volume_score;
            
            // DEX reputation score (0-20 points)
            let dex_score = match pool.dex_type {
                DexType::Raydium => 20.0,
                DexType::Orca => 20.0,
                DexType::OrcaWhirlpool => 20.0,
                DexType::Meteora => 18.0,
                DexType::Lifinity => 16.0,
                DexType::Phoenix => 15.0,
                DexType::Saber => 14.0,
                _ => 10.0,
            };
            health_score += dex_score;
            
            // Fee tier efficiency score (0-10 points)
            let fee_score = if pool.fee_tier <= 0.01 {
                10.0
            } else if pool.fee_tier <= 0.05 {
                8.0
            } else if pool.fee_tier <= 0.1 {
                5.0
            } else {
                2.0
            };
            health_score += fee_score;
            
            pool.health_score = health_score;
            scored_pools.push(pool);
        }
        
        // Sort by health score (best first)
        scored_pools.sort_by(|a, b| b.health_score.partial_cmp(&a.health_score).unwrap());
        
        Ok(scored_pools)
    }
    
    pub async fn get_top_pools(&self, limit: usize) -> Vec<DiscoveredPool> {
        let mut pools: Vec<_> = self.pool_cache.pools.values().cloned().collect();
        pools.sort_by(|a, b| b.health_score.partial_cmp(&a.health_score).unwrap());
        pools.into_iter().take(limit).collect()
    }
    
    pub async fn get_pools_by_pair(&self, token_a: &str, token_b: &str) -> Vec<DiscoveredPool> {
        self.pool_cache.pools.values()
            .filter(|pool| {
                (pool.token_a == token_a && pool.token_b == token_b) ||
                (pool.token_a == token_b && pool.token_b == token_a)
            })
            .cloned()
            .collect()
    }
}

impl PoolCache {
    pub fn new() -> Self {
        Self {
            pools: HashMap::new(),
            last_update: Instant::now(),
            cache_duration: Duration::from_secs(10 * 60), // 10 minutes cache
        }
    }
    
    pub async fn update_pools(&mut self, pools: Vec<DiscoveredPool>) {
        for pool in pools {
            self.pools.insert(pool.address.clone(), pool);
        }
        self.last_update = Instant::now();
        println!("💾 [CACHE] Updated with {} pools", self.pools.len());
    }
    
    pub fn is_cache_fresh(&self) -> bool {
        self.last_update.elapsed() < self.cache_duration
    }
    
    pub fn get_pool_count(&self) -> usize {
        self.pools.len()
    }
}

impl PoolHealthMonitor {
    pub fn new() -> Self {
        Self {
            min_tvl_usd: 200_000.0,        // Minimum $200k TVL
            min_volume_24h_usd: 50_000.0,  // Minimum $50k daily volume
            max_age_hours: 24,             // Max 24 hours old data
            health_check_interval: Duration::from_secs(30 * 60),
        }
    }
    
    pub async fn monitor_pool_health(&self, pools: &[DiscoveredPool]) -> Vec<DiscoveredPool> {
        pools.iter()
            .filter(|pool| {
                pool.tvl_usd >= self.min_tvl_usd &&
                pool.volume_24h_usd >= self.min_volume_24h_usd &&
                pool.discovered_at.elapsed().as_secs() < (self.max_age_hours * 3600)
            })
            .cloned()
            .collect()
    }
}

// Meteora DEX Integration
#[derive(Debug)]
pub struct MeteoraIntegration {
    client: Client,
    api_base: String,
}

impl MeteoraIntegration {
    pub fn new() -> Self {
        Self {
            client: Client::new(),
            api_base: "https://app.meteora.ag/api".to_string(),
        }
    }
}

#[async_trait::async_trait]
impl DexIntegration for MeteoraIntegration {
    async fn get_pools(&self) -> Result<Vec<DiscoveredPool>, Box<dyn std::error::Error>> {
        let url = format!("{}/pools", self.api_base);
        
        match self.client.get(&url).send().await {
            Ok(response) => {
                let pools_data: serde_json::Value = response.json().await?;
                let mut pools = Vec::new();
                
                if let Some(pools_array) = pools_data.as_array() {
                    for pool_data in pools_array {
                        if let Ok(pool) = self.parse_meteora_pool(pool_data) {
                            pools.push(pool);
                        }
                    }
                }
                
                println!("📊 [METEORA] Retrieved {} pools", pools.len());
                Ok(pools)
            }
            Err(e) => {
                println!("❌ [METEORA] API error: {}", e);
                Ok(Vec::new()) // Return empty vector on error
            }
        }
    }
    
    async fn get_pool_info(&self, address: &str) -> Result<DiscoveredPool, Box<dyn std::error::Error>> {
        let url = format!("{}/pools/{}", self.api_base, address);
        let response = self.client.get(&url).send().await?;
        let pool_data: serde_json::Value = response.json().await?;
        self.parse_meteora_pool(&pool_data)
    }
    
    async fn validate_pool(&self, pool: &DiscoveredPool) -> Result<bool, Box<dyn std::error::Error>> {
        // Validate Meteora pool characteristics
        Ok(pool.tvl_usd > 100_000.0 && 
           pool.volume_24h_usd > 10_000.0 &&
           pool.fee_tier < 0.1)
    }
    
    fn get_dex_name(&self) -> &str {
        "meteora"
    }
}

impl MeteoraIntegration {
    fn parse_meteora_pool(&self, pool_data: &serde_json::Value) -> Result<DiscoveredPool, Box<dyn std::error::Error>> {
        Ok(DiscoveredPool {
            address: pool_data["address"].as_str().unwrap_or("").to_string(),
            dex_type: DexType::Meteora,
            token_a: pool_data["token_a"]["symbol"].as_str().unwrap_or("").to_string(),
            token_b: pool_data["token_b"]["symbol"].as_str().unwrap_or("").to_string(),
            tvl_usd: pool_data["tvl"].as_f64().unwrap_or(0.0),
            volume_24h_usd: pool_data["volume_24h"].as_f64().unwrap_or(0.0),
            fee_tier: pool_data["fee_tier"].as_f64().unwrap_or(0.003),
            discovered_at: Instant::now(),
            health_score: 0.0,
            liquidity_concentration: pool_data["concentration_ratio"].as_f64(),
        })
    }
}

// Lifinity DEX Integration
#[derive(Debug)]
pub struct LifinityIntegration {
    client: Client,
    api_base: String,
}

impl LifinityIntegration {
    pub fn new() -> Self {
        Self {
            client: Client::new(),
            api_base: "https://api.lifinity.io".to_string(),
        }
    }
}

#[async_trait::async_trait]
impl DexIntegration for LifinityIntegration {
    async fn get_pools(&self) -> Result<Vec<DiscoveredPool>, Box<dyn std::error::Error>> {
        let url = format!("{}/pools", self.api_base);
        
        match self.client.get(&url).send().await {
            Ok(response) => {
                let pools_data: serde_json::Value = response.json().await?;
                let mut pools = Vec::new();
                
                if let Some(pools_array) = pools_data.as_array() {
                    for pool_data in pools_array {
                        if let Ok(pool) = self.parse_lifinity_pool(pool_data) {
                            pools.push(pool);
                        }
                    }
                }
                
                println!("📊 [LIFINITY] Retrieved {} pools", pools.len());
                Ok(pools)
            }
            Err(e) => {
                println!("❌ [LIFINITY] API error: {}", e);
                Ok(Vec::new())
            }
        }
    }
    
    async fn get_pool_info(&self, address: &str) -> Result<DiscoveredPool, Box<dyn std::error::Error>> {
        let url = format!("{}/pools/{}", self.api_base, address);
        let response = self.client.get(&url).send().await?;
        let pool_data: serde_json::Value = response.json().await?;
        self.parse_lifinity_pool(&pool_data)
    }
    
    async fn validate_pool(&self, pool: &DiscoveredPool) -> Result<bool, Box<dyn std::error::Error>> {
        Ok(pool.tvl_usd > 150_000.0 && pool.volume_24h_usd > 20_000.0)
    }
    
    fn get_dex_name(&self) -> &str {
        "lifinity"
    }
}

impl LifinityIntegration {
    fn parse_lifinity_pool(&self, pool_data: &serde_json::Value) -> Result<DiscoveredPool, Box<dyn std::error::Error>> {
        Ok(DiscoveredPool {
            address: pool_data["address"].as_str().unwrap_or("").to_string(),
            dex_type: DexType::Lifinity,
            token_a: pool_data["token_a"]["symbol"].as_str().unwrap_or("").to_string(),
            token_b: pool_data["token_b"]["symbol"].as_str().unwrap_or("").to_string(),
            tvl_usd: pool_data["tvl"].as_f64().unwrap_or(0.0),
            volume_24h_usd: pool_data["volume_24h"].as_f64().unwrap_or(0.0),
            fee_tier: pool_data["fee"].as_f64().unwrap_or(0.005),
            discovered_at: Instant::now(),
            health_score: 0.0,
            liquidity_concentration: None,
        })
    }
}

// Phoenix DEX Integration
#[derive(Debug)]
pub struct PhoenixIntegration {
    client: Client,
    api_base: String,
}

impl PhoenixIntegration {
    pub fn new() -> Self {
        Self {
            client: Client::new(),
            api_base: "https://api.phoenix.trade".to_string(),
        }
    }
}

#[async_trait::async_trait]
impl DexIntegration for PhoenixIntegration {
    async fn get_pools(&self) -> Result<Vec<DiscoveredPool>, Box<dyn std::error::Error>> {
        let url = format!("{}/markets", self.api_base);
        
        match self.client.get(&url).send().await {
            Ok(response) => {
                let markets_data: serde_json::Value = response.json().await?;
                let mut pools = Vec::new();
                
                if let Some(markets_array) = markets_data.as_array() {
                    for market_data in markets_array {
                        if let Ok(pool) = self.parse_phoenix_market(market_data) {
                            pools.push(pool);
                        }
                    }
                }
                
                println!("📊 [PHOENIX] Retrieved {} markets", pools.len());
                Ok(pools)
            }
            Err(e) => {
                println!("❌ [PHOENIX] API error: {}", e);
                Ok(Vec::new())
            }
        }
    }
    
    async fn get_pool_info(&self, address: &str) -> Result<DiscoveredPool, Box<dyn std::error::Error>> {
        let url = format!("{}/markets/{}", self.api_base, address);
        let response = self.client.get(&url).send().await?;
        let market_data: serde_json::Value = response.json().await?;
        self.parse_phoenix_market(&market_data)
    }
    
    async fn validate_pool(&self, pool: &DiscoveredPool) -> Result<bool, Box<dyn std::error::Error>> {
        Ok(pool.tvl_usd > 100_000.0 && pool.volume_24h_usd > 15_000.0)
    }
    
    fn get_dex_name(&self) -> &str {
        "phoenix"
    }
}

impl PhoenixIntegration {
    fn parse_phoenix_market(&self, market_data: &serde_json::Value) -> Result<DiscoveredPool, Box<dyn std::error::Error>> {
        Ok(DiscoveredPool {
            address: market_data["address"].as_str().unwrap_or("").to_string(),
            dex_type: DexType::Phoenix,
            token_a: market_data["base_token"]["symbol"].as_str().unwrap_or("").to_string(),
            token_b: market_data["quote_token"]["symbol"].as_str().unwrap_or("").to_string(),
            tvl_usd: market_data["total_liquidity"].as_f64().unwrap_or(0.0),
            volume_24h_usd: market_data["volume_24h"].as_f64().unwrap_or(0.0),
            fee_tier: market_data["taker_fee"].as_f64().unwrap_or(0.002),
            discovered_at: Instant::now(),
            health_score: 0.0,
            liquidity_concentration: None,
        })
    }
}

// Saber DEX Integration (for stablecoins)
#[derive(Debug)]
pub struct SaberIntegration {
    client: Client,
    api_base: String,
}

impl SaberIntegration {
    pub fn new() -> Self {
        Self {
            client: Client::new(),
            api_base: "https://api.saber.so".to_string(),
        }
    }
}

#[async_trait::async_trait]
impl DexIntegration for SaberIntegration {
    async fn get_pools(&self) -> Result<Vec<DiscoveredPool>, Box<dyn std::error::Error>> {
        let url = format!("{}/pools", self.api_base);
        
        match self.client.get(&url).send().await {
            Ok(response) => {
                let pools_data: serde_json::Value = response.json().await?;
                let mut pools = Vec::new();
                
                if let Some(pools_array) = pools_data.as_array() {
                    for pool_data in pools_array {
                        if let Ok(pool) = self.parse_saber_pool(pool_data) {
                            // Only include stablecoin pairs
                            if self.is_stablecoin_pair(&pool) {
                                pools.push(pool);
                            }
                        }
                    }
                }
                
                println!("📊 [SABER] Retrieved {} stablecoin pools", pools.len());
                Ok(pools)
            }
            Err(e) => {
                println!("❌ [SABER] API error: {}", e);
                Ok(Vec::new())
            }
        }
    }
    
    async fn get_pool_info(&self, address: &str) -> Result<DiscoveredPool, Box<dyn std::error::Error>> {
        let url = format!("{}/pools/{}", self.api_base, address);
        let response = self.client.get(&url).send().await?;
        let pool_data: serde_json::Value = response.json().await?;
        self.parse_saber_pool(&pool_data)
    }
    
    async fn validate_pool(&self, pool: &DiscoveredPool) -> Result<bool, Box<dyn std::error::Error>> {
        Ok(pool.tvl_usd > 200_000.0 && 
           pool.volume_24h_usd > 30_000.0 &&
           self.is_stablecoin_pair(pool))
    }
    
    fn get_dex_name(&self) -> &str {
        "saber"
    }
}

impl SaberIntegration {
    fn parse_saber_pool(&self, pool_data: &serde_json::Value) -> Result<DiscoveredPool, Box<dyn std::error::Error>> {
        Ok(DiscoveredPool {
            address: pool_data["address"].as_str().unwrap_or("").to_string(),
            dex_type: DexType::Saber,
            token_a: pool_data["token_a"]["symbol"].as_str().unwrap_or("").to_string(),
            token_b: pool_data["token_b"]["symbol"].as_str().unwrap_or("").to_string(),
            tvl_usd: pool_data["tvl"].as_f64().unwrap_or(0.0),
            volume_24h_usd: pool_data["volume_24h"].as_f64().unwrap_or(0.0),
            fee_tier: pool_data["swap_fee"].as_f64().unwrap_or(0.0006),
            discovered_at: Instant::now(),
            health_score: 0.0,
            liquidity_concentration: None,
        })
    }
    
    fn is_stablecoin_pair(&self, pool: &DiscoveredPool) -> bool {
        let stablecoins = ["USDC", "USDT", "UST", "DAI", "FRAX", "BUSD", "TUSD"];
        stablecoins.contains(&pool.token_a.as_str()) || 
        stablecoins.contains(&pool.token_b.as_str())
    }
}
