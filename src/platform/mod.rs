pub mod bot_manager;
pub mod event_bus;
pub mod resource_coordinator;

use anyhow::Result;
use std::sync::Arc;
use tokio::sync::{broadcast, RwLock};
use tracing::{error, info, warn};

use crate::config::Config;
use crate::shared::SharedServices;
use crate::types::{BotId, BotType, PlatformMetrics};

use bot_manager::BotManager;
use event_bus::{EventBus, PlatformEvent};
use resource_coordinator::ResourceCoordinator;

pub struct SniperForgePlatform {
    config: Config,
    bot_manager: Arc<BotManager>,
    event_bus: Arc<EventBus>,
    resource_coordinator: Arc<ResourceCoordinator>,
    shared_services: Arc<SharedServices>,
    shutdown_tx: broadcast::Sender<()>,
    is_running: Arc<RwLock<bool>>,
}

impl SniperForgePlatform {
    pub async fn new(config: Config) -> Result<Self> {
        info!("🏗️ Initializing SniperForge Platform");

        // Validate configuration
        config.validate()?;

        // Create shutdown channel
        let (shutdown_tx, _) = broadcast::channel(1);

        // Initialize shared services
        info!("🔧 Initializing shared services");
        let shared_services = Arc::new(SharedServices::new(&config).await?);

        // Initialize event bus
        info!("📡 Initializing event bus");
        let event_bus = Arc::new(EventBus::new(Some(config.platform.event_bus_buffer_size)));

        // Initialize resource coordinator
        info!("⚖️ Initializing resource coordinator");
        let resource_coordinator = Arc::new(ResourceCoordinator::new());

        // Initialize bot manager
        info!("🤖 Initializing bot manager");
        let bot_manager = Arc::new(
            BotManager::new(
                config.clone(),
                shared_services.clone(),
                event_bus.clone(),
                resource_coordinator.clone(),
            )
            .await?,
        );

        let platform = Self {
            config,
            bot_manager,
            event_bus,
            resource_coordinator,
            shared_services,
            shutdown_tx,
            is_running: Arc::new(RwLock::new(false)),
        };

        info!("✅ Platform initialization completed");
        Ok(platform)
    }

    pub async fn start_platform(&self) -> Result<()> {
        info!("🚀 Starting SniperForge Platform");

        *self.is_running.write().await = true;

        // Start shared services
        self.shared_services.start().await?;

        // Start event bus
        self.event_bus.start().await?;

        // Start resource coordinator
        self.resource_coordinator.start().await?;

        // Start enabled bots
        self.bot_manager.start_enabled_bots().await?;
        // Publish platform started event
        self.event_bus
            .publish_simple(
                event_bus::EventType::SystemMetrics,
                "platform".to_string(),
                serde_json::json!({
                    "event": "platform_started",
                    "timestamp": chrono::Utc::now()
                }),
                Some(event_bus::EventPriority::Normal),
            )
            .await?;

        info!("✅ Platform started successfully");
        Ok(())
    }

    pub async fn start_specific_bots(&self, bot_types: Vec<String>) -> Result<()> {
        info!("🤖 Starting specific bots: {:?}", bot_types);

        *self.is_running.write().await = true;

        // Start shared services
        self.shared_services.start().await?;

        // Start event bus
        self.event_bus.start().await?;

        // Start resource coordinator
        self.resource_coordinator.start().await?;

        // Start specific bots
        for bot_type_str in bot_types {
            if let Some(bot_type) = self.parse_bot_type(&bot_type_str) {
                self.bot_manager.start_bot(bot_type).await?;
            } else {
                warn!("Unknown bot type: {}", bot_type_str);
            }
        }

        info!("✅ Specific bots started successfully");
        Ok(())
    }

    pub async fn run(&self) -> Result<()> {
        info!("🔄 Platform running, waiting for events...");

        // Subscribe to shutdown signal
        let mut shutdown_rx = self.shutdown_tx.subscribe();

        // Main event loop
        loop {
            tokio::select! {
                // Handle shutdown signal
                _ = shutdown_rx.recv() => {
                    info!("📡 Shutdown signal received");
                    break;
                }

                // Handle platform events (placeholder for now)
                _ = tokio::time::sleep(tokio::time::Duration::from_secs(1)) => {
                    // Health check and metrics collection
                    if let Err(e) = self.perform_health_check().await {
                        error!("❌ Health check failed: {}", e);
                    }
                }
            }
        }

        Ok(())
    }

