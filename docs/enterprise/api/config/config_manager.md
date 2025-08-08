# ConfigManager API Documentation

## Overview

The **ConfigManager** is the enterprise configuration management system for SniperForge, providing dynamic configuration loading, validation, hot-reloading, and template management. It supports YAML-based configuration with enterprise-grade validation and security.

## Architecture

```
┌─────────────────────────────────────────────────────────────┐
│                    ConfigManager                            │
├─────────────────────────────────────────────────────────────┤
│  Core Components:                                           │
│  • ConfigLoader (YAML/JSON loading)                        │
│  • ConfigValidator (Schema validation)                     │
│  • TemplateManager (Configuration templates)               │
│  • HotReloadSystem (Real-time config updates)              │
│  • ConfigCache (Performance optimization)                  │
│  • SecurityValidator (Security compliance)                 │
│  • AuditLogger (Configuration change tracking)             │
└─────────────────────────────────────────────────────────────┘
```

## Constructor

### `ConfigManager::new(config_path: &str) -> Self`

Creates a new ConfigManager with specified configuration directory.

**Parameters:**
- `config_path: &str` - Path to configuration directory

**Returns:**
- `ConfigManager` - Configured manager instance

**Directory Structure:**
```
config/
├── system.yaml           # System-wide configuration
├── templates/            # Configuration templates
│   ├── bot_templates.yaml
│   ├── trading_templates.yaml
│   └── security_templates.yaml
├── bots/                # Bot-specific configurations
│   ├── {bot_id}.yaml
│   └── defaults.yaml
├── environments/        # Environment-specific configs
│   ├── development.yaml
│   ├── staging.yaml
│   └── production.yaml
└── schemas/            # Validation schemas
    ├── bot_schema.yaml
    └── system_schema.yaml
```

**Example:**
```rust
let config_manager = ConfigManager::new("./config");
```

---

## System Configuration Management

### `load_system_config() -> Result<SystemConfig>`

Loads and validates system-wide configuration.

**Returns:**
- `Result<SystemConfig>` - Complete system configuration

**SystemConfig Structure:**
```rust
pub struct SystemConfig {
    pub database: DatabaseConfig,
    pub api: ApiConfig,
    pub logging: LoggingConfig,
    pub security: SecurityConfig,
    pub monitoring: MonitoringConfig,
    pub trading: TradingConfig,
    pub performance: PerformanceConfig,
}
```

**Example:**
```rust
let system_config = config_manager.load_system_config().await?;
println!("Database host: {}", system_config.database.host);
println!("API port: {}", system_config.api.port);
```

**Configuration Validation:**
- Schema validation against JSON Schema
- Security policy compliance checking
- Performance parameter validation
- Cross-reference consistency validation

### `save_system_config(config: SystemConfig) -> Result<()>`

Saves system configuration with validation and backup.

**Parameters:**
- `config: SystemConfig` - System configuration to save

**Returns:**
- `Result<()>` - Success or error

**Process:**
1. Validates configuration against schema
2. Creates backup of existing configuration
3. Performs security compliance check
4. Saves new configuration atomically
5. Triggers configuration change event

**Example:**
```rust
let mut system_config = config_manager.load_system_config().await?;
system_config.api.port = 9000;
system_config.api.enable_cors = true;

config_manager.save_system_config(system_config).await?;
```

---

## Bot Configuration Management

### `load_bot_config(bot_id: Uuid) -> Result<BotConfig>`

Loads configuration for a specific bot with template inheritance.

**Parameters:**
- `bot_id: Uuid` - Unique identifier of the bot

**Returns:**
- `Result<BotConfig>` - Bot configuration with applied templates

**Template Inheritance:**
1. Load base template for bot type
2. Apply environment-specific overrides
3. Apply bot-specific configuration
4. Validate final configuration

**Example:**
```rust
let bot_config = config_manager.load_bot_config(bot_id).await?;
println!("Bot type: {:?}", bot_config.bot_type);
println!("Trading pairs: {:?}", bot_config.trading_pairs);
```

### `save_bot_config(bot_id: Uuid, config: &BotConfig) -> Result<()>`

Saves bot configuration with validation and versioning.

**Parameters:**
- `bot_id: Uuid` - Unique identifier of the bot
- `config: &BotConfig` - Bot configuration to save

**Returns:**
- `Result<()>` - Success or error

