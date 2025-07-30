// ================================================================================
// ARBITRAGE CONFIG - Configuración del Bot de Arbitraje
// ================================================================================

use serde::{Deserialize, Serialize};
use sniperforge_core::BotConfigurable;

/// Configuración principal del bot de arbitraje
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ArbitrageConfig {
    pub general: GeneralConfig,
    pub engine: EngineConfig,
    pub triangular: TriangularConfig,
    pub ml: MlConfig,
    pub trading: TradingConfig,
    pub phases: PhaseConfig,
    pub jupiter: JupiterConfig,
    pub wallet: WalletConfig,
    pub fees: FeesConfig,
    pub apis: ApiConfig,
    pub risk: RiskConfig,
}

/// Configuración general
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GeneralConfig {
    pub enabled: bool,
    pub max_concurrent_trades: u32,
    pub cycle_interval_ms: u64,
    pub log_level: String,
    pub emergency_stop_loss: f64,
}

/// Configuración del motor de arbitraje
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EngineConfig {
    pub enabled: bool,
    pub min_profit_sol: f64,
    pub min_profit_percentage: f64,
    pub max_hops: u8,
    pub timeout_ms: u64,
    pub price_impact_threshold: f64,
    pub supported_dexes: Vec<String>,
}

/// Configuración de arbitraje triangular
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TriangularConfig {
    pub enabled: bool,
    pub min_profit_sol: f64,
    pub base_tokens: Vec<String>,
    pub max_combinations: u32,
    pub timeout_ms: u64,
}

/// Configuración de Machine Learning
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MlConfig {
    pub enabled: bool,
    pub min_score_threshold: f64,
    pub pattern_history_size: u32,
    pub learning_rate: f64,
    pub model_update_interval_minutes: u32,
}

/// Configuración de trading
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TradingConfig {
    pub max_slippage_bps: u16,
    pub priority_fee_lamports: u64,
    pub compute_unit_limit: u32,
    pub compute_unit_price: u64,
    pub confirmation_timeout_ms: u64,
    pub retry_attempts: u8,
}

/// Configuración de fases avanzadas
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PhaseConfig {
    pub quantum: QuantumConfig,
    pub autonomous: AutonomousConfig,
    pub ecosystem: EcosystemConfig,
}

/// Configuración Phase 9: Quantum
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct QuantumConfig {
    pub enabled: bool,
    pub quantum_iterations: u32,
    pub superposition_analysis: bool,
    pub entanglement_detection: bool,
    pub quantum_speedup_factor: f64,
}

/// Configuración Phase 10: Autonomous
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AutonomousConfig {
    pub enabled: bool,
    pub ai_decision_threshold: f64,
    pub auto_parameter_tuning: bool,
    pub adaptive_risk_management: bool,
    pub learning_feedback_loop: bool,
}

/// Configuración Phase 11: Ecosystem
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EcosystemConfig {
    pub enabled: bool,
    pub cross_chain_enabled: bool,
    pub supported_chains: Vec<String>,
    pub ecosystem_multiplier: f64,
    pub network_effect_bonus: f64,
}

/// Configuración de Jupiter
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct JupiterConfig {
    pub api_url: String,
    pub api_key: Option<String>,
    pub timeout_ms: u64,
    pub rate_limit_per_second: u32,
    pub use_versioned_transactions: bool,
}

/// Configuración de wallet
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WalletConfig {
    pub private_key_path: String,
    pub rpc_url: String,
    pub commitment: String,
    pub min_balance_sol: f64,
    pub max_balance_usage_percentage: f64,
}

/// Configuración de fees
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FeesConfig {
    pub platform_fee_bps: u16,
    pub jupiter_fee_bps: u16,
    pub priority_fee_multiplier: f64,
    pub gas_optimization: bool,
}

/// Configuración de APIs
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ApiConfig {
    pub dexscreener_enabled: bool,
    pub coingecko_enabled: bool,
    pub birdeye_enabled: bool,
    pub helius_enabled: bool,
    pub update_interval_ms: u64,
}

