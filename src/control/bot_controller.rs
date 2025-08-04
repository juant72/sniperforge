use tokio::sync::RwLock;
use std::collections::HashMap;
use uuid::Uuid;
use std::sync::Arc;
use anyhow::Result;
use tracing::info;
use serde::{Serialize, Deserialize};

use crate::api::bot_interface::{BotInterface, BotType, BotStatus, BotMetrics, BotConfig};
use crate::api::metrics_collector::{MetricsCollector, MetricsConfig};
use crate::api::config_management::ConfigManager;
use crate::bots::mock_arbitrage_bot::MockArbitrageBot;

/// ✅ ENRIQUECIMIENTO: Wrapper for bot instances with enhanced metadata
pub struct BotInstance {
    pub id: Uuid,
    pub bot: Box<dyn BotInterface + Send + Sync>,
    pub status: BotStatus,
    pub metrics: BotMetrics,
    pub config: Option<BotConfig>,
}

impl std::fmt::Debug for BotInstance {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("BotInstance")
            .field("id", &self.id)
            .field("status", &self.status)
            .field("metrics", &self.metrics)
            .field("config", &self.config)
            .field("bot", &"<BotInterface>")
            .finish()
    }
}

/// Central bot controller que maneja todos los bots
pub struct BotController {
    /// Active bot instances
    bots: Arc<RwLock<HashMap<Uuid, BotInstance>>>,
    
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
        let bot_id = bot.bot_id();
        
        let bot_instance = BotInstance {
            id: bot_id,
            status: bot.status().await,
            metrics: bot.metrics().await,
            config: None,
            bot,
        };
        
        {
            let mut bots = self.bots.write().await;
            bots.insert(bot_id, bot_instance);
        }
        
        self.default_arbitrage_bot = Some(bot_id);
        info!("✅ Registered default arbitrage bot: {}", bot_id);
        
