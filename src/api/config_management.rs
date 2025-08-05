//! Configuration Management Module
//! 
//! This module handles dynamic configuration management for bots and the system.
//! It provides functionality for loading, saving, validating, and hot-reloading configurations.

use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::{Path, PathBuf};
use tokio::fs;
use tokio::sync::RwLock;
use uuid::Uuid;

use crate::api::bot_interface::{BotConfig, BotType};

/// Configuration management errors
#[derive(Debug, thiserror::Error)]
pub enum ConfigError {
    #[error("Configuration file not found: {0}")]
    FileNotFound(String),
    
    #[error("Invalid configuration format: {0}")]
    InvalidFormat(String),
    
    #[error("Configuration validation failed: {0}")]
    ValidationFailed(String),
    
    #[error("IO error: {0}")]
    IoError(#[from] std::io::Error),
    
    #[error("Serialization error: {0}")]
    SerializationError(#[from] serde_json::Error),
}

/// System-wide configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SystemConfig {
    pub database: DatabaseConfig,
    pub api: ApiConfig,
    pub logging: LoggingConfig,
    pub security: SecurityConfig,
    pub monitoring: MonitoringConfig,
}

/// Database configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DatabaseConfig {
    pub host: String,
    pub port: u16,
    pub database: String,
    pub username: String,
    pub password: String,
    pub pool_size: u32,
    pub connection_timeout_ms: u64,
}

/// API configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ApiConfig {
    pub host: String,
    pub port: u16,
    pub cors_enabled: bool,
    pub rate_limit_per_minute: u32,
    pub max_request_size_mb: u32,
    pub jwt_secret: String,
    pub session_timeout_hours: u32,
}

/// Logging configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LoggingConfig {
    pub level: String,
    pub file_path: Option<String>,
    pub max_file_size_mb: u32,
    pub max_files: u32,
    pub structured_logging: bool,
}

/// Security configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SecurityConfig {
    pub encryption_key: String,
    pub api_key_required: bool,
    pub rate_limiting_enabled: bool,
    pub audit_logging: bool,
    pub ssl_enabled: bool,
    pub ssl_cert_path: Option<String>,
    pub ssl_key_path: Option<String>,
}

/// Monitoring configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MonitoringConfig {
    pub metrics_enabled: bool,
    pub prometheus_endpoint: String,
    pub health_check_interval_seconds: u64,
    pub alert_webhook_url: Option<String>,
    pub performance_tracking: bool,
}

/// Bot configuration template
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BotConfigTemplate {
    pub bot_type: BotType,
    pub name: String,
    pub description: String,
    pub default_config: BotConfig,
    pub required_parameters: Vec<String>,
    pub optional_parameters: Vec<String>,
    pub validation_rules: HashMap<String, ValidationRule>,
}

/// Configuration validation rule
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ValidationRule {
    pub rule_type: ValidationType,
    pub parameters: HashMap<String, serde_json::Value>,
    pub error_message: String,
}

/// Types of validation
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ValidationType {
    Range { min: f64, max: f64 },
    Pattern(String),
    Required,
    OneOf(Vec<String>),
    Custom(String),
}

/// Configuration manager
pub struct ConfigManager {
    system_config: RwLock<SystemConfig>,
    bot_configs: RwLock<HashMap<Uuid, BotConfig>>,
    bot_templates: RwLock<HashMap<BotType, BotConfigTemplate>>,
    config_path: PathBuf,
}

impl ConfigManager {
    /// Create a new configuration manager
    pub fn new<P: AsRef<Path>>(config_path: P) -> Self {
        Self {
            system_config: RwLock::new(SystemConfig::default()),
            bot_configs: RwLock::new(HashMap::new()),
            bot_templates: RwLock::new(HashMap::new()),
            config_path: config_path.as_ref().to_path_buf(),
        }
    }

    /// Load system configuration from file
    pub async fn load_system_config(&self) -> Result<(), ConfigError> {
        let config_file = self.config_path.join("system.json");
        
        if !config_file.exists() {
            // Create default configuration
            let default_config = SystemConfig::default();
            self.save_system_config(&default_config).await?;
            *self.system_config.write().await = default_config;
            return Ok(());
        }

        let content = fs::read_to_string(&config_file).await?;
        let config: SystemConfig = serde_json::from_str(&content)?;
        
        self.validate_system_config(&config)?;
        *self.system_config.write().await = config;
        
        Ok(())
    }

