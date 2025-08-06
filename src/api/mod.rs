pub mod bot_interface;
pub mod gateway;
pub mod config_management;
pub mod health_monitoring;
pub mod metrics_collector;
pub mod state_persistence;
pub mod yaml_config;

// Re-export main API types
pub use bot_interface::{
    BotInterface, BotType, BotStatus, BotConfig, BotMetrics, HealthStatus, 
    BotCapabilities, ValidationResult, BotError, Environment
};
pub use gateway::{ApiGateway, GatewayConfig, AppState};
pub use config_management::{ConfigManager, BotConfigTemplate, SystemConfig};
pub use health_monitoring::{HealthMonitor, HealthReport, SystemHealthMetrics};
pub use metrics_collector::{MetricsCollector, SystemMetricsSummary, MetricsConfig};
pub use state_persistence::{
    StatePersistenceManager, PersistedBotState, PersistedSystemMetrics, 
    SystemStateSnapshot, PersistenceError
};
pub use yaml_config::{
    YamlConfigManager, SystemConfigYaml, BotConfigYaml, YamlConfigError
};
