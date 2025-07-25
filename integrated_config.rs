// ===== CONFIGURATION FOR COMPLETE INTEGRATED SYSTEM =====
// All phases configuration in one place - 100% real parameters

use std::time::Duration;
use serde::{Deserialize, Serialize};
use solana_sdk::pubkey::Pubkey;

// ===== MAIN ARBITRAGE CONFIGURATION =====
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ArbitrageConfig {
    pub rpc_url: String,
    pub max_slippage_bps: u64,
    pub min_profit_threshold: u64,
    pub max_trade_amount: u64,
    pub pool_refresh_interval: Duration,
    pub price_update_interval: Duration,
    pub target_tokens: Vec<String>,
}

impl Default for ArbitrageConfig {
    fn default() -> Self {
        Self {
            rpc_url: "https://api.mainnet-beta.solana.com".to_string(),
            max_slippage_bps: 100, // 1.0%
            min_profit_threshold: 5, // 0.05%
            max_trade_amount: 10_000_000_000, // 10 SOL
            pool_refresh_interval: Duration::from_secs(30),
            price_update_interval: Duration::from_secs(5),
            target_tokens: vec![
                "EPjFWdd5AufqSSqeM2qN1xzybapC8G4wEGGkZwyTDt1v".to_string(), // USDC
                "Es9vMFrzaCERmJfrF4H2FYD4KCoNkY11McCe8BenwNYB".to_string(), // USDT
                "4k3Dyjzvzp8eMZWUXbBCjEvwSkkk59S5iCNLY3QrkX6R".to_string(), // RAY
                "DezXAZ8z7PnrnRJjz3wXBoRgixCa6xjnB7YaB1pPB263".to_string(), // BONK
            ],
        }
    }
}

// ===== PERFORMANCE METRICS =====
#[derive(Debug, Clone, Default)]
pub struct PerformanceMetrics {
    pub total_trades: u64,
    pub successful_trades: u64,
    pub total_profit_usd: f64,
    pub average_execution_time_ms: f64,
    pub best_profit_opportunity: f64,
    pub hourly_pnl: std::collections::VecDeque<f64>,
}

// ===== ADAPTIVE CONFIGURATION =====
#[derive(Debug, Clone)]
pub struct AdaptiveConfig {
    pub max_slippage_bps: u64,
    pub min_profit_threshold: u64,
    pub max_trade_amount: u64,
    pub risk_multiplier: f64,
    pub volatility_adjustment: f64,
    pub latency_compensation: f64,
}

impl Default for AdaptiveConfig {
    fn default() -> Self {
        Self {
            max_slippage_bps: 100,
            min_profit_threshold: 5,
            max_trade_amount: 10_000_000_000,
            risk_multiplier: 1.0,
            volatility_adjustment: 1.0,
            latency_compensation: 1.0,
        }
    }
}

// ===== EXECUTION MODE =====
#[derive(Debug, Clone, PartialEq)]
pub enum ExecutionMode {
    Simulation,
    RealMoney,
    DryRun,
}

// ===== PHASE-SPECIFIC CONFIGURATIONS =====

// Phase 1: Jupiter Advanced Config
#[derive(Debug, Clone)]
pub struct JupiterAdvancedConfig {
    pub api_endpoint: String,
    pub swap_endpoint: String,
    pub max_accounts: u8,
    pub restrict_intermediate_tokens: bool,
    pub intermediate_tokens: Vec<Pubkey>,
    pub dynamic_slippage: bool,
    pub min_profit_threshold_bps: u64,
    pub max_route_complexity: usize,
    pub timeout_seconds: u64,
}

impl Default for JupiterAdvancedConfig {
    fn default() -> Self {
        Self {
            api_endpoint: "https://quote-api.jup.ag/v6".to_string(),
            swap_endpoint: "https://quote-api.jup.ag/v6/swap".to_string(),
            max_accounts: 16,
            restrict_intermediate_tokens: true,
            intermediate_tokens: vec![
                Pubkey::try_from("EPjFWdd5AufqSSqeM2qN1xzybapC8G4wEGGkZwyTDt1v").unwrap(), // USDC
                Pubkey::try_from("Es9vMFrzaCERmJfrF4H2FYD4KCoNkY11McCe8BenwNYB").unwrap(), // USDT
                Pubkey::try_from("4k3Dyjzvzp8eMZWUXbBCjEvwSkkk59S5iCNLY3QrkX6R").unwrap(), // RAY
            ],
            dynamic_slippage: true,
            min_profit_threshold_bps: 50,
            max_route_complexity: 4,
            timeout_seconds: 15,
        }
    }
}

