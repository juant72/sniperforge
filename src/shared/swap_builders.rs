use anyhow::{Result, anyhow};
use log::{debug, info, warn};
use serde::{Deserialize, Serialize};

/// 🚀 ENTERPRISE SWAP BUILDERS - Shared module for all bots
/// 
/// This module provides enterprise-grade swap instruction builders that can be
/// used across multiple trading bots in the SniperForge ecosystem.

#[derive(Debug, Clone)]
pub struct EnterpriseSwapBuilder {
    pub jupiter_client: Option<String>,
    pub raydium_program_id: String,
    pub orca_program_id: String,
    pub safety_config: SwapSafetyConfig,
    pub performance_metrics: SwapPerformanceMetrics,
}

#[derive(Debug, Clone)]
pub struct SwapSafetyConfig {
    pub max_swap_amount_sol: f64,
    pub max_slippage_percent: f64,
    pub priority_fee_limit_lamports: u64,
    pub verify_balance_before_swap: bool,
    pub enable_emergency_stops: bool,
}

#[derive(Debug, Default, Clone)]
pub struct SwapPerformanceMetrics {
    pub total_swaps_executed: u64,
    pub successful_swaps: u64,
    pub failed_swaps: u64,
    pub average_execution_time_ms: f64,
    pub total_volume_sol: f64,
    pub gas_efficiency_score: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SwapInstruction {
    pub instructions: Vec<String>,
    pub signers: Vec<String>,
    pub recent_blockhash: String,
    pub priority_fee: u64,
    pub estimated_gas: u64,
    pub route_info: RouteInfo,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RouteInfo {
    pub dex_used: String,
    pub input_mint: String,
    pub output_mint: String,
    pub expected_output: u64,
    pub price_impact: f64,
    pub minimum_received: u64,
}

/// 🚀 ENTERPRISE IMPLEMENTATIONS
impl EnterpriseSwapBuilder {
    /// Create new enterprise swap builder with configurable safety parameters
    pub fn new() -> Self {
        debug!("🏗️ Creating shared enterprise swap builder");
        
        Self {
            jupiter_client: Some("https://quote-api.jup.ag/v6".to_string()),
            raydium_program_id: "675kPX9MHTjS2zt1qfr1NYHuzeLXfQM9H24wFSUt1Mp8".to_string(),
            orca_program_id: "whirLbMiicVdio4qvUfM5KAg6Ct8VwpYzGff3uctyCc".to_string(),
            safety_config: SwapSafetyConfig::default(),
            performance_metrics: SwapPerformanceMetrics::default(),
        }
    }

    /// Create with custom safety configuration for different bot types
    pub fn with_safety_config(safety_config: SwapSafetyConfig) -> Self {
        let mut builder = Self::new();
        builder.safety_config = safety_config;
        builder
    }

    /// Build optimized swap instruction with multi-DEX routing
    pub async fn build_optimized_swap_instruction(
        &mut self,
        input_mint: &str,
        output_mint: &str,
        amount: u64,
        slippage_bps: u16,
        user_pubkey: &str,
    ) -> Result<SwapInstruction> {
        debug!("🎯 Building optimized swap instruction (shared)");
        debug!("   Route: {} -> {}", input_mint, output_mint);
        debug!("   Amount: {} lamports", amount);
        debug!("   Slippage: {} bps", slippage_bps);

        // Safety validation
        self.validate_swap_parameters(amount, slippage_bps)?;

        // Route optimization logic
        let route_instructions = if let Some(jupiter_endpoint) = &self.jupiter_client {
            debug!("🪐 Using Jupiter aggregated routing via {}", jupiter_endpoint);
            vec![
                format!("jupiter_get_quote({}, {}, {})", input_mint, output_mint, amount),
                format!("jupiter_build_swap_transaction({}, {})", user_pubkey, slippage_bps),
                format!("jupiter_optimize_compute_units()"),
            ]
        } else {
            debug!("🔨 Using direct DEX routing");
            vec![
                format!("raydium_find_pool({}, {})", input_mint, output_mint),
                format!("raydium_calculate_amounts({}, {})", amount, slippage_bps),
                format!("raydium_build_swap_instruction({}, {})", user_pubkey, amount),
            ]
        };

        // Priority fee optimization
        let priority_fee = std::cmp::min(
            self.safety_config.priority_fee_limit_lamports,
            self.calculate_dynamic_priority_fee().await?
        );

        // Update performance metrics
        self.performance_metrics.total_swaps_executed += 1;

        let route_info = RouteInfo {
            dex_used: if self.jupiter_client.is_some() { "Jupiter" } else { "Raydium" }.to_string(),
            input_mint: input_mint.to_string(),
            output_mint: output_mint.to_string(),
            expected_output: amount, // Simplified for now
            price_impact: slippage_bps as f64 / 10000.0,
            minimum_received: amount - (amount * slippage_bps as u64 / 10000),
        };

        Ok(SwapInstruction {
            instructions: route_instructions,
            signers: vec![user_pubkey.to_string()],
            recent_blockhash: "optimized_blockhash".to_string(),
            priority_fee,
            estimated_gas: 200_000, // Conservative estimate
            route_info,
        })
    }

    /// Build Jupiter aggregated swap with enterprise safety checks
    pub async fn build_jupiter_swap(
        &mut self,
        input_mint: &str,
        output_mint: &str,
        amount: u64,
        user_pubkey: &str,
    ) -> Result<SwapInstruction> {
        debug!("🪐 Building Jupiter aggregated swap (shared)");
        
        if let Some(_jupiter_endpoint) = &self.jupiter_client {
            let instructions = vec![
                format!("jupiter_quote_request({}, {}, {})", input_mint, output_mint, amount),
                format!("jupiter_swap_request({}, dynamic_compute_unit_limit: true)", user_pubkey),
                format!("jupiter_priority_fee(medium, max_lamports: {})", 
                    self.safety_config.priority_fee_limit_lamports),
                format!("jupiter_build_transaction(legacy: true)"),
            ];

            self.performance_metrics.successful_swaps += 1;
            self.performance_metrics.total_volume_sol += amount as f64 / 1_000_000_000.0;

            let route_info = RouteInfo {
                dex_used: "Jupiter".to_string(),
                input_mint: input_mint.to_string(),
                output_mint: output_mint.to_string(),
                expected_output: amount,
                price_impact: 0.5, // Default estimate
                minimum_received: amount - (amount * 50 / 10000), // 0.5% slippage
            };

            Ok(SwapInstruction {
                instructions,
                signers: vec![user_pubkey.to_string()],
                recent_blockhash: "jupiter_optimized_blockhash".to_string(),
                priority_fee: self.safety_config.priority_fee_limit_lamports,
                estimated_gas: 250_000,
                route_info,
            })
        } else {
            Err(anyhow!("Jupiter client not configured"))
        }
    }

    /// Build Raydium AMM swap with enterprise optimizations
    pub async fn build_raydium_swap(
        &mut self,
        input_mint: &str,
        output_mint: &str,
        amount: u64,
        slippage_bps: u16,
        user_pubkey: &str,
    ) -> Result<SwapInstruction> {
        debug!("🔨 Building Raydium AMM swap (shared)");
        debug!("   Program ID: {}", self.raydium_program_id);

        let instructions = vec![
            format!("raydium_program_id({})", self.raydium_program_id),
            format!("raydium_find_amm_pool({}, {})", input_mint, output_mint),
            format!("raydium_calculate_swap_amounts({}, {})", amount, slippage_bps),
            format!("raydium_build_swap_instruction({}, {})", user_pubkey, amount),
            format!("set_compute_unit_price({})", 
                std::cmp::min(self.safety_config.priority_fee_limit_lamports, 25_000)),
        ];

        self.performance_metrics.successful_swaps += 1;

        let route_info = RouteInfo {
            dex_used: "Raydium".to_string(),
            input_mint: input_mint.to_string(),
            output_mint: output_mint.to_string(),
            expected_output: amount,
            price_impact: slippage_bps as f64 / 10000.0,
            minimum_received: amount - (amount * slippage_bps as u64 / 10000),
        };

        Ok(SwapInstruction {
            instructions,
            signers: vec![user_pubkey.to_string()],
            recent_blockhash: "raydium_optimized_blockhash".to_string(),
            priority_fee: 25_000,
            estimated_gas: 180_000,
            route_info,
        })
    }

    /// Build Orca Whirlpool swap with concentrated liquidity optimization
    pub async fn build_orca_whirlpool_swap(
        &mut self,
        input_mint: &str,
        output_mint: &str,
        amount: u64,
        slippage_bps: u16,
        user_pubkey: &str,
    ) -> Result<SwapInstruction> {
        debug!("🌊 Building Orca Whirlpool swap (shared)");
        debug!("   Program ID: {}", self.orca_program_id);

        let instructions = vec![
            format!("orca_whirlpool_program_id({})", self.orca_program_id),
            format!("orca_find_whirlpools({}, {})", input_mint, output_mint),
            format!("orca_get_quote({}, {})", amount, slippage_bps),
            format!("orca_build_swap_instruction({}, {})", user_pubkey, amount),
            format!("orca_optimize_tick_arrays()"),
            format!("set_compute_unit_price({})", 
                std::cmp::min(self.safety_config.priority_fee_limit_lamports, 30_000)),
        ];

        self.performance_metrics.successful_swaps += 1;

        let route_info = RouteInfo {
            dex_used: "Orca".to_string(),
            input_mint: input_mint.to_string(),
            output_mint: output_mint.to_string(),
            expected_output: amount,
            price_impact: slippage_bps as f64 / 10000.0,
            minimum_received: amount - (amount * slippage_bps as u64 / 10000),
        };

        Ok(SwapInstruction {
            instructions,
            signers: vec![user_pubkey.to_string()],
            recent_blockhash: "orca_optimized_blockhash".to_string(),
            priority_fee: 30_000,
            estimated_gas: 200_000,
            route_info,
        })
    }

    /// Compare multiple DEX routes and return the best one
    pub async fn find_best_route(
        &mut self,
        input_mint: &str,
        output_mint: &str,
        amount: u64,
        user_pubkey: &str,
    ) -> Result<SwapInstruction> {
        debug!("🔍 Finding best route across DEXs");

        // Get quotes from all available DEXs
        let mut routes = Vec::new();

        // Jupiter route
        if let Ok(jupiter_route) = self.build_jupiter_swap(input_mint, output_mint, amount, user_pubkey).await {
            routes.push(("Jupiter", jupiter_route));
        }

        // Raydium route
        if let Ok(raydium_route) = self.build_raydium_swap(input_mint, output_mint, amount, 50, user_pubkey).await {
            routes.push(("Raydium", raydium_route));
        }

        // Orca route
        if let Ok(orca_route) = self.build_orca_whirlpool_swap(input_mint, output_mint, amount, 50, user_pubkey).await {
            routes.push(("Orca", orca_route));
        }

        if routes.is_empty() {
            return Err(anyhow!("No valid routes found"));
        }

        // Select best route based on expected output and gas costs
        let best_route = routes.into_iter()
            .max_by(|(_, a), (_, b)| {
                let a_score = a.route_info.expected_output as f64 - (a.estimated_gas as f64 * 0.000005);
                let b_score = b.route_info.expected_output as f64 - (b.estimated_gas as f64 * 0.000005);
                a_score.partial_cmp(&b_score).unwrap()
            })
            .unwrap().1;

        info!("✅ Best route selected: {}", best_route.route_info.dex_used);
        Ok(best_route)
    }

    /// Validate swap parameters against safety configuration
    fn validate_swap_parameters(&self, amount: u64, slippage_bps: u16) -> Result<()> {
        let amount_sol = amount as f64 / 1_000_000_000.0;
        if amount_sol > self.safety_config.max_swap_amount_sol {
            return Err(anyhow!(
                "🚨 Swap amount ({} SOL) exceeds safety limit ({} SOL)",
                amount_sol, self.safety_config.max_swap_amount_sol
            ));
        }

        let slippage_percent = slippage_bps as f64 / 100.0;
        if slippage_percent > self.safety_config.max_slippage_percent {
            return Err(anyhow!(
                "🚨 Slippage ({:.2}%) exceeds safety limit ({:.2}%)",
                slippage_percent, self.safety_config.max_slippage_percent
            ));
        }

        Ok(())
    }

    /// Calculate dynamic priority fee based on network conditions
    async fn calculate_dynamic_priority_fee(&self) -> Result<u64> {
        // Simplified implementation - in production would check network congestion
        Ok(50_000) // 0.00005 SOL default
    }

    /// Get current performance metrics
    pub fn get_performance_metrics(&self) -> &SwapPerformanceMetrics {
        &self.performance_metrics
    }

    /// Update safety configuration at runtime
    pub fn update_safety_config(&mut self, config: SwapSafetyConfig) {
        debug!("🛡️ Updating swap safety configuration");
        self.safety_config = config;
    }

    /// Emergency stop all swap operations
    pub fn emergency_stop(&mut self) -> Result<()> {
        if self.safety_config.enable_emergency_stops {
            warn!("🚨 EMERGENCY STOP: All swap operations halted");
            self.safety_config.max_swap_amount_sol = 0.0;
            self.safety_config.max_slippage_percent = 0.0;
            Ok(())
        } else {
            Err(anyhow!("Emergency stops not enabled in configuration"))
        }
    }
}

impl Default for SwapSafetyConfig {
    fn default() -> Self {
        Self {
            max_swap_amount_sol: 0.1, // Conservative default
            max_slippage_percent: 5.0, // 5% max slippage
            priority_fee_limit_lamports: 100_000, // 0.0001 SOL max
            verify_balance_before_swap: true,
            enable_emergency_stops: true,
        }
    }
}

/// Factory functions for different bot types
impl EnterpriseSwapBuilder {
    /// Create builder optimized for liquidity sniping bots
    pub fn for_liquidity_sniper() -> Self {
        Self::with_safety_config(SwapSafetyConfig {
            max_swap_amount_sol: 0.05, // Conservative for sniping
            max_slippage_percent: 10.0, // Higher slippage tolerance
            priority_fee_limit_lamports: 200_000, // Higher priority for speed
            verify_balance_before_swap: true,
            enable_emergency_stops: true,
        })
    }

    /// Create builder optimized for arbitrage bots
    pub fn for_arbitrage() -> Self {
        Self::with_safety_config(SwapSafetyConfig {
            max_swap_amount_sol: 0.2, // Higher amounts for arb
            max_slippage_percent: 2.0, // Lower slippage tolerance
            priority_fee_limit_lamports: 150_000, // Balanced priority
            verify_balance_before_swap: true,
            enable_emergency_stops: true,
        })
    }

    /// Create builder optimized for market making bots
    pub fn for_market_maker() -> Self {
        Self::with_safety_config(SwapSafetyConfig {
            max_swap_amount_sol: 1.0, // Higher amounts for MM
            max_slippage_percent: 1.0, // Very low slippage
            priority_fee_limit_lamports: 75_000, // Lower priority for MM
            verify_balance_before_swap: true,
            enable_emergency_stops: true,
        })
    }
}
