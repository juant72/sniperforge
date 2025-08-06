//! YAML Configuration Manager
//! 
//! Enterprise-grade configuration management with YAML support for better readability
//! and maintainability. Supports hot-reload, validation, and environment overrides.

use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::{Path, PathBuf};
use tokio::fs;
use tokio::sync::RwLock;
use uuid::Uuid;
use chrono::{DateTime, Utc};
use anyhow::Result;
use tracing::{info, error};

use crate::api::bot_interface::{BotType, Environment};

/// YAML Configuration management errors
#[derive(Debug, thiserror::Error)]
pub enum YamlConfigError {
    #[error("Configuration file not found: {0}")]
    FileNotFound(String),
    
    #[error("Invalid YAML format: {0}")]
    InvalidYamlFormat(String),
    
    #[error("Configuration validation failed: {0}")]
    ValidationFailed(String),
    
    #[error("Environment variable not found: {0}")]
    EnvVarNotFound(String),
    
    #[error("IO error: {0}")]
    IoError(#[from] std::io::Error),
    
    #[error("YAML parsing error: {0}")]
    YamlError(#[from] serde_yaml::Error),
}

/// System-wide configuration in YAML format
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SystemConfigYaml {
    /// Application metadata
    pub app: AppConfig,
    
    /// Server configuration
    pub server: ServerConfig,
    
    /// Database/persistence configuration
    pub persistence: PersistenceConfig,
    
    /// Security settings
    pub security: SecurityConfigYaml,
    
    /// Logging configuration
    pub logging: LoggingConfig,
    
    /// Bot default settings
    pub bot_defaults: BotDefaultsConfig,
    
    /// Environment-specific overrides
    pub environments: HashMap<String, EnvironmentOverrides>,
}

/// Application metadata
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AppConfig {
    pub name: String,
    pub version: String,
    pub description: String,
    pub author: String,
    pub license: String,
}

/// Server configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ServerConfig {
    /// TCP control server settings
    pub tcp: TcpServerConfig,
    
    /// HTTP API server settings
    pub http: HttpServerConfig,
    
    /// WebSocket server settings
    pub websocket: WebSocketConfig,
    
    /// Performance settings
    pub performance: PerformanceConfig,
}

/// TCP server configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TcpServerConfig {
    pub enabled: bool,
    pub host: String,
    pub port: u16,
    pub max_connections: u32,
    pub timeout_seconds: u64,
    pub buffer_size: usize,
}

/// HTTP server configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HttpServerConfig {
    pub enabled: bool,
    pub host: String,
    pub port: u16,
    pub cors_origins: Vec<String>,
    pub max_request_size_mb: usize,
    pub rate_limit_per_minute: u32,
}

/// WebSocket configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WebSocketConfig {
    pub enabled: bool,
    pub port: u16,
    pub heartbeat_interval_seconds: u64,
    pub max_frame_size_mb: usize,
}

/// Performance configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PerformanceConfig {
    pub worker_threads: Option<usize>,
    pub max_blocking_threads: Option<usize>,
    pub enable_profiling: bool,
    pub metrics_collection_interval_seconds: u64,
}

/// Persistence configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PersistenceConfig {
    pub enabled: bool,
    pub storage_type: StorageType,
    pub data_directory: String,
    pub backup_directory: String,
    pub auto_backup_enabled: bool,
    pub backup_interval_hours: u32,
    pub retention_days: u32,
    pub compression_enabled: bool,
}

/// Storage type options
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum StorageType {
    #[serde(rename = "file")]
    File,
    #[serde(rename = "sqlite")]
    Sqlite,
    #[serde(rename = "postgresql")]
    PostgreSQL,
    #[serde(rename = "redis")]
    Redis,
}

/// Security configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SecurityConfigYaml {
    pub encryption: EncryptionConfig,
    pub authentication: AuthConfig,
    pub api_security: ApiSecurityConfig,
    pub wallet_security: WalletSecurityConfig,
}

/// Encryption configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EncryptionConfig {
    pub enabled: bool,
    pub algorithm: String,
    pub key_rotation_days: u32,
    pub encrypt_at_rest: bool,
    pub encrypt_in_transit: bool,
}

/// Authentication configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AuthConfig {
    pub required: bool,
    pub method: AuthMethod,
    pub session_timeout_minutes: u32,
    pub max_failed_attempts: u32,
    pub lockout_duration_minutes: u32,
}

/// Authentication methods
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum AuthMethod {
    #[serde(rename = "api_key")]
    ApiKey,
    #[serde(rename = "jwt")]
    JWT,
    #[serde(rename = "oauth2")]
    OAuth2,
    #[serde(rename = "certificate")]
    Certificate,
}

/// API security configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ApiSecurityConfig {
    pub rate_limiting: bool,
    pub ip_whitelist: Vec<String>,
    pub require_https: bool,
    pub allowed_origins: Vec<String>,
}

/// Wallet security configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WalletSecurityConfig {
    pub use_env_keys: bool,
    pub key_derivation_method: String,
    pub multi_sig_enabled: bool,
    pub hardware_wallet_support: bool,
}

