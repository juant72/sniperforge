//! Configuration management for SniperForge

pub mod api_credentials;
pub mod enterprise;

use serde::{Deserialize, Serialize};
use std::{path::Path, collections::HashMap};
use crate::types::Result;
pub use api_credentials::{ApiCredentials, WebSocketConfig};
pub use enterprise::{EnterpriseConfig, SolanaConfig as EnterpriseSolanaConfig, 
                    ApiConfig as EnterpriseApiConfig, TradingConfig as EnterpriseTradingConfig};

/// Simple configuration alias for backward compatibility
pub type Config = SniperForgeConfig;

/// Main configuration structure
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SniperForgeConfig {
    /// Trading configuration
    pub trading: TradingConfig,
    /// Security configuration
    pub security: SecurityConfig,
    /// API configuration
    pub apis: ApiConfig,
    /// Performance configuration
    pub performance: PerformanceConfig,
}

impl SniperForgeConfig {
    /// Create a simplified config for the arbitrage engine
    pub fn to_simple_config(&self) -> SimpleConfig {
        SimpleConfig {
            solana_rpc_url: self.security.rpc_url.clone(),
            solana_ws_url: self.security.rpc_url.replace("http", "ws"), // Convert HTTP to WS
            max_slippage: self.trading.max_slippage_bps as f64 / 10000.0, // Convert bps to decimal
            min_profit_threshold: self.trading.min_profit_bps as f64 / 10000.0, // Convert bps to decimal
            max_position_size: self.trading.max_trade_size_sol,
            private_key_path: self.security.wallet_path.clone(),
            enable_simulation: !self.trading.enabled, // If trading disabled, use simulation
            log_level: "info".to_string(),
            dexscreener_base_url: "https://api.dexscreener.com".to_string(), // Default value
            max_requests_per_second: 10, // Default value
            cooldown_period_ms: 1000, // Default value
            max_history_size: 1000, // Default value
        }
    }
}

/// Simplified configuration for arbitrage engine
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SimpleConfig {
    pub solana_rpc_url: String,
    pub solana_ws_url: String,
    pub max_slippage: f64,
    pub min_profit_threshold: f64,
    pub max_position_size: f64,
    pub private_key_path: String,
    pub enable_simulation: bool,
    pub log_level: String,
    pub dexscreener_base_url: String,
    pub max_requests_per_second: u32,
    pub cooldown_period_ms: u64,
    pub max_history_size: usize,
}

impl Default for SimpleConfig {
    fn default() -> Self {
        Self {
            solana_rpc_url: "https://api.mainnet-beta.solana.com".to_string(),
            solana_ws_url: "wss://api.mainnet-beta.solana.com/".to_string(),
            max_slippage: 0.005,
            min_profit_threshold: 0.001,
            max_position_size: 0.1,
            private_key_path: "./wallet.json".to_string(),
            enable_simulation: false,  // MAINNET = NO SIMULATION
            log_level: "info".to_string(),
            dexscreener_base_url: "https://api.dexscreener.com".to_string(),
            max_requests_per_second: 10,
            cooldown_period_ms: 100,
            max_history_size: 1000,
        }
    }
}

impl SimpleConfig {
    pub fn from_env() -> Result<Self> {
        Self::load_from_file(".env.mainnet")
    }
    