// Phase 2: MEV Protection Config
#[derive(Debug, Clone)]
pub struct MEVProtectionConfig {
    pub jito_url: String,
    pub max_priority_fee: u64,
    pub min_bundle_tip: u64,
    pub max_bundle_wait_ms: u64,
    pub enable_sandwich_detection: bool,
    pub enable_frontrun_protection: bool,
    pub max_bundle_retries: u32,
    pub congestion_multiplier: f64,
}

impl Default for MEVProtectionConfig {
    fn default() -> Self {
        Self {
            jito_url: "https://mainnet.block-engine.jito.wtf".to_string(),
            max_priority_fee: 1_000_000, // 0.001 SOL
            min_bundle_tip: 100_000,     // 0.0001 SOL
            max_bundle_wait_ms: 30_000,  // 30 seconds
            enable_sandwich_detection: true,
            enable_frontrun_protection: true,
            max_bundle_retries: 3,
            congestion_multiplier: 2.0,
        }
    }
}

// Phase 3: DEX Specialization Config
#[derive(Debug, Clone)]
pub struct DEXSpecializationConfig {
    pub enable_raydium_clmm: bool,
    pub enable_orca_whirlpool: bool,
    pub enable_phoenix_orderbook: bool,
    pub enable_meteora_vaults: bool,
    pub enable_cross_dex_arbitrage: bool,
    pub raydium_clmm_pools: Vec<Pubkey>,
    pub orca_whirlpool_pools: Vec<Pubkey>,
    pub phoenix_markets: Vec<Pubkey>,
    pub meteora_vaults: Vec<Pubkey>,
    pub max_specialized_opportunities: usize,
    pub specialization_timeout_ms: u64,
}

impl Default for DEXSpecializationConfig {
    fn default() -> Self {
        Self {
            enable_raydium_clmm: true,
            enable_orca_whirlpool: true,
            enable_phoenix_orderbook: true,
            enable_meteora_vaults: true,
            enable_cross_dex_arbitrage: true,
            raydium_clmm_pools: vec![],
            orca_whirlpool_pools: vec![],
            phoenix_markets: vec![],
            meteora_vaults: vec![],
            max_specialized_opportunities: 20,
            specialization_timeout_ms: 1000,
        }
    }
}

// Phase 4: Event-Driven Config
#[derive(Debug, Clone)]
pub struct EventDrivenConfig {
    pub max_concurrent_events: usize,
    pub event_buffer_size: usize,
    pub price_update_threshold: f64,
    pub liquidity_update_threshold: f64,
    pub opportunity_evaluation_timeout_ms: u64,
    pub max_event_processing_time_ms: u64,
}

impl Default for EventDrivenConfig {
    fn default() -> Self {
        Self {
            max_concurrent_events: 100,
            event_buffer_size: 1000,
            price_update_threshold: 0.001, // 0.1%
            liquidity_update_threshold: 0.05, // 5%
            opportunity_evaluation_timeout_ms: 500,
            max_event_processing_time_ms: 100,
        }
    }
}

// Phase 4: Parallel Execution Config
#[derive(Debug, Clone)]
pub struct ParallelExecutionConfig {
    pub max_concurrent_opportunities: usize,
    pub max_concurrent_executions: usize,
    pub opportunity_timeout_seconds: u64,
    pub execution_timeout_seconds: u64,
    pub max_queue_size: usize,
    pub enable_execution_prioritization: bool,
    pub min_profit_threshold_lamports: u64,
}

impl Default for ParallelExecutionConfig {
    fn default() -> Self {
        Self {
            max_concurrent_opportunities: 10,
            max_concurrent_executions: 3,
            opportunity_timeout_seconds: 30,
            execution_timeout_seconds: 60,
            max_queue_size: 50,
            enable_execution_prioritization: true,
            min_profit_threshold_lamports: 1_000_000, // 0.001 SOL
        }
    }
}