/// Logging configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LoggingConfig {
    pub level: String,
    pub format: LogFormat,
    pub output: Vec<LogOutput>,
    pub rotation: LogRotationConfig,
    pub filters: Vec<LogFilter>,
}

/// Log format options
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum LogFormat {
    #[serde(rename = "json")]
    Json,
    #[serde(rename = "pretty")]
    Pretty,
    #[serde(rename = "compact")]
    Compact,
}

/// Log output configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LogOutput {
    pub target: LogTarget,
    pub enabled: bool,
    pub level_override: Option<String>,
}

/// Log targets
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum LogTarget {
    #[serde(rename = "console")]
    Console,
    #[serde(rename = "file")]
    File { path: String },
    #[serde(rename = "syslog")]
    Syslog,
    #[serde(rename = "elasticsearch")]
    ElasticSearch { endpoint: String },
}

/// Log rotation configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LogRotationConfig {
    pub enabled: bool,
    pub max_size_mb: u64,
    pub max_files: u32,
    pub rotation_interval: RotationInterval,
}

/// Rotation intervals
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum RotationInterval {
    #[serde(rename = "hourly")]
    Hourly,
    #[serde(rename = "daily")]
    Daily,
    #[serde(rename = "weekly")]
    Weekly,
}

/// Log filters
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LogFilter {
    pub module: String,
    pub level: String,
}

/// Bot default configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BotDefaultsConfig {
    pub environment: Environment,
    pub resources: DefaultResourceLimits,
    pub network: DefaultNetworkConfig,
    pub security: DefaultSecurityConfig,
    pub timeouts: DefaultTimeouts,
}

/// Default resource limits
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DefaultResourceLimits {
    pub max_cpu_cores: f64,
    pub max_memory_mb: u64,
    pub max_disk_mb: u64,
    pub max_network_mbps: Option<u64>,
}

/// Default network configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DefaultNetworkConfig {
    pub solana_rpc_urls: Vec<String>,
    pub websocket_urls: Vec<String>,
    pub connection_pool_size: u32,
    pub request_timeout_seconds: u64,
    pub retry_attempts: u32,
}

/// Default security configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DefaultSecurityConfig {
    pub encryption_enabled: bool,
    pub auth_required: bool,
    pub use_env_keys: bool,
    pub validate_signatures: bool,
}

/// Default timeouts
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DefaultTimeouts {
    pub startup_timeout_seconds: u64,
    pub shutdown_timeout_seconds: u64,
    pub health_check_interval_seconds: u64,
    pub metrics_update_interval_seconds: u64,
}

/// Environment-specific overrides
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EnvironmentOverrides {
    pub server: Option<ServerConfig>,
    pub persistence: Option<PersistenceConfig>,
    pub security: Option<SecurityConfigYaml>,
    pub logging: Option<LoggingConfig>,
    pub bot_defaults: Option<BotDefaultsConfig>,
}

/// Bot-specific YAML configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BotConfigYaml {
    /// Bot metadata
    pub metadata: BotMetadata,
    
    /// Bot type and configuration
    pub bot: BotConfiguration,
    
    /// Resource allocation
    pub resources: BotResourceConfig,
    
    /// Network settings
    pub network: BotNetworkConfig,
    
    /// Security settings
    pub security: BotSecurityConfig,
    
    /// Strategy parameters
    pub strategy: StrategyConfig,
}

/// Bot metadata
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BotMetadata {
    pub id: Uuid,
    pub name: String,
    pub description: String,
    pub version: String,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    pub created_by: String,
    pub tags: Vec<String>,
}

/// Bot configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BotConfiguration {
    pub bot_type: BotType,
    pub environment: Environment,
    pub enabled: bool,
    pub auto_start: bool,
    pub auto_restart: bool,
    pub priority: u8, // 1-10, 10 being highest
}

/// Bot resource configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BotResourceConfig {
    pub cpu_limit: f64,
    pub memory_limit_mb: u64,
    pub disk_limit_mb: u64,
    pub network_limit_mbps: Option<u64>,
    pub priority_class: ResourcePriorityClass,
}

/// Resource priority classes
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ResourcePriorityClass {
    #[serde(rename = "low")]
    Low,
    #[serde(rename = "normal")]
    Normal,
    #[serde(rename = "high")]
    High,
    #[serde(rename = "critical")]
    Critical,
}

/// Bot network configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BotNetworkConfig {
    pub solana_rpc_urls: Vec<String>,
    pub websocket_urls: Vec<String>,
    pub backup_rpc_urls: Vec<String>,
    pub connection_timeouts: NetworkTimeoutsYaml,
    pub retry_policy: RetryPolicyConfig,
}

/// Network timeouts
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NetworkTimeoutsYaml {
    pub connect_timeout_seconds: u64,
    pub request_timeout_seconds: u64,
    pub read_timeout_seconds: u64,
    pub write_timeout_seconds: u64,
}

