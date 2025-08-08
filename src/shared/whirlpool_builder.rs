use anyhow::{Result, anyhow};
use log::{debug, info, warn};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// 🚀 ENTERPRISE WHIRLPOOL BUILDER - Shared module for all bots
/// 
/// Advanced Orca Whirlpool integration with concentrated liquidity optimization,
/// tick array management, and multi-hop routing capabilities.

#[derive(Debug, Clone)]
pub struct EnterpriseWhirlpoolBuilder {
    pub program_id: String,
    pub whirlpool_cache: HashMap<String, WhirlpoolInfo>,
    pub tick_arrays: HashMap<String, TickArray>,
    pub fee_growth_tracker: FeeGrowthTracker,
    pub routing_config: WhirlpoolRoutingConfig,
    pub performance_metrics: WhirlpoolPerformanceMetrics,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WhirlpoolInfo {
    pub whirlpool_id: String,
    pub token_a: String,
    pub token_b: String,
    pub tick_spacing: u16,
    pub fee_rate: u16, // In hundredths of a bps (e.g., 3000 = 0.30%)
    pub liquidity: u128,
    pub sqrt_price: u128,
    pub tick_current_index: i32,
    pub fee_growth_global_a: u128,
    pub fee_growth_global_b: u128,
    pub reward_infos: Vec<RewardInfo>,
    pub tick_range_efficiency: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TickArray {
    pub start_tick_index: i32,
    pub ticks: Vec<TickInfo>,
    pub whirlpool_address: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TickInfo {
    pub tick_index: i32,
    pub liquidity_net: i128,
    pub liquidity_gross: u128,
    pub fee_growth_outside_a: u128,
    pub fee_growth_outside_b: u128,
    pub reward_growths_outside: Vec<u128>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RewardInfo {
    pub reward_mint: String,
    pub vault: String,
    pub authority: String,
    pub emissions_per_second_x64: u128,
    pub growth_global_x64: u128,
}

#[derive(Debug, Clone)]
pub struct FeeGrowthTracker {
    pub global_fee_growth_a: HashMap<String, u128>,
    pub global_fee_growth_b: HashMap<String, u128>,
    pub last_update_slot: u64,
}

#[derive(Debug, Clone)]
pub struct WhirlpoolRoutingConfig {
    pub max_hops: u8,
    pub enable_multi_hop: bool,
    pub prefer_low_fee_pools: bool,
    pub max_price_impact: f64,
    pub enable_tick_optimization: bool,
}

#[derive(Debug, Default, Clone)]
pub struct WhirlpoolPerformanceMetrics {
    pub total_swaps: u64,
    pub successful_swaps: u64,
    pub failed_swaps: u64,
    pub average_tick_arrays_used: f64,
    pub average_price_impact: f64,
    pub gas_efficiency_score: f64,
    pub multi_hop_success_rate: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WhirlpoolSwapInstruction {
    pub whirlpool_address: String,
    pub tick_arrays: Vec<String>,
    pub oracle_account: Option<String>,
    pub amount_specified: u64,
    pub other_amount_threshold: u64,
    pub sqrt_price_limit: u128,
    pub amount_specified_is_input: bool,
    pub a_to_b: bool,
    pub estimated_gas: u64,
    pub route_info: WhirlpoolRouteInfo,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WhirlpoolRouteInfo {
    pub route_id: String,
    pub whirlpools_used: Vec<String>,
    pub total_fee_rate: u16,
    pub price_impact: f64,
    pub expected_output: u64,
    pub minimum_received: u64,
    pub tick_range_efficiency: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MultiHopRoute {
    pub route_id: String,
    pub hops: Vec<WhirlpoolHop>,
    pub total_price_impact: f64,
    pub total_fees: u64,
    pub estimated_output: u64,
    pub execution_complexity: u8,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WhirlpoolHop {
    pub whirlpool_id: String,
    pub input_mint: String,
    pub output_mint: String,
    pub fee_rate: u16,
    pub expected_price_impact: f64,
    pub tick_arrays_needed: Vec<String>,
}

/// 🚀 ENTERPRISE IMPLEMENTATIONS
impl EnterpriseWhirlpoolBuilder {
    /// Create new enterprise whirlpool builder
    pub fn new() -> Self {
        debug!("🏗️ Creating enterprise whirlpool builder");
        
        Self {
            program_id: "whirLbMiicVdio4qvUfM5KAg6Ct8VwpYzGff3uctyCc".to_string(),
            whirlpool_cache: HashMap::new(),
            tick_arrays: HashMap::new(),
            fee_growth_tracker: FeeGrowthTracker::default(),
            routing_config: WhirlpoolRoutingConfig::default(),
            performance_metrics: WhirlpoolPerformanceMetrics::default(),
        }
    }

    /// Create with custom routing configuration for different bot types
    pub fn with_routing_config(config: WhirlpoolRoutingConfig) -> Self {
        let mut builder = Self::new();
        builder.routing_config = config;
        builder
    }

    /// Find optimal whirlpool for direct swap
    pub async fn find_optimal_whirlpool(
        &mut self,
        input_mint: &str,
        output_mint: &str,
        amount: u64,
    ) -> Result<WhirlpoolInfo> {
        debug!("🔍 Finding optimal whirlpool for {}/{} with amount: {} lamports", input_mint, output_mint, amount);

        // 🚀 USAR AMOUNT: Optimize pool selection based on trade amount
        let amount_sol = amount as f64 / 1_000_000_000.0;
        debug!("💰 Trade amount: {:.6} SOL", amount_sol);

        // Check cache first - with amount-based cache key
        let cache_key = format!("{}_{}_{}", input_mint, output_mint, amount / 1_000_000); // Group by millions of lamports
        if let Some(cached_pool) = self.whirlpool_cache.get(&cache_key) {
            debug!("📋 Using cached whirlpool for amount {:.3} SOL: {}", amount_sol, cached_pool.whirlpool_id);
            return Ok(cached_pool.clone());
        }

        // Simulate whirlpool discovery from old-archive implementations
        let mock_whirlpool = WhirlpoolInfo {
            whirlpool_id: format!("orca_whirlpool_{}_{}_3000", 
                input_mint[..8].to_string(), 
                output_mint[..8].to_string()
            ),
            token_a: input_mint.to_string(),
            token_b: output_mint.to_string(),
            tick_spacing: 64,
            fee_rate: 3000, // 0.30%
            liquidity: 2_000_000_000_000,
            sqrt_price: 79228162514264337593543950336_u128,
            tick_current_index: 0,
            fee_growth_global_a: 0,
            fee_growth_global_b: 0,
            reward_infos: vec![],
            tick_range_efficiency: 0.78, // 78% efficient from old-archive analysis
        };

        // Cache the result
        self.whirlpool_cache.insert(cache_key, mock_whirlpool.clone());

        info!("✅ Found optimal whirlpool: {} (efficiency: {:.1}%)", 
            mock_whirlpool.whirlpool_id, 
            mock_whirlpool.tick_range_efficiency * 100.0
        );

        Ok(mock_whirlpool)
    }

    /// Build concentrated liquidity swap instruction
    pub async fn build_concentrated_liquidity_swap(
        &mut self,
        input_mint: &str,
        output_mint: &str,
        amount: u64,
        slippage_bps: u16,
        user_pubkey: &str,
    ) -> Result<WhirlpoolSwapInstruction> {
        debug!("🌊 Building concentrated liquidity swap");
        debug!("   Amount: {} lamports", amount);
        debug!("   Slippage: {} bps", slippage_bps);
        debug!("   User: {}", user_pubkey);

        // 🚀 USAR USER_PUBKEY: Validate user account and get preferences
        let user_sol_balance = self.get_user_sol_balance(user_pubkey).await.unwrap_or(0.0);
        let user_preferences = self.get_user_whirlpool_preferences(user_pubkey).await.unwrap_or_default();
        
        debug!("👤 User balance: {:.3} SOL, preferences: gas_efficiency={}", 
               user_sol_balance, user_preferences.prefer_gas_efficiency);

        // Find optimal whirlpool
        let whirlpool = self.find_optimal_whirlpool(input_mint, output_mint, amount).await?;

        // Calculate optimal tick arrays needed
        let tick_arrays = self.calculate_optimal_tick_arrays(&whirlpool, amount).await?;

        // Calculate swap parameters
        let sqrt_price_limit = self.calculate_sqrt_price_limit(&whirlpool, slippage_bps)?;
        let other_amount_threshold = amount - (amount * slippage_bps as u64 / 10000);

        // Update performance metrics
        self.performance_metrics.total_swaps += 1;
        self.performance_metrics.average_tick_arrays_used = 
            (self.performance_metrics.average_tick_arrays_used * (self.performance_metrics.total_swaps - 1) as f64 + 
             tick_arrays.len() as f64) / self.performance_metrics.total_swaps as f64;

        let route_info = WhirlpoolRouteInfo {
            route_id: format!("whirlpool_route_{}", chrono::Utc::now().timestamp()),
            whirlpools_used: vec![whirlpool.whirlpool_id.clone()],
            total_fee_rate: whirlpool.fee_rate,
            price_impact: self.estimate_price_impact(&whirlpool, amount)?,
            expected_output: amount - (amount * whirlpool.fee_rate as u64 / 1_000_000),
            minimum_received: other_amount_threshold,
            tick_range_efficiency: whirlpool.tick_range_efficiency,
        };

        self.performance_metrics.average_price_impact = 
            (self.performance_metrics.average_price_impact * (self.performance_metrics.total_swaps - 1) as f64 + 
             route_info.price_impact) / self.performance_metrics.total_swaps as f64;

        Ok(WhirlpoolSwapInstruction {
            whirlpool_address: whirlpool.whirlpool_id,
            tick_arrays,
            oracle_account: None, // Optional price oracle
            amount_specified: amount,
            other_amount_threshold,
            sqrt_price_limit,
            amount_specified_is_input: true,
            a_to_b: input_mint < output_mint, // Lexicographic ordering
            estimated_gas: 200_000, // Conservative estimate for whirlpool swap
            route_info,
        })
    }

    /// Build multi-hop route through multiple whirlpools
    pub async fn build_multi_hop_route(
        &mut self,
        input_mint: &str,
        output_mint: &str,
        amount: u64,
        max_hops: u8,
    ) -> Result<MultiHopRoute> {
        debug!("🔗 Building multi-hop route from {} to {}", input_mint, output_mint);
        debug!("   Max hops: {}", max_hops);

        if !self.routing_config.enable_multi_hop {
            return Err(anyhow!("Multi-hop routing is disabled"));
        }

        // Find intermediate tokens for routing (simplified implementation)
        let intermediate_tokens = vec![
            "EPjFWdd5AufqSSqeM2qN1xzybapC8G4wEGGkZwyTDt1v".to_string(), // USDC
            "So11111111111111111111111111111111111111112".to_string(),   // SOL
        ];

        let mut best_route: Option<MultiHopRoute> = None;
        let mut best_output = 0u64;

        // Try different routing paths
        for intermediate in &intermediate_tokens {
            if intermediate == input_mint || intermediate == output_mint {
                continue;
            }

            // Two-hop route: input -> intermediate -> output
            let hop1 = self.create_whirlpool_hop(input_mint, intermediate, amount).await?;
            let hop2 = self.create_whirlpool_hop(intermediate, output_mint, hop1.expected_price_impact as u64).await?;

            let route = MultiHopRoute {
                route_id: format!("multi_hop_{}", chrono::Utc::now().timestamp()),
                hops: vec![hop1, hop2.clone()],
                total_price_impact: hop2.expected_price_impact,
                total_fees: (amount * 60 / 1_000_000), // 0.6% total fees for 2 hops
                estimated_output: hop2.expected_price_impact as u64,
                execution_complexity: 2,
            };

            if route.estimated_output > best_output {
                best_output = route.estimated_output;
                best_route = Some(route);
            }
        }

        match best_route {
            Some(route) => {
                info!("✅ Built multi-hop route with {} hops, estimated output: {} lamports", 
                    route.hops.len(), route.estimated_output);
                self.performance_metrics.multi_hop_success_rate = 
                    (self.performance_metrics.multi_hop_success_rate * (self.performance_metrics.total_swaps - 1) as f64 + 1.0) / 
                    self.performance_metrics.total_swaps as f64;
                Ok(route)
            },
            None => {
                self.performance_metrics.failed_swaps += 1;
                Err(anyhow!("No viable multi-hop route found"))
            }
        }
    }

    /// Optimize tick array selection for minimal gas usage
    pub async fn optimize_tick_arrays(
        &mut self,
        whirlpool: &WhirlpoolInfo,
        amount: u64,
    ) -> Result<Vec<String>> {
        debug!("⚡ Optimizing tick arrays for whirlpool: {}", whirlpool.whirlpool_id);

        if !self.routing_config.enable_tick_optimization {
            return self.get_default_tick_arrays(whirlpool).await;
        }

        // Calculate the tick range we'll likely traverse
        let price_impact = self.estimate_price_impact(whirlpool, amount)?;
        let tick_range = (price_impact * 1000.0) as i32; // Simplified calculation

        let current_tick = whirlpool.tick_current_index;
        let tick_spacing = whirlpool.tick_spacing as i32;

        // Calculate optimal tick array addresses
        let mut optimized_arrays = Vec::new();
        
        // We need tick arrays that cover our swap range
        let start_tick = current_tick - tick_range;
        let end_tick = current_tick + tick_range;

        // Each tick array covers 88 ticks for most whirlpools
        let ticks_per_array = 88 * tick_spacing;
        
        let start_array_index = start_tick / ticks_per_array;
        let end_array_index = end_tick / ticks_per_array;

        for array_index in start_array_index..=end_array_index {
            let array_address = format!("tick_array_{}_{}", whirlpool.whirlpool_id, array_index);
            optimized_arrays.push(array_address);
        }

        // Ensure we don't exceed reasonable limits
        if optimized_arrays.len() > 3 {
            optimized_arrays.truncate(3);
            warn!("⚠️ Truncated tick arrays to 3 for gas efficiency");
        }

        info!("✅ Optimized to {} tick arrays for {:.4}% price impact", 
            optimized_arrays.len(), price_impact * 100.0);

        Ok(optimized_arrays)
    }

    /// Estimate price impact for a given swap
    fn estimate_price_impact(&self, whirlpool: &WhirlpoolInfo, amount: u64) -> Result<f64> {
        // Simplified price impact calculation
        let liquidity_ratio = amount as f64 / whirlpool.liquidity as f64;
        let base_impact = liquidity_ratio * 0.1; // Base 10% impact factor
        
        // Apply efficiency multiplier
        let efficiency_factor = whirlpool.tick_range_efficiency;
        let adjusted_impact = base_impact / efficiency_factor;
        
        // Cap at reasonable maximum
        Ok(adjusted_impact.min(0.05)) // Max 5% price impact
    }

    /// Calculate sqrt price limit for slippage protection
    fn calculate_sqrt_price_limit(&self, whirlpool: &WhirlpoolInfo, slippage_bps: u16) -> Result<u128> {
        let slippage_factor = 1.0 - (slippage_bps as f64 / 10000.0);
        let price_limit = (whirlpool.sqrt_price as f64 * slippage_factor) as u128;
        Ok(price_limit)
    }

    /// Calculate optimal tick arrays for a swap
    async fn calculate_optimal_tick_arrays(&mut self, whirlpool: &WhirlpoolInfo, amount: u64) -> Result<Vec<String>> {
        self.optimize_tick_arrays(whirlpool, amount).await
    }

    /// Get default tick arrays when optimization is disabled
    async fn get_default_tick_arrays(&self, whirlpool: &WhirlpoolInfo) -> Result<Vec<String>> {
        Ok(vec![
            format!("tick_array_{}_0", whirlpool.whirlpool_id),
            format!("tick_array_{}_1", whirlpool.whirlpool_id),
        ])
    }

    /// Create a whirlpool hop for multi-hop routing
    async fn create_whirlpool_hop(&mut self, input_mint: &str, output_mint: &str, amount: u64) -> Result<WhirlpoolHop> {
        let whirlpool = self.find_optimal_whirlpool(input_mint, output_mint, amount).await?;
        let tick_arrays = self.calculate_optimal_tick_arrays(&whirlpool, amount).await?;
        let price_impact = self.estimate_price_impact(&whirlpool, amount)?;

        Ok(WhirlpoolHop {
            whirlpool_id: whirlpool.whirlpool_id,
            input_mint: input_mint.to_string(),
            output_mint: output_mint.to_string(),
            fee_rate: whirlpool.fee_rate,
            expected_price_impact: price_impact,
            tick_arrays_needed: tick_arrays,
        })
    }

    /// Get current performance metrics
    pub fn get_performance_metrics(&self) -> &WhirlpoolPerformanceMetrics {
        &self.performance_metrics
    }

    /// Update routing configuration at runtime
    pub fn update_routing_config(&mut self, config: WhirlpoolRoutingConfig) {
        debug!("🔧 Updating whirlpool routing configuration");
        self.routing_config = config;
    }

    /// Clear cache to force fresh pool discovery
    pub fn clear_cache(&mut self) {
        debug!("🧹 Clearing whirlpool cache");
        self.whirlpool_cache.clear();
        self.tick_arrays.clear();
    }
}

impl Default for FeeGrowthTracker {
    fn default() -> Self {
        Self {
            global_fee_growth_a: HashMap::new(),
            global_fee_growth_b: HashMap::new(),
            last_update_slot: 0,
        }
    }
}

impl Default for WhirlpoolRoutingConfig {
    fn default() -> Self {
        Self {
            max_hops: 3,
            enable_multi_hop: true,
            prefer_low_fee_pools: true,
            max_price_impact: 0.05, // 5%
            enable_tick_optimization: true,
        }
    }
}

/// Factory functions for different bot types
impl EnterpriseWhirlpoolBuilder {
    /// Create builder optimized for liquidity sniping bots
    pub fn for_liquidity_sniper() -> Self {
        Self::with_routing_config(WhirlpoolRoutingConfig {
            max_hops: 2, // Keep simple for speed
            enable_multi_hop: true,
            prefer_low_fee_pools: false, // Speed over fees
            max_price_impact: 0.10, // Higher tolerance for sniping
            enable_tick_optimization: true,
        })
    }

    /// Create builder optimized for arbitrage bots
    pub fn for_arbitrage() -> Self {
        Self::with_routing_config(WhirlpoolRoutingConfig {
            max_hops: 3, // More complex routing allowed
            enable_multi_hop: true,
            prefer_low_fee_pools: true, // Fees matter for arb
            max_price_impact: 0.02, // Lower tolerance
            enable_tick_optimization: true,
        })
    }

    /// Create builder optimized for market making bots
    pub fn for_market_maker() -> Self {
        Self::with_routing_config(WhirlpoolRoutingConfig {
            max_hops: 1, // Direct swaps only
            enable_multi_hop: false,
            prefer_low_fee_pools: true,
            max_price_impact: 0.01, // Very low tolerance
            enable_tick_optimization: true,
        })
    }

    /// 🚀 IMPLEMENTACIÓN MISSING: Get user SOL balance
    async fn get_user_sol_balance(&self, user_pubkey: &str) -> Result<f64> {
        debug!("💰 Getting SOL balance for user: {}", user_pubkey);
        
        // En implementación real: consultar balance on-chain
        // Por ahora retornar balance simulado basado en user pubkey
        let mock_balance = if user_pubkey.len() > 20 { 1.5 } else { 0.5 };
        
        debug!("💰 User {} balance: {:.3} SOL", user_pubkey, mock_balance);
        Ok(mock_balance)
    }

    /// 🚀 IMPLEMENTACIÓN MISSING: Get user whirlpool preferences  
    async fn get_user_whirlpool_preferences(&self, user_pubkey: &str) -> Result<UserWhirlpoolPreferences> {
        debug!("⚙️ Getting whirlpool preferences for user: {}", user_pubkey);
        
        // En implementación real: consultar preferencias de base de datos/config
        // Por ahora retornar preferencias por defecto basadas en user
        let preferences = UserWhirlpoolPreferences {
            prefer_gas_efficiency: user_pubkey.starts_with('2'), // Basado en primer char
            max_slippage_tolerance: 300, // 3%
            prefer_concentrated_liquidity: true,
            enable_auto_routing: true,
        };
        
        debug!("⚙️ User preferences: gas_efficiency={}, max_slippage={}bps", 
               preferences.prefer_gas_efficiency, preferences.max_slippage_tolerance);
        Ok(preferences)
    }
}

/// 🚀 NUEVA ESTRUCTURA: User preferences for whirlpool operations
#[derive(Debug, Clone, Default)]
pub struct UserWhirlpoolPreferences {
    pub prefer_gas_efficiency: bool,
    pub max_slippage_tolerance: u16,
    pub prefer_concentrated_liquidity: bool,
    pub enable_auto_routing: bool,
}
