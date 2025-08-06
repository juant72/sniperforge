//! Bot Interface - Core abstraction for containerized bots
//! 
//! This module defines the standard interface that all bots must implement
//! for the containerized ecosystem management.

use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use uuid::Uuid;
use chrono::{DateTime, Utc};
use anyhow::Result;

/// Core bot interface that all trading bots must implement
#[async_trait]
pub trait BotInterface: Send + Sync {
    /// Unique bot identifier
    fn bot_id(&self) -> Uuid;
    
    /// Bot type classification
    fn bot_type(&self) -> BotType;
    
    /// Bot version for compatibility tracking
    fn version(&self) -> String;
    
    /// Current bot status
    async fn status(&self) -> BotStatus;
    
    /// Start bot with configuration
    async fn start(&mut self, config: BotConfig) -> Result<(), BotError>;
    
    /// Stop bot gracefully
    async fn stop(&mut self) -> Result<(), BotError>;
    
    /// Pause bot operations (temporary)
    async fn pause(&mut self) -> Result<(), BotError>;
    
    /// Resume bot operations
    async fn resume(&mut self) -> Result<(), BotError>;
    
    /// Update configuration with hot-reload
    async fn update_config(&mut self, config: BotConfig) -> Result<(), BotError>;
    
    /// Get current performance metrics
    async fn metrics(&self) -> BotMetrics;
    
    /// Health check for container orchestration
    async fn health_check(&self) -> HealthStatus;
    
    /// Get bot capabilities and supported features
    fn capabilities(&self) -> BotCapabilities;
    
    /// Validate configuration before applying
    async fn validate_config(&self, config: &BotConfig) -> Result<ValidationResult, BotError>;
}

/// Bot type enumeration for the ecosystem
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Hash)]
pub enum BotType {
    /// Enhanced arbitrage with ML analysis
    EnhancedArbitrage,
    /// Triangular arbitrage detection
    TriangularArbitrage,
    /// Flash loan arbitrage opportunities
    FlashLoanArbitrage,
    /// Cross-chain arbitrage
    CrossChainArbitrage,
    /// ML analytics and predictions
    MLAnalytics,
    /// Portfolio management and optimization
    PortfolioManager,
    /// Real-time dashboard and monitoring
    RealTimeDashboard,
    /// Performance profiling and optimization
    PerformanceProfiler,
    /// Pattern analysis and recognition
    PatternAnalyzer,
}

impl BotType {
    /// Convert bot type to string representation for filenames and display
    pub fn as_str(&self) -> &'static str {
        match self {
            BotType::EnhancedArbitrage => "enhanced-arbitrage",
            BotType::TriangularArbitrage => "triangular-arbitrage",
            BotType::FlashLoanArbitrage => "flash-loan-arbitrage",
            BotType::CrossChainArbitrage => "cross-chain-arbitrage",
            BotType::MLAnalytics => "ml-analytics",
            BotType::PortfolioManager => "portfolio-manager",
            BotType::RealTimeDashboard => "real-time-dashboard",
            BotType::PerformanceProfiler => "performance-profiler",
            BotType::PatternAnalyzer => "pattern-analyzer",
        }
    }
}

/// Bot operational status
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum BotStatus {
    /// Bot is initializing
    Initializing,
    /// Bot is running normally
    Running,
    /// Bot is paused (temporary stop)
    Paused,
    /// Bot is stopping gracefully
    Stopping,
    /// Bot is stopped
    Stopped,
    /// Bot encountered an error
    Error(String),
    /// Bot is updating configuration
    ConfigurationUpdate,
}

/// Bot configuration structure
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BotConfig {
    /// Unique configuration ID
    pub config_id: Uuid,
    /// Bot instance ID this config applies to
    pub bot_id: Uuid,
    /// Bot type
    pub bot_type: BotType,
    /// Environment (dev, test, prod)
    pub environment: Environment,
    /// Dynamic parameters specific to bot type
    pub parameters: serde_json::Value,
    /// Resource limits for container
    pub resources: ResourceLimits,
    /// Network configuration
    pub network: NetworkConfig,
    /// Security settings
    pub security: SecurityConfig,
    /// Configuration metadata
    pub metadata: ConfigMetadata,
}

/// Environment classification
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum Environment {
    Development,
    Testing,
    Staging,
    Production,
    Mainnet,
    Testnet,
}