/// Retry policy configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RetryPolicyConfig {
    pub max_attempts: u32,
    pub base_delay_ms: u64,
    pub max_delay_ms: u64,
    pub exponential_backoff: bool,
    pub jitter_enabled: bool,
}

/// Bot security configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BotSecurityConfig {
    pub wallet_config: WalletConfigYaml,
    pub api_keys: HashMap<String, String>,
    pub encryption_enabled: bool,
    pub signature_validation: bool,
    pub secure_communication: bool,
}

/// Wallet configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WalletConfigYaml {
    pub wallet_type: String,
    pub address: Option<String>,
    pub private_key_env_var: Option<String>,
    pub private_key_file: Option<String>,
    pub use_hardware_wallet: bool,
    pub derivation_path: Option<String>,
}

/// Strategy configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StrategyConfig {
    pub parameters: HashMap<String, serde_yaml::Value>,
    pub risk_management: RiskManagementConfig,
    pub execution: ExecutionConfig,
    pub monitoring: MonitoringConfig,
}

/// Risk management configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RiskManagementConfig {
    pub max_position_size_usd: f64,
    pub max_daily_loss_usd: f64,
    pub max_drawdown_percent: f64,
    pub stop_loss_percent: Option<f64>,
    pub take_profit_percent: Option<f64>,
    pub position_size_percent: f64,
}

/// Execution configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExecutionConfig {
    pub execution_mode: ExecutionMode,
    pub slippage_tolerance_percent: f64,
    pub max_execution_time_ms: u64,
    pub gas_price_strategy: GasPriceStrategy,
    pub priority_fee_strategy: PriorityFeeStrategy,
}

/// Execution modes
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ExecutionMode {
    #[serde(rename = "simulation")]
    Simulation,
    #[serde(rename = "live")]
    Live,
    #[serde(rename = "paper_trading")]
    PaperTrading,
}

/// Gas price strategies
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum GasPriceStrategy {
    #[serde(rename = "conservative")]
    Conservative,
    #[serde(rename = "normal")]
    Normal,
    #[serde(rename = "aggressive")]
    Aggressive,
    #[serde(rename = "custom")]
    Custom { value: u64 },
}

/// Priority fee strategies
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum PriorityFeeStrategy {
    #[serde(rename = "none")]
    None,
    #[serde(rename = "low")]
    Low,
    #[serde(rename = "medium")]
    Medium,
    #[serde(rename = "high")]
    High,
    #[serde(rename = "custom")]
    Custom { microlamports: u64 },
}

/// Monitoring configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MonitoringConfig {
    pub metrics_enabled: bool,
    pub health_checks_enabled: bool,
    pub performance_tracking: bool,
    pub alerts: AlertConfig,
    pub reporting: ReportingConfig,
}

/// Alert configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AlertConfig {
    pub enabled: bool,
    pub channels: Vec<AlertChannel>,
    pub thresholds: AlertThresholds,
}

/// Alert channels
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum AlertChannel {
    #[serde(rename = "console")]
    Console,
    #[serde(rename = "file")]
    File { path: String },
    #[serde(rename = "email")]
    Email { recipient: String },
    #[serde(rename = "webhook")]
    Webhook { url: String },
    #[serde(rename = "discord")]
    Discord { webhook_url: String },
    #[serde(rename = "telegram")]
    Telegram { bot_token: String, chat_id: String },
}

/// Alert thresholds
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AlertThresholds {
    pub error_rate_percent: f64,
    pub latency_ms: u64,
    pub memory_usage_percent: f64,
    pub profit_loss_usd: f64,
    pub health_score_minimum: f64,
}

/// Reporting configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ReportingConfig {
    pub enabled: bool,
    pub format: ReportFormat,
    pub frequency: ReportFrequency,
    pub recipients: Vec<String>,
    pub include_charts: bool,
}

/// Report formats
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ReportFormat {
    #[serde(rename = "json")]
    Json,
    #[serde(rename = "html")]
    Html,
    #[serde(rename = "pdf")]
    Pdf,
    #[serde(rename = "csv")]
    Csv,
}

/// Report frequencies
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ReportFrequency {
    #[serde(rename = "hourly")]
    Hourly,
    #[serde(rename = "daily")]
    Daily,
    #[serde(rename = "weekly")]
    Weekly,
    #[serde(rename = "monthly")]
    Monthly,
}

/// YAML Configuration Manager
pub struct YamlConfigManager {
    config_path: PathBuf,
    system_config: RwLock<SystemConfigYaml>,
    bot_configs: RwLock<HashMap<Uuid, BotConfigYaml>>,
    environment: String,
}

impl YamlConfigManager {
    /// Create new YAML configuration manager
    pub fn new<P: AsRef<Path>>(config_path: P, environment: String) -> Self {
        Self {
            config_path: config_path.as_ref().to_path_buf(),
            system_config: RwLock::new(SystemConfigYaml::default()),
            bot_configs: RwLock::new(HashMap::new()),
            environment,
        }
    }
    
