// ===== ARBITRAGE SETTINGS MODULE =====
// Configuration management for arbitrage bot settings

use serde::{Deserialize, Serialize};
use std::fs;
use anyhow::{Result, anyhow};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TradingConfig {
    pub mode: String,
    pub force_real_transactions: bool,
    pub max_trade_sol: f64,
    pub min_profit_threshold_sol: f64,
    pub min_confidence_threshold: f64,
    pub max_concurrent_trades: u32,
    pub trade_timeout_seconds: u64,
    pub min_trade_size_sol: f64,
    pub max_trade_size_sol: f64,
    pub max_slippage_bps: u64,
    pub military_min_profit_bps: u64,
    pub base_profit_percentage: f64,
    pub max_profit_percentage: f64,
    pub default_trade_amount_usd: f64,
    pub estimated_gas_cost_usd: f64,
    pub estimated_execution_time_ms: u64,
    pub pre_execution_validation: bool,
    pub program_id_whitelist: Vec<String>,
    pub simulation_before_execution: bool,
    pub max_simulation_retries: u32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WalletConfig {
    pub keypair_file: String,
    pub backup_keypair_file: String,
    pub use_env_private_key: bool,
    pub env_key_name: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RpcConfig {
    pub primary_url: String,
    pub backup_urls: Vec<String>,
    pub timeout_seconds: u64,
    pub retry_attempts: u32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ApiConfig {
    pub enabled: bool,
    pub timeout_seconds: u64,
    pub max_retries: u32,
    pub batch_requests: bool,
    pub rate_limit_per_sec: u32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ApisConfig {
    pub dexscreener: ApiConfig,
    pub jupiter: ApiConfig,
    pub coingecko: ApiConfig,
    pub chainstack: ApiConfig,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PerformanceConfig {
    pub max_concurrent_discoveries: usize,
    pub cache_ttl_seconds: u64,
    pub parallel_api_calls: bool,
    pub latency_target_ms: u64,
    pub max_execution_time_ms: u64,
    pub memory_limit_mb: u64,
    pub cpu_usage_threshold: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MlAnalysisConfig {
    pub enabled: bool,
    pub pattern_recognition_enabled: bool,
    pub adaptive_parameters_enabled: bool,
    pub ml_confidence_threshold: f64,
    pub historical_data_days: u32,
    pub model_update_frequency_hours: u32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SecurityConfig {
    pub emergency_stop_loss_pct: f64,
    pub max_daily_trades: u32,
    pub circuit_breaker_enabled: bool,
    pub audit_logging: bool,
    pub encrypted_wallet: bool,
    pub whitelist_enforcement: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ArbitrageSettings {
    pub trading: TradingConfig,
    pub wallet: WalletConfig,
    pub rpc: RpcConfig,
    pub apis: ApisConfig,
    pub performance: PerformanceConfig,
    pub ml_analysis: MlAnalysisConfig,
    pub security: SecurityConfig,
}

impl ArbitrageSettings {
    /// Load settings from default arbitrage_settings.json
    pub fn load_default() -> Result<Self> {
        Self::load_from_file("arbitrage_settings.json")
    }
    
    /// Load settings from specified file
    pub fn load_from_file(file_path: &str) -> Result<Self> {
        let content = fs::read_to_string(file_path)
            .map_err(|e| anyhow!("Failed to read settings file {}: {}", file_path, e))?;
        
        let settings: ArbitrageSettings = serde_json::from_str(&content)
            .map_err(|e| anyhow!("Failed to parse settings JSON: {}", e))?;
        
        Ok(settings)
    }
}

impl Default for ArbitrageSettings {
    fn default() -> Self {
        Self {
            trading: TradingConfig {
                mode: "simulation".to_string(),
                force_real_transactions: false,
                max_trade_sol: 0.08,
                min_profit_threshold_sol: 0.004,
                min_confidence_threshold: 0.7,
                max_concurrent_trades: 1,
                trade_timeout_seconds: 45,
                min_trade_size_sol: 0.005,
                max_trade_size_sol: 0.03,
                max_slippage_bps: 30,
                military_min_profit_bps: 1,
                base_profit_percentage: 0.0005,
                max_profit_percentage: 0.008,
                default_trade_amount_usd: 75.0,
                estimated_gas_cost_usd: 0.5,
                estimated_execution_time_ms: 5000,
                pre_execution_validation: true,
                program_id_whitelist: vec![
                    "JUP6LkbZbjS1jKKwapdHNy74zcZ3tLUZoi5QNyVTaV4".to_string(),
                    "TokenkegQfeZyiNwAJbNbGKPFXCWuBvf9Ss623VQ5DA".to_string(),
                ],
                simulation_before_execution: true,
                max_simulation_retries: 2,
            },
            wallet: WalletConfig {
                keypair_file: "./keypair.json".to_string(),
                backup_keypair_file: "~/.config/solana/id.json".to_string(),
                use_env_private_key: false,
                env_key_name: "SOLANA_PRIVATE_KEY".to_string(),
            },
            rpc: RpcConfig {
                primary_url: "https://api.mainnet-beta.solana.com".to_string(),
                backup_urls: vec!["https://api.mainnet-beta.solana.com".to_string()],
                timeout_seconds: 20,
                retry_attempts: 2,
            },
            apis: ApisConfig {
                dexscreener: ApiConfig {
                    enabled: true,
                    timeout_seconds: 4,
                    max_retries: 1,
                    batch_requests: false,
                    rate_limit_per_sec: 5,
                },
                jupiter: ApiConfig {
                    enabled: true,
                    timeout_seconds: 8,
                    max_retries: 2,
                    batch_requests: true,
                    rate_limit_per_sec: 10,
                },
                coingecko: ApiConfig {
                    enabled: true,
                    timeout_seconds: 6,
                    max_retries: 1,
                    batch_requests: false,
                    rate_limit_per_sec: 3,
                },
                chainstack: ApiConfig {
                    enabled: false,
                    timeout_seconds: 10,
                    max_retries: 2,
                    batch_requests: true,
                    rate_limit_per_sec: 8,
                },
            },
            performance: PerformanceConfig {
                max_concurrent_discoveries: 4,
                cache_ttl_seconds: 30,
                parallel_api_calls: true,
                latency_target_ms: 1000,
                max_execution_time_ms: 30000,
                memory_limit_mb: 512,
                cpu_usage_threshold: 80.0,
            },
            ml_analysis: MlAnalysisConfig {
                enabled: false,
                pattern_recognition_enabled: false,
                adaptive_parameters_enabled: false,
                ml_confidence_threshold: 0.8,
                historical_data_days: 7,
                model_update_frequency_hours: 24,
            },
            security: SecurityConfig {
                emergency_stop_loss_pct: 10.0,
                max_daily_trades: 100,
                circuit_breaker_enabled: true,
                audit_logging: true,
                encrypted_wallet: false,
                whitelist_enforcement: true,
            },
        }
    }
}