/// Configuración de gestión de riesgo
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RiskConfig {
    pub max_position_size_sol: f64,
    pub max_daily_trades: u32,
    pub max_daily_loss_sol: f64,
    pub stop_loss_percentage: f64,
    pub exposure_limits: ExposureLimits,
}

/// Límites de exposición
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExposureLimits {
    pub max_exposure_per_token: f64,
    pub max_exposure_per_dex: f64,
    pub max_total_exposure: f64,
    pub concentration_limit: f64,
}

impl Default for ArbitrageConfig {
    fn default() -> Self {
        Self {
            general: GeneralConfig {
                enabled: true,
                max_concurrent_trades: 3,
                cycle_interval_ms: 5000,
                log_level: "info".to_string(),
                emergency_stop_loss: 0.1, // 10%
            },
            engine: EngineConfig {
                enabled: true,
                min_profit_sol: 0.005,
                min_profit_percentage: 0.005, // 0.5%
                max_hops: 3,
                timeout_ms: 30000,
                price_impact_threshold: 0.03, // 3%
                supported_dexes: vec![
                    "Raydium".to_string(),
                    "Orca".to_string(),
                    "Serum".to_string(),
                    "Phoenix".to_string(),
                ],
            },
            triangular: TriangularConfig {
                enabled: true,
                min_profit_sol: 0.008,
                base_tokens: vec![
                    "SOL".to_string(),
                    "USDC".to_string(),
                    "USDT".to_string(),
                ],
                max_combinations: 50,
                timeout_ms: 20000,
            },
            ml: MlConfig {
                enabled: true,
                min_score_threshold: 0.7,
                pattern_history_size: 1000,
                learning_rate: 0.01,
                model_update_interval_minutes: 60,
            },
            trading: TradingConfig {
                max_slippage_bps: 50, // 0.5%
                priority_fee_lamports: 10000,
                compute_unit_limit: 200000,
                compute_unit_price: 1000,
                confirmation_timeout_ms: 30000,
                retry_attempts: 3,
            },
            phases: PhaseConfig {
                quantum: QuantumConfig {
                    enabled: false, // Resource intensive
                    quantum_iterations: 100,
                    superposition_analysis: true,
                    entanglement_detection: true,
                    quantum_speedup_factor: 2.5,
                },
                autonomous: AutonomousConfig {
                    enabled: true,
                    ai_decision_threshold: 0.8,
                    auto_parameter_tuning: true,
                    adaptive_risk_management: true,
                    learning_feedback_loop: true,
                },
                ecosystem: EcosystemConfig {
                    enabled: false, // Cross-chain not implemented yet
                    cross_chain_enabled: false,
                    supported_chains: vec!["Solana".to_string()],
                    ecosystem_multiplier: 1.5,
                    network_effect_bonus: 0.1,
                },
            },
            jupiter: JupiterConfig {
                api_url: "https://quote-api.jup.ag/v6".to_string(),
                api_key: None,
                timeout_ms: 15000,
                rate_limit_per_second: 10,
                use_versioned_transactions: true,
            },
            wallet: WalletConfig {
                private_key_path: "wallet.json".to_string(),
                rpc_url: "https://api.mainnet-beta.solana.com".to_string(),
                commitment: "confirmed".to_string(),
                min_balance_sol: 0.05,
                max_balance_usage_percentage: 0.8, // 80%
            },
            fees: FeesConfig {
                platform_fee_bps: 0, // No platform fee for arbitrage
                jupiter_fee_bps: 0, // Jupiter fee handled separately
                priority_fee_multiplier: 1.5,
                gas_optimization: true,
            },
            apis: ApiConfig {
                dexscreener_enabled: true,
                coingecko_enabled: true,
                birdeye_enabled: false, // Requires API key
                helius_enabled: false, // Requires API key
                update_interval_ms: 10000,
            },
            risk: RiskConfig {
                max_position_size_sol: 1.0,
                max_daily_trades: 100,
                max_daily_loss_sol: 0.1,
                stop_loss_percentage: 0.05, // 5%
                exposure_limits: ExposureLimits {
                    max_exposure_per_token: 0.3, // 30%
                    max_exposure_per_dex: 0.5, // 50%
                    max_total_exposure: 0.8, // 80%
                    concentration_limit: 0.4, // 40%
                },
            },
        }
    }
}

