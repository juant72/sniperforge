// Multi-DEX Integration Module
// Integrates multi-dex scanner with existing arbitrage system

use crate::multi_dex_scanner::*;
use crate::types::*;
use std::collections::HashMap;
use tokio::time::Duration;

pub struct EnhancedPoolDiscovery {
    pub multi_dex_scanner: MultiDexPoolScanner,
    pub legacy_pools: Vec<(String, DexType, String, String)>, // Original 3 pools
    pub enhanced_pools: HashMap<String, DiscoveredPool>,
    pub discovery_interval: Duration,
}

impl EnhancedPoolDiscovery {
    pub fn new() -> Self {
        let mut scanner = Self {
            multi_dex_scanner: MultiDexPoolScanner::new(),
            legacy_pools: vec![
                ("58oQChx4yWmvKdwLLZzBi4ChoCc2fqCUWBkwMihLYQo2".to_string(), DexType::Raydium, "SOL".to_string(), "USDC".to_string()),
                ("HJPjoWUrhoZzkNfRpHuieeFk9WcZWjwy6PBjZ81ngndJ".to_string(), DexType::OrcaWhirlpool, "SOL".to_string(), "USDC".to_string()),
                ("9W959DqEETiGZocYWCQPaJ6sBmUzgfxXfqGeTEdp3aQP".to_string(), DexType::Orca, "SOL".to_string(), "USDC".to_string()),
            ],
            enhanced_pools: HashMap::new(),
            discovery_interval: Duration::from_secs(15 * 60), // Discovery every 15 minutes
        };
        
        // Initialize with legacy pools
        scanner.initialize_legacy_pools();
        scanner
    }
    
    fn initialize_legacy_pools(&mut self) {
        for (address, dex_type, token_a, token_b) in &self.legacy_pools {
            let pool = DiscoveredPool {
                address: address.clone(),
                dex_type: dex_type.clone(),
                token_a: token_a.clone(),
                token_b: token_b.clone(),
                tvl_usd: 500_000.0, // Assume good TVL for legacy pools
                volume_24h_usd: 100_000.0, // Assume good volume
                fee_tier: 0.003,
                discovered_at: tokio::time::Instant::now(),
                health_score: 90.0, // High score for verified pools
                liquidity_concentration: None,
            };
            
            self.enhanced_pools.insert(address.clone(), pool);
        }
        
        println!("✅ [ENHANCED] Initialized with {} legacy pools", self.legacy_pools.len());
    }
    
    pub async fn execute_comprehensive_pool_discovery(&mut self) -> Result<HashMap<String, DiscoveredPool>, Box<dyn std::error::Error>> {
        println!("🚀 [ENHANCED] Starting comprehensive multi-DEX pool discovery...");
        
        // 1. Discover new pools from all DEXs
        let discovered_pools = self.multi_dex_scanner.discover_all_pools().await?;
        
        // 2. Merge with existing pools
        for pool in discovered_pools {
            self.enhanced_pools.insert(pool.address.clone(), pool);
        }
        
        // 3. Get top performing pools
        let top_pools = self.get_top_pools_by_health_score(50).await;
        
        println!("🏆 [ENHANCED] Total pools available: {}", self.enhanced_pools.len());
        println!("🎯 [ENHANCED] Top performing pools: {}", top_pools.len());
        
        // 4. Log DEX distribution
        self.log_dex_distribution().await;
        
        Ok(self.enhanced_pools.clone())
    }
    
    pub async fn get_pools_for_arbitrage(&self, token_a: &str, token_b: &str) -> Vec<DiscoveredPool> {
        let mut matching_pools = Vec::new();
        
        for pool in self.enhanced_pools.values() {
            if (pool.token_a == token_a && pool.token_b == token_b) ||
               (pool.token_a == token_b && pool.token_b == token_a) {
                matching_pools.push(pool.clone());
            }
        }
        
        // Sort by health score
        matching_pools.sort_by(|a, b| b.health_score.partial_cmp(&a.health_score).unwrap());
        
        println!("🔍 [ENHANCED] Found {} pools for {}/{} pair", matching_pools.len(), token_a, token_b);
        matching_pools
    }
    
