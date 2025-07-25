// ===== CONFIGURACIÓN UNIFICADA PHASE 4.5 =====
// Sistema de configuración centralizado para todas las fases 1-4

use std::collections::HashMap;
use solana_sdk::pubkey::Pubkey;
use serde::{Deserialize, Serialize};

/// Configuración unificada para todas las fases del sistema
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UnifiedPhase45Config {
    // ===== CONFIGURACIÓN BASE =====
    pub min_profit_bps: u64,
    pub max_slippage_bps: u64,
    pub max_trade_sol: f64,
    pub min_trade_sol: f64,
    pub api_timeout_ms: u64,
    
    // ===== PHASE 1: JUPITER ADVANCED =====
    pub jupiter_advanced_enabled: bool,
    pub jupiter_max_accounts: u8,
    pub jupiter_restrict_intermediate_tokens: bool,
    pub jupiter_intermediate_tokens: Vec<String>, // Mint addresses as strings
    pub jupiter_dynamic_slippage: bool,
    pub jupiter_max_route_complexity: usize,
    pub jupiter_timeout_seconds: u64,
    
    // ===== PHASE 2: MEV PROTECTION =====
    pub mev_protection_enabled: bool,
    // === RPC Configuration ===
    pub rpc_endpoint: String,
    pub jito_rpc_url: String,
    pub jito_tip_lamports: u64,
    pub jito_max_priority_fee: u64,
    pub jito_bundle_max_wait_ms: u64,
    pub jito_max_retries: u32,
    pub enable_sandwich_detection: bool,
    pub enable_frontrun_protection: bool,
    
    // ===== PHASE 3: DEX SPECIALIZATION =====
    pub dex_specialization_enabled: bool,
    pub enable_raydium_clmm: bool,
    pub enable_orca_whirlpools: bool,
    pub enable_phoenix_orderbook: bool,
    pub enable_meteora_vaults: bool,
    pub dex_min_liquidity_threshold: u64,
    pub dex_max_price_impact_bps: u16,
    pub dex_preferred_tick_spacings: Vec<u16>,
    
    // ===== PHASE 4A: EVENT-DRIVEN =====
    pub event_driven_enabled: bool,
    pub max_concurrent_events: usize,
    pub event_buffer_size: usize,
    pub price_update_threshold: f64, // Percentage change to trigger event
    pub liquidity_update_threshold: f64,
    pub enable_websocket_feeds: bool,
    pub websocket_reconnect_attempts: u32,
    
    // ===== PHASE 4B: PARALLEL EXECUTION =====
    pub parallel_execution_enabled: bool,
    pub max_concurrent_executions: usize,
    pub max_concurrent_per_dex: usize,
    pub max_concurrent_per_token: usize,
    pub execution_timeout_ms: u64,
    pub retry_attempts: u8,
    pub retry_delay_ms: u64,
    pub enable_resource_isolation: bool,
    pub enable_execution_batching: bool,
    pub batch_size: usize,
    pub batch_timeout_ms: u64,
    
    // ===== PHASE 4C: REAL-TIME MONITORING =====
    pub real_time_monitoring_enabled: bool,
    pub monitoring_update_interval_ms: u64,
    pub enable_performance_tracking: bool,
    pub enable_health_checks: bool,
    pub health_check_interval_ms: u64,
    pub enable_alerts: bool,
    pub alert_thresholds: MonitoringThresholds,
    
    // ===== ADVANCED FEATURES =====
    pub enable_machine_learning: bool,
    pub enable_predictive_analysis: bool,
    pub enable_risk_scoring: bool,
    pub enable_adaptive_parameters: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MonitoringThresholds {
    pub max_execution_time_ms: u64,
    pub min_success_rate_percent: f64,
    pub max_failed_executions_per_hour: u32,
    pub min_profit_per_hour_sol: f64,
    pub max_gas_cost_per_execution_sol: f64,
}

impl Default for UnifiedPhase45Config {
    fn default() -> Self {
        Self {
            // Base configuration (ultra-conservative)
            min_profit_bps: 25,           // 0.25% minimum profit
            max_slippage_bps: 15,         // 0.15% maximum slippage
            max_trade_sol: 0.005,         // 0.005 SOL max (micro-trading)
            min_trade_sol: 0.002,         // 0.002 SOL min (micro-trading)
            api_timeout_ms: 8000,
            
            // Phase 1: Jupiter Advanced
            jupiter_advanced_enabled: true,
            jupiter_max_accounts: 20,
            jupiter_restrict_intermediate_tokens: true,
            jupiter_intermediate_tokens: vec![
                "EPjFWdd5AufqSSqeM2qN1xzybapC8G4wEGGkZwyTDt1v".to_string(), // USDC
                "Es9vMFrzaCERmJfrF4H2FYD4KCoNkY11McCe8BenwNYB".to_string(), // USDT
                "4k3Dyjzvzp8eMZWUXbBCjEvwSkkk59S5iCNLY3QrkX6R".to_string(), // RAY
            ],
            jupiter_dynamic_slippage: true,
            jupiter_max_route_complexity: 4,
            jupiter_timeout_seconds: 15,
            
            // Phase 2: MEV Protection
            mev_protection_enabled: true,
            // === RPC Configuration ===
            rpc_endpoint: "https://api.mainnet-beta.solana.com".to_string(),
            jito_rpc_url: "https://mainnet.block-engine.jito.wtf".to_string(),
            jito_tip_lamports: 10_000,     // 0.00001 SOL tip (ultra-conservative)
            jito_max_priority_fee: 100_000, // 0.0001 SOL max priority fee
            jito_bundle_max_wait_ms: 30_000,
            jito_max_retries: 3,
            enable_sandwich_detection: true,
            enable_frontrun_protection: true,
            
            // Phase 3: DEX Specialization
            dex_specialization_enabled: true,
            enable_raydium_clmm: true,
            enable_orca_whirlpools: true,
            enable_phoenix_orderbook: false, // Disabled by default (complex)
            enable_meteora_vaults: false,    // Disabled by default (complex)
            dex_min_liquidity_threshold: 10_000_000_000, // 10 SOL minimum liquidity
            dex_max_price_impact_bps: 50,    // 0.5% max price impact
            dex_preferred_tick_spacings: vec![1, 4, 16, 64],
            
            // Phase 4A: Event-Driven
            event_driven_enabled: false,    // Disabled by default (requires WebSocket)
            max_concurrent_events: 100,
            event_buffer_size: 1000,
            price_update_threshold: 0.005,  // 0.5% price change trigger
            liquidity_update_threshold: 0.1, // 10% liquidity change trigger
            enable_websocket_feeds: false,  // Disabled by default (complex setup)
            websocket_reconnect_attempts: 5,
            
            // Phase 4B: Parallel Execution
            parallel_execution_enabled: false, // Disabled by default (conservative)
            max_concurrent_executions: 3,   // Very conservative
            max_concurrent_per_dex: 2,      // Very conservative
            max_concurrent_per_token: 1,    // Very conservative
            execution_timeout_ms: 10_000,
            retry_attempts: 2,
            retry_delay_ms: 1000,
            enable_resource_isolation: true,
            enable_execution_batching: false, // Disabled by default
            batch_size: 3,
            batch_timeout_ms: 2000,
            
            // Phase 4C: Real-Time Monitoring
            real_time_monitoring_enabled: true, // Always enabled
            monitoring_update_interval_ms: 5000, // 5 seconds
            enable_performance_tracking: true,
            enable_health_checks: true,
            health_check_interval_ms: 30_000, // 30 seconds
            enable_alerts: true,
            alert_thresholds: MonitoringThresholds::default(),
            
            // Advanced Features (disabled by default)
            enable_machine_learning: false,
            enable_predictive_analysis: false,
            enable_risk_scoring: true,      // Enabled by default (important for safety)
            enable_adaptive_parameters: false,
        }
    }
}

impl Default for MonitoringThresholds {
    fn default() -> Self {
        Self {
            max_execution_time_ms: 15_000,      // 15 seconds max execution
            min_success_rate_percent: 60.0,     // 60% minimum success rate
            max_failed_executions_per_hour: 20, // Max 20 failures per hour
            min_profit_per_hour_sol: 0.001,     // Minimum 0.001 SOL profit per hour
            max_gas_cost_per_execution_sol: 0.0001, // Max 0.0001 SOL gas per execution
        }
    }
}

/// Configuración de fase específica
#[derive(Debug, Clone)]
pub enum PhaseConfig {
    Conservative,  // Most conservative settings
    Balanced,      // Balanced performance vs safety
    Aggressive,    // Maximum performance (higher risk)
    Custom(UnifiedPhase45Config),
}

impl PhaseConfig {
    pub fn to_unified_config(&self) -> UnifiedPhase45Config {
        match self {
            PhaseConfig::Conservative => {
                let mut config = UnifiedPhase45Config::default();
                // Make even more conservative
                config.max_trade_sol = 0.002;
                config.min_trade_sol = 0.001;
                config.min_profit_bps = 50; // 0.5% minimum
                config.max_slippage_bps = 10; // 0.1% max slippage
                config.parallel_execution_enabled = false;
                config.event_driven_enabled = false;
                config.max_concurrent_executions = 1;
                config
            },
            PhaseConfig::Balanced => UnifiedPhase45Config::default(),
            PhaseConfig::Aggressive => {
                let mut config = UnifiedPhase45Config::default();
                // More aggressive settings
                config.max_trade_sol = 0.01;
                config.min_profit_bps = 15; // 0.15% minimum
                config.max_slippage_bps = 25; // 0.25% max slippage
                config.parallel_execution_enabled = true;
                config.event_driven_enabled = true;
                config.max_concurrent_executions = 5;
                config.enable_websocket_feeds = true;
                config
            },
            PhaseConfig::Custom(config) => config.clone(),
        }
    }
}

/// Builder para configuración fácil
#[derive(Debug)]
pub struct ConfigBuilder {
    config: UnifiedPhase45Config,
}

impl ConfigBuilder {
    pub fn new() -> Self {
        Self {
            config: UnifiedPhase45Config::default(),
        }
    }
    
    pub fn from_preset(preset: PhaseConfig) -> Self {
        Self {
            config: preset.to_unified_config(),
        }
    }
    
    // Phase 1 configuration
    pub fn enable_jupiter_advanced(mut self, enabled: bool) -> Self {
        self.config.jupiter_advanced_enabled = enabled;
        self
    }
    
    pub fn jupiter_max_accounts(mut self, max_accounts: u8) -> Self {
        self.config.jupiter_max_accounts = max_accounts;
        self
    }
    
    // Phase 2 configuration
    pub fn enable_mev_protection(mut self, enabled: bool) -> Self {
        self.config.mev_protection_enabled = enabled;
        self
    }
    
    pub fn jito_tip_lamports(mut self, tip: u64) -> Self {
        self.config.jito_tip_lamports = tip;
        self
    }
    
    pub fn jito_max_priority_fee(mut self, fee: u64) -> Self {
        self.config.jito_max_priority_fee = fee;
        self
    }
    
    // Phase 3 configuration
    pub fn enable_dex_specialization(mut self, enabled: bool) -> Self {
        self.config.dex_specialization_enabled = enabled;
        self
    }
    
    pub fn enable_raydium_clmm(mut self, enabled: bool) -> Self {
        self.config.enable_raydium_clmm = enabled;
        self
    }
    
    pub fn enable_orca_whirlpools(mut self, enabled: bool) -> Self {
        self.config.enable_orca_whirlpools = enabled;
        self
    }
    
    // Phase 4 configuration
    pub fn enable_event_driven(mut self, enabled: bool) -> Self {
        self.config.event_driven_enabled = enabled;
        self
    }
    
    pub fn enable_parallel_execution(mut self, enabled: bool) -> Self {
        self.config.parallel_execution_enabled = enabled;
        self
    }
    
    pub fn max_concurrent_executions(mut self, max: usize) -> Self {
        self.config.max_concurrent_executions = max;
        self
    }
    
    // Trading parameters
    pub fn trading_range(mut self, min_sol: f64, max_sol: f64) -> Self {
        self.config.min_trade_sol = min_sol;
        self.config.max_trade_sol = max_sol;
        self
    }
    
    pub fn profit_requirements(mut self, min_profit_bps: u64, max_slippage_bps: u64) -> Self {
        self.config.min_profit_bps = min_profit_bps;
        self.config.max_slippage_bps = max_slippage_bps;
        self
    }
    
    pub fn build(self) -> UnifiedPhase45Config {
        self.config
    }
}

/// Validator para configuración
pub struct ConfigValidator;

impl ConfigValidator {
    pub fn validate(config: &UnifiedPhase45Config) -> Result<(), String> {
        // Validate trading parameters
        if config.min_trade_sol <= 0.0 {
            return Err("min_trade_sol must be positive".to_string());
        }
        
        if config.max_trade_sol <= config.min_trade_sol {
            return Err("max_trade_sol must be greater than min_trade_sol".to_string());
        }
        
        if config.min_profit_bps == 0 {
            return Err("min_profit_bps must be positive".to_string());
        }
        
        if config.max_slippage_bps == 0 {
            return Err("max_slippage_bps must be positive".to_string());
        }
        
        // Validate Jupiter configuration
        if config.jupiter_advanced_enabled {
            if config.jupiter_max_accounts == 0 {
                return Err("jupiter_max_accounts must be positive".to_string());
            }
            
            if config.jupiter_max_route_complexity == 0 {
                return Err("jupiter_max_route_complexity must be positive".to_string());
            }
        }
        
        // Validate MEV protection configuration
        if config.mev_protection_enabled {
            if config.jito_tip_lamports == 0 {
                return Err("jito_tip_lamports must be positive".to_string());
            }
            
            if config.jito_rpc_url.is_empty() {
                return Err("jito_rpc_url cannot be empty".to_string());
            }
        }
        
        // Validate parallel execution configuration
        if config.parallel_execution_enabled {
            if config.max_concurrent_executions == 0 {
                return Err("max_concurrent_executions must be positive".to_string());
            }
            
            if config.max_concurrent_per_dex == 0 {
                return Err("max_concurrent_per_dex must be positive".to_string());
            }
            
            if config.max_concurrent_per_token == 0 {
                return Err("max_concurrent_per_token must be positive".to_string());
            }
        }
        
        // Validate monitoring configuration
        if config.real_time_monitoring_enabled {
            if config.monitoring_update_interval_ms == 0 {
                return Err("monitoring_update_interval_ms must be positive".to_string());
            }
            
            if config.alert_thresholds.min_success_rate_percent < 0.0 || 
               config.alert_thresholds.min_success_rate_percent > 100.0 {
                return Err("min_success_rate_percent must be between 0 and 100".to_string());
            }
        }
        
        Ok(())
    }
    
    pub fn validate_for_balance(config: &UnifiedPhase45Config, balance_sol: f64) -> Result<(), String> {
        // Validate that trading parameters are appropriate for the balance
        if config.max_trade_sol > balance_sol * 0.1 {
            return Err(format!(
                "max_trade_sol ({:.3}) is too high for balance ({:.3} SOL). Recommended: max {:.3} SOL",
                config.max_trade_sol, balance_sol, balance_sol * 0.1
            ));
        }
        
        if config.min_trade_sol > balance_sol * 0.05 {
            return Err(format!(
                "min_trade_sol ({:.3}) is too high for balance ({:.3} SOL). Recommended: max {:.3} SOL",
                config.min_trade_sol, balance_sol, balance_sol * 0.05
            ));
        }
        
        // Check if balance is sufficient for any meaningful trading
        if balance_sol < 0.01 {
            return Err(format!(
                "Balance ({:.6} SOL) is too low for any trading. Minimum recommended: 0.01 SOL",
                balance_sol
            ));
        }
        
        Ok(())
    }
}

/// Helper functions para configuraciones predefinidas
impl UnifiedPhase45Config {
    pub fn micro_trading() -> Self {
        ConfigBuilder::from_preset(PhaseConfig::Conservative)
            .trading_range(0.001, 0.002)
            .profit_requirements(50, 10) // 0.5% profit, 0.1% slippage
            .build()
    }
    
    pub fn safe_trading() -> Self {
        ConfigBuilder::from_preset(PhaseConfig::Balanced)
            .trading_range(0.002, 0.005)
            .profit_requirements(25, 15) // 0.25% profit, 0.15% slippage
            .build()
    }
    
    pub fn active_trading() -> Self {
        ConfigBuilder::from_preset(PhaseConfig::Aggressive)
            .trading_range(0.005, 0.02)
            .profit_requirements(15, 25) // 0.15% profit, 0.25% slippage
            .enable_parallel_execution(true)
            .max_concurrent_executions(5)
            .build()
    }
    
    pub fn professional_trading() -> Self {
        ConfigBuilder::from_preset(PhaseConfig::Aggressive)
            .trading_range(0.01, 0.1)
            .profit_requirements(10, 30) // 0.1% profit, 0.3% slippage
            .enable_event_driven(true)
            .enable_parallel_execution(true)
            .max_concurrent_executions(10)
            .build()
    }
    
    /// Configuración para TRADING REAL 100% en mainnet
    pub fn real_trading_mainnet() -> Self {
        ConfigBuilder::from_preset(PhaseConfig::Aggressive)
            .trading_range(0.01, 0.05)  // Entre 0.01 y 0.05 SOL por trade
            .profit_requirements(20, 25) // 0.20% profit mínimo, 0.25% slippage máximo
            .enable_jupiter_advanced(true)  // Jupiter avanzado para mejores rutas
            .enable_mev_protection(true)    // MEV protection obligatorio para trading real
            .enable_dex_specialization(true) // Especialización DEX para más oportunidades
            .enable_parallel_execution(false) // Deshabilitado para trading real conservador
            .max_concurrent_executions(1)   // Solo 1 trade a la vez para trading real
            .jito_tip_lamports(50000)       // 0.00005 SOL tip para Jito
            .jito_max_priority_fee(100000)  // 0.0001 SOL max priority fee
            .build()
    }
    
    /// Configuración para trading real agresivo (más riesgo, más profit)
    pub fn aggressive_real_trading() -> Self {
        ConfigBuilder::from_preset(PhaseConfig::Aggressive)
            .trading_range(0.02, 0.1)   // Entre 0.02 y 0.1 SOL por trade
            .profit_requirements(15, 30) // 0.15% profit mínimo, 0.30% slippage máximo
            .enable_jupiter_advanced(true)
            .enable_mev_protection(true)
            .enable_dex_specialization(true)
            .enable_event_driven(true)      // Event-driven para trading agresivo
            .enable_parallel_execution(true)
            .max_concurrent_executions(3)   // Hasta 3 trades simultáneos
            .jito_tip_lamports(100000)      // 0.0001 SOL tip más alto
            .jito_max_priority_fee(200000)  // 0.0002 SOL max priority fee
            .build()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_default_config_validation() {
        let config = UnifiedPhase45Config::default();
        assert!(ConfigValidator::validate(&config).is_ok());
    }
    
    #[test]
    fn test_config_builder() {
        let config = ConfigBuilder::new()
            .enable_jupiter_advanced(true)
            .enable_mev_protection(true)
            .trading_range(0.001, 0.01)
            .build();
            
        assert!(config.jupiter_advanced_enabled);
        assert!(config.mev_protection_enabled);
        assert_eq!(config.min_trade_sol, 0.001);
        assert_eq!(config.max_trade_sol, 0.01);
    }
    
    #[test]
    fn test_balance_validation() {
        let config = UnifiedPhase45Config::default();
        
        // Should pass with sufficient balance
        assert!(ConfigValidator::validate_for_balance(&config, 1.0).is_ok());
        
        // Should fail with insufficient balance
        assert!(ConfigValidator::validate_for_balance(&config, 0.001).is_err());
    }
}