/// Resource limits for container orchestration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ResourceLimits {
    /// Maximum CPU usage (cores)
    pub max_cpu: f64,
    /// Maximum memory usage (MB)
    pub max_memory_mb: u64,
    /// Maximum disk usage (MB)
    pub max_disk_mb: u64,
    /// Network bandwidth limit (Mbps)
    pub max_network_mbps: Option<u32>,
}

/// Network configuration for bot
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NetworkConfig {
    /// Solana RPC endpoints
    pub solana_rpc_urls: Vec<String>,
    /// WebSocket endpoints
    pub websocket_urls: Vec<String>,
    /// API endpoints (Jupiter, DexScreener, etc.)
    pub api_endpoints: HashMap<String, String>,
    /// Network timeout settings
    pub timeouts: NetworkTimeouts,
}

/// Network timeout configurations
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NetworkTimeouts {
    /// RPC request timeout (seconds)
    pub rpc_timeout_seconds: u64,
    /// WebSocket connection timeout (seconds)
    pub websocket_timeout_seconds: u64,
    /// API request timeout (seconds)
    pub api_timeout_seconds: u64,
}

/// Security configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SecurityConfig {
    /// Wallet configuration
    pub wallet: WalletConfig,
    /// API key management
    pub api_keys: HashMap<String, String>,
    /// Encryption settings
    pub encryption_enabled: bool,
    /// Authentication settings
    pub auth_required: bool,
}

/// Wallet configuration for trading bots
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WalletConfig {
    /// Wallet type (keypair, hardware, etc.)
    pub wallet_type: String,
    /// Wallet address
    pub address: String,
    /// Encrypted private key path (if applicable)
    pub private_key_path: Option<String>,
    /// Whether to use environment variables for keys
    pub use_env_keys: bool,
}

/// Configuration metadata
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConfigMetadata {
    /// Configuration name/description
    pub name: String,
    /// Configuration version
    pub version: String,
    /// Creation timestamp
    pub created_at: DateTime<Utc>,
    /// Last update timestamp
    pub updated_at: DateTime<Utc>,
    /// Creator/owner
    pub created_by: String,
    /// Configuration tags
    pub tags: Vec<String>,
}

/// Bot performance metrics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BotMetrics {
    /// Basic operational metrics
    pub operational: OperationalMetrics,
    /// Trading-specific metrics
    pub trading: TradingMetrics,
    /// System performance metrics
    pub performance: PerformanceMetrics,
    /// Custom metrics specific to bot type
    pub custom: serde_json::Value,
    /// Metrics timestamp
    pub timestamp: DateTime<Utc>,
}

/// Operational metrics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OperationalMetrics {
    /// Bot uptime in seconds
    pub uptime_seconds: u64,
    /// Number of restarts
    pub restart_count: u32,
    /// Last restart timestamp
    pub last_restart: Option<DateTime<Utc>>,
    /// Configuration update count
    pub config_updates: u32,
    /// Error count
    pub error_count: u32,
}

/// Trading performance metrics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TradingMetrics {
    /// Total trades executed
    pub trades_executed: u64,
    /// Successful trades
    pub successful_trades: u64,
    /// Total profit/loss in USD
    pub total_pnl_usd: f64,
    /// Success rate percentage
    pub success_rate: f64,
    /// Average profit per trade
    pub avg_profit_per_trade: f64,
    /// Total volume traded
    pub total_volume_usd: f64,
    /// Sharpe ratio (if applicable)
    pub sharpe_ratio: Option<f64>,
}

/// System performance metrics
/// ✅ ENRIQUECIMIENTO: Comprehensive trade metrics for bot performance tracking
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct TradeMetrics {
    /// Total number of trades executed
    pub total_trades: u64,
    /// Total trading volume in USD
    pub total_volume_usd: f64,
    /// Total profit/loss in USD
    pub total_pnl_usd: f64,
    /// Successful trades count
    pub successful_trades: u64,
    /// Failed trades count
    pub failed_trades: u64,
    /// Average trade size in USD
    pub avg_trade_size_usd: f64,
    /// Best trade profit in USD
    pub best_trade_usd: f64,
    /// Worst trade loss in USD
    pub worst_trade_usd: f64,
    /// Total fees paid in USD
    pub total_fees_usd: f64,
    /// Win rate percentage (0-100)
    pub win_rate_percent: f64,
}