// Phase 4: Monitoring Config
#[derive(Debug, Clone)]
pub struct MonitoringConfig {
    pub metrics_update_interval_seconds: u64,
    pub dashboard_port: u16,
    pub enable_web_dashboard: bool,
    pub enable_prometheus_metrics: bool,
    pub log_level: String,
    pub alert_thresholds: AlertThresholds,
}

#[derive(Debug, Clone, Default)]
pub struct AlertThresholds {
    pub max_failed_executions_per_hour: u64,
    pub min_success_rate_percentage: f64,
    pub max_execution_time_ms: u64,
    pub min_profit_per_hour_usd: f64,
}

impl Default for MonitoringConfig {
    fn default() -> Self {
        Self {
            metrics_update_interval_seconds: 5,
            dashboard_port: 8080,
            enable_web_dashboard: true,
            enable_prometheus_metrics: false,
            log_level: "info".to_string(),
            alert_thresholds: AlertThresholds {
                max_failed_executions_per_hour: 10,
                min_success_rate_percentage: 70.0,
                max_execution_time_ms: 5000,
                min_profit_per_hour_usd: 10.0,
            },
        }
    }
}

// Phase 4: Benchmark Config
#[derive(Debug, Clone)]
pub struct BenchmarkConfig {
    pub benchmark_interval_minutes: u64,
    pub historical_data_retention_hours: u64,
    pub performance_targets: PerformanceTargets,
    pub enable_continuous_benchmarking: bool,
}

#[derive(Debug, Clone, Default)]
pub struct PerformanceTargets {
    pub target_opportunities_per_hour: u64,
    pub target_success_rate_percentage: f64,
    pub target_average_profit_usd: f64,
    pub target_execution_time_ms: u64,
}

impl Default for BenchmarkConfig {
    fn default() -> Self {
        Self {
            benchmark_interval_minutes: 5,
            historical_data_retention_hours: 24,
            performance_targets: PerformanceTargets {
                target_opportunities_per_hour: 20,
                target_success_rate_percentage: 80.0,
                target_average_profit_usd: 5.0,
                target_execution_time_ms: 2000,
            },
            enable_continuous_benchmarking: true,
        }
    }
}

// Phase 4: Integrated System Config
#[derive(Debug, Clone)]
pub struct IntegratedSystemConfig {
    pub enable_all_phases: bool,
    pub max_daily_trades: u64,
    pub max_daily_volume_sol: f64,
    pub risk_management_level: String,
    pub auto_rebalance_enabled: bool,
    pub emergency_stop_loss_percentage: f64,
}

impl Default for IntegratedSystemConfig {
    fn default() -> Self {
        Self {
            enable_all_phases: true,
            max_daily_trades: 100,
            max_daily_volume_sol: 1000.0,
            risk_management_level: "conservative".to_string(),
            auto_rebalance_enabled: true,
            emergency_stop_loss_percentage: 5.0,
        }
    }
}

// ===== HELPER FUNCTIONS =====

/// Load configuration from environment variables
pub fn load_config_from_env() -> ArbitrageConfig {
    ArbitrageConfig {
        rpc_url: std::env::var("SOLANA_RPC_URL")
            .unwrap_or_else(|_| "https://api.mainnet-beta.solana.com".to_string()),
        max_slippage_bps: std::env::var("MAX_SLIPPAGE_BPS")
            .unwrap_or_else(|_| "100".to_string())
            .parse()
            .unwrap_or(100),
        min_profit_threshold: std::env::var("MIN_PROFIT_THRESHOLD")
            .unwrap_or_else(|_| "5".to_string())
            .parse()
            .unwrap_or(5),
        max_trade_amount: std::env::var("MAX_TRADE_AMOUNT")
            .unwrap_or_else(|_| "10000000000".to_string())
            .parse()
            .unwrap_or(10_000_000_000),
        ..Default::default()
    }
}

/// Validate configuration parameters
pub fn validate_config(config: &ArbitrageConfig) -> Result<(), String> {
    if config.max_slippage_bps > 1000 {
        return Err("Max slippage too high (>10%)".to_string());
    }
    
    if config.min_profit_threshold < 1 {
        return Err("Min profit threshold too low (<0.01%)".to_string());
    }
    
    if config.max_trade_amount > 100_000_000_000 {
        return Err("Max trade amount too high (>100 SOL)".to_string());
    }
    
    Ok(())
}
