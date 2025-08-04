use std::sync::Arc;
use std::collections::HashMap;
use tokio::sync::RwLock;
use uuid::Uuid;
use async_trait::async_trait;
use chrono::{DateTime, Utc};

use crate::api::bot_interface::{
    BotInterface, BotType, BotStatus, BotConfig, BotMetrics, BotError, HealthStatus,
    BotCapabilities, ValidationResult, ValidationError, HealthLevel, OperationalMetrics, TradingMetrics, 
    PerformanceMetrics, NetworkIOMetrics, ApiCallMetrics, BotFeature, ConfigOption
};

/// Mock arbitrage bot for testing control system
#[derive(Debug)]
pub struct MockArbitrageBot {
    pub id: Uuid,
    pub name: String,
    pub status: Arc<RwLock<BotStatus>>,
    pub metrics: Arc<RwLock<BotMetrics>>,
    pub config: Arc<RwLock<Option<BotConfig>>>,
    pub start_time: Option<DateTime<Utc>>,
}

impl MockArbitrageBot {
    pub fn new(name: String) -> Self {
        Self {
            id: Uuid::new_v4(),
            name,
            status: Arc::new(RwLock::new(BotStatus::Stopped)),
            metrics: Arc::new(RwLock::new(Self::create_default_metrics())),
            config: Arc::new(RwLock::new(None)),
            start_time: None,
        }
    }

    fn create_default_metrics() -> BotMetrics {
        BotMetrics {
            operational: OperationalMetrics {
                uptime_seconds: 0,
                restart_count: 0,
                last_restart: None,
                config_updates: 0,
                error_count: 0,
            },
            trading: TradingMetrics {
                trades_executed: 0,
                successful_trades: 0,
                total_pnl_usd: 0.0,
                success_rate: 0.0,
                avg_profit_per_trade: 0.0,
                total_volume_usd: 0.0,
                sharpe_ratio: None,
            },
            performance: PerformanceMetrics {
                cpu_usage_percent: 0.0,
                memory_usage_mb: 0,
                network_io: NetworkIOMetrics {
                    bytes_sent: 0,
                    bytes_received: 0,
                    packets_sent: 0,
                    packets_received: 0,
                },
                api_calls: ApiCallMetrics {
                    total_calls: 0,
                    successful_calls: 0,
                    failed_calls: 0,
                    avg_response_time_ms: 0.0,
                },
                avg_response_time_ms: 0.0,
                throughput_per_second: 0.0,
            },
            custom: serde_json::Value::Null,
            timestamp: Utc::now(),
        }
    }
}

#[async_trait]
impl BotInterface for MockArbitrageBot {
    fn bot_id(&self) -> Uuid {
        self.id
    }

    fn bot_type(&self) -> BotType {
        BotType::EnhancedArbitrage
    }

    fn version(&self) -> String {
        "1.0.0".to_string()
    }

    async fn start(&mut self, _config: BotConfig) -> Result<(), BotError> {
        *self.status.write().await = BotStatus::Running;
        // Simulate some metrics update
        {
            let mut metrics = self.metrics.write().await;
            metrics.trading.trades_executed = 5;
            metrics.trading.total_pnl_usd = 125.50;
            metrics.trading.success_rate = 0.85;
            metrics.operational.uptime_seconds = 9000; // 2.5 hours
        }
        Ok(())
    }

    async fn stop(&mut self) -> Result<(), BotError> {
        *self.status.write().await = BotStatus::Stopped;
        Ok(())
    }

    async fn pause(&mut self) -> Result<(), BotError> {
        *self.status.write().await = BotStatus::Paused;
        Ok(())
    }

    async fn resume(&mut self) -> Result<(), BotError> {
        *self.status.write().await = BotStatus::Running;
        Ok(())
    }

    async fn status(&self) -> BotStatus {
        self.status.read().await.clone()
    }

    async fn update_config(&mut self, config: BotConfig) -> Result<(), BotError> {
        *self.config.write().await = Some(config);
        Ok(())
    }

    async fn metrics(&self) -> BotMetrics {
        self.metrics.read().await.clone()
    }

    async fn health_check(&self) -> HealthStatus {
        HealthStatus {
            status: HealthLevel::Healthy,
            checks: vec![],
            timestamp: Utc::now(),
            details: HashMap::new(),
        }
    }

    fn capabilities(&self) -> BotCapabilities {
        BotCapabilities {
            networks: vec!["ethereum".to_string(), "polygon".to_string()],
            dexs: vec!["uniswap".to_string(), "sushiswap".to_string()],
            token_types: vec!["ERC20".to_string()],
            features: vec![
                BotFeature::RealTimeTrading,
                BotFeature::SimulationMode,
                BotFeature::RiskManagement,
            ],
            config_options: vec![],
        }
    }

    async fn validate_config(&self, _config: &BotConfig) -> Result<ValidationResult, BotError> {
        Ok(ValidationResult {
            is_valid: true,
            errors: vec![],
            warnings: vec![],
        })
    }
}