**Features:**
- Configuration versioning for rollback
- Hot-reload notification to running bots
- Validation against bot-specific schemas
- Audit logging for configuration changes

**Example:**
```rust
let mut bot_config = config_manager.load_bot_config(bot_id).await?;
bot_config.max_position_size = 2000.0;
bot_config.risk_level = "Conservative".to_string();

config_manager.save_bot_config(bot_id, &bot_config).await?;
```

### `validate_bot_config(bot_type: &BotType, config: &BotConfig) -> Result<()>`

Validates bot configuration against type-specific schema and business rules.

**Parameters:**
- `bot_type: &BotType` - Type of bot for validation
- `config: &BotConfig` - Configuration to validate

**Returns:**
- `Result<()>` - Success or validation error

**Validation Rules:**
- **LiquiditySniper**: Minimum liquidity thresholds, valid trading pairs
- **ArbitrageBot**: Exchange connectivity, profit thresholds
- **MLAnalytics**: Model parameters, data source validation
- **PortfolioManager**: Position limits, rebalancing rules

**Example:**
```rust
let validation_result = config_manager.validate_bot_config(&BotType::LiquiditySniper, &config).await;
match validation_result {
    Ok(_) => println!("✅ Configuration valid"),
    Err(e) => println!("❌ Validation failed: {}", e),
}
```

---

## Template Management

### `load_template(template_name: &str) -> Result<ConfigTemplate>`

Loads a configuration template for reuse across multiple bots.

**Parameters:**
- `template_name: &str` - Name of the template to load

**Returns:**
- `Result<ConfigTemplate>` - Template configuration

**Built-in Templates:**
- `conservative_liquidity_sniper` - Low-risk liquidity sniping
- `aggressive_arbitrage` - High-frequency arbitrage
- `balanced_portfolio` - Diversified portfolio management
- `ml_enhanced_trading` - AI-powered trading strategies

**Example:**
```rust
let template = config_manager.load_template("conservative_liquidity_sniper").await?;
let bot_config = template.instantiate_for_bot(bot_id, custom_params)?;
```

### `create_template(name: &str, config: ConfigTemplate) -> Result<()>`

Creates a new configuration template for reuse.

**Parameters:**
- `name: &str` - Name for the new template
- `config: ConfigTemplate` - Template configuration

**Returns:**
- `Result<()>` - Success or error

**ConfigTemplate Structure:**
```rust
pub struct ConfigTemplate {
    pub name: String,
    pub description: String,
    pub bot_type: BotType,
    pub parameters: HashMap<String, TemplateParameter>,
    pub defaults: BotConfig,
    pub validation_rules: Vec<ValidationRule>,
}
```

**Example:**
```rust
let template = ConfigTemplate {
    name: "custom_sniper".to_string(),
    description: "Custom liquidity sniper configuration".to_string(),
    bot_type: BotType::LiquiditySniper,
    parameters: HashMap::from([
        ("max_position".to_string(), TemplateParameter::Float {
            default: 1000.0,
            min: 100.0,
            max: 10000.0,
        }),
        ("risk_level".to_string(), TemplateParameter::String {
            default: "Moderate".to_string(),
            allowed_values: vec!["Conservative".to_string(), "Moderate".to_string(), "Aggressive".to_string()],
        }),
    ]),
    defaults: BotConfig::default_for_type(BotType::LiquiditySniper),
    validation_rules: vec![
        ValidationRule::PositionSizeLimit(10000.0),
        ValidationRule::RequiredTradingPairs(1),
    ],
};

config_manager.create_template("custom_sniper", template).await?;
```

### `list_templates() -> Result<Vec<TemplateInfo>>`

Lists all available configuration templates.

**Returns:**
- `Result<Vec<TemplateInfo>>` - List of template information

**TemplateInfo Structure:**
```rust
pub struct TemplateInfo {
    pub name: String,
    pub description: String,
    pub bot_type: BotType,
    pub parameter_count: usize,
    pub created_at: DateTime<Utc>,
    pub usage_count: u64,
}
```

**Example:**
```rust
let templates = config_manager.list_templates().await?;
for template in templates {
    println!("{}: {} ({})", template.name, template.description, template.bot_type);
}
```

---

## Hot-Reload System

### `hot_reload() -> Result<()>`

Performs hot-reload of all configurations without system restart.

**Returns:**
- `Result<()>` - Success or error