    pub async fn shutdown(&self) -> Result<()> {
        info!("🛑 Shutting down SniperForge Platform");

        *self.is_running.write().await = false;
        // Publish shutdown event
        if let Err(e) = self
            .event_bus
            .publish_simple(
                event_bus::EventType::SystemMetrics,
                "platform".to_string(),
                serde_json::json!({
                    "event": "platform_stopping",
                    "timestamp": chrono::Utc::now()
                }),
                Some(event_bus::EventPriority::High),
            )
            .await
        {
            error!("Failed to publish shutdown event: {}", e);
        }

        // Stop bots first
        if let Err(e) = self.bot_manager.stop_all_bots().await {
            error!("❌ Error stopping bots: {}", e);
        }

        // Stop coordinators
        if let Err(e) = self.resource_coordinator.stop().await {
            error!("❌ Error stopping resource coordinator: {}", e);
        }

        // Stop event bus
        if let Err(e) = self.event_bus.stop().await {
            error!("❌ Error stopping event bus: {}", e);
        }

        // Stop shared services last
        if let Err(e) = self.shared_services.stop().await {
            error!("❌ Error stopping shared services: {}", e);
        }

        // Send shutdown signal
        if let Err(e) = self.shutdown_tx.send(()) {
            error!("❌ Error sending shutdown signal: {}", e);
        }

        info!("✅ Platform shutdown completed");
        Ok(())
    }

    pub async fn get_metrics(&self) -> Result<PlatformMetrics> {
        let bot_metrics = self.bot_manager.get_all_metrics().await?;
        let system_metrics = self.shared_services.get_metrics().await?;

        Ok(PlatformMetrics {
            total_bots: bot_metrics.len(),
            active_bots: bot_metrics
                .iter()
                .filter(|m| matches!(m.1, crate::types::BotStatus::Running))
                .count(),
            total_trades: bot_metrics
                .iter()
                .map(|(metrics, _)| metrics.operations_count)
                .sum(),
            successful_trades: 0,  // TODO: Calculate from trade results
            total_volume_usd: 0.0, // TODO: Calculate from trade results
            total_profit_usd: 0.0, // TODO: Calculate from trade results
            avg_latency_ms: if !bot_metrics.is_empty() {
                bot_metrics
                    .iter()
                    .map(|(metrics, _)| metrics.avg_latency_ms)
                    .sum::<f64>()
                    / bot_metrics.len() as f64
            } else {
                0.0
            },
            cpu_usage_percent: system_metrics.cpu_usage_percent,
            memory_usage_mb: system_metrics.memory_usage_mb,
            uptime_seconds: system_metrics.uptime_seconds,
            last_updated: chrono::Utc::now(),
        })
    }

    async fn perform_health_check(&self) -> Result<()> {
        // Check shared services health
        let shared_health = self.shared_services.health_check().await?;
        if !shared_health.is_healthy {
            warn!(
                "🔧 Shared services health check failed: {:?}",
                shared_health.message
            );
        }

        // Check bot manager health
        let bot_health = self.bot_manager.health_check().await?;
        if !bot_health.is_healthy {
            warn!(
                "🤖 Bot manager health check failed: {:?}",
                bot_health.message
            );
        }

        // Check resource coordinator health
        let resource_health = self.resource_coordinator.health_check().await?;
        if !resource_health.is_healthy {
            warn!(
                "⚖️ Resource coordinator health check failed: {:?}",
                resource_health.message
            );
        }

        Ok(())
    }

    fn parse_bot_type(&self, bot_type_str: &str) -> Option<BotType> {
        match bot_type_str.to_lowercase().as_str() {
            "lp-sniper" | "lp_sniper" | "lpsniper" => Some(BotType::LpSniper),
            "copy-trading" | "copy_trading" | "copytrading" => Some(BotType::CopyTrading),
            "arbitrage" => Some(BotType::Arbitrage),
            "mev" => Some(BotType::Mev),
            "grid-trading" | "grid_trading" | "gridtrading" => Some(BotType::GridTrading),
            "dca" | "dollar-cost-average" | "dollar_cost_average" => {
                Some(BotType::DollarCostAverage)
            }
            _ => None,
        }
    }

    /// Check if platform is running
    pub async fn is_running(&self) -> bool {
        *self.is_running.read().await
    }
}