    pub fn load_from_file(config_file: &str) -> Result<Self> {
        // Cargar el archivo de configuración
        let config_map = Self::parse_env_file(config_file)?;
        let mut config = Self::default();
        
        // Basic connection settings
        if let Some(rpc_url) = config_map.get("SOLANA_RPC_URL") {
            config.solana_rpc_url = rpc_url.clone();
        }
        
        if let Some(ws_url) = config_map.get("SOLANA_WS_URL") {
            config.solana_ws_url = ws_url.clone();
        }
        
        // Trading parameters
        if let Some(slippage) = config_map.get("MAX_SLIPPAGE") {
            config.max_slippage = slippage.parse()
                .map_err(|_| "Invalid MAX_SLIPPAGE value".to_string())?;
        }
        
        if let Some(min_profit) = config_map.get("MIN_PROFIT_THRESHOLD") {
            config.min_profit_threshold = min_profit.parse()
                .map_err(|_| "Invalid MIN_PROFIT_THRESHOLD value".to_string())?;
        }
        
        if let Some(max_position) = config_map.get("MAX_POSITION_SIZE") {
            config.max_position_size = max_position.parse()
                .map_err(|_| "Invalid MAX_POSITION_SIZE value".to_string())?;
        }
        
        // Wallet configuration
        if let Some(wallet_path) = config_map.get("WALLET_PATH") {
            config.private_key_path = wallet_path.clone();
        }
        
        // Simulation mode
        if let Some(enable_sim) = config_map.get("ENABLE_SIMULATION") {
            config.enable_simulation = enable_sim.parse()
                .map_err(|_| "Invalid ENABLE_SIMULATION value".to_string())?;
        }
        
        // Network configuration
        if let Some(max_rps) = config_map.get("MAX_REQUESTS_PER_SECOND") {
            config.max_requests_per_second = max_rps.parse()
                .map_err(|_| "Invalid MAX_REQUESTS_PER_SECOND value".to_string())?;
        }
        
        if let Some(cooldown) = config_map.get("COOLDOWN_PERIOD_MS") {
            config.cooldown_period_ms = cooldown.parse()
                .map_err(|_| "Invalid COOLDOWN_PERIOD_MS value".to_string())?;
        }
        
        if let Some(history_size) = config_map.get("MAX_HISTORY_SIZE") {
            config.max_history_size = history_size.parse()
                .map_err(|_| "Invalid MAX_HISTORY_SIZE value".to_string())?;
        }
        
        // API URLs
        if let Some(dex_url) = config_map.get("DEXSCREENER_API_URL") {
            config.dexscreener_base_url = dex_url.clone();
        }
        
        // Logging
        if let Some(log_level) = config_map.get("LOG_LEVEL") {
            config.log_level = log_level.clone();
        }
        
        Ok(config)
    }
    
    /// Parse .env file manually without using environment variables
    fn parse_env_file(file_path: &str) -> Result<HashMap<String, String>> {
        let mut config_map = HashMap::new();
        
        if !std::path::Path::new(file_path).exists() {
            return Err(format!("Configuration file '{}' not found", file_path).into());
        }
        
        let content = std::fs::read_to_string(file_path)
            .map_err(|e| format!("Failed to read config file '{}': {}", file_path, e))?;
        
        for line in content.lines() {
            let line = line.trim();
            
            // Skip empty lines and comments
            if line.is_empty() || line.starts_with('#') {
                continue;
            }
            
            // Parse key=value pairs
            if let Some(eq_pos) = line.find('=') {
                let key = line[..eq_pos].trim().to_string();
                let value = line[eq_pos + 1..].trim().to_string();
                config_map.insert(key, value);
            }
        }
        
        Ok(config_map)
    }
    
    /// Get a configuration value from the loaded config
    pub fn get_config_value(key: &str, default: &str) -> String {
        match Self::parse_env_file(".env.mainnet") {
            Ok(config_map) => config_map.get(key).cloned().unwrap_or_else(|| default.to_string()),
            Err(_) => default.to_string(),
        }
    }
}

/// Trading configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TradingConfig {
    /// Enable real trading (vs simulation)
    pub enabled: bool,
    /// Maximum trade size in SOL
    pub max_trade_size_sol: f64,
    /// Minimum profit threshold in basis points
    pub min_profit_bps: u16,
    /// Maximum slippage in basis points
    pub max_slippage_bps: u16,
    /// Timeout for trades in seconds
    pub trade_timeout_seconds: u64,
}

/// Security configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SecurityConfig {
    /// Wallet file path
    pub wallet_path: String,
    /// RPC endpoint URL
    pub rpc_url: String,
    /// Enable transaction verification
    pub verify_transactions: bool,
    /// Maximum concurrent transactions
    pub max_concurrent_tx: u32,
}

/// API configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ApiConfig {
    /// Jupiter API configuration
    pub jupiter: JupiterConfig,
    /// Orca API configuration
    pub orca: OrcaConfig,
    /// General API settings
    pub general: GeneralApiConfig,
}

/// Jupiter API configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct JupiterConfig {
    /// Enable Jupiter integration
    pub enabled: bool,
    /// API base URL
    pub base_url: String,
    /// Rate limit (requests per second)
    pub rate_limit_rps: u32,
    /// Request timeout in seconds
    pub timeout_seconds: u64,
}