/// Performance metrics for bot monitoring and optimization
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PerformanceMetrics {
    /// Current CPU usage percentage
    pub cpu_usage_percent: f64,
    /// Current memory usage in MB
    pub memory_usage_mb: u64,
    /// Network I/O statistics
    pub network_io: NetworkIOMetrics,
    /// API call statistics
    pub api_calls: ApiCallMetrics,
    /// Average response time for operations
    pub avg_response_time_ms: f64,
    /// Operations per second throughput
    pub throughput_per_second: f64,
}

/// Network I/O metrics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NetworkIOMetrics {
    /// Bytes sent
    pub bytes_sent: u64,
    /// Bytes received
    pub bytes_received: u64,
    /// Packets sent
    pub packets_sent: u64,
    /// Packets received
    pub packets_received: u64,
}

/// API call metrics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ApiCallMetrics {
    /// Total API calls made
    pub total_calls: u64,
    /// Successful API calls
    pub successful_calls: u64,
    /// Failed API calls
    pub failed_calls: u64,
    /// Average API response time
    pub avg_response_time_ms: f64,
}

/// Health status for container orchestration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HealthStatus {
    /// Overall health status
    pub status: HealthLevel,
    /// Detailed health checks
    pub checks: Vec<HealthCheck>,
    /// Health check timestamp
    pub timestamp: DateTime<Utc>,
    /// Additional health information
    pub details: HashMap<String, serde_json::Value>,
}

/// Health level enumeration
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum HealthLevel {
    /// Bot is healthy and operating normally
    Healthy,
    /// Bot has minor issues but is functional
    Warning,
    /// Bot has serious issues affecting operation
    Critical,
    /// Bot is unhealthy and not functioning
    Unhealthy,
    /// Health status is unknown
    Unknown,
}

/// Individual health check result
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HealthCheck {
    /// Check name/identifier
    pub name: String,
    /// Check status
    pub status: HealthLevel,
    /// Check description
    pub description: String,
    /// Check execution time
    pub execution_time_ms: u64,
    /// Additional check data
    pub data: Option<serde_json::Value>,
}

/// Bot capabilities and features
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BotCapabilities {
    /// Supported trading networks
    pub networks: Vec<String>,
    /// Supported DEXs
    pub dexs: Vec<String>,
    /// Supported token types
    pub token_types: Vec<String>,
    /// Features supported by this bot
    pub features: Vec<BotFeature>,
    /// Configuration options available
    pub config_options: Vec<ConfigOption>,
}

/// Bot feature enumeration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum BotFeature {
    /// Real-time trading execution
    RealTimeTrading,
    /// Simulation/paper trading
    SimulationMode,
    /// Machine learning analysis
    MLAnalysis,
    /// Risk management
    RiskManagement,
    /// Performance analytics
    PerformanceAnalytics,
    /// Hot configuration reload
    HotConfigReload,
    /// Multi-DEX support
    MultiDexSupport,
    /// Cross-chain operations
    CrossChainSupport,
    /// Dashboard and monitoring
    Dashboard,
}

/// Configuration option definition
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConfigOption {
    /// Option name
    pub name: String,
    /// Option type (number, string, boolean, etc.)
    pub option_type: String,
    /// Default value
    pub default_value: serde_json::Value,
    /// Validation rules
    pub validation: Option<ValidationRules>,
    /// Option description
    pub description: String,
    /// Whether option is required
    pub required: bool,
}

/// Validation rules for configuration options
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ValidationRules {
    /// Minimum value (for numbers)
    pub min: Option<f64>,
    /// Maximum value (for numbers)
    pub max: Option<f64>,
    /// Allowed values (enum-like)
    pub allowed_values: Option<Vec<serde_json::Value>>,
    /// Regular expression pattern (for strings)
    pub pattern: Option<String>,
}

/// Configuration validation result
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ValidationResult {
    /// Whether configuration is valid
    pub is_valid: bool,
    /// Validation errors
    pub errors: Vec<ValidationError>,
    /// Validation warnings
    pub warnings: Vec<ValidationWarning>,
}

/// Configuration validation error
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ValidationError {
    /// Error field path
    pub field: String,
    /// Error message
    pub message: String,
    /// Error code
    pub code: String,
}

/// Configuration validation warning
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ValidationWarning {
    /// Warning field path
    pub field: String,
    /// Warning message
    pub message: String,
    /// Warning code
    pub code: String,
}

/// Bot error enumeration
#[derive(Debug, thiserror::Error)]
pub enum BotError {
    #[error("Configuration error: {0}")]
    Configuration(String),
    
    #[error("Network error: {0}")]
    Network(String),
    