    /// Save system configuration to file
    pub async fn save_system_config(&self, config: &SystemConfig) -> Result<(), ConfigError> {
        self.validate_system_config(config)?;
        
        let config_file = self.config_path.join("system.json");
        
        // Ensure directory exists
        if let Some(parent) = config_file.parent() {
            fs::create_dir_all(parent).await?;
        }
        
        let content = serde_json::to_string_pretty(config)?;
        fs::write(&config_file, content).await?;
        
        *self.system_config.write().await = config.clone();
        
        Ok(())
    }

    /// Get current system configuration
    pub async fn get_system_config(&self) -> SystemConfig {
        self.system_config.read().await.clone()
    }

    /// Update system configuration
    pub async fn update_system_config(&self, config: SystemConfig) -> Result<(), ConfigError> {
        self.save_system_config(&config).await
    }

    /// Load bot configuration templates
    pub async fn load_bot_templates(&self) -> Result<(), ConfigError> {
        let templates_file = self.config_path.join("bot_templates.json");
        
        if !templates_file.exists() {
            let default_templates = self.create_default_templates();
            self.save_bot_templates(&default_templates).await?;
            return Ok(());
        }

        let content = fs::read_to_string(&templates_file).await?;
        let templates: HashMap<BotType, BotConfigTemplate> = serde_json::from_str(&content)?;
        
        *self.bot_templates.write().await = templates;
        
        Ok(())
    }

    /// Save bot configuration templates
    pub async fn save_bot_templates(
        &self,
        templates: &HashMap<BotType, BotConfigTemplate>
    ) -> Result<(), ConfigError> {
        let templates_file = self.config_path.join("bot_templates.json");
        
        if let Some(parent) = templates_file.parent() {
            fs::create_dir_all(parent).await?;
        }
        
        let content = serde_json::to_string_pretty(templates)?;
        fs::write(&templates_file, content).await?;
        
        *self.bot_templates.write().await = templates.clone();
        
        Ok(())
    }

    /// Get bot configuration template
    pub async fn get_bot_template(&self, bot_type: &BotType) -> Option<BotConfigTemplate> {
        self.bot_templates.read().await.get(bot_type).cloned()
    }

    /// Validate bot configuration against template
    pub async fn validate_bot_config(
        &self,
        bot_type: &BotType,
        config: &BotConfig,
    ) -> Result<(), ConfigError> {
        let templates = self.bot_templates.read().await;
        
        if let Some(template) = templates.get(bot_type) {
            self.validate_config_against_template(config, template)?;
        }
        
        Ok(())
    }

    /// Save bot configuration
    pub async fn save_bot_config(&self, bot_id: Uuid, config: &BotConfig) -> Result<(), ConfigError> {
        if let Some(parent) = self.config_path.join("bots").parent() {
            fs::create_dir_all(parent).await?;
        }
        
        // Ensure bots directory exists
        let bots_dir = self.config_path.join("bots");
        fs::create_dir_all(&bots_dir).await?;
        
        // ESTRATEGIA HOTRELOAD: Usar bot_id como identificador único
        // El servidor NO mantiene archivos abiertos, solo lee cuando necesita
        let final_config_file = bots_dir.join(format!("{}.json", bot_id));
        
        tracing::info!("📁 Saving bot config to: {}", final_config_file.display());
        
        let content = serde_json::to_string_pretty(config)?;
        
        // ESTRATEGIA HOTRELOAD: Sobrescribir archivo existente de forma segura
        // 1. Escribir a archivo temporal
        // 2. Renombrar (operación atómica en Windows)
        // 3. El servidor hará hot-reload automático cuando reciba comandos CLI
        
        let temp_file = final_config_file.with_extension("tmp");
        
        // Escribir contenido al archivo temporal
        match fs::write(&temp_file, &content).await {
            Ok(()) => {
                // Renombrar archivo temporal al final (operación atómica)
                match fs::rename(&temp_file, &final_config_file).await {
                    Ok(()) => {
                        // Actualizar cache en memoria DESPUÉS de escribir el archivo
                        self.bot_configs.write().await.insert(bot_id, config.clone());
                        tracing::info!("✅ Bot configuration saved to: {}", final_config_file.display());
                        tracing::info!("🔄 Hot-reload: Server will reload on next CLI command");
                        Ok(())
                    },
                    Err(e) => {
                        // Limpiar archivo temporal si falló el rename
                        let _ = fs::remove_file(&temp_file).await;
                        tracing::error!("❌ Failed to rename config file: {}", e);
                        Err(ConfigError::IoError(e))
                    }
                }
            },
            Err(e) => {
                tracing::error!("❌ Failed to write temp config file: {}", e);
                Err(ConfigError::IoError(e))
            }
        }
    }

