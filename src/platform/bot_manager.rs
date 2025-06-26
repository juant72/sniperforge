use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::{RwLock, mpsc};
use anyhow::Result;
use tracing::{info, warn, error, debug};
use uuid::Uuid;

use crate::config::Config;
use crate::shared::SharedServices;
use crate::platform::event_bus::EventBus;
use crate::platform::resource_coordinator::ResourceCoordinator;
use crate::types::{BotType, BotStatus, BotConfig, BotInstance, BotCommand, BotEvent, PlatformError, BotMetrics};

/// Manages all bot instances in the platform
pub struct BotManager {
    config: Config,
    bots: Arc<RwLock<HashMap<Uuid, BotInstance>>>,
    shared_services: Arc<SharedServices>,
    event_bus: Arc<EventBus>,
    resource_coordinator: Arc<ResourceCoordinator>,
    command_tx: mpsc::UnboundedSender<BotCommand>,
    event_rx: mpsc::UnboundedReceiver<BotEvent>,
    shutdown_tx: mpsc::Sender<()>,
}

impl BotManager {
    pub async fn new(
        config: Config,
        shared_services: Arc<SharedServices>,
        event_bus: Arc<EventBus>,
        resource_coordinator: Arc<ResourceCoordinator>,
    ) -> Result<Self> {
        let (command_tx, _command_rx) = mpsc::unbounded_channel();
        let (_event_tx, event_rx) = mpsc::unbounded_channel();
        let (shutdown_tx, _) = mpsc::channel(1);

        let manager = Self {
            config,
            bots: Arc::new(RwLock::new(HashMap::new())),
            shared_services,
            event_bus,
            resource_coordinator,
            command_tx,
            event_rx,
            shutdown_tx,
        };

        Ok(manager)
    }

    /// Start the bot manager
    pub async fn start(&mut self) -> Result<()> {
        info!("Starting Bot Manager");
        
        // Initialize bot management loop
        self.run_management_loop().await?;
        
        Ok(())
    }

    /// Stop the bot manager and all bots
    pub async fn stop(&self) -> Result<()> {
        info!("Stopping Bot Manager and all bots");
        
        let bots = self.bots.read().await;
        for (bot_id, bot) in bots.iter() {
            info!("Stopping bot: {} ({})", bot.name, bot_id);
            // Send stop command to bot
            // Implementation will depend on bot architecture
        }
        
        // Send shutdown signal
        let _ = self.shutdown_tx.send(()).await;
        
        Ok(())
    }

    /// Create and register a new bot instance
    pub async fn create_bot(&self, bot_type: BotType, config: BotConfig) -> Result<Uuid> {
        let bot_id = Uuid::new_v4();
        
        let bot_instance = BotInstance {
            id: bot_id,
            bot_type: bot_type.clone(),
            name: format!("{:?}-{}", bot_type, &bot_id.to_string()[..8]),
            status: BotStatus::Stopped,
            config,
            created_at: chrono::Utc::now(),
            last_activity: chrono::Utc::now(),
            metrics: Default::default(),
        };

        let mut bots = self.bots.write().await;
        bots.insert(bot_id, bot_instance);
        
        info!("Created bot: {} ({})", bot_type, bot_id);
        Ok(bot_id)
    }

    /// Start enabled bots from configuration
    pub async fn start_enabled_bots(&self) -> Result<()> {
        info!("🤖 Starting enabled bots from configuration");
        
        // For now, we'll start LP Sniper if enabled
        if self.config.bots.lp_sniper.enabled {
            let bot_type = BotType::LpSniper;
            self.start_bot(bot_type).await?;
        }
        
        Ok(())
    }