    #[error("Trading error: {0}")]
    Trading(String),
    
    #[error("Security error: {0}")]
    Security(String),
    
    #[error("Resource error: {0}")]
    Resource(String),
    
    #[error("Internal error: {0}")]
    Internal(String),
    
    #[error("Invalid state: {0}")]
    InvalidState(String),
    
    #[error("Timeout error: {0}")]
    Timeout(String),
}

impl Default for ResourceLimits {
    fn default() -> Self {
        Self {
            max_cpu: 1.0,
            max_memory_mb: 512,
            max_disk_mb: 1024,
            max_network_mbps: None,
        }
    }
}

impl Default for NetworkTimeouts {
    fn default() -> Self {
        Self {
            rpc_timeout_seconds: 30,
            websocket_timeout_seconds: 60,
            api_timeout_seconds: 15,
        }
    }
}

impl Default for BotMetrics {
    fn default() -> Self {
        Self {
            operational: OperationalMetrics::default(),
            trading: TradingMetrics::default(),
            performance: PerformanceMetrics::default(),
            custom: serde_json::Value::Null,
            timestamp: chrono::Utc::now(),
        }
    }
}

impl Default for OperationalMetrics {
    fn default() -> Self {
        Self {
            uptime_seconds: 0,
            restart_count: 0,
            last_restart: None,
            config_updates: 0,
            error_count: 0,
        }
    }
}

impl Default for TradingMetrics {
    fn default() -> Self {
        Self {
            trades_executed: 0,
            successful_trades: 0,
            total_pnl_usd: 0.0,
            success_rate: 0.0,
            avg_profit_per_trade: 0.0,
            total_volume_usd: 0.0,
            sharpe_ratio: None,
        }
    }
}

impl Default for PerformanceMetrics {
    fn default() -> Self {
        Self {
            cpu_usage_percent: 0.0,
            memory_usage_mb: 0,
            network_io: NetworkIOMetrics::default(),
            api_calls: ApiCallMetrics::default(),
            avg_response_time_ms: 0.0,
            throughput_per_second: 0.0,
        }
    }
}

impl Default for NetworkIOMetrics {
    fn default() -> Self {
        Self {
            bytes_sent: 0,
            bytes_received: 0,
            packets_sent: 0,
            packets_received: 0,
        }
    }
}

impl Default for ApiCallMetrics {
    fn default() -> Self {
        Self {
            total_calls: 0,
            successful_calls: 0,
            failed_calls: 0,
            avg_response_time_ms: 0.0,
        }
    }
}

impl BotConfig {
    /// Create a default configuration for a specific bot ID
    pub fn default_for_id(bot_id: Uuid) -> Self {
        let now = Utc::now();
        
        Self {
            config_id: Uuid::new_v4(),
            bot_id,
            bot_type: BotType::EnhancedArbitrage, // Default to enhanced arbitrage
            environment: Environment::Development,
            parameters: serde_json::json!({
                "pairs": ["BTC/USDT", "ETH/USDT"],
                "exchanges": ["binance", "kraken"],
                "min_profit_threshold": 0.01,
                "max_position_size": 1000.0,
                "execution_timeout_ms": 5000
            }),
            resources: ResourceLimits {
                max_cpu: 1.0,
                max_memory_mb: 256,
                max_disk_mb: 128,
                max_network_mbps: Some(10),
            },
            network: NetworkConfig {
                solana_rpc_urls: vec![
                    "https://mainnet.helius-rpc.com/?api-key=062bf3dd-23d4-4ffd-99fd-6e397ee59d6c".to_string()
                ],
                websocket_urls: vec![],
                api_endpoints: HashMap::new(),
                timeouts: NetworkTimeouts {
                    rpc_timeout_seconds: 10,
                    websocket_timeout_seconds: 10,
                    api_timeout_seconds: 10,
                },
            },
            security: SecurityConfig {
                wallet: WalletConfig {
                    wallet_type: "keypair".to_string(),
                    address: "".to_string(),
                    private_key_path: None,
                    use_env_keys: true,
                },
                api_keys: HashMap::new(),
                encryption_enabled: true,
                auth_required: false,
            },
            metadata: ConfigMetadata {
                name: format!("Default Config for Bot {}", bot_id),
                version: "1.0.0".to_string(),
                created_at: now,
                updated_at: now,
                created_by: "System".to_string(),
                tags: vec!["default".to_string(), "auto-generated".to_string()],
            },
        }
    }
}