/// Orca API configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OrcaConfig {
    /// Enable Orca integration
    pub enabled: bool,
    /// API base URL
    pub base_url: String,
    /// Rate limit (requests per second)
    pub rate_limit_rps: u32,
}

/// General API configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GeneralApiConfig {
    /// User agent for HTTP requests
    pub user_agent: String,
    /// Default timeout for API calls
    pub default_timeout_seconds: u64,
    /// Maximum retry attempts
    pub max_retries: u32,
}

/// Performance configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PerformanceConfig {
    /// Number of worker threads
    pub worker_threads: u32,
    /// Enable performance monitoring
    pub monitoring_enabled: bool,
    /// Metrics collection interval in seconds
    pub metrics_interval_seconds: u64,
}

impl Default for SniperForgeConfig {
    fn default() -> Self {
        Self {
            trading: TradingConfig {
                enabled: false, // Safety: disabled by default
                max_trade_size_sol: 0.1,
                min_profit_bps: 50, // 0.5%
                max_slippage_bps: 100, // 1%
                trade_timeout_seconds: 30,
            },
            security: SecurityConfig {
                wallet_path: "./wallet.json".to_string(),
                rpc_url: "https://api.mainnet-beta.solana.com".to_string(),
                verify_transactions: true,
                max_concurrent_tx: 5,
            },
            apis: ApiConfig {
                jupiter: JupiterConfig {
                    enabled: true,
                    base_url: "https://quote-api.jup.ag/v6".to_string(),
                    rate_limit_rps: 10,
                    timeout_seconds: 10,
                },
                orca: OrcaConfig {
                    enabled: true,
                    base_url: "https://api.orca.so".to_string(),
                    rate_limit_rps: 5,
                },
                general: GeneralApiConfig {
                    user_agent: "SniperForge/1.0".to_string(),
                    default_timeout_seconds: 10,
                    max_retries: 3,
                },
            },
            performance: PerformanceConfig {
                worker_threads: 4,
                monitoring_enabled: true,
                metrics_interval_seconds: 60,
            },
        }
    }
}

impl SniperForgeConfig {
    /// Load configuration from file
    pub fn load_from_file<P: AsRef<Path>>(path: P) -> Result<Self> {
        let content = std::fs::read_to_string(path)
            .map_err(|e| crate::types::SniperForgeError::Config(format!("Failed to read config file: {}", e)))?;
        
        let config: Self = serde_json::from_str(&content)
            .map_err(|e| crate::types::SniperForgeError::Config(format!("Failed to parse config: {}", e)))?;
        
        config.validate()?;
        Ok(config)
    }
    
    /// Save configuration to file
    pub fn save_to_file<P: AsRef<Path>>(&self, path: P) -> Result<()> {
        let content = serde_json::to_string_pretty(self)
            .map_err(|e| crate::types::SniperForgeError::Config(format!("Failed to serialize config: {}", e)))?;
        
        std::fs::write(path, content)
            .map_err(|e| crate::types::SniperForgeError::Config(format!("Failed to write config file: {}", e)))?;
        
        Ok(())
    }
    
    /// Validate configuration
    pub fn validate(&self) -> Result<()> {
        // Validate trading config
        if self.trading.max_trade_size_sol <= 0.0 {
            return Err(crate::types::SniperForgeError::Config(
                "max_trade_size_sol must be positive".to_string()
            ));
        }
        
        if self.trading.min_profit_bps == 0 {
            return Err(crate::types::SniperForgeError::Config(
                "min_profit_bps must be positive".to_string()
            ));
        }
        
        // Validate security config
        if self.security.wallet_path.is_empty() {
            return Err(crate::types::SniperForgeError::Config(
                "wallet_path cannot be empty".to_string()
            ));
        }
        
        if self.security.rpc_url.is_empty() {
            return Err(crate::types::SniperForgeError::Config(
                "rpc_url cannot be empty".to_string()
            ));
        }
        
        Ok(())
    }
    
    /// Create a safe configuration for production
    pub fn production_safe() -> Self {
        let mut config = Self::default();
        config.trading.enabled = false; // Extra safety
        config.trading.max_trade_size_sol = 0.01; // Very conservative
        config.trading.min_profit_bps = 100; // 1% minimum
        config
    }
}