    /// Start a bot of specific type
    pub async fn start_bot(&self, bot_type: BotType) -> Result<()> {
        info!("🚀 Starting bot: {:?}", bot_type);
        
        match bot_type {
            BotType::LpSniper => {                // Create and start LP Sniper bot  
                let bot_config = self.config.bots.lp_sniper.clone();                let bot_id = self.create_bot(bot_type, BotConfig::LpSniper(crate::types::LpSniperConfig {
                    enabled: bot_config.enabled,
                    trade_amount_sol: 0.5, // Default values
                    max_slippage_percent: 5.0,
                    min_liquidity_usd: 10000.0,
                    max_pool_age_seconds: 3600,
                    risk_per_trade: 5.0, // Max 5% of balance per trade
                    stop_loss_percent: 20.0, // 20% stop loss
                    take_profit_percent: 50.0, // 50% take profit
                    trading_wallet_name: "trading".to_string(), // Default wallet name
                    devnet_mode: true, // Default to devnet for safety
                    monitoring_interval_ms: 1000, // Check every second
                    target_pools: vec![], // No specific pools by default
                    max_market_cap: 1000000.0, // $1M max market cap
                    slippage_tolerance: 5.0, // 5% slippage tolerance
                    settings: HashMap::new(), // Empty for now
                })).await?;
                self.start_bot_instance(bot_id).await?;
            }
            _ => {
                warn!("Bot type {:?} not yet implemented", bot_type);
            }
        }
        
        Ok(())
    }

    /// Stop all running bots
    pub async fn stop_all_bots(&self) -> Result<()> {
        info!("🛑 Stopping all bots");
        
        let bots = self.bots.read().await;
        for (bot_id, _) in bots.iter() {
            if let Err(e) = self.stop_bot(*bot_id).await {
                error!("Failed to stop bot {}: {}", bot_id, e);
            }
        }
        
        Ok(())
    }

    /// Get metrics from all bots
    pub async fn get_all_metrics(&self) -> Result<Vec<(crate::types::BotMetrics, BotStatus)>> {
        let bots = self.bots.read().await;
        let mut metrics = Vec::new();
        
        for bot in bots.values() {
            metrics.push((bot.metrics.clone(), bot.status.clone()));
        }
        
        Ok(metrics)
    }

    /// Health check for bot manager
    pub async fn health_check(&self) -> Result<crate::types::HealthStatus> {
        let bots = self.bots.read().await;
        let total_bots = bots.len();
        let error_bots = bots.values().filter(|b| matches!(b.status, BotStatus::Error(_))).count();
          Ok(crate::types::HealthStatus {
            is_healthy: error_bots == 0,
            component: "bot_manager".to_string(),
            message: if error_bots > 0 {
                Some(format!("{} bots in error state out of {}", error_bots, total_bots))
            } else {
                None
            },
            checked_at: chrono::Utc::now(),
            metrics: HashMap::new(),
        })
    }

    /// Start a specific bot instance (internal method)
    async fn start_bot_instance(&self, bot_id: Uuid) -> Result<()> {
        let mut bots = self.bots.write().await;
        
        if let Some(bot) = bots.get_mut(&bot_id) {
            match bot.status {
                BotStatus::Stopped => {
                    bot.status = BotStatus::Starting;
                    info!("Starting bot: {} ({})", bot.name, bot_id);
                    
                    // In a real implementation, this would spawn the actual bot
                    // For now, we'll just mark it as running
                    bot.status = BotStatus::Running;
                    bot.last_activity = chrono::Utc::now();
                    
                    Ok(())
                }
                _ => {
                    warn!("Bot {} is already running or starting", bot_id);
                    Err(PlatformError::BotManagement("Bot is not in stopped state".to_string()).into())
                }
            }
        } else {
            Err(PlatformError::BotManagement("Bot not found".to_string()).into())
        }
    }

