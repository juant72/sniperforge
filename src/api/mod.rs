pub mod bot_interface;
pub mod gateway;
pub mod config_management;
pub mod health_monitoring;
pub mod metrics_collector;

// Re-export main API types
pub use bot_interface::{
    BotInterface, BotType, BotStatus, BotConfig, BotMetrics, HealthStatus, 
    BotCapabilities, ValidationResult, BotError, Environment
};
pub use gateway::{ApiGateway, ApiRoutes};
pub use config_management::{ConfigManager, ConfigTemplate};
pub use health_monitoring::{HealthMonitor, SystemHealth};
pub use metrics_collector::{MetricsCollector, MetricsStorage};