    /// Initialize configuration system
    pub async fn initialize(&self) -> Result<(), YamlConfigError> {
        info!("🔧 Initializing YAML configuration system...");
        
        // Create config directories
        fs::create_dir_all(&self.config_path).await?;
        fs::create_dir_all(self.config_path.join("bots")).await?;
        fs::create_dir_all(self.config_path.join("templates")).await?;
        fs::create_dir_all(self.config_path.join("environments")).await?;
        
        // Load or create system configuration
        self.load_system_config().await?;
        
        // Load bot configurations
        self.load_all_bot_configs().await?;
        
        info!("✅ YAML configuration system initialized");
        Ok(())
    }
    
    /// Load system configuration with environment overrides
    pub async fn load_system_config(&self) -> Result<(), YamlConfigError> {
        let config_file = self.config_path.join("system.yaml");
        
        if !config_file.exists() {
            info!("📝 Creating default system configuration...");
            let default_config = SystemConfigYaml::default();
            self.save_system_config(&default_config).await?;
            return Ok(());
        }
        
        let content = fs::read_to_string(&config_file).await?;
        let mut config: SystemConfigYaml = serde_yaml::from_str(&content)?;
        
        // Apply environment-specific overrides
        if let Some(env_overrides) = config.environments.get(&self.environment).cloned() {
            info!("🔄 Applying {} environment overrides", self.environment);
            self.apply_environment_overrides(&mut config, &env_overrides);
        }
        
        // Apply environment variable overrides
        self.apply_env_var_overrides(&mut config)?;
        
        *self.system_config.write().await = config;
        
        info!("✅ System configuration loaded for environment: {}", self.environment);
        Ok(())
    }
    
    /// Save system configuration
    pub async fn save_system_config(&self, config: &SystemConfigYaml) -> Result<(), YamlConfigError> {
        let config_file = self.config_path.join("system.yaml");
        let yaml_content = serde_yaml::to_string(config)?;
        fs::write(&config_file, yaml_content).await?;
        
        *self.system_config.write().await = config.clone();
        Ok(())
    }
    
    /// Load all bot configurations
    pub async fn load_all_bot_configs(&self) -> Result<(), YamlConfigError> {
        let bots_dir = self.config_path.join("bots");
        
        if !bots_dir.exists() {
            return Ok(());
        }
        
        let mut dir = fs::read_dir(&bots_dir).await?;
        let mut loaded_count = 0;
        
        while let Some(entry) = dir.next_entry().await? {
            if let Some(file_name) = entry.file_name().to_str() {
                if file_name.ends_with(".yaml") || file_name.ends_with(".yml") {
                    if let Some(bot_id_str) = file_name.strip_suffix(".yaml").or_else(|| file_name.strip_suffix(".yml")) {
                        if let Ok(bot_id) = Uuid::parse_str(bot_id_str) {
                            if let Ok(_) = self.load_bot_config(bot_id).await {
                                loaded_count += 1;
                            }
                        }
                    }
                }
            }
        }
        
        info!("📂 Loaded {} bot configurations", loaded_count);
        Ok(())
    }
    
    /// Load specific bot configuration
    pub async fn load_bot_config(&self, bot_id: Uuid) -> Result<BotConfigYaml, YamlConfigError> {
        let config_file = self.config_path.join("bots").join(format!("{}.yaml", bot_id));
        
        if !config_file.exists() {
            return Err(YamlConfigError::FileNotFound(config_file.to_string_lossy().to_string()));
        }
        
        let content = fs::read_to_string(&config_file).await?;
        let config: BotConfigYaml = serde_yaml::from_str(&content)?;
        
        self.bot_configs.write().await.insert(bot_id, config.clone());
        
        Ok(config)
    }
    
    /// Save bot configuration
    pub async fn save_bot_config(&self, bot_id: Uuid, config: &BotConfigYaml) -> Result<(), YamlConfigError> {
        let config_file = self.config_path.join("bots").join(format!("{}.yaml", bot_id));
        let yaml_content = serde_yaml::to_string(config)?;
        
        // Atomic write using temporary file
        let temp_file = config_file.with_extension("tmp");
        fs::write(&temp_file, yaml_content).await?;
        fs::rename(&temp_file, &config_file).await?;
        
        self.bot_configs.write().await.insert(bot_id, config.clone());
        
        info!("💾 Saved bot configuration: {}", bot_id);
        Ok(())
    }
    
    /// Get system configuration
    pub async fn get_system_config(&self) -> SystemConfigYaml {
        self.system_config.read().await.clone()
    }
    
    /// Get bot configuration
    pub async fn get_bot_config(&self, bot_id: Uuid) -> Option<BotConfigYaml> {
        self.bot_configs.read().await.get(&bot_id).cloned()
    }
    
    /// Create bot configuration from template
    pub async fn create_bot_config_from_template(
        &self,
        bot_id: Uuid,
        bot_type: BotType,
        name: String,
    ) -> Result<BotConfigYaml, YamlConfigError> {
        let template_file = self.config_path.join("templates").join(format!("{:?}.yaml", bot_type));
        
        let config = if template_file.exists() {
            let content = fs::read_to_string(&template_file).await?;
            let mut template: BotConfigYaml = serde_yaml::from_str(&content)?;
            template.metadata.id = bot_id;
            template.metadata.name = name;
            template.metadata.created_at = Utc::now();
            template.metadata.updated_at = Utc::now();
            template
        } else {
            BotConfigYaml::default_for_type(bot_id, bot_type, name)
        };
        
        self.save_bot_config(bot_id, &config).await?;
        Ok(config)
    }
    
