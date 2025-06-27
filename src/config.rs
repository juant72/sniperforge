use anyhow::Result;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::Path;

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct Config {
    pub platform: PlatformConfig,
    pub trading: Option<TradingConfig>,  // New for Sprint 1.5
    pub wallets: Option<WalletsConfig>,  // New for Sprint 1.5
    pub network: NetworkConfig,
    pub shared_services: SharedServicesConfig,
    pub security: SecurityConfig,
    pub logging: LoggingConfig,
    pub bots: BotsConfig,
    pub development: DevelopmentConfig,
    pub performance: PerformanceConfig,
    pub pool_detection: Option<PoolDetectionConfig>, // New for Phase 5B
    pub trading_session: Option<TradingSessionConfig>, // New for Phase 5B
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct PlatformConfig {
    pub name: String,
    pub version: String,
    pub max_concurrent_bots: usize,
    pub resource_allocation_strategy: String,
    pub event_bus_buffer_size: usize,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct NetworkConfig {
    pub environment: String,  // "devnet" or "mainnet"
    
    // Devnet configuration (optional - only needed in devnet configs)
    #[serde(default)]
    pub devnet_primary_rpc: Option<String>,
    #[serde(default)]
    pub devnet_backup_rpc: Option<Vec<String>>,
    #[serde(default)]
    pub devnet_websocket_url: Option<String>,
    
    // Mainnet configuration (optional - only needed in mainnet configs)
    #[serde(default)]
    pub mainnet_primary_rpc: Option<String>,
    #[serde(default)]
    pub mainnet_backup_rpc: Option<Vec<String>>,
    #[serde(default)]
    pub mainnet_websocket_url: Option<String>,
    
    // Connection settings
    pub connection_timeout_ms: u64,
    pub request_timeout_ms: u64,
}

impl NetworkConfig {
    /// Get the primary RPC URL for the current environment
    pub fn primary_rpc(&self) -> &str {
        match self.environment.as_str() {
            "devnet" => {
                self.devnet_primary_rpc.as_ref()
                    .map(|s| s.as_str())
                    .unwrap_or("https://api.devnet.solana.com")
            },
            "mainnet" => {
                self.mainnet_primary_rpc.as_ref()
                    .map(|s| s.as_str())
                    .unwrap_or("https://api.mainnet-beta.solana.com")
            },
            _ => "https://api.devnet.solana.com", // Default to devnet for safety
        }
    }
    
    /// Get backup RPC URLs for the current environment
    pub fn backup_rpc(&self) -> Vec<String> {
        match self.environment.as_str() {
            "devnet" => {
                self.devnet_backup_rpc.as_ref()
                    .cloned()
                    .unwrap_or_else(|| vec![
                        "https://devnet.helius-rpc.com".to_string(),
                        "https://rpc-devnet.hellomoon.io".to_string()
                    ])
            },
            "mainnet" => {
                self.mainnet_backup_rpc.as_ref()
                    .cloned()
                    .unwrap_or_else(|| vec![
                        "https://solana-api.projectserum.com".to_string(),
                        "https://rpc.ankr.com/solana".to_string()
                    ])
            },
            _ => vec![
                "https://devnet.helius-rpc.com".to_string(),
                "https://rpc-devnet.hellomoon.io".to_string()
            ], // Default to devnet for safety
        }
    }
    
    /// Get WebSocket URL for the current environment
    pub fn websocket_url(&self) -> &str {
        match self.environment.as_str() {
            "devnet" => {
                self.devnet_websocket_url.as_ref()
                    .map(|s| s.as_str())
                    .unwrap_or("wss://api.devnet.solana.com")
            },
            "mainnet" => {
                self.mainnet_websocket_url.as_ref()
                    .map(|s| s.as_str())
                    .unwrap_or("wss://api.mainnet-beta.solana.com")
            },
            _ => "wss://api.devnet.solana.com", // Default to devnet for safety
        }
    }
    
    /// Check if we're running on devnet
    pub fn is_devnet(&self) -> bool {
        self.environment == "devnet"
    }
    
    /// Check if we're running on mainnet
    pub fn is_mainnet(&self) -> bool {
        self.environment == "mainnet"
    }
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct SharedServicesConfig {
    pub rpc_pool_size: usize,
    pub wallet_isolation: bool,
    pub data_feed_aggregation: bool,
    pub monitoring_enabled: bool,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct SecurityConfig {
    pub wallet_encryption_enabled: bool,
    pub key_derivation_iterations: u32,
    pub session_timeout_minutes: u32,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct LoggingConfig {
    pub level: String,
    pub file_enabled: bool,
    pub console_enabled: bool,
    pub max_file_size_mb: u64,
    pub max_files: u32,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct BotsConfig {
    pub lp_sniper: BotConfig,
    pub copy_trading: BotConfig,
    pub arbitrage: BotConfig,
    pub mev: BotConfig,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct BotConfig {
    pub enabled: bool,
    pub priority: String,
    pub resource_allocation: ResourceAllocation,
    #[serde(flatten)]
    pub specific_config: HashMap<String, toml::Value>,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct ResourceAllocation {
    pub cpu: String,
    pub memory: String,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct DevelopmentConfig {
    pub simulate_trades: bool,
    pub use_devnet: bool,
    pub verbose_logging: bool,
    pub enable_debug_mode: bool,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct PerformanceConfig {
    pub target_latency_ms: u64,
    pub max_memory_usage_mb: u64,
    pub gc_interval_seconds: u64,
    pub metrics_collection_interval_ms: u64,
}

// New configurations for Sprint 1.5
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct TradingConfig {
    pub mode: String,  // "hybrid", "devnet_only", "mainnet_only"
    pub devnet_real_trading: bool,
    pub mainnet_paper_trading: bool,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct WalletsConfig {
    pub auto_generate: bool,
    pub auto_airdrop_devnet: bool,
    pub devnet_airdrop_amount: f64,
    pub devnet: WalletEnvironmentConfig,
    pub mainnet: WalletEnvironmentConfig,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct WalletEnvironmentConfig {
    pub enabled: bool,
    pub real_trading: Option<bool>,
    pub paper_trading: Option<bool>,
    pub initial_balance_sol: Option<f64>,
    pub virtual_balance_sol: Option<f64>,
    pub max_trade_amount_sol: f64,
}

// New configurations for Phase 5B - Pool Detection
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct PoolDetectionConfig {
    pub min_liquidity_usd: Option<f64>,
    pub max_price_impact_1k: Option<f64>,
    pub min_risk_score: Option<f64>,
    pub monitoring_interval_ms: Option<u64>,
    pub max_tracked_pools: Option<u64>,
    pub min_profit_threshold_usd: Option<f64>,
    pub min_confidence_score: Option<f64>,
    pub max_execution_time_ms: Option<u64>,
    pub enable_new_pool_detection: Option<bool>,
    pub enable_websocket_triggers: Option<bool>,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct TradingSessionConfig {
    pub default_max_capital: Option<f64>,
    pub default_max_trade: Option<f64>,
    pub default_daily_limit: Option<f64>,
    pub default_duration_minutes: Option<u64>,
}

impl Config {
    pub fn load<P: AsRef<Path>>(path: P) -> Result<Self> {
        let content = std::fs::read_to_string(path)?;
        let config: Config = toml::from_str(&content)?;
        Ok(config)
    }
    
    pub fn save<P: AsRef<Path>>(&self, path: P) -> Result<()> {
        let content = toml::to_string_pretty(self)?;
        std::fs::write(path, content)?;
        Ok(())
    }
    
    pub fn is_bot_enabled(&self, bot_type: &str) -> bool {
        match bot_type {
            "lp_sniper" | "lp-sniper" => self.bots.lp_sniper.enabled,
            "copy_trading" | "copy-trading" => self.bots.copy_trading.enabled,
            "arbitrage" => self.bots.arbitrage.enabled,
            "mev" => self.bots.mev.enabled,
            _ => false,
        }
    }
    
    pub fn get_bot_config(&self, bot_type: &str) -> Option<&BotConfig> {
        match bot_type {
            "lp_sniper" | "lp-sniper" => Some(&self.bots.lp_sniper),
            "copy_trading" | "copy-trading" => Some(&self.bots.copy_trading),
            "arbitrage" => Some(&self.bots.arbitrage),
            "mev" => Some(&self.bots.mev),
            _ => None,
        }
    }
    
    /// Check if devnet real trading is enabled
    pub fn is_devnet_real_trading_enabled(&self) -> bool {
        self.trading.as_ref()
            .map(|t| t.devnet_real_trading)
            .unwrap_or(false)
    }
    
    /// Check if mainnet paper trading is enabled
    pub fn is_mainnet_paper_trading_enabled(&self) -> bool {
        self.trading.as_ref()
            .map(|t| t.mainnet_paper_trading)
            .unwrap_or(false)
    }
    
    /// Get devnet wallet config
    pub fn devnet_wallet_config(&self) -> Option<&WalletEnvironmentConfig> {
        self.wallets.as_ref().map(|w| &w.devnet)
    }
    
    /// Get mainnet wallet config
    pub fn mainnet_wallet_config(&self) -> Option<&WalletEnvironmentConfig> {
        self.wallets.as_ref().map(|w| &w.mainnet)
    }
    
    pub fn validate(&self) -> Result<()> {        // Validate RPC URLs
        if self.network.primary_rpc().is_empty() {
            return Err(anyhow::anyhow!("Primary RPC URL cannot be empty"));
        }
        
        // Validate resource allocations don't exceed 100%
        let mut total_cpu = 0.0;
        let mut total_memory = 0.0;
        
        for (_bot_name, bot_config) in [
            ("lp_sniper", &self.bots.lp_sniper),
            ("copy_trading", &self.bots.copy_trading),
            ("arbitrage", &self.bots.arbitrage),
            ("mev", &self.bots.mev),
        ] {
            if bot_config.enabled {
                // Parse CPU percentage
                let cpu_str = &bot_config.resource_allocation.cpu;
                if let Some(cpu_percent) = cpu_str.strip_suffix('%') {
                    if let Ok(cpu) = cpu_percent.parse::<f64>() {
                        total_cpu += cpu;
                    }
                }
                
                // Parse memory in GB
                let mem_str = &bot_config.resource_allocation.memory;
                if let Some(mem_gb) = mem_str.strip_suffix("GB") {
                    if let Ok(memory) = mem_gb.parse::<f64>() {
                        total_memory += memory;
                    }
                }
            }
        }
        
        if total_cpu > 100.0 {
            return Err(anyhow::anyhow!(
                "Total CPU allocation ({:.1}%) exceeds 100%", 
                total_cpu
            ));
        }
        
        if total_memory > 8.0 {  // Assuming 8GB max
            tracing::warn!(
                "High memory allocation: {:.1}GB. Ensure system has sufficient RAM", 
                total_memory
            );
        }
        
        Ok(())
    }
}

impl Default for Config {
    fn default() -> Self {
        Self {
            platform: PlatformConfig {
                name: "SniperForge".to_string(),
                version: "0.1.0".to_string(),
                max_concurrent_bots: 5,
                resource_allocation_strategy: "priority_based".to_string(),
                event_bus_buffer_size: 10000,
            },            network: NetworkConfig {
                environment: "devnet".to_string(),
                devnet_primary_rpc: Some("https://api.devnet.solana.com".to_string()),
                devnet_backup_rpc: Some(vec![
                    "https://devnet.helius-rpc.com".to_string(),
                    "https://rpc-devnet.hellomoon.io".to_string(),
                ]),
                devnet_websocket_url: Some("wss://api.devnet.solana.com".to_string()),
                mainnet_primary_rpc: Some("https://api.mainnet-beta.solana.com".to_string()),
                mainnet_backup_rpc: Some(vec![
                    "https://solana-api.projectserum.com".to_string(),
                    "https://rpc.ankr.com/solana".to_string(),
                ]),
                mainnet_websocket_url: Some("wss://api.mainnet-beta.solana.com".to_string()),
                connection_timeout_ms: 5000,
                request_timeout_ms: 10000,
            },
            shared_services: SharedServicesConfig {
                rpc_pool_size: 20,
                wallet_isolation: true,
                data_feed_aggregation: true,
                monitoring_enabled: true,
            },
            security: SecurityConfig {
                wallet_encryption_enabled: true,
                key_derivation_iterations: 100000,
                session_timeout_minutes: 30,
            },
            logging: LoggingConfig {
                level: "info".to_string(),
                file_enabled: true,
                console_enabled: true,
                max_file_size_mb: 100,
                max_files: 10,
            },
            bots: BotsConfig {
                lp_sniper: BotConfig {
                    enabled: true,
                    priority: "high".to_string(),
                    resource_allocation: ResourceAllocation {
                        cpu: "40%".to_string(),
                        memory: "1GB".to_string(),
                    },
                    specific_config: HashMap::new(),
                },
                copy_trading: BotConfig {
                    enabled: false,
                    priority: "medium".to_string(),
                    resource_allocation: ResourceAllocation {
                        cpu: "20%".to_string(),
                        memory: "512MB".to_string(),
                    },
                    specific_config: HashMap::new(),
                },
                arbitrage: BotConfig {
                    enabled: false,
                    priority: "high".to_string(),
                    resource_allocation: ResourceAllocation {
                        cpu: "25%".to_string(),
                        memory: "512MB".to_string(),
                    },
                    specific_config: HashMap::new(),
                },
                mev: BotConfig {
                    enabled: false,
                    priority: "critical".to_string(),
                    resource_allocation: ResourceAllocation {
                        cpu: "30%".to_string(),
                        memory: "1GB".to_string(),
                    },
                    specific_config: HashMap::new(),
                },
            },
            development: DevelopmentConfig {
                simulate_trades: true,
                use_devnet: false,
                verbose_logging: true,
                enable_debug_mode: true,
            },
            performance: PerformanceConfig {
                target_latency_ms: 50,
                max_memory_usage_mb: 2048,
                gc_interval_seconds: 60,
                metrics_collection_interval_ms: 1000,
            },
            trading: Some(TradingConfig {
                mode: "hybrid".to_string(),
                devnet_real_trading: true,
                mainnet_paper_trading: false,
            }),
            wallets: Some(WalletsConfig {
                auto_generate: true,
                auto_airdrop_devnet: true,
                devnet_airdrop_amount: 2.0,
                devnet: WalletEnvironmentConfig {
                    enabled: true,
                    real_trading: Some(true),
                    paper_trading: Some(false),
                    initial_balance_sol: Some(1.0),
                    virtual_balance_sol: Some(0.5),
                    max_trade_amount_sol: 0.1,
                },
                mainnet: WalletEnvironmentConfig {
                    enabled: true,
                    real_trading: Some(true),
                    paper_trading: Some(false),
                    initial_balance_sol: Some(0.5),
                    virtual_balance_sol: Some(0.1),
                    max_trade_amount_sol: 0.01,
                },            }),
            pool_detection: None, // Will be loaded from config file
            trading_session: None, // Will be loaded from config file
        }
    }
}