    /// Load bot configuration from file

    /// Load bot configuration
    pub async fn load_bot_config(&self, bot_id: Uuid) -> Result<BotConfig, ConfigError> {
        let config_file = self.config_path.join("bots").join(format!("{}.json", bot_id));
        
        if !config_file.exists() {
            return Err(ConfigError::FileNotFound(config_file.to_string_lossy().to_string()));
        }
        
        let content = fs::read_to_string(&config_file).await?;
        let config: BotConfig = serde_json::from_str(&content)?;
        
        self.bot_configs.write().await.insert(bot_id, config.clone());
        
        Ok(config)
    }

    /// Get bot configuration from memory
    pub async fn get_bot_config(&self, bot_id: Uuid) -> Option<BotConfig> {
        self.bot_configs.read().await.get(&bot_id).cloned()
    }

    /// Delete bot configuration
    pub async fn delete_bot_config(&self, bot_id: Uuid) -> Result<(), ConfigError> {
        let config_file = self.config_path.join("bots").join(format!("{}.json", bot_id));
        
        if config_file.exists() {
            fs::remove_file(&config_file).await?;
        }
        
        self.bot_configs.write().await.remove(&bot_id);
        
        Ok(())
    }

    /// Hot reload all configurations
    pub async fn hot_reload(&self) -> Result<(), ConfigError> {
        self.load_system_config().await?;
        self.load_bot_templates().await?;
        
        // Reload all bot configurations
        let bots_dir = self.config_path.join("bots");
        if bots_dir.exists() {
            let mut dir = fs::read_dir(&bots_dir).await?;
            while let Some(entry) = dir.next_entry().await? {
                if let Some(file_name) = entry.file_name().to_str() {
                    if file_name.ends_with(".json") {
                        if let Some(bot_id_str) = file_name.strip_suffix(".json") {
                            if let Ok(bot_id) = Uuid::parse_str(bot_id_str) {
                                let _ = self.load_bot_config(bot_id).await;
                            }
                        }
                    }
                }
            }
        }
        
        Ok(())
    }

    /// Validate system configuration
    fn validate_system_config(&self, config: &SystemConfig) -> Result<(), ConfigError> {
        // Validate database configuration
        if config.database.host.is_empty() {
            return Err(ConfigError::ValidationFailed("Database host cannot be empty".to_string()));
        }
        
        if config.database.port == 0 {
            return Err(ConfigError::ValidationFailed("Database port must be greater than 0".to_string()));
        }
        
        // Validate API configuration
        if config.api.port == 0 {
            return Err(ConfigError::ValidationFailed("API port must be greater than 0".to_string()));
        }
        
        if config.api.jwt_secret.len() < 32 {
            return Err(ConfigError::ValidationFailed("JWT secret must be at least 32 characters".to_string()));
        }
        
        Ok(())
    }

    /// Validate configuration against template
    fn validate_config_against_template(
        &self,
        config: &BotConfig,
        template: &BotConfigTemplate,
    ) -> Result<(), ConfigError> {
        // Check required parameters
        for required_param in &template.required_parameters {
            if let Some(params_obj) = config.parameters.as_object() {
                if !params_obj.contains_key(required_param) {
                    return Err(ConfigError::ValidationFailed(
                        format!("Required parameter '{}' is missing", required_param)
                    ));
                }
            } else {
                return Err(ConfigError::ValidationFailed(
                    "Configuration parameters must be a JSON object".to_string()
                ));
            }
        }
        
        // Validate parameters against rules
        if let Some(params_obj) = config.parameters.as_object() {
            for (param_name, value) in params_obj {
                if let Some(rule) = template.validation_rules.get(param_name) {
                    self.validate_parameter_value(param_name, value, rule)?;
                }
            }
        }
        
        Ok(())
    }

