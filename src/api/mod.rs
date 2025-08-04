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
pub use gateway::{ApiGateway, GatewayConfig, AppState};
pub use config_management::{ConfigManager, BotConfigTemplate, SystemConfig};
pub use health_monitoring::{HealthMonitor, HealthReport, SystemHealthMetrics};
pub use metrics_collector::{MetricsCollector, MetricsSummary, MetricsConfig};