    /// Validate configuration
    pub async fn validate_system_config(&self, config: &SystemConfigYaml) -> Result<(), YamlConfigError> {
        // Validate TCP port range (only check minimum since u16 max is 65535)
        if config.server.tcp.port < 1024 {
            return Err(YamlConfigError::ValidationFailed("TCP port must be 1024 or higher".to_string()));
        }
        
        // Validate HTTP port range (only check minimum since u16 max is 65535)
        if config.server.http.port < 1024 {
            return Err(YamlConfigError::ValidationFailed("HTTP port must be 1024 or higher".to_string()));
        }
        
        // Validate logging level
        let valid_levels = ["trace", "debug", "info", "warn", "error"];
        if !valid_levels.contains(&config.logging.level.as_str()) {
            return Err(YamlConfigError::ValidationFailed("Invalid logging level".to_string()));
        }
        
        // Add more validation rules as needed...
        
        Ok(())
    }
    
    /// Apply environment overrides
    fn apply_environment_overrides(&self, config: &mut SystemConfigYaml, overrides: &EnvironmentOverrides) {
        if let Some(server_override) = &overrides.server {
            config.server = server_override.clone();
        }
        if let Some(persistence_override) = &overrides.persistence {
            config.persistence = persistence_override.clone();
        }
        if let Some(security_override) = &overrides.security {
            config.security = security_override.clone();
        }
        if let Some(logging_override) = &overrides.logging {
            config.logging = logging_override.clone();
        }
        if let Some(bot_defaults_override) = &overrides.bot_defaults {
            config.bot_defaults = bot_defaults_override.clone();
        }
    }
    
    /// Apply environment variable overrides
    fn apply_env_var_overrides(&self, config: &mut SystemConfigYaml) -> Result<(), YamlConfigError> {
        // TCP port override
        if let Ok(port_str) = std::env::var("SNIPERFORGE_TCP_PORT") {
            if let Ok(port) = port_str.parse::<u16>() {
                config.server.tcp.port = port;
                info!("🔧 TCP port overridden by environment: {}", port);
            }
        }
        
        // HTTP port override
        if let Ok(port_str) = std::env::var("SNIPERFORGE_HTTP_PORT") {
            if let Ok(port) = port_str.parse::<u16>() {
                config.server.http.port = port;
                info!("🔧 HTTP port overridden by environment: {}", port);
            }
        }
        
        // Log level override
        if let Ok(level) = std::env::var("SNIPERFORGE_LOG_LEVEL") {
            config.logging.level = level;
            info!("🔧 Log level overridden by environment: {}", config.logging.level);
        }
        
        // Data directory override
        if let Ok(data_dir) = std::env::var("SNIPERFORGE_DATA_DIR") {
            config.persistence.data_directory = data_dir;
            info!("🔧 Data directory overridden by environment: {}", config.persistence.data_directory);
        }
        
        Ok(())
    }
}