    /// Stop a specific bot
    pub async fn stop_bot(&self, bot_id: Uuid) -> Result<()> {
        let mut bots = self.bots.write().await;
        
        if let Some(bot) = bots.get_mut(&bot_id) {
            match bot.status {
                BotStatus::Running | BotStatus::Error(_) => {
                    bot.status = BotStatus::Stopping;
                    info!("Stopping bot: {} ({})", bot.name, bot_id);
                    
                    // Send stop command
                    let command = BotCommand::Stop { bot_id };
                    self.command_tx.send(command)
                        .map_err(|_| PlatformError::BotManagement("Failed to send stop command".to_string()))?;
                    
                    bot.status = BotStatus::Stopped;
                    Ok(())
                }
                _ => {
                    warn!("Bot {} is not running", bot_id);
                    Err(PlatformError::BotManagement("Bot is not in running state".to_string()).into())
                }
            }
        } else {
            Err(PlatformError::BotManagement("Bot not found".to_string()).into())
        }
    }

    /// Get bot status and metrics
    pub async fn get_bot_status(&self, bot_id: Uuid) -> Option<BotInstance> {
        let bots = self.bots.read().await;
        bots.get(&bot_id).cloned()
    }

    /// List all bots
    pub async fn list_bots(&self) -> Vec<BotInstance> {
        let bots = self.bots.read().await;
        bots.values().cloned().collect()
    }    /// Update bot metrics
    pub async fn update_bot_metrics(&self, bot_id: Uuid, metrics: BotMetrics) -> Result<()> {
        let mut bots = self.bots.write().await;
        
        if let Some(bot) = bots.get_mut(&bot_id) {
            bot.metrics = metrics;
            bot.last_activity = chrono::Utc::now();
            debug!("Updated metrics for bot: {}", bot_id);
            Ok(())
        } else {
            Err(PlatformError::BotManagement("Bot not found".to_string()).into())
        }    }

    /// Handle bot events
    async fn handle_bot_event(&self, event: BotEvent) -> Result<()> {
        match event {
            BotEvent::StatusChanged(status) => {
                info!("Global bot status changed to: {:?}", status);
            }            BotEvent::BotError(bot_id, error) => {
                let mut bots = self.bots.write().await;
                if let Some(bot) = bots.get_mut(&bot_id.0) {
                    bot.status = BotStatus::Error(error.clone());
                    bot.last_activity = chrono::Utc::now();
                    warn!("Bot {} error: {}", bot_id, error);
                }
            }
            BotEvent::BotStarted(bot_id, bot_type) => {
                info!("Bot {} ({:?}) started", bot_id, bot_type);
            }
            BotEvent::BotStopped(bot_id, bot_type) => {
                info!("Bot {} ({:?}) stopped", bot_id, bot_type);
            }
            BotEvent::TradeExecuted(bot_id, trade_result) => {
                info!("Bot {} executed trade: {:?}", bot_id, trade_result.status);
            }
            BotEvent::OpportunityDetected(bot_id, opportunity) => {
                info!("Bot {} detected opportunity: {}", bot_id, opportunity.id);
            }
            _ => {
                // Handle other events
            }
        }
        Ok(())
    }

    /// Main management loop
    async fn run_management_loop(&mut self) -> Result<()> {
        loop {
            tokio::select! {
                Some(event) = self.event_rx.recv() => {
                    if let Err(e) = self.handle_bot_event(event).await {
                        error!("Error handling bot event: {}", e);
                    }
                }
                _ = tokio::time::sleep(tokio::time::Duration::from_secs(10)) => {
                    // Periodic health checks
                    self.perform_health_checks().await;
                }
            }
        }
    }

    /// Perform health checks on all bots
    async fn perform_health_checks(&self) {
        let bots = self.bots.read().await;
        let now = chrono::Utc::now();
        
        for (bot_id, bot) in bots.iter() {
            // Check if bot is inactive for too long
            let inactive_duration = now.signed_duration_since(bot.last_activity);
            if inactive_duration.num_minutes() > 5 && bot.status == BotStatus::Running {
                warn!("Bot {} appears inactive for {} minutes", bot_id, inactive_duration.num_minutes());
            }        }
    }
}