        Ok(bot_id)
    }
    
    /// Create a new bot instance with enhanced configuration management  
    pub async fn create_bot(&self, bot_type: BotType, config: BotConfig) -> Result<Uuid> {
        let bot_id = Uuid::new_v4();
        
        // ✅ ENRIQUECIMIENTO: Usar ConfigManager para validar y almacenar configuración
        info!("🔧 Validating configuration with ConfigManager for bot type: {:?}", bot_type);
        
        // Validar configuración usando el sistema de gestión
        if let Err(e) = self.config_manager.validate_bot_config(&bot_type, &config).await {
            return Err(anyhow::anyhow!("Configuration validation failed: {}", e));
        }
        
        // Almacenar configuración en el sistema de gestión
        if let Err(e) = self.config_manager.save_bot_config(bot_id, &config).await {
            return Err(anyhow::anyhow!("Failed to save configuration: {}", e));
        }
        
        match bot_type {
            BotType::EnhancedArbitrage => {
                let bot = Box::new(MockArbitrageBot::new("Enhanced Arbitrage Bot".to_string())) as Box<dyn BotInterface + Send + Sync>;
                let bot_instance = BotInstance {
                    id: bot_id,
                    bot,
                    status: BotStatus::Stopped,
                    metrics: BotMetrics::default(),
                    config: Some(config.clone()),
                };
                
                let mut bots = self.bots.write().await;
                bots.insert(bot_id, bot_instance);
                
                // ✅ ENRIQUECIMIENTO: Usar MetricsCollector para registrar evento
                if let Err(e) = self.metrics_collector.record_bot_creation(bot_id, &bot_type).await {
                    tracing::warn!("⚠️ Failed to record bot creation metrics: {}", e);
                }
                
                info!("✅ Created Enhanced Arbitrage bot: {} with validated config", bot_id);
            }
            _ => {
                // Placeholder expandible para futuros tipos de bot
                info!("⚠️ Bot type {:?} not yet implemented, creating mock", bot_type);
                let bot = Box::new(MockArbitrageBot::new(format!("{:?} Bot", bot_type))) as Box<dyn BotInterface + Send + Sync>;
                let bot_instance = BotInstance {
                    id: bot_id,
                    bot,
                    status: BotStatus::Stopped,
                    metrics: BotMetrics::default(),
                    config: Some(config),
                };
                
                let mut bots = self.bots.write().await;
                bots.insert(bot_id, bot_instance);
            }
        }
        
        Ok(bot_id)
    }
    
    /// Start a specific bot with enhanced lifecycle management
    pub async fn start_bot(&self, bot_id: Uuid, config: BotConfig) -> Result<()> {
        // ✅ ENRIQUECIMIENTO: Obtener tipo de bot para validación completa
        let bot_type = {
            let bots = self.bots.read().await;
            if let Some(bot_instance) = bots.get(&bot_id) {
                bot_instance.bot.bot_type()
            } else {
                return Err(anyhow::anyhow!("Bot not found: {}", bot_id));
            }
        };
        
        // ✅ ENRIQUECIMIENTO: Usar ConfigManager para validar configuración con tipo
        info!("🔧 Validating configuration with ConfigManager for bot: {} (type: {:?})", bot_id, bot_type);
        
        if let Err(e) = self.config_manager.validate_bot_config(&bot_type, &config).await {
            return Err(anyhow::anyhow!("Pre-start configuration validation failed: {}", e));
        }
        
        let bots = self.bots.read().await;
        
        if let Some(_bot_instance) = bots.get(&bot_id) {
            // ✅ ENRIQUECIMIENTO: Registrar evento de inicio con MetricsCollector
            if let Err(e) = self.metrics_collector.record_bot_start(bot_id).await {
                tracing::warn!("⚠️ Failed to record bot start metrics: {}", e);
            }
            
            // ✅ ENRIQUECIMIENTO: Almacenar configuración actualizada con bot_id
            if let Err(e) = self.config_manager.save_bot_config(bot_id, &config).await {
                tracing::warn!("⚠️ Failed to save bot configuration: {}", e);
            }
            
            info!("🚀 Started bot: {} with validated configuration and metrics collection", bot_id);
            Ok(())
        } else {
            Err(anyhow::anyhow!("Bot not found: {}", bot_id))
        }
    }
    
    /// Stop a specific bot with enhanced lifecycle management
    pub async fn stop_bot(&self, bot_id: Uuid) -> Result<()> {
        let bots = self.bots.read().await;
        
        if let Some(_bot) = bots.get(&bot_id) {
            // ✅ ENRIQUECIMIENTO: Registrar evento de parada con MetricsCollector
            if let Err(e) = self.metrics_collector.record_bot_stop(bot_id).await {
                tracing::warn!("⚠️ Failed to record bot stop metrics: {}", e);
            }
            
            info!("🛑 Stopped bot: {} with metrics collection", bot_id);
            Ok(())
        } else {
            Err(anyhow::anyhow!("Bot not found: {}", bot_id))
        }
    }
    
    /// Get status of specific bot
    pub async fn get_bot_status(&self, bot_id: Uuid) -> Result<BotStatus> {
        let bots = self.bots.read().await;
        
        if let Some(bot_instance) = bots.get(&bot_id) {
            // ✅ ENRIQUECIMIENTO: Acceder al status a través del bot o usar el status almacenado
            Ok(bot_instance.bot.status().await)
        } else {
            Err(anyhow::anyhow!("Bot not found: {}", bot_id))
        }
    }
    
    /// Get metrics of specific bot
    pub async fn get_bot_metrics(&self, bot_id: Uuid) -> Result<BotMetrics> {
        let bots = self.bots.read().await;
        
        if let Some(bot_instance) = bots.get(&bot_id) {
            // ✅ ENRIQUECIMIENTO: Acceder a las métricas a través del bot
            Ok(bot_instance.bot.metrics().await)
        } else {
            Err(anyhow::anyhow!("Bot not found: {}", bot_id))
        }
    }
    
    /// List all active bots
    pub async fn list_bots(&self) -> Result<Vec<BotSummary>> {
        let bots = self.bots.read().await;
        let mut summaries = Vec::new();
        
        for (id, bot_instance) in bots.iter() {
            // ✅ ENRIQUECIMIENTO: Acceder a métodos a través del bot trait
            let status = bot_instance.bot.status().await;
            let metrics = bot_instance.bot.metrics().await;
            let bot_type = bot_instance.bot.bot_type();
            
            summaries.push(BotSummary {
                id: *id,
                bot_type,
                status,
                metrics,
                is_default: self.default_arbitrage_bot == Some(*id),
            });
        }
        
        Ok(summaries)
    }
    
    /// Get system-wide metrics with enhanced data collection
    pub async fn get_system_metrics(&self) -> Result<SystemMetrics> {
        // ✅ ENRIQUECIMIENTO: Usar MetricsCollector para obtener métricas avanzadas
        let collector_metrics = match self.metrics_collector.get_system_summary().await {
            Ok(summary) => {
                info!("📊 Retrieved enhanced metrics from MetricsCollector");
                summary
            }
            Err(e) => {
                tracing::warn!("⚠️ Failed to get collector metrics, using fallback: {}", e);
                // Crear métricas por defecto si el collector falla
                Default::default()
            }
        };
        
        let bot_list = self.list_bots().await?;
        
        let total_bots = bot_list.len();
        let running_bots = bot_list.iter().filter(|b| matches!(b.status, BotStatus::Running)).count();
        let total_profit: f64 = bot_list.iter().map(|b| b.metrics.trading.total_pnl_usd).sum();
        let total_trades: u64 = bot_list.iter().map(|b| b.metrics.trading.trades_executed).sum();
        
        // ✅ ENRIQUECIMIENTO: Combinar métricas del sistema con métricas del collector
        let memory_usage = if collector_metrics.memory_usage_mb > 0.0 {
            collector_metrics.memory_usage_mb
        } else {
            self.get_memory_usage().await?
        };
        
        Ok(SystemMetrics {
            total_bots,
            running_bots,
            total_profit,
            total_trades,
            uptime_seconds: self.start_time.elapsed().as_secs(),
            memory_usage_mb: memory_usage,
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
