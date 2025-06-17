use anyhow::Result;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::Path;

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct Config {
    pub platform: PlatformConfig,
    pub network: NetworkConfig,
    pub shared_services: SharedServicesConfig,
    pub security: SecurityConfig,
    pub logging: LoggingConfig,
    pub bots: BotsConfig,
    pub development: DevelopmentConfig,
    pub performance: PerformanceConfig,
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
    pub primary_rpc: String,
    pub backup_rpc: Vec<String>,
    pub websocket_url: String,
    pub connection_timeout_ms: u64,
    pub request_timeout_ms: u64,
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
    
    pub fn validate(&self) -> Result<()> {
        // Validate RPC URLs
        if self.network.primary_rpc.is_empty() {
            return Err(anyhow::anyhow!("Primary RPC URL cannot be empty"));
        }
        
        // Validate resource allocations don't exceed 100%
        let mut total_cpu = 0.0;
        let mut total_memory = 0.0;
        
        for (bot_name, bot_config) in [
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
            },
            network: NetworkConfig {
                primary_rpc: "https://api.mainnet-beta.solana.com".to_string(),
                backup_rpc: vec![
                    "https://solana-api.projectserum.com".to_string(),
                    "https://rpc.ankr.com/solana".to_string(),
                ],
                websocket_url: "wss://api.mainnet-beta.solana.com".to_string(),
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
        }
    }
}