    /// Validate parameter value against rule
    fn validate_parameter_value(
        &self,
        param_name: &str,
        value: &serde_json::Value,
        rule: &ValidationRule,
    ) -> Result<(), ConfigError> {
        match &rule.rule_type {
            ValidationType::Range { min, max } => {
                if let Some(num) = value.as_f64() {
                    if num < *min || num > *max {
                        return Err(ConfigError::ValidationFailed(
                            format!("Parameter '{}': {}", param_name, rule.error_message)
                        ));
                    }
                } else {
                    return Err(ConfigError::ValidationFailed(
                        format!("Parameter '{}' must be a number", param_name)
                    ));
                }
            },
            ValidationType::Required => {
                if value.is_null() {
                    return Err(ConfigError::ValidationFailed(
                        format!("Parameter '{}' is required", param_name)
                    ));
                }
            },
            ValidationType::OneOf(options) => {
                if let Some(str_val) = value.as_str() {
                    if !options.contains(&str_val.to_string()) {
                        return Err(ConfigError::ValidationFailed(
                            format!("Parameter '{}': {}", param_name, rule.error_message)
                        ));
                    }
                } else {
                    return Err(ConfigError::ValidationFailed(
                        format!("Parameter '{}' must be a string", param_name)
                    ));
                }
            },
            ValidationType::Pattern(pattern) => {
                if let Some(str_val) = value.as_str() {
                    let regex = regex::Regex::new(pattern).map_err(|_| {
                        ConfigError::ValidationFailed("Invalid regex pattern".to_string())
                    })?;
                    
                    if !regex.is_match(str_val) {
                        return Err(ConfigError::ValidationFailed(
                            format!("Parameter '{}': {}", param_name, rule.error_message)
                        ));
                    }
                } else {
                    return Err(ConfigError::ValidationFailed(
                        format!("Parameter '{}' must be a string", param_name)
                    ));
                }
            },
            ValidationType::Custom(_) => {
                // Custom validation would be implemented here
                // For now, we'll skip custom validation
            }
        }
        
        Ok(())
    }

    /// Create default bot configuration templates
    fn create_default_templates(&self) -> HashMap<BotType, BotConfigTemplate> {
        let mut templates = HashMap::new();
        
        // Enhanced Arbitrage Bot Template
        templates.insert(
            BotType::EnhancedArbitrage,
            BotConfigTemplate {
                bot_type: BotType::EnhancedArbitrage,
                name: "Enhanced Arbitrage Bot".to_string(),
                description: "Advanced arbitrage bot with ML-enhanced opportunity detection".to_string(),
                default_config: BotConfig {
                    config_id: Uuid::new_v4(),
                    bot_id: Uuid::new_v4(),
                    bot_type: BotType::EnhancedArbitrage,
                    environment: crate::api::bot_interface::Environment::Development,
                    parameters: serde_json::json!({
                        "min_profit_threshold": 0.01,
                        "max_position_size": 1000.0,
                        "execution_timeout_ms": 5000,
                        "exchanges": ["binance", "kraken"],
                        "pairs": ["BTC/USDT", "ETH/USDT"]
                    }),
                    resources: crate::api::bot_interface::ResourceLimits {
                        max_cpu: 1.0,
                        max_memory_mb: 512,
                        max_disk_mb: 1024,
                        max_network_mbps: Some(100),
                    },
                    network: crate::api::bot_interface::NetworkConfig {
                        solana_rpc_urls: vec!["https://api.mainnet-beta.solana.com".to_string()],
                        websocket_urls: vec!["wss://api.mainnet-beta.solana.com".to_string()],
                        api_endpoints: std::collections::HashMap::new(),
                        timeouts: crate::api::bot_interface::NetworkTimeouts {
                            rpc_timeout_seconds: 30,
                            websocket_timeout_seconds: 30,
                            api_timeout_seconds: 30,
                        },
                    },
                    security: crate::api::bot_interface::SecurityConfig {
                        wallet: crate::api::bot_interface::WalletConfig {
                            wallet_type: "keypair".to_string(),
                            address: "".to_string(),
                            private_key_path: Some("./wallet.json".to_string()),
                            use_env_keys: false,
                        },
                        api_keys: std::collections::HashMap::new(),
                        encryption_enabled: true,
                        auth_required: true,
                    },
                    metadata: crate::api::bot_interface::ConfigMetadata {
                        name: "Enhanced Arbitrage Configuration".to_string(),
                        version: "1.0.0".to_string(),
                        created_at: chrono::Utc::now(),
                        updated_at: chrono::Utc::now(),
                        created_by: "system".to_string(),
                        tags: vec!["arbitrage".to_string(), "trading".to_string()],
                    },
                },
                required_parameters: vec![
                    "min_profit_threshold".to_string(),
                    "max_position_size".to_string(),
                    "exchanges".to_string(),
                ],
                optional_parameters: vec![
                    "execution_timeout_ms".to_string(),
                    "pairs".to_string(),
                ],
                validation_rules: HashMap::new(),
            }
        );
        
        templates
    }
}