// Default implementations
impl Default for SystemConfigYaml {
    fn default() -> Self {
        Self {
            app: AppConfig {
                name: "SniperForge".to_string(),
                version: env!("CARGO_PKG_VERSION").to_string(),
                description: "Professional trading bot suite for Solana DeFi".to_string(),
                author: "SniperForge Team".to_string(),
                license: "MIT".to_string(),
            },
            server: ServerConfig {
                tcp: TcpServerConfig {
                    enabled: true,
                    host: "127.0.0.1".to_string(),
                    port: 8888,
                    max_connections: 100,
                    timeout_seconds: 30,
                    buffer_size: 8192,
                },
                http: HttpServerConfig {
                    enabled: true,
                    host: "127.0.0.1".to_string(),
                    port: 8080,
                    cors_origins: vec!["*".to_string()],
                    max_request_size_mb: 10,
                    rate_limit_per_minute: 60,
                },
                websocket: WebSocketConfig {
                    enabled: true,
                    port: 8081,
                    heartbeat_interval_seconds: 30,
                    max_frame_size_mb: 1,
                },
                performance: PerformanceConfig {
                    worker_threads: None,
                    max_blocking_threads: None,
                    enable_profiling: false,
                    metrics_collection_interval_seconds: 60,
                },
            },
            persistence: PersistenceConfig {
                enabled: true,
                storage_type: StorageType::File,
                data_directory: "./state".to_string(),
                backup_directory: "./state/backups".to_string(),
                auto_backup_enabled: true,
                backup_interval_hours: 24,
                retention_days: 30,
                compression_enabled: true,
            },
            security: SecurityConfigYaml {
                encryption: EncryptionConfig {
                    enabled: true,
                    algorithm: "AES-256-GCM".to_string(),
                    key_rotation_days: 90,
                    encrypt_at_rest: true,
                    encrypt_in_transit: true,
                },
                authentication: AuthConfig {
                    required: true,
                    method: AuthMethod::ApiKey,
                    session_timeout_minutes: 60,
                    max_failed_attempts: 3,
                    lockout_duration_minutes: 15,
                },
                api_security: ApiSecurityConfig {
                    rate_limiting: true,
                    ip_whitelist: vec![],
                    require_https: false,
                    allowed_origins: vec!["localhost".to_string()],
                },
                wallet_security: WalletSecurityConfig {
                    use_env_keys: true,
                    key_derivation_method: "BIP44".to_string(),
                    multi_sig_enabled: false,
                    hardware_wallet_support: false,
                },
            },
            logging: LoggingConfig {
                level: "info".to_string(),
                format: LogFormat::Pretty,
                output: vec![
                    LogOutput {
                        target: LogTarget::Console,
                        enabled: true,
                        level_override: None,
                    },
                    LogOutput {
                        target: LogTarget::File {
                            path: "./logs/sniperforge.log".to_string(),
                        },
                        enabled: true,
                        level_override: None,
                    },
                ],
                rotation: LogRotationConfig {
                    enabled: true,
                    max_size_mb: 100,
                    max_files: 10,
                    rotation_interval: RotationInterval::Daily,
                },
                filters: vec![],
            },
            bot_defaults: BotDefaultsConfig {
                environment: Environment::Development,
                resources: DefaultResourceLimits {
                    max_cpu_cores: 1.0,
                    max_memory_mb: 512,
                    max_disk_mb: 1024,
                    max_network_mbps: Some(100),
                },
                network: DefaultNetworkConfig {
                    solana_rpc_urls: vec!["https://api.devnet.solana.com".to_string()],
                    websocket_urls: vec!["wss://api.devnet.solana.com".to_string()],
                    connection_pool_size: 10,
                    request_timeout_seconds: 30,
                    retry_attempts: 3,
                },
                security: DefaultSecurityConfig {
                    encryption_enabled: true,
                    auth_required: true,
                    use_env_keys: true,
                    validate_signatures: true,
                },
                timeouts: DefaultTimeouts {
                    startup_timeout_seconds: 60,
                    shutdown_timeout_seconds: 30,
                    health_check_interval_seconds: 30,
                    metrics_update_interval_seconds: 60,
                },
            },
            environments: HashMap::new(),
        }
    }
}

impl BotConfigYaml {
    pub fn default_for_type(bot_id: Uuid, bot_type: BotType, name: String) -> Self {
        Self {
            metadata: BotMetadata {
                id: bot_id,
                name,
                description: format!("Default {} bot configuration", bot_type.as_str()),
                version: "1.0.0".to_string(),
                created_at: Utc::now(),
                updated_at: Utc::now(),
                created_by: "system".to_string(),
                tags: vec!["default".to_string(), bot_type.as_str().to_lowercase()],
            },
            bot: BotConfiguration {
                bot_type,
                environment: Environment::Development,
                enabled: true,
                auto_start: false,
                auto_restart: true,
                priority: 5,
            },
            resources: BotResourceConfig {
                cpu_limit: 1.0,
                memory_limit_mb: 512,
                disk_limit_mb: 1024,
                network_limit_mbps: Some(100),
                priority_class: ResourcePriorityClass::Normal,
            },
            network: BotNetworkConfig {
                solana_rpc_urls: vec!["https://api.devnet.solana.com".to_string()],
                websocket_urls: vec!["wss://api.devnet.solana.com".to_string()],
                backup_rpc_urls: vec!["https://api.mainnet-beta.solana.com".to_string()],
                connection_timeouts: NetworkTimeoutsYaml {
                    connect_timeout_seconds: 10,
                    request_timeout_seconds: 30,
                    read_timeout_seconds: 30,
                    write_timeout_seconds: 30,
                },
                retry_policy: RetryPolicyConfig {
                    max_attempts: 3,
                    base_delay_ms: 1000,
                    max_delay_ms: 10000,
                    exponential_backoff: true,
                    jitter_enabled: true,
                },
            },
            security: BotSecurityConfig {
                wallet_config: WalletConfigYaml {
                    wallet_type: "keypair".to_string(),
                    address: None,
                    private_key_env_var: Some("SOLANA_PRIVATE_KEY".to_string()),
                    private_key_file: Some("./wallet.json".to_string()),
                    use_hardware_wallet: false,
                    derivation_path: Some("m/44'/501'/0'/0'".to_string()),
                },
                api_keys: HashMap::new(),
                encryption_enabled: true,
                signature_validation: true,
                secure_communication: true,
            },
            strategy: StrategyConfig {
                parameters: HashMap::new(),
                risk_management: RiskManagementConfig {
                    max_position_size_usd: 1000.0,
                    max_daily_loss_usd: 100.0,
                    max_drawdown_percent: 10.0,
                    stop_loss_percent: Some(5.0),
                    take_profit_percent: Some(10.0),
                    position_size_percent: 1.0,
                },
                execution: ExecutionConfig {
                    execution_mode: ExecutionMode::Simulation,
                    slippage_tolerance_percent: 1.0,
                    max_execution_time_ms: 5000,
                    gas_price_strategy: GasPriceStrategy::Normal,
                    priority_fee_strategy: PriorityFeeStrategy::Medium,
                },
                monitoring: MonitoringConfig {
                    metrics_enabled: true,
                    health_checks_enabled: true,
                    performance_tracking: true,
                    alerts: AlertConfig {
                        enabled: true,
                        channels: vec![AlertChannel::Console],
                        thresholds: AlertThresholds {
                            error_rate_percent: 5.0,
                            latency_ms: 1000,
                            memory_usage_percent: 80.0,
                            profit_loss_usd: -50.0,
                            health_score_minimum: 0.8,
                        },
                    },
                    reporting: ReportingConfig {
                        enabled: true,
                        format: ReportFormat::Json,
                        frequency: ReportFrequency::Daily,
                        recipients: vec![],
                        include_charts: false,
                    },
                },
            },
        }
    }
}