**Hot-Reload Process:**
1. Scans all configuration files for changes
2. Validates new configurations
3. Backs up current configurations
4. Applies new configurations atomically
5. Notifies all affected components
6. Logs all configuration changes

**Example:**
```rust
config_manager.hot_reload().await?;
println!("✅ All configurations reloaded successfully");
```

**Performance:**
- Configuration reload: < 200ms
- Zero downtime for running bots
- Atomic configuration updates
- Automatic rollback on validation failure

### `watch_config_changes() -> Result<ConfigWatcher>`

Sets up automatic configuration change monitoring.

**Returns:**
- `Result<ConfigWatcher>` - Configuration change watcher

**Features:**
- File system change monitoring
- Automatic hot-reload on changes
- Change notification to subscribers
- Debounced reload to prevent thrashing

**Example:**
```rust
let watcher = config_manager.watch_config_changes().await?;

watcher.on_change(|change_event| {
    println!("Configuration changed: {:?}", change_event);
    // Handle configuration change
}).await;
```

### `subscribe_to_changes(bot_id: Option<Uuid>) -> Result<ConfigChangeReceiver>`

Subscribes to configuration change notifications.

**Parameters:**
- `bot_id: Option<Uuid>` - Specific bot ID to watch, or None for all changes

**Returns:**
- `Result<ConfigChangeReceiver>` - Change notification receiver

**Example:**
```rust
let receiver = config_manager.subscribe_to_changes(Some(bot_id)).await?;

while let Some(change) = receiver.recv().await {
    match change.change_type {
        ConfigChangeType::BotConfig => {
            // Reload bot configuration
            let new_config = config_manager.load_bot_config(bot_id).await?;
            bot.update_config(new_config).await?;
        }
        ConfigChangeType::SystemConfig => {
            // Handle system configuration change
        }
    }
}
```

---

## Environment Management

### `load_environment_config(environment: &str) -> Result<EnvironmentConfig>`

Loads environment-specific configuration (development, staging, production).

**Parameters:**
- `environment: &str` - Environment name ("development", "staging", "production")

**Returns:**
- `Result<EnvironmentConfig>` - Environment-specific configuration

**EnvironmentConfig Structure:**
```rust
pub struct EnvironmentConfig {
    pub name: String,
    pub database_url: String,
    pub api_endpoints: HashMap<String, String>,
    pub log_level: String,
    pub debug_enabled: bool,
    pub rate_limits: RateLimitConfig,
    pub security_settings: SecurityConfig,
    pub feature_flags: HashMap<String, bool>,
}
```

**Example:**
```rust
let env_config = config_manager.load_environment_config("production").await?;
println!("Environment: {}", env_config.name);
println!("Debug enabled: {}", env_config.debug_enabled);
```

### `set_active_environment(environment: &str) -> Result<()>`

Sets the active environment for configuration resolution.

**Parameters:**
- `environment: &str` - Environment to activate

**Returns:**
- `Result<()>` - Success or error

**Effect:**
- All subsequent configuration loads will use environment-specific overrides
- Template instantiation will apply environment defaults
- Validation rules will use environment-specific constraints

**Example:**
```rust
config_manager.set_active_environment("production").await?;
// All configs now use production environment settings
```

---

## Configuration Validation

### `validate_configuration_integrity() -> Result<ValidationReport>`

Performs comprehensive validation of all configurations.

**Returns:**
- `Result<ValidationReport>` - Complete validation report

**ValidationReport Structure:**
```rust
pub struct ValidationReport {
    pub overall_status: ValidationStatus,
    pub system_config_status: ValidationStatus,
    pub bot_configs_status: HashMap<Uuid, ValidationStatus>,
    pub template_status: HashMap<String, ValidationStatus>,
    pub errors: Vec<ConfigError>,
    pub warnings: Vec<ConfigWarning>,
    pub recommendations: Vec<String>,
}
```

**Validation Checks:**
- Schema compliance validation
- Cross-reference consistency
- Security policy compliance
- Performance constraint validation
- Business rule compliance

**Example:**
```rust
let report = config_manager.validate_configuration_integrity().await?;
println!("Overall status: {:?}", report.overall_status);

for error in report.errors {
    println!("❌ Error: {}", error);
}

for warning in report.warnings {
    println!("⚠️ Warning: {}", warning);
}
```

### `validate_schema(config_type: ConfigType, data: &serde_json::Value) -> Result<()>`

Validates configuration data against JSON Schema.

