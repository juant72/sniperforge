// ===== UNIFIED CONFIG MODULE =====
// Centralized configuration management for all bot components

use anyhow::Result;
use serde::{Serialize, Deserialize};
use std::collections::HashMap;
use std::path::PathBuf;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UnifiedConfig {
    pub trading: TradingConfig,
    pub risk_management: RiskManagementConfig,
    pub arbitrage: ArbitrageConfig,
    pub monitoring: MonitoringConfig,
    pub api: ApiConfig,
    pub performance: PerformanceConfig,
    pub ml: MLConfig,
    pub network: NetworkConfig,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TradingConfig {
    pub enabled: bool,
    pub execution_mode: String, // "simulation", "testnet", "mainnet"
    pub default_slippage_bps: u16,
    pub max_trade_size_usd: f64,
    pub min_profit_threshold_bps: u16,
    pub priority_fee_lamports: u64,
    pub confirmation_timeout_ms: u64,
    pub retry_attempts: u32,
    pub retry_delay_ms: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RiskManagementConfig {
    pub max_position_size_usd: f64,
    pub max_daily_loss_usd: f64,
    pub max_consecutive_losses: u32,
    pub drawdown_limit_percent: f64,
    pub stop_loss_enabled: bool,
    pub position_sizing_method: String, // "fixed", "kelly", "percent_risk"
    pub emergency_stop_triggers: Vec<String>,
    pub cooling_period_minutes: u32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ArbitrageConfig {
    pub min_profit_bps: u16,
    pub max_hops: u32,
    pub max_routes_to_check: u32,
    pub price_update_interval_ms: u64,
    pub stale_price_threshold_ms: u64,
    pub triangular_enabled: bool,
    pub cross_dex_enabled: bool,
    pub flashloan_enabled: bool,
    pub supported_tokens: Vec<String>,
    pub blacklisted_tokens: Vec<String>,
    pub dex_priorities: HashMap<String, u32>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MonitoringConfig {
    pub log_level: String,
    pub metrics_enabled: bool,
    pub metrics_port: u16,
    pub health_check_interval_ms: u64,
    pub performance_tracking: bool,
    pub alerts_enabled: bool,
    pub alert_channels: Vec<String>,
    pub dashboard_enabled: bool,
    pub export_trades_csv: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ApiConfig {
    pub jupiter_enabled: bool,
    pub jupiter_rate_limit_ms: u64,
    pub dexscreener_enabled: bool,
    pub dexscreener_rate_limit_ms: u64,
    pub birdeye_enabled: bool,
    pub birdeye_api_key: Option<String>,
    pub coinapi_enabled: bool,
    pub coinapi_key: Option<String>,
    pub timeout_ms: u64,
    pub max_retries: u32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PerformanceConfig {
    pub parallel_route_checking: bool,
    pub max_concurrent_requests: u32,
    pub cache_enabled: bool,
    pub cache_ttl_ms: u64,
    pub precompute_routes: bool,
    pub optimization_level: String, // "fast", "balanced", "thorough"
    pub memory_limit_mb: u64,
    pub cpu_usage_limit_percent: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MLConfig {
    pub enabled: bool,
    pub pattern_recognition: bool,
    pub prediction_models: bool,
    pub model_update_interval_hours: u32,
    pub training_data_days: u32,
    pub confidence_threshold: f64,
    pub feature_engineering: bool,
    pub ensemble_methods: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NetworkConfig {
    pub rpc_endpoint: String,
    pub backup_rpc_endpoints: Vec<String>,
    pub commitment_level: String, // "processed", "confirmed", "finalized"
    pub connection_timeout_ms: u64,
    pub request_timeout_ms: u64,
    pub max_connections: u32,
    pub keepalive_enabled: bool,
}

impl Default for UnifiedConfig {
    fn default() -> Self {
        Self {
            trading: TradingConfig {
                enabled: false, // Safety: disabled by default
                execution_mode: "simulation".to_string(),
                default_slippage_bps: 50, // 0.5%
                max_trade_size_usd: 1000.0,
                min_profit_threshold_bps: 30, // 0.3%
                priority_fee_lamports: 5000,
                confirmation_timeout_ms: 30000,
                retry_attempts: 3,
                retry_delay_ms: 1000,
            },
            risk_management: RiskManagementConfig {
                max_position_size_usd: 5000.0,
                max_daily_loss_usd: 500.0,
                max_consecutive_losses: 5,
                drawdown_limit_percent: 10.0,
                stop_loss_enabled: true,
                position_sizing_method: "fixed".to_string(),
                emergency_stop_triggers: vec![
                    "max_daily_loss".to_string(),
                    "consecutive_losses".to_string(),
                    "drawdown_limit".to_string(),
                ],
                cooling_period_minutes: 30,
            },
            arbitrage: ArbitrageConfig {
                min_profit_bps: 30, // 0.3%
                max_hops: 3,
                max_routes_to_check: 50,
                price_update_interval_ms: 1000,
                stale_price_threshold_ms: 5000,
                triangular_enabled: true,
                cross_dex_enabled: true,
                flashloan_enabled: false, // Disabled by default
                supported_tokens: vec![
                    "SOL".to_string(),
                    "USDC".to_string(),
                    "USDT".to_string(),
                    "RAY".to_string(),
                    "SRM".to_string(),
                    "ORCA".to_string(),
                ],
                blacklisted_tokens: vec![],
                dex_priorities: {
                    let mut priorities = HashMap::new();
                    priorities.insert("Jupiter".to_string(), 1);
                    priorities.insert("Raydium".to_string(), 2);
                    priorities.insert("Orca".to_string(), 3);
                    priorities
                },
            },
            monitoring: MonitoringConfig {
                log_level: "info".to_string(),
                metrics_enabled: true,
                metrics_port: 9090,
                health_check_interval_ms: 10000,
                performance_tracking: true,
                alerts_enabled: true,
                alert_channels: vec!["console".to_string()],
                dashboard_enabled: false,
                export_trades_csv: true,
            },
            api: ApiConfig {
                jupiter_enabled: true,
                jupiter_rate_limit_ms: 100,
                dexscreener_enabled: true,
                dexscreener_rate_limit_ms: 1000,
                birdeye_enabled: false,
                birdeye_api_key: None,
                coinapi_enabled: false,
                coinapi_key: None,
                timeout_ms: 10000,
                max_retries: 3,
            },
            performance: PerformanceConfig {
                parallel_route_checking: true,
                max_concurrent_requests: 10,
                cache_enabled: true,
                cache_ttl_ms: 5000,
                precompute_routes: false,
                optimization_level: "balanced".to_string(),
                memory_limit_mb: 512,
                cpu_usage_limit_percent: 80.0,
            },
            ml: MLConfig {
                enabled: false, // Disabled by default for safety
                pattern_recognition: false,
                prediction_models: false,
                model_update_interval_hours: 24,
                training_data_days: 30,
                confidence_threshold: 0.7,
                feature_engineering: false,
                ensemble_methods: false,
            },
            network: NetworkConfig {
                rpc_endpoint: "https://api.mainnet-beta.solana.com".to_string(),
                backup_rpc_endpoints: vec![
                    "https://solana-api.projectserum.com".to_string(),
                    "https://rpc.ankr.com/solana".to_string(),
                ],
                commitment_level: "confirmed".to_string(),
                connection_timeout_ms: 5000,
                request_timeout_ms: 10000,
                max_connections: 20,
                keepalive_enabled: true,
            },
        }
    }
}

impl UnifiedConfig {
    /// Load configuration from file
    pub fn load_from_file(path: &PathBuf) -> Result<Self> {
        let content = std::fs::read_to_string(path)?;
        let config: UnifiedConfig = serde_json::from_str(&content)?;
        config.validate()?;
        Ok(config)
    }
    
    /// Save configuration to file
    pub fn save_to_file(&self, path: &PathBuf) -> Result<()> {
        let content = serde_json::to_string_pretty(self)?;
        std::fs::write(path, content)?;
        Ok(())
    }
    
    /// Load from environment variables
    pub fn load_from_env() -> Result<Self> {
        let mut config = Self::default();
        
        // Load key settings from environment
        if let Ok(enabled) = std::env::var("TRADING_ENABLED") {
            config.trading.enabled = enabled.parse().unwrap_or(false);
        }
        
        if let Ok(mode) = std::env::var("EXECUTION_MODE") {
            config.trading.execution_mode = mode;
        }
        
        if let Ok(rpc) = std::env::var("RPC_ENDPOINT") {
            config.network.rpc_endpoint = rpc;
        }
        
        if let Ok(api_key) = std::env::var("BIRDEYE_API_KEY") {
            config.api.birdeye_api_key = Some(api_key);
            config.api.birdeye_enabled = true;
        }
        
        if let Ok(api_key) = std::env::var("COINAPI_KEY") {
            config.api.coinapi_key = Some(api_key);
            config.api.coinapi_enabled = true;
        }
        
        config.validate()?;
        Ok(config)
    }
    
    /// Validate configuration
    pub fn validate(&self) -> Result<()> {
        // Validate execution mode
        match self.trading.execution_mode.as_str() {
            "simulation" | "testnet" | "mainnet" => {},
            _ => return Err(anyhow::anyhow!("Invalid execution mode")),
        }
        
        // Validate basic limits
        if self.trading.max_trade_size_usd <= 0.0 {
            return Err(anyhow::anyhow!("Max trade size must be positive"));
        }
        
        if self.arbitrage.min_profit_bps == 0 {
            return Err(anyhow::anyhow!("Minimum profit must be positive"));
        }
        
        if self.arbitrage.max_hops == 0 || self.arbitrage.max_hops > 10 {
            return Err(anyhow::anyhow!("Max hops must be between 1 and 10"));
        }
        
        // Validate risk limits
        if self.risk_management.max_daily_loss_usd <= 0.0 {
            return Err(anyhow::anyhow!("Max daily loss must be positive"));
        }
        
        if self.risk_management.drawdown_limit_percent <= 0.0 || 
           self.risk_management.drawdown_limit_percent > 100.0 {
            return Err(anyhow::anyhow!("Drawdown limit must be between 0 and 100%"));
        }
        
        // Validate performance settings
        if self.performance.max_concurrent_requests == 0 {
            return Err(anyhow::anyhow!("Max concurrent requests must be positive"));
        }
        
        if self.performance.cpu_usage_limit_percent <= 0.0 || 
           self.performance.cpu_usage_limit_percent > 100.0 {
            return Err(anyhow::anyhow!("CPU usage limit must be between 0 and 100%"));
        }
        
        Ok(())
    }
    
    /// Get safe defaults for production
    pub fn production_safe() -> Self {
        let mut config = Self::default();
        
        // Extra safe production settings
        config.trading.enabled = false;
        config.trading.execution_mode = "simulation".to_string();
        config.trading.max_trade_size_usd = 100.0; // Very conservative
        config.risk_management.max_daily_loss_usd = 50.0;
        config.risk_management.max_position_size_usd = 200.0;
        config.arbitrage.min_profit_bps = 50; // Higher threshold
        config.ml.enabled = false; // Disable ML for safety
        
        config
    }
    
    /// Get aggressive settings for maximum profit (use with caution)
    pub fn aggressive() -> Self {
        let mut config = Self::default();
        
        config.trading.max_trade_size_usd = 10000.0;
        config.trading.min_profit_threshold_bps = 20; // Lower threshold
        config.risk_management.max_position_size_usd = 50000.0;
        config.risk_management.max_daily_loss_usd = 2000.0;
        config.arbitrage.min_profit_bps = 20;
        config.arbitrage.max_routes_to_check = 200;
        config.performance.max_concurrent_requests = 50;
        config.ml.enabled = true;
        
        config
    }
    
    /// Get development settings
    pub fn development() -> Self {
        let mut config = Self::default();
        
        config.trading.execution_mode = "simulation".to_string();
        config.monitoring.log_level = "debug".to_string();
        config.monitoring.dashboard_enabled = true;
        config.performance.cache_enabled = false; // For testing
        config.api.jupiter_rate_limit_ms = 50; // Faster for development
        
        config
    }
    
    /// Update a specific section of the config
    pub fn update_trading(&mut self, new_trading: TradingConfig) -> Result<()> {
        self.trading = new_trading;
        self.validate()
    }
    
    pub fn update_risk_management(&mut self, new_risk: RiskManagementConfig) -> Result<()> {
        self.risk_management = new_risk;
        self.validate()
    }
    
    /// Get a summary of current settings
    pub fn summary(&self) -> HashMap<String, serde_json::Value> {
        let mut summary = HashMap::new();
        summary.insert("trading_enabled".to_string(), self.trading.enabled.into());
        summary.insert("execution_mode".to_string(), self.trading.execution_mode.clone().into());
        summary.insert("max_trade_size_usd".to_string(), self.trading.max_trade_size_usd.into());
        summary.insert("min_profit_bps".to_string(), self.arbitrage.min_profit_bps.into());
        summary.insert("max_daily_loss_usd".to_string(), self.risk_management.max_daily_loss_usd.into());
        summary.insert("ml_enabled".to_string(), self.ml.enabled.into());
        summary.insert("supported_tokens".to_string(), self.arbitrage.supported_tokens.len().into());
        summary
    }
    
    /// Check if configuration is safe for production
    pub fn is_production_safe(&self) -> bool {
        self.trading.max_trade_size_usd <= 1000.0 &&
        self.risk_management.max_daily_loss_usd <= 500.0 &&
        self.arbitrage.min_profit_bps >= 30 &&
        (self.trading.execution_mode == "simulation" || self.trading.execution_mode == "testnet")
    }
}