// =====================================================================================
// 🎯 DECLARATIVE DESIRED STATE CONFIGURATION
// =====================================================================================

/// Desired State Configuration - Declarative approach for bot management
/// This represents the target state that the system should maintain
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DesiredStateConfig {
    /// Reconciliation settings
    pub reconciliation: ReconciliationConfig,
    
    /// Desired bot configurations
    pub bots: Vec<DesiredBotState>,
    
    /// System-wide desired state
    pub system: DesiredSystemState,
}

/// Reconciliation configuration for desired state convergence
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ReconciliationConfig {
    /// Enable automatic reconciliation
    pub enabled: bool,
    
    /// Reconciliation interval in seconds
    pub interval_seconds: u64,
    
    /// Maximum retries for failed reconciliation attempts
    pub max_retries: u32,
    
    /// Timeout for individual reconciliation operations
    pub timeout_seconds: u64,
    
    /// Enable drift detection
    pub drift_detection: bool,
    
    /// Drift tolerance before triggering reconciliation
    pub drift_tolerance_percent: f64,
}

/// Desired state for a single bot
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DesiredBotState {
    /// Bot identifier (must be unique)
    pub id: String,
    
    /// Bot type
    #[serde(rename = "type")]
    pub bot_type: BotType,
    
    /// Desired operational status
    pub desired_status: DesiredBotStatus,
    
    /// Bot configuration
    pub config: DesiredBotConfig,
    
    /// Resource allocation
    pub resources: BotResourceConfig,
    
    /// Health check configuration
    pub health_checks: Option<DesiredHealthConfig>,
    
    /// Auto-scaling configuration
    pub auto_scaling: Option<AutoScalingConfig>,
}

/// Desired bot operational status
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "lowercase")]
pub enum DesiredBotStatus {
    /// Bot should be running and active
    Running,
    
    /// Bot should be stopped
    Stopped,
    
    /// Bot should be paused (ready but not executing)
    Paused,
    
    /// Bot should be in maintenance mode
    Maintenance,
}

/// Desired bot configuration parameters
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DesiredBotConfig {
    /// Trading pairs for arbitrage bots
    pub pairs: Option<Vec<String>>,
    
    /// Supported exchanges
    pub exchanges: Option<Vec<String>>,
    
    /// Minimum profit threshold
    pub min_profit_threshold: Option<f64>,
    
    /// Maximum position size
    pub max_position_size: Option<f64>,
    
    /// Execution timeout in milliseconds
    pub execution_timeout_ms: Option<u64>,
    
    /// Analysis interval for ML bots (seconds)
    pub analysis_interval_seconds: Option<u64>,
    
    /// Prediction horizon for ML bots (hours)
    pub prediction_horizon_hours: Option<u64>,
    
    /// Confidence threshold for ML predictions
    pub confidence_threshold: Option<f64>,
    
    /// ML models to use
    pub models: Option<Vec<String>>,
    
    /// Data sources for sentiment analysis
    pub sources: Option<Vec<String>>,
    
    /// Update interval for sentiment analysis (seconds)
    pub update_interval_seconds: Option<u64>,
    
    /// Sentiment threshold
    pub sentiment_threshold: Option<f64>,
    
    /// Language filter for sentiment analysis
    pub language_filter: Option<Vec<String>>,
    
    /// Custom parameters
    pub custom_parameters: Option<HashMap<String, serde_yaml::Value>>,
}

/// Health check configuration for desired state
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DesiredHealthConfig {
    /// Enable health checks
    pub enabled: bool,
    
    /// Health check interval in seconds
    pub interval_seconds: u64,
    
    /// Health check timeout in seconds
    pub timeout_seconds: u64,
    
    /// Restart bot on health check failure
    pub restart_on_failure: bool,
    
    /// Maximum consecutive failures before marking as unhealthy
    pub max_failures: u32,
}

