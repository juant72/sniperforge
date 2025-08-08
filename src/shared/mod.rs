/// 🚀 SHARED MODULES - Enterprise components for all bots
/// 
/// This module contains shared enterprise-grade components that can be
/// used across multiple trading bots in the SniperForge ecosystem.

pub mod swap_builders;
pub mod whirlpool_builder;
pub mod aggregator_interface;

// Re-export key types for easy access
pub use swap_builders::{
    EnterpriseSwapBuilder, 
    SwapSafetyConfig, 
    SwapPerformanceMetrics,
    SwapInstruction,
    RouteInfo,
};

pub use whirlpool_builder::{
    EnterpriseWhirlpoolBuilder,
    WhirlpoolInfo,
    WhirlpoolRoutingConfig,
    WhirlpoolSwapInstruction,
    MultiHopRoute,
    WhirlpoolPerformanceMetrics,
};

pub use aggregator_interface::{
    EnterpriseAggregatorInterface,
    DexConfiguration,
    PriceQuote,
    AggregatedQuote,
    ArbitrageOpportunity,
    OptimizationStrategy,
    AggregatorPerformanceMetrics,
};

/// Factory trait for creating enterprise components optimized for different bot types
pub trait BotOptimized {
    /// Create instance optimized for liquidity sniping bots
    fn for_liquidity_sniper() -> Self;
    
    /// Create instance optimized for arbitrage bots  
    fn for_arbitrage() -> Self;
    
    /// Create instance optimized for market making bots
    fn for_market_maker() -> Self;
}

impl BotOptimized for EnterpriseSwapBuilder {
    fn for_liquidity_sniper() -> Self {
        Self::for_liquidity_sniper()
    }
    
    fn for_arbitrage() -> Self {
        Self::for_arbitrage()
    }
    
    fn for_market_maker() -> Self {
        Self::for_market_maker()
    }
}

impl BotOptimized for EnterpriseWhirlpoolBuilder {
    fn for_liquidity_sniper() -> Self {
        Self::for_liquidity_sniper()
    }
    
    fn for_arbitrage() -> Self {
        Self::for_arbitrage()
    }
    
    fn for_market_maker() -> Self {
        Self::for_market_maker()
    }
}

impl BotOptimized for EnterpriseAggregatorInterface {
    fn for_liquidity_sniper() -> Self {
        Self::for_liquidity_sniper()
    }
    
    fn for_arbitrage() -> Self {
        Self::for_arbitrage()
    }
    
    fn for_market_maker() -> Self {
        Self::for_market_maker()
    }
}

/// Utility functions for enterprise component initialization
pub mod utils {
    use super::*;
    use anyhow::Result;
    
    /// Create a complete enterprise trading stack for a specific bot type
    pub fn create_enterprise_stack_for_bot(bot_type: &str) -> Result<(EnterpriseSwapBuilder, EnterpriseWhirlpoolBuilder, EnterpriseAggregatorInterface)> {
        let (swap_builder, whirlpool_builder, aggregator) = match bot_type.to_lowercase().as_str() {
            "liquidity_sniper" | "sniper" => (
                EnterpriseSwapBuilder::for_liquidity_sniper(),
                EnterpriseWhirlpoolBuilder::for_liquidity_sniper(), 
                EnterpriseAggregatorInterface::for_liquidity_sniper(),
            ),
            "arbitrage" | "arb" => (
                EnterpriseSwapBuilder::for_arbitrage(),
                EnterpriseWhirlpoolBuilder::for_arbitrage(),
                EnterpriseAggregatorInterface::for_arbitrage(),
            ),
            "market_maker" | "mm" => (
                EnterpriseSwapBuilder::for_market_maker(),
                EnterpriseWhirlpoolBuilder::for_market_maker(),
                EnterpriseAggregatorInterface::for_market_maker(),
            ),
            _ => {
                return Err(anyhow::anyhow!("Unknown bot type: {}. Supported: liquidity_sniper, arbitrage, market_maker", bot_type));
            }
        };
        
        Ok((swap_builder, whirlpool_builder, aggregator))
    }
    
    /// Get recommended configuration for a bot type
    pub fn get_bot_recommendations(bot_type: &str) -> Result<BotRecommendations> {
        match bot_type.to_lowercase().as_str() {
            "liquidity_sniper" | "sniper" => Ok(BotRecommendations {
                max_swap_amount_sol: 0.05,
                max_slippage_percent: 10.0,
                priority_fee_lamports: 200_000,
                enable_multi_hop: true,
                prefer_speed_over_cost: true,
                arbitrage_threshold_sol: 0.005,
                description: "Optimized for speed and opportunity capture".to_string(),
            }),
            "arbitrage" | "arb" => Ok(BotRecommendations {
                max_swap_amount_sol: 0.2,
                max_slippage_percent: 2.0,
                priority_fee_lamports: 150_000,
                enable_multi_hop: true,
                prefer_speed_over_cost: false,
                arbitrage_threshold_sol: 0.0005,
                description: "Optimized for profit maximization and cross-DEX opportunities".to_string(),
            }),
            "market_maker" | "mm" => Ok(BotRecommendations {
                max_swap_amount_sol: 1.0,
                max_slippage_percent: 1.0,
                priority_fee_lamports: 75_000,
                enable_multi_hop: false,
                prefer_speed_over_cost: false,
                arbitrage_threshold_sol: 0.001,
                description: "Optimized for gas efficiency and tight spreads".to_string(),
            }),
            _ => Err(anyhow::anyhow!("Unknown bot type: {}", bot_type)),
        }
    }
    
    #[derive(Debug, Clone)]
    pub struct BotRecommendations {
        pub max_swap_amount_sol: f64,
        pub max_slippage_percent: f64,
        pub priority_fee_lamports: u64,
        pub enable_multi_hop: bool,
        pub prefer_speed_over_cost: bool,
        pub arbitrage_threshold_sol: f64,
        pub description: String,
    }
}