**Parameters:**
- `config_type: ConfigType` - Type of configuration to validate
- `data: &serde_json::Value` - Configuration data to validate

**Returns:**
- `Result<()>` - Success or validation error

**ConfigType Options:**
- `SystemConfig` - System-wide configuration
- `BotConfig(BotType)` - Bot-specific configuration
- `TemplateConfig` - Template configuration
- `EnvironmentConfig` - Environment configuration

**Example:**
```rust
let config_json = serde_json::to_value(&bot_config)?;
config_manager.validate_schema(ConfigType::BotConfig(BotType::LiquiditySniper), &config_json).await?;
```

---

## Configuration Backup & Recovery

### `create_configuration_backup() -> Result<BackupInfo>`

Creates a backup of all current configurations.

**Returns:**
- `Result<BackupInfo>` - Information about created backup

**BackupInfo Structure:**
```rust
pub struct BackupInfo {
    pub backup_id: String,
    pub created_at: DateTime<Utc>,
    pub backup_path: PathBuf,
    pub file_count: usize,
    pub total_size_bytes: u64,
    pub checksum: String,
}
```

**Backup Contents:**
- All system configurations
- All bot configurations
- Configuration templates
- Environment configurations
- Validation schemas

**Example:**
```rust
let backup = config_manager.create_configuration_backup().await?;
println!("Backup created: {} ({})", backup.backup_id, backup.backup_path.display());
```

### `restore_from_backup(backup_id: &str) -> Result<()>`

Restores configurations from a previous backup.

**Parameters:**
- `backup_id: &str` - Identifier of the backup to restore

**Returns:**
- `Result<()>` - Success or error

**Restore Process:**
1. Validates backup integrity
2. Creates current state backup
3. Restores configurations atomically
4. Validates restored configurations
5. Triggers hot-reload of all components

**Example:**
```rust
config_manager.restore_from_backup("backup_20250808_143022").await?;
println!("✅ Configuration restored successfully");
```

### `list_backups() -> Result<Vec<BackupInfo>>`

Lists all available configuration backups.

**Returns:**
- `Result<Vec<BackupInfo>>` - List of available backups

**Example:**
```rust
let backups = config_manager.list_backups().await?;
for backup in backups {
    println!("{}: {} ({} files)", backup.backup_id, backup.created_at, backup.file_count);
}
```

---

## Configuration Auditing

### `get_configuration_audit_log(filter: AuditFilter) -> Result<Vec<ConfigAuditEntry>>`

Retrieves configuration change audit log.

**Parameters:**
- `filter: AuditFilter` - Filter criteria for audit entries

**Returns:**
- `Result<Vec<ConfigAuditEntry>>` - Filtered audit entries

**ConfigAuditEntry Structure:**
```rust
pub struct ConfigAuditEntry {
    pub timestamp: DateTime<Utc>,
    pub change_type: ConfigChangeType,
    pub config_path: String,
    pub user_id: Option<String>,
    pub changes: Vec<ConfigChange>,
    pub validation_status: ValidationStatus,
}
```

**Example:**
```rust
let filter = AuditFilter {
    start_time: Some(Utc::now() - chrono::Duration::hours(24)),
    end_time: Some(Utc::now()),
    change_types: vec![ConfigChangeType::BotConfig],
    user_id: None,
};

let audit_entries = config_manager.get_configuration_audit_log(filter).await?;
for entry in audit_entries {
    println!("{}: {} changes to {}", entry.timestamp, entry.changes.len(), entry.config_path);
}
```

### `track_configuration_usage() -> Result<ConfigUsageStats>`

Tracks configuration usage statistics for optimization.

**Returns:**
- `Result<ConfigUsageStats>` - Configuration usage statistics

**ConfigUsageStats Structure:**
```rust
pub struct ConfigUsageStats {
    pub most_used_templates: Vec<(String, u64)>,
    pub configuration_load_frequency: HashMap<String, u64>,
    pub hot_reload_frequency: u64,
    pub validation_failure_rate: f64,
    pub average_load_time_ms: f64,
}
```

**Example:**
```rust
let stats = config_manager.track_configuration_usage().await?;
println!("Most used template: {:?}", stats.most_used_templates.first());
println!("Average load time: {:.2}ms", stats.average_load_time_ms);
```

---

## Performance Optimization

### `optimize_configuration_cache() -> Result<CacheOptimizationResult>`

Optimizes configuration cache for improved performance.