impl BotConfigurable for ArbitrageConfig {
    fn validate(&self) -> sniperforge_core::CoreResult<()> {
        // Validate general config
        if self.general.max_concurrent_trades == 0 {
            return Err(sniperforge_core::CoreError::Configuration(
                "max_concurrent_trades must be > 0".to_string()
            ));
        }
        
        if self.general.emergency_stop_loss <= 0.0 || self.general.emergency_stop_loss >= 1.0 {
            return Err(sniperforge_core::CoreError::Configuration(
                "emergency_stop_loss must be between 0 and 1".to_string()
            ));
        }
        
        // Validate engine config
        if self.engine.min_profit_sol <= 0.0 {
            return Err(sniperforge_core::CoreError::Configuration(
                "min_profit_sol must be > 0".to_string()
            ));
        }
        
        if self.engine.min_profit_percentage <= 0.0 {
            return Err(sniperforge_core::CoreError::Configuration(
                "min_profit_percentage must be > 0".to_string()
            ));
        }
        
        // Validate trading config
        if self.trading.max_slippage_bps > 1000 { // 10%
            return Err(sniperforge_core::CoreError::Configuration(
                "max_slippage_bps should not exceed 1000 (10%)".to_string()
            ));
        }
        
        // Validate wallet config
        if self.wallet.min_balance_sol <= 0.0 {
            return Err(sniperforge_core::CoreError::Configuration(
                "min_balance_sol must be > 0".to_string()
            ));
        }
        
        if self.wallet.max_balance_usage_percentage <= 0.0 || self.wallet.max_balance_usage_percentage > 1.0 {
            return Err(sniperforge_core::CoreError::Configuration(
                "max_balance_usage_percentage must be between 0 and 1".to_string()
            ));
        }
        
        // Validate risk config
        if self.risk.max_position_size_sol <= 0.0 {
            return Err(sniperforge_core::CoreError::Configuration(
                "max_position_size_sol must be > 0".to_string()
            ));
        }
        
        Ok(())
    }
    
    fn get_name(&self) -> &str {
        "ArbitrageBot"
    }
    
    fn get_version(&self) -> &str {
        "1.0.0"
    }
}

/// Helper para crear configuración de desarrollo
impl ArbitrageConfig {
    /// Configuración para devnet/testnet
    pub fn devnet() -> Self {
        let mut config = Self::default();
        config.wallet.rpc_url = "https://api.devnet.solana.com".to_string();
        config.engine.min_profit_sol = 0.001; // Lower for testing
        config.engine.min_profit_percentage = 0.001;
        config.risk.max_position_size_sol = 0.1; // Lower for testing
        config.risk.max_daily_loss_sol = 0.05;
        config
    }
    
    /// Configuración conservadora para mainnet
    pub fn conservative() -> Self {
        let mut config = Self::default();
        config.engine.min_profit_sol = 0.01; // Higher minimum
        config.engine.min_profit_percentage = 0.01; // 1%
        config.trading.max_slippage_bps = 30; // 0.3%
        config.risk.max_position_size_sol = 0.5;
        config.general.max_concurrent_trades = 1;
        config
    }
    
    /// Configuración agresiva para traders experimentados
    pub fn aggressive() -> Self {
        let mut config = Self::default();
        config.engine.min_profit_sol = 0.003;
        config.engine.min_profit_percentage = 0.003; // 0.3%
        config.trading.max_slippage_bps = 100; // 1%
        config.risk.max_position_size_sol = 2.0;
        config.general.max_concurrent_trades = 5;
        config.phases.quantum.enabled = true; // Enable quantum for max performance
        config
    }
}