    pub async fn get_top_pools_by_health_score(&self, limit: usize) -> Vec<DiscoveredPool> {
        let mut pools: Vec<_> = self.enhanced_pools.values().cloned().collect();
        pools.sort_by(|a, b| b.health_score.partial_cmp(&a.health_score).unwrap());
        pools.into_iter().take(limit).collect()
    }
    
    pub async fn get_pools_by_dex(&self, dex_type: &DexType) -> Vec<DiscoveredPool> {
        self.enhanced_pools.values()
            .filter(|pool| &pool.dex_type == dex_type)
            .cloned()
            .collect()
    }
    
    pub async fn get_high_volume_pools(&self, min_volume_usd: f64) -> Vec<DiscoveredPool> {
        self.enhanced_pools.values()
            .filter(|pool| pool.volume_24h_usd >= min_volume_usd)
            .cloned()
            .collect()
    }
    
    async fn log_dex_distribution(&self) {
        let mut dex_counts: HashMap<String, usize> = HashMap::new();
        let mut total_tvl_by_dex: HashMap<String, f64> = HashMap::new();
        
        for pool in self.enhanced_pools.values() {
            let dex_name = format!("{}", pool.dex_type);
            *dex_counts.entry(dex_name.clone()).or_insert(0) += 1;
            *total_tvl_by_dex.entry(dex_name).or_insert(0.0) += pool.tvl_usd;
        }
        
        println!("📊 [ENHANCED] DEX Distribution:");
        for (dex, count) in &dex_counts {
            let avg_tvl = total_tvl_by_dex.get(dex).unwrap_or(&0.0) / *count as f64;
            println!("   {} {} pools (avg TVL: ${:.0})", 
                    dex, count, avg_tvl);
        }
    }
    
    pub async fn generate_opportunity_matrix(&self) -> Vec<ArbitrageOpportunityMatrix> {
        let mut opportunity_matrix = Vec::new();
        let sol_usdc_pools = self.get_pools_for_arbitrage("SOL", "USDC").await;
        
        // Generate all possible pool combinations for arbitrage
        for (i, pool_a) in sol_usdc_pools.iter().enumerate() {
            for (j, pool_b) in sol_usdc_pools.iter().enumerate() {
                if i != j && pool_a.dex_type != pool_b.dex_type {
                    let opportunity = ArbitrageOpportunityMatrix {
                        pool_a: pool_a.clone(),
                        pool_b: pool_b.clone(),
                        estimated_gas_cost: self.estimate_gas_cost(&pool_a.dex_type, &pool_b.dex_type),
                        liquidity_score: (pool_a.health_score + pool_b.health_score) / 2.0,
                        cross_dex_difficulty: self.calculate_cross_dex_difficulty(&pool_a.dex_type, &pool_b.dex_type),
                    };
                    opportunity_matrix.push(opportunity);
                }
            }
        }
        
        // Sort by liquidity score
        opportunity_matrix.sort_by(|a, b| b.liquidity_score.partial_cmp(&a.liquidity_score).unwrap());
        
        println!("🎯 [ENHANCED] Generated {} arbitrage opportunities", opportunity_matrix.len());
        opportunity_matrix
    }
    
    fn estimate_gas_cost(&self, dex_a: &DexType, dex_b: &DexType) -> f64 {
        // Estimate gas costs based on DEX complexity
        let base_cost = 0.01; // 0.01 SOL base cost
        
        let dex_a_complexity = match dex_a {
            DexType::Raydium => 1.0,
            DexType::Orca => 1.0,
            DexType::OrcaWhirlpool => 1.2,
            DexType::Meteora => 1.3,
            DexType::Lifinity => 1.1,
            DexType::Phoenix => 1.4,
            DexType::Saber => 1.1,
            _ => 1.5,
        };
        
        let dex_b_complexity = match dex_b {
            DexType::Raydium => 1.0,
            DexType::Orca => 1.0,
            DexType::OrcaWhirlpool => 1.2,
            DexType::Meteora => 1.3,
            DexType::Lifinity => 1.1,
            DexType::Phoenix => 1.4,
            DexType::Saber => 1.1,
            _ => 1.5,
        };
        
        base_cost * (dex_a_complexity + dex_b_complexity)
    }
    