**Returns:**
- `Result<CacheOptimizationResult>` - Optimization results

**Optimization Techniques:**
- Preloading frequently accessed configurations
- Compressing rarely used configurations
- Memory-mapping large configuration files
- Predictive caching based on usage patterns

**Example:**
```rust
let optimization = config_manager.optimize_configuration_cache().await?;
println!("Cache hit rate improved by: {:.2}%", optimization.hit_rate_improvement);
```

### `get_configuration_performance_metrics() -> Result<ConfigPerformanceMetrics>`

Retrieves performance metrics for configuration operations.

**Returns:**
- `Result<ConfigPerformanceMetrics>` - Performance metrics

**ConfigPerformanceMetrics Structure:**
```rust
pub struct ConfigPerformanceMetrics {
    pub load_time_percentiles: HashMap<String, f64>,
    pub cache_hit_rate: f64,
    pub validation_time_avg_ms: f64,
    pub hot_reload_time_avg_ms: f64,
    pub memory_usage_mb: f64,
    pub disk_usage_mb: f64,
}
```

**Example:**
```rust
let metrics = config_manager.get_configuration_performance_metrics().await?;
println!("Cache hit rate: {:.2}%", metrics.cache_hit_rate * 100.0);
println!("P95 load time: {:.2}ms", metrics.load_time_percentiles.get("p95").unwrap_or(&0.0));
```

---

## Advanced Features

### `merge_configurations(base: &BotConfig, override_config: &BotConfig) -> Result<BotConfig>`

Merges two configurations with conflict resolution.

**Parameters:**
- `base: &BotConfig` - Base configuration
- `override_config: &BotConfig` - Override configuration

**Returns:**
- `Result<BotConfig>` - Merged configuration

**Merge Rules:**
- Override values take precedence
- Arrays are merged, not replaced
- Null values in override remove base values
- Validation is performed on merged result

**Example:**
```rust
let base_config = config_manager.load_template("conservative_base").await?;
let custom_overrides = BotConfig {
    max_position_size: 5000.0,
    trading_pairs: vec!["SOL/USDC".to_string()],
    ..Default::default()
};

let merged = config_manager.merge_configurations(&base_config.defaults, &custom_overrides).await?;
```

### `generate_configuration_diff(old: &BotConfig, new: &BotConfig) -> ConfigDiff`

Generates a detailed diff between two configurations.

**Parameters:**
- `old: &BotConfig` - Original configuration
- `new: &BotConfig` - Updated configuration

**Returns:**
- `ConfigDiff` - Detailed difference report

**ConfigDiff Structure:**
```rust
pub struct ConfigDiff {
    pub added_fields: HashMap<String, serde_json::Value>,
    pub removed_fields: HashMap<String, serde_json::Value>,
    pub modified_fields: HashMap<String, (serde_json::Value, serde_json::Value)>,
    pub unchanged_fields: Vec<String>,
}
```

**Example:**
```rust
let diff = config_manager.generate_configuration_diff(&old_config, &new_config);
for (field, (old_val, new_val)) in diff.modified_fields {
    println!("Changed {}: {} -> {}", field, old_val, new_val);
}
```

---

## Error Handling

### Configuration Error Types

```rust
#[derive(Debug, thiserror::Error)]
pub enum ConfigError {
    #[error("Configuration file not found: {0}")]
    FileNotFound(String),
    
    #[error("Invalid configuration format: {0}")]
    InvalidFormat(String),
    
    #[error("Configuration validation failed: {0}")]
    ValidationFailed(String),
    
    #[error("Template not found: {0}")]
    TemplateNotFound(String),
    
    #[error("Schema validation error: {0}")]
    SchemaValidationError(String),
    
    #[error("Environment not found: {0}")]
    EnvironmentNotFound(String),
    
    #[error("Configuration merge conflict: {0}")]
    MergeConflict(String),
}
```

### Error Recovery

The ConfigManager implements robust error recovery:

1. **File Not Found**: Automatic template fallback and defaults
2. **Validation Failures**: Rollback to last known good configuration
3. **Hot-Reload Errors**: Partial update with error reporting
4. **Schema Errors**: Detailed validation reports with fix suggestions

---

## Enterprise Integration Example