/// Auto-scaling configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AutoScalingConfig {
    /// Enable auto-scaling
    pub enabled: bool,
    
    /// Minimum number of instances
    pub min_instances: u32,
    
    /// Maximum number of instances
    pub max_instances: u32,
    
    /// CPU utilization threshold for scaling up (percentage)
    pub scale_up_cpu_threshold: f64,
    
    /// CPU utilization threshold for scaling down (percentage)
    pub scale_down_cpu_threshold: f64,
    
    /// Memory utilization threshold for scaling up (percentage)
    pub scale_up_memory_threshold: f64,
    
    /// Cool-down period between scaling operations (seconds)
    pub cooldown_seconds: u64,
}

/// Desired system-wide state
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DesiredSystemState {
    /// Total number of bots that should be running
    pub target_running_bots: Option<u32>,
    
    /// Maximum system resource utilization
    pub max_system_cpu_percent: Option<f64>,
    
    /// Maximum system memory utilization
    pub max_system_memory_percent: Option<f64>,
    
    /// Desired system health score
    pub target_health_score: Option<f64>,
    
    /// Maintenance mode flag
    pub maintenance_mode: bool,
    
    /// Load balancing configuration
    pub load_balancing: Option<LoadBalancingConfig>,
}

/// Load balancing configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LoadBalancingConfig {
    /// Enable load balancing
    pub enabled: bool,
    
    /// Load balancing strategy
    pub strategy: LoadBalancingStrategy,
    
    /// Target CPU utilization for load balancing
    pub target_cpu_utilization: f64,
    
    /// Target memory utilization for load balancing
    pub target_memory_utilization: f64,
}

/// Load balancing strategies
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum LoadBalancingStrategy {
    /// Round-robin distribution
    RoundRobin,
    
    /// Least connections
    LeastConnections,
    
    /// Resource-based (CPU/Memory)
    ResourceBased,
    
    /// Performance-based
    PerformanceBased,
}

/// Implementation for DesiredStateConfig
impl DesiredStateConfig {
    /// Create a new desired state configuration with defaults
    pub fn new() -> Self {
        Self {
            reconciliation: ReconciliationConfig {
                enabled: true,
                interval_seconds: 30,
                max_retries: 3,
                timeout_seconds: 300,
                drift_detection: true,
                drift_tolerance_percent: 5.0,
            },
            bots: Vec::new(),
            system: DesiredSystemState {
                target_running_bots: None,
                max_system_cpu_percent: Some(80.0),
                max_system_memory_percent: Some(85.0),
                target_health_score: Some(0.95),
                maintenance_mode: false,
                load_balancing: Some(LoadBalancingConfig {
                    enabled: true,
                    strategy: LoadBalancingStrategy::ResourceBased,
                    target_cpu_utilization: 70.0,
                    target_memory_utilization: 75.0,
                }),
            },
        }
    }
    
    /// Add a desired bot state
    pub fn add_bot(&mut self, bot_state: DesiredBotState) {
        self.bots.push(bot_state);
    }
    
    /// Find a bot by ID
    pub fn find_bot(&self, id: &str) -> Option<&DesiredBotState> {
        self.bots.iter().find(|bot| bot.id == id)
    }
    
    /// Get all bots with a specific desired status
    pub fn get_bots_by_status(&self, status: &DesiredBotStatus) -> Vec<&DesiredBotState> {
        self.bots.iter().filter(|bot| bot.desired_status == *status).collect()
    }
    
    /// Validate the configuration
    pub fn validate(&self) -> Result<(), YamlConfigError> {
        // Check for duplicate bot IDs
        let mut seen_ids = std::collections::HashSet::new();
        for bot in &self.bots {
            if !seen_ids.insert(&bot.id) {
                return Err(YamlConfigError::ValidationFailed(
                    format!("Duplicate bot ID: {}", bot.id)
                ));
            }
        }
        
        // Validate reconciliation settings
        if self.reconciliation.interval_seconds == 0 {
            return Err(YamlConfigError::ValidationFailed(
                "Reconciliation interval must be greater than 0".to_string()
            ));
        }
        
        if self.reconciliation.timeout_seconds == 0 {
            return Err(YamlConfigError::ValidationFailed(
                "Reconciliation timeout must be greater than 0".to_string()
            ));
        }
        
        Ok(())
    }
}

impl Default for DesiredStateConfig {
    fn default() -> Self {
        Self::new()
    }
}

impl Default for ReconciliationConfig {
    fn default() -> Self {
        Self {
            enabled: true,
            interval_seconds: 30,
            max_retries: 3,
            timeout_seconds: 300,
            drift_detection: true,
            drift_tolerance_percent: 5.0,
        }
    }
}

impl Default for DesiredSystemState {
    fn default() -> Self {
        Self {
            target_running_bots: None,
            max_system_cpu_percent: Some(80.0),
            max_system_memory_percent: Some(85.0),
            target_health_score: Some(0.95),
            maintenance_mode: false,
            load_balancing: Some(LoadBalancingConfig {
                enabled: true,
                strategy: LoadBalancingStrategy::ResourceBased,
                target_cpu_utilization: 70.0,
                target_memory_utilization: 75.0,
            }),
        }
    }
}
