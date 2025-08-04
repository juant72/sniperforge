use tokio::sync::RwLock;
use std::collections::HashMap;
use std::time::Duration;
use uuid::Uuid;
use std::sync::Arc;
use anyhow::Result;
use tracing::info;
use serde::{Serialize, Deserialize};

use crate::api::bot_interface::{BotInterface, BotType, BotStatus, BotMetrics, BotConfig};
use crate::api::metrics_collector::{MetricsCollector, MetricsConfig};
use crate::api::config_management::ConfigManager;
use crate::bots::mock_arbitrage_bot::MockArbitrageBot;

/// Central bot controller que maneja todos los bots
pub struct BotController {
    /// Active bot instances
    bots: Arc<RwLock<HashMap<Uuid, Box<dyn BotInterface + Send + Sync>>>>,
    
    /// Default arbitrage bot (running by default)
    default_arbitrage_bot: Option<Uuid>,
    
    /// Configuration manager
    config_manager: ConfigManager,
    
    /// Metrics collector
    metrics_collector: MetricsCollector,
    
    /// Server start time for uptime calculation
    start_time: std::time::Instant,
}

impl BotController {
    pub async fn new() -> Result<Self> {
        let config_path = "config.json";
        let metrics_config = MetricsConfig {
            collection_interval_seconds: 60,
            retention_hours: 24,
            max_points_per_metric: 1000,
            aggregation_windows: vec![],
            enable_percentiles: false,
            enable_trading_metrics: true,
            custom_metrics_enabled: false,
        };

        Ok(Self {
            bots: Arc::new(RwLock::new(HashMap::new())),
            default_arbitrage_bot: None,
            config_manager: ConfigManager::new(config_path),
            metrics_collector: MetricsCollector::new(metrics_config),
            start_time: std::time::Instant::now(),
        })
    }
    
    /// Register the default arbitrage bot that's already running
    pub async fn register_default_arbitrage_bot(
        &mut self, 
        bot: Box<dyn BotInterface + Send + Sync>
    ) -> Result<Uuid> {
        let bot_id = Uuid::new_v4();
        
        {
            let mut bots = self.bots.write().await;
            bots.insert(bot_id, bot);
        }
        
        self.default_arbitrage_bot = Some(bot_id);
        info!("✅ Registered default arbitrage bot: {}", bot_id);
        
        Ok(bot_id)
    }
    
    /// Create a new bot instance (placeholder for future sniper bot)
    pub async fn create_bot(&self, bot_type: BotType, config: BotConfig) -> Result<Uuid> {
        let bot_id = Uuid::new_v4();
        
        // For now, return error as sniper bot not implemented yet
        match bot_type {
            BotType::EnhancedArbitrage => {
                return Err(anyhow::anyhow!("Enhanced Arbitrage bot should be registered, not created"));
            }
            _ => {
                return Err(anyhow::anyhow!("Bot type {:?} not supported yet", bot_type));
            }
        }
    }
    
    /// Start a specific bot
    pub async fn start_bot(&self, bot_id: Uuid, config: BotConfig) -> Result<()> {
        let bots = self.bots.read().await;
        
        if let Some(bot) = bots.get(&bot_id) {
            // Note: This will require mutable access, so we'll need to refactor
            // For now, return success if bot exists
            info!("🚀 Would start bot: {} (implementation pending)", bot_id);
            Ok(())
        } else {
            Err(anyhow::anyhow!("Bot not found: {}", bot_id))
        }
    }
    
    /// Stop a specific bot
    pub async fn stop_bot(&self, bot_id: Uuid) -> Result<()> {
        let bots = self.bots.read().await;
        
        if let Some(bot) = bots.get(&bot_id) {
            // Note: This will require mutable access, so we'll need to refactor
            // For now, return success if bot exists
            info!("🛑 Would stop bot: {} (implementation pending)", bot_id);
            Ok(())
        } else {
            Err(anyhow::anyhow!("Bot not found: {}", bot_id))
        }
    }
    
    /// Get status of specific bot
    pub async fn get_bot_status(&self, bot_id: Uuid) -> Result<BotStatus> {
        let bots = self.bots.read().await;
        
        if let Some(bot) = bots.get(&bot_id) {
            Ok(bot.status().await)
        } else {
            Err(anyhow::anyhow!("Bot not found: {}", bot_id))
        }
    }
    
    /// Get metrics of specific bot
    pub async fn get_bot_metrics(&self, bot_id: Uuid) -> Result<BotMetrics> {
        let bots = self.bots.read().await;
        
        if let Some(bot) = bots.get(&bot_id) {
            Ok(bot.metrics().await)
        } else {
            Err(anyhow::anyhow!("Bot not found: {}", bot_id))
        }
    }
    
    /// List all active bots
    pub async fn list_bots(&self) -> Result<Vec<BotSummary>> {
        let bots = self.bots.read().await;
        let mut summaries = Vec::new();
        
        for (id, bot) in bots.iter() {
            let status = bot.status().await;
            let metrics = bot.metrics().await;
            
            summaries.push(BotSummary {
                id: *id,
                bot_type: bot.bot_type(),
                status,
                metrics,
                is_default: self.default_arbitrage_bot == Some(*id),
            });
        }
        
        Ok(summaries)
    }
    
    /// Get system-wide metrics
    pub async fn get_system_metrics(&self) -> Result<SystemMetrics> {
        let bot_list = self.list_bots().await?;
        
        let total_bots = bot_list.len();
        let running_bots = bot_list.iter().filter(|b| matches!(b.status, BotStatus::Running)).count();
        let total_profit: f64 = bot_list.iter().map(|b| b.metrics.trading.total_pnl_usd).sum();
        let total_trades: u64 = bot_list.iter().map(|b| b.metrics.trading.trades_executed).sum();
        
        Ok(SystemMetrics {
            total_bots,
            running_bots,
            total_profit,
            total_trades,
            uptime_seconds: self.start_time.elapsed().as_secs(),
            memory_usage_mb: self.get_memory_usage().await?,
        })
    }
    
    async fn get_memory_usage(&self) -> Result<f64> {
        // Get current process memory usage
        #[cfg(target_os = "windows")]
        {
            // For now, return a placeholder value
            // TODO: Implement actual Windows memory usage
            Ok(50.0) // 50MB placeholder
        }
        
        #[cfg(not(target_os = "windows"))]
        {
            // Unix-like systems memory usage
            Ok(50.0) // 50MB placeholder
        }
    }
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct BotSummary {
    pub id: Uuid,
    pub bot_type: BotType,
    pub status: BotStatus,
    pub metrics: BotMetrics,
    pub is_default: bool,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct SystemMetrics {
    pub total_bots: usize,
    pub running_bots: usize,
    pub total_profit: f64,
    pub total_trades: u64,
    pub uptime_seconds: u64,
    pub memory_usage_mb: f64,
}