```rust
use sniperforge::api::config_management::ConfigManager;

async fn enterprise_config_setup() -> anyhow::Result<()> {
    // Initialize configuration manager
    let config_manager = ConfigManager::new("./config");
    
    // Set production environment
    config_manager.set_active_environment("production").await?;
    
    // Load system configuration
    let system_config = config_manager.load_system_config().await?;
    println!("Loaded system config for: {}", system_config.environment);
    
    // Create bot from template
    let template = config_manager.load_template("enterprise_liquidity_sniper").await?;
    let bot_config = template.instantiate_for_bot(
        bot_id,
        HashMap::from([
            ("max_position_size".to_string(), "10000.0".to_string()),
            ("risk_level".to_string(), "Conservative".to_string()),
        ])
    )?;
    
    // Validate and save bot configuration
    config_manager.validate_bot_config(&BotType::LiquiditySniper, &bot_config).await?;
    config_manager.save_bot_config(bot_id, &bot_config).await?;
    
    // Set up hot-reload monitoring
    let receiver = config_manager.subscribe_to_changes(Some(bot_id)).await?;
    
    tokio::spawn(async move {
        while let Some(change) = receiver.recv().await {
            println!("Configuration changed: {:?}", change);
            // Handle configuration change in running bot
        }
    });
    
    // Create daily backup
    let backup = config_manager.create_configuration_backup().await?;
    println!("Daily backup created: {}", backup.backup_id);
    
    // Monitor configuration performance
    let metrics = config_manager.get_configuration_performance_metrics().await?;
    println!("Config performance:");
    println!("- Cache hit rate: {:.2}%", metrics.cache_hit_rate * 100.0);
    println!("- Average load time: {:.2}ms", metrics.validation_time_avg_ms);
    
    // Validate all configurations
    let report = config_manager.validate_configuration_integrity().await?;
    match report.overall_status {
        ValidationStatus::Valid => println!("✅ All configurations valid"),
        ValidationStatus::Warning => {
            println!("⚠️ Configuration warnings found:");
            for warning in report.warnings {
                println!("  - {}", warning);
            }
        }
        ValidationStatus::Error => {
            println!("❌ Configuration errors found:");
            for error in report.errors {
                println!("  - {}", error);
            }
        }
    }
    
    Ok(())
}
```

---

## Performance Characteristics

### Latency Guarantees

- Configuration load: < 50ms
- Hot-reload: < 200ms
- Validation: < 100ms
- Template instantiation: < 20ms
- Cache lookup: < 1ms

### Throughput

- Concurrent config loads: 500/second
- Hot-reload frequency: 10/second
- Validation operations: 1000/second
- Template instantiations: 200/second

### Memory Usage

- Base overhead: ~5MB
- Cached configurations: ~1MB per 100 configs
- Template cache: ~2MB for standard templates
- Hot-reload monitoring: ~500KB

---

## Configuration Schema Examples

### Bot Configuration Schema

```yaml
# schemas/bot_schema.yaml
type: object
properties:
  bot_type:
    type: string
    enum: ["LiquiditySniper", "EnhancedArbitrage", "MLAnalytics"]
  trading_pairs:
    type: array
    items:
      type: string
      pattern: "^[A-Z]+/[A-Z]+$"
    minItems: 1
    maxItems: 50
  max_position_size:
    type: number
    minimum: 1.0
    maximum: 1000000.0
  risk_level:
    type: string
    enum: ["Conservative", "Moderate", "Aggressive"]
  enable_ai_analysis:
    type: boolean
    default: false
required:
  - bot_type
  - trading_pairs
  - max_position_size
```

### Template Configuration Example

```yaml
# templates/conservative_liquidity_sniper.yaml
name: "conservative_liquidity_sniper"
description: "Conservative liquidity sniping configuration"
bot_type: "LiquiditySniper"
parameters:
  max_position_size:
    type: "float"
    default: 1000.0
    min: 100.0
    max: 5000.0
  risk_level:
    type: "string"
    default: "Conservative"
    allowed_values: ["Conservative", "Moderate"]
  slippage_tolerance:
    type: "float"
    default: 0.5
    min: 0.1
    max: 2.0
defaults:
  bot_type: "LiquiditySniper"
  trading_pairs: ["SOL/USDC", "BTC/USDT"]
  max_position_size: 1000.0
  risk_level: "Conservative"
  slippage_tolerance: 0.5
  enable_ai_analysis: true
  min_liquidity_threshold: 50000.0
validation_rules:
  - type: "position_size_limit"
    value: 5000.0
  - type: "required_trading_pairs"
    value: 1
```