    fn calculate_cross_dex_difficulty(&self, dex_a: &DexType, dex_b: &DexType) -> f64 {
        // Calculate difficulty score for cross-DEX arbitrage
        let compatibility_matrix = [
            // Raydium compatibility
            (DexType::Raydium, DexType::Orca, 0.9),
            (DexType::Raydium, DexType::Meteora, 0.8),
            (DexType::Raydium, DexType::Lifinity, 0.7),
            // Orca compatibility
            (DexType::Orca, DexType::OrcaWhirlpool, 0.95),
            (DexType::Orca, DexType::Phoenix, 0.6),
            // Default difficulty
        ];
        
        // Find specific compatibility or return default
        for (dex1, dex2, score) in &compatibility_matrix {
            if (dex1 == dex_a && dex2 == dex_b) || (dex1 == dex_b && dex2 == dex_a) {
                return *score;
            }
        }
        
        0.5 // Default medium difficulty
    }
}

#[derive(Debug, Clone)]
pub struct ArbitrageOpportunityMatrix {
    pub pool_a: DiscoveredPool,
    pub pool_b: DiscoveredPool,
    pub estimated_gas_cost: f64,
    pub liquidity_score: f64,
    pub cross_dex_difficulty: f64,
}

impl ArbitrageOpportunityMatrix {
    pub fn get_profit_potential_score(&self) -> f64 {
        // Calculate overall profit potential
        let liquidity_factor = self.liquidity_score / 100.0;
        let gas_penalty = 1.0 - (self.estimated_gas_cost / 0.1); // Penalty for high gas
        let difficulty_factor = self.cross_dex_difficulty;
        
        (liquidity_factor * 0.5 + gas_penalty * 0.3 + difficulty_factor * 0.2) * 100.0
    }
    
    pub fn is_viable_opportunity(&self) -> bool {
        self.liquidity_score > 60.0 && 
        self.estimated_gas_cost < 0.05 && 
        self.cross_dex_difficulty > 0.4
    }
}

// Integration with existing system
pub async fn execute_enhanced_pool_discovery() -> Result<Vec<(String, DexType, String, String)>, Box<dyn std::error::Error>> {
    let mut enhanced_discovery = EnhancedPoolDiscovery::new();
    
    // Execute comprehensive discovery
    let discovered_pools = enhanced_discovery.execute_comprehensive_pool_discovery().await?;
    
    // Convert to legacy format for backward compatibility
    let legacy_format: Vec<(String, DexType, String, String)> = discovered_pools.values()
        .filter(|pool| pool.health_score > 70.0) // Only high-quality pools
        .map(|pool| (
            pool.address.clone(),
            pool.dex_type.clone(),
            pool.token_a.clone(),
            pool.token_b.clone()
        ))
        .collect();
    
    println!("🔄 [INTEGRATION] Converted {} pools to legacy format", legacy_format.len());
    
    Ok(legacy_format)
}

// Enhanced opportunity discovery function
pub async fn discover_enhanced_opportunities() -> Result<Vec<ArbitrageOpportunityMatrix>, Box<dyn std::error::Error>> {
    let mut enhanced_discovery = EnhancedPoolDiscovery::new();
    
    // Execute discovery
    enhanced_discovery.execute_comprehensive_pool_discovery().await?;
    
    // Generate opportunity matrix
    let opportunities = enhanced_discovery.generate_opportunity_matrix().await;
    
    // Filter viable opportunities
    let viable_opportunities: Vec<ArbitrageOpportunityMatrix> = opportunities.into_iter()
        .filter(|opp| opp.is_viable_opportunity())
        .collect();
    
    println!("✅ [ENHANCED] Found {} viable arbitrage opportunities", viable_opportunities.len());
    
    Ok(viable_opportunities)
}
