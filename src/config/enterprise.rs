use serde::{Deserialize, Serialize};
use std::{env, path::PathBuf};
use crate::errors::{SniperForgeError, SniperResult};

/// Enterprise-grade configuration management system
/// Supports environment variables, validation, and secure defaults

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EnterpriseConfig {
    /// Solana blockchain configuration
    pub solana: SolanaConfig,
    
    /// API credentials and endpoints
    pub apis: ApiConfig,
    
    /// Trading parameters and limits
    pub trading: TradingConfig,
    
    /// Security and wallet configuration
    pub security: SecurityConfig,
    
    /// System and operational settings
    pub system: SystemConfig,
    
    /// Monitoring and alerting configuration
    pub monitoring: MonitoringConfig,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SolanaConfig {
    pub rpc_url: String,
    pub ws_url: String,
    pub commitment_level: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ApiConfig {
    pub helius_api_key: String,
    pub helius_rpc_url: String,
    pub jupiter_api_url: String,
    pub dexscreener_api_url: String,
    pub pyth_api_url: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TradingConfig {
    pub max_slippage: f64,
    pub min_profit_threshold: f64,
    pub max_position_size: f64,
    pub max_position_size_usd: f64,
    pub max_requests_per_second: u32,
    pub cooldown_period_ms: u64,
    pub max_history_size: usize,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SecurityConfig {
    pub private_key_path: PathBuf,
    pub wallet_password: Option<String>,
    pub enable_simulation: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SystemConfig {
    pub log_level: String,
    pub log_file_path: Option<PathBuf>,
    pub metrics_port: u16,
    pub health_check_port: u16,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MonitoringConfig {
    pub prometheus_enabled: bool,
    pub prometheus_port: u16,
    pub alert_webhook_url: Option<String>,
    pub alert_email: Option<String>,
}

impl EnterpriseConfig {
    /// Load configuration from environment variables
    pub fn from_env() -> SniperResult<Self> {
        // Load .env file if it exists
        if let Err(_) = dotenv::dotenv() {
            tracing::warn!("No .env file found, using system environment variables only");
        }

        let config = EnterpriseConfig {
            solana: SolanaConfig {
                rpc_url: env::var("SOLANA_RPC_URL")
                    .unwrap_or_else(|_| "https://api.devnet.solana.com".to_string()),
                ws_url: env::var("SOLANA_WS_URL")
                    .unwrap_or_else(|_| "wss://api.devnet.solana.com".to_string()),
                commitment_level: env::var("SOLANA_COMMITMENT_LEVEL")
                    .unwrap_or_else(|_| "confirmed".to_string()),
            },
            apis: ApiConfig {
                helius_api_key: env::var("HELIUS_API_KEY")
                    .map_err(|_| SniperForgeError::config("HELIUS_API_KEY is required"))?,
                helius_rpc_url: env::var("HELIUS_RPC_URL")
                    .unwrap_or_else(|_| "https://devnet.helius-rpc.com/?api-key=".to_string()),
                jupiter_api_url: env::var("JUPITER_API_URL")
                    .unwrap_or_else(|_| "https://quote-api.jup.ag/v6".to_string()),
                dexscreener_api_url: env::var("DEXSCREENER_API_URL")
                    .unwrap_or_else(|_| "https://api.dexscreener.com/latest".to_string()),
                pyth_api_url: env::var("PYTH_API_URL")
                    .unwrap_or_else(|_| "https://hermes.pyth.network/api".to_string()),
            },
            trading: TradingConfig {
                max_slippage: parse_env_f64("MAX_SLIPPAGE", 0.005)?,
                min_profit_threshold: parse_env_f64("MIN_PROFIT_THRESHOLD", 0.001)?,
                max_position_size: parse_env_f64("MAX_POSITION_SIZE", 0.1)?,
                max_position_size_usd: parse_env_f64("MAX_POSITION_SIZE_USD", 1000.0)?,
                max_requests_per_second: parse_env_u32("MAX_REQUESTS_PER_SECOND", 50)?,
                cooldown_period_ms: parse_env_u64("COOLDOWN_PERIOD_MS", 20)?,
                max_history_size: parse_env_usize("MAX_HISTORY_SIZE", 10000)?,
            },
            security: SecurityConfig {
                private_key_path: PathBuf::from(
                    env::var("PRIVATE_KEY_PATH")
                        .unwrap_or_else(|_| "./config/wallet.json".to_string())
                ),
                wallet_password: env::var("WALLET_PASSWORD").ok(),
                enable_simulation: parse_env_bool("ENABLE_SIMULATION", true)?,
            },
            system: SystemConfig {
                log_level: env::var("LOG_LEVEL")
                    .unwrap_or_else(|_| "info".to_string()),
                log_file_path: env::var("LOG_FILE_PATH")
                    .ok()
                    .map(PathBuf::from),
                metrics_port: parse_env_u16("METRICS_PORT", 8080)?,
                health_check_port: parse_env_u16("HEALTH_CHECK_PORT", 8081)?,
            },
            monitoring: MonitoringConfig {
                prometheus_enabled: parse_env_bool("PROMETHEUS_ENABLED", true)?,
                prometheus_port: parse_env_u16("PROMETHEUS_PORT", 9090)?,
                alert_webhook_url: env::var("ALERT_WEBHOOK_URL").ok(),
                alert_email: env::var("ALERT_EMAIL").ok(),
            },
        };

        config.validate()?;
        Ok(config)
    }

    /// Validate configuration parameters
    pub fn validate(&self) -> SniperResult<()> {
        // Validate trading parameters
        if self.trading.max_slippage <= 0.0 || self.trading.max_slippage > 1.0 {
            return Err(SniperForgeError::config(
                "max_slippage must be between 0 and 1"
            ));
        }

        if self.trading.min_profit_threshold <= 0.0 {
            return Err(SniperForgeError::config(
                "min_profit_threshold must be positive"
            ));
        }

        if self.trading.max_position_size <= 0.0 {
            return Err(SniperForgeError::config(
                "max_position_size must be positive"
            ));
        }

        if self.trading.max_requests_per_second == 0 {
            return Err(SniperForgeError::config(
                "max_requests_per_second must be positive"
            ));
        }

        // Validate URLs
        if !self.solana.rpc_url.starts_with("http") {
            return Err(SniperForgeError::config(
                "Invalid Solana RPC URL format"
            ));
        }

        if !self.solana.ws_url.starts_with("ws") {
            return Err(SniperForgeError::config(
                "Invalid Solana WebSocket URL format"
            ));
        }

        // Validate commitment level
        match self.solana.commitment_level.as_str() {
            "processed" | "confirmed" | "finalized" => {},
            _ => return Err(SniperForgeError::config(
                "Invalid commitment level, must be: processed, confirmed, or finalized"
            )),
        }

        // Validate log level
        match self.system.log_level.as_str() {
            "trace" | "debug" | "info" | "warn" | "error" => {},
            _ => return Err(SniperForgeError::config(
                "Invalid log level, must be one of: trace, debug, info, warn, error"
            )),
        }

        // Validate wallet file exists if not in simulation mode
        if !self.security.enable_simulation && !self.security.private_key_path.exists() {
            return Err(SniperForgeError::config(
                format!("Wallet file not found: {:?}", self.security.private_key_path)
            ));
        }

        Ok(())
    }

    /// Get Helius RPC URL with API key
    pub fn get_helius_url(&self) -> String {
        format!("{}{}", self.apis.helius_rpc_url, self.apis.helius_api_key)
    }
}

/// Helper functions for parsing environment variables

fn parse_env_f64(key: &str, default: f64) -> SniperResult<f64> {
    env::var(key)
        .map(|v| v.parse::<f64>()
            .map_err(|_| SniperForgeError::config(format!("Invalid {} value: {}", key, v))))
        .unwrap_or(Ok(default))
}

fn parse_env_u32(key: &str, default: u32) -> SniperResult<u32> {
    env::var(key)
        .map(|v| v.parse::<u32>()
            .map_err(|_| SniperForgeError::config(format!("Invalid {} value: {}", key, v))))
        .unwrap_or(Ok(default))
}

fn parse_env_u64(key: &str, default: u64) -> SniperResult<u64> {
    env::var(key)
        .map(|v| v.parse::<u64>()
            .map_err(|_| SniperForgeError::config(format!("Invalid {} value: {}", key, v))))
        .unwrap_or(Ok(default))
}

fn parse_env_u16(key: &str, default: u16) -> SniperResult<u16> {
    env::var(key)
        .map(|v| v.parse::<u16>()
            .map_err(|_| SniperForgeError::config(format!("Invalid {} value: {}", key, v))))
        .unwrap_or(Ok(default))
}

fn parse_env_usize(key: &str, default: usize) -> SniperResult<usize> {
    env::var(key)
        .map(|v| v.parse::<usize>()
            .map_err(|_| SniperForgeError::config(format!("Invalid {} value: {}", key, v))))
        .unwrap_or(Ok(default))
}

fn parse_env_bool(key: &str, default: bool) -> SniperResult<bool> {
    env::var(key)
        .map(|v| match v.to_lowercase().as_str() {
            "true" | "1" | "yes" | "on" => Ok(true),
            "false" | "0" | "no" | "off" => Ok(false),
            _ => Err(SniperForgeError::config(format!("Invalid {} value: {}", key, v)))
        })
        .unwrap_or(Ok(default))
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::env;

    #[test]
    fn test_config_validation() {
        let mut config = create_test_config();
        assert!(config.validate().is_ok());

        // Test invalid slippage
        config.trading.max_slippage = 1.5;
        assert!(config.validate().is_err());
    }

    #[test]
    fn test_parse_env_values() {
        env::set_var("TEST_F64", "0.5");
        env::set_var("TEST_BOOL", "true");
        
        assert_eq!(parse_env_f64("TEST_F64", 0.0).unwrap(), 0.5);
        assert_eq!(parse_env_bool("TEST_BOOL", false).unwrap(), true);
        
        env::remove_var("TEST_F64");
        env::remove_var("TEST_BOOL");
    }

    fn create_test_config() -> EnterpriseConfig {
        EnterpriseConfig {
            solana: SolanaConfig {
                rpc_url: "https://api.devnet.solana.com".to_string(),
                ws_url: "wss://api.devnet.solana.com".to_string(),
                commitment_level: "confirmed".to_string(),
            },
            apis: ApiConfig {
                helius_api_key: "test_key".to_string(),
                helius_rpc_url: "https://test.com/".to_string(),
                jupiter_api_url: "https://jupiter.com".to_string(),
                dexscreener_api_url: "https://dexscreener.com".to_string(),
                pyth_api_url: "https://pyth.com".to_string(),
            },
            trading: TradingConfig {
                max_slippage: 0.005,
                min_profit_threshold: 0.001,
                max_position_size: 0.1,
                max_position_size_usd: 1000.0,
                max_requests_per_second: 50,
                cooldown_period_ms: 20,
                max_history_size: 10000,
            },
            security: SecurityConfig {
                private_key_path: PathBuf::from("./test_wallet.json"),
                wallet_password: None,
                enable_simulation: true,
            },
            system: SystemConfig {
                log_level: "info".to_string(),
                log_file_path: None,
                metrics_port: 8080,
                health_check_port: 8081,
            },
            monitoring: MonitoringConfig {
                prometheus_enabled: true,
                prometheus_port: 9090,
                alert_webhook_url: None,
                alert_email: None,
            },
        }
    }
}