impl Default for SystemConfig {
    fn default() -> Self {
        Self {
            database: DatabaseConfig {
                host: "localhost".to_string(),
                port: 5432,
                database: "sniperforge".to_string(),
                username: "sniperforge".to_string(),
                password: "password".to_string(),
                pool_size: 10,
                connection_timeout_ms: 30000,
            },
            api: ApiConfig {
                host: "0.0.0.0".to_string(),
                port: 8080,
                cors_enabled: true,
                rate_limit_per_minute: 1000,
                max_request_size_mb: 10,
                jwt_secret: "your-secret-key-change-this-in-production".to_string(),
                session_timeout_hours: 24,
            },
            logging: LoggingConfig {
                level: "info".to_string(),
                file_path: Some("logs/sniperforge.log".to_string()),
                max_file_size_mb: 100,
                max_files: 10,
                structured_logging: true,
            },
            security: SecurityConfig {
                encryption_key: "your-encryption-key-change-this".to_string(),
                api_key_required: true,
                rate_limiting_enabled: true,
                audit_logging: true,
                ssl_enabled: false,
                ssl_cert_path: None,
                ssl_key_path: None,
            },
            monitoring: MonitoringConfig {
                metrics_enabled: true,
                prometheus_endpoint: "http://localhost:9090".to_string(),
                health_check_interval_seconds: 30,
                alert_webhook_url: None,
                performance_tracking: true,
            },
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    #[tokio::test]
    async fn test_config_manager_creation() {
        let temp_dir = TempDir::new().unwrap();
        let manager = ConfigManager::new(temp_dir.path());
        
        // Test loading system config (should create default)
        manager.load_system_config().await.unwrap();
        
        let config = manager.get_system_config().await;
        assert_eq!(config.api.port, 8080);
    }

    #[tokio::test]
    async fn test_bot_config_save_load() {
        let temp_dir = TempDir::new().unwrap();
        let manager = ConfigManager::new(temp_dir.path());
        
        let bot_id = Uuid::new_v4();
        let config = BotConfig {
            config_id: Uuid::new_v4(),
            bot_id: bot_id,
            bot_type: crate::api::bot_interface::BotType::EnhancedArbitrage,
            environment: crate::api::bot_interface::Environment::Development,
            parameters: serde_json::json!({
                "test_param": "test_value"
            }),
            resources: crate::api::bot_interface::ResourceLimits {
                max_cpu: 1.0,
                max_memory_mb: 512,
                max_disk_mb: 1024,
                max_network_mbps: Some(100),
            },
            network: crate::api::bot_interface::NetworkConfig {
                solana_rpc_urls: vec!["https://api.mainnet-beta.solana.com".to_string()],
                websocket_urls: vec!["wss://api.mainnet-beta.solana.com".to_string()],
                api_endpoints: std::collections::HashMap::new(),
                timeouts: crate::api::bot_interface::NetworkTimeouts {
                    rpc_timeout_seconds: 30,
                    websocket_timeout_seconds: 30,
                    api_timeout_seconds: 30,
                },
            },
            security: crate::api::bot_interface::SecurityConfig {
                wallet: crate::api::bot_interface::WalletConfig {
                    wallet_type: "keypair".to_string(),
                    address: "".to_string(),
                    private_key_path: Some("./wallet.json".to_string()),
                    use_env_keys: false,
                },
                api_keys: std::collections::HashMap::new(),
                encryption_enabled: true,
                auth_required: true,
            },
            metadata: crate::api::bot_interface::ConfigMetadata {
                name: "Test Configuration".to_string(),
                version: "1.0.0".to_string(),
                created_at: chrono::Utc::now(),
                updated_at: chrono::Utc::now(),
                created_by: "test".to_string(),
                tags: vec!["test".to_string()],
            },
        };
        
        // Save configuration
        manager.save_bot_config(bot_id, &config).await.unwrap();
        
        // Load configuration
        let loaded_config = manager.load_bot_config(bot_id).await.unwrap();
        assert_eq!(loaded_config.parameters.get("test_param").unwrap(), "test_value");
    }
}
