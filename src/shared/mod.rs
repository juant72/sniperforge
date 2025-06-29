pub mod rpc_pool;
pub mod rpc_health_persistence;  // NEW: RPC endpoint health persistence
pub mod premium_rpc_manager;     // NEW: Premium RPC endpoints with API keys
pub mod tatum_rpc_client;        // NEW: Tatum client with header authentication
pub mod wallet_manager;
pub mod data_feeds;
pub mod monitoring;
pub mod jupiter;
pub mod real_data_manager;  // NEW: Centralized real data management
pub mod real_trade_executor; // NEW: Real-only trade execution
pub mod trade_executor;
pub mod risk_manager;
pub mod automated_trader;
pub mod performance_tracker;
pub mod websocket_manager;
pub mod websocket_price_feed;
pub mod syndica_websocket;
pub mod helius_websocket;  // NUEVO: Cliente Helius WebSocket
pub mod pool_detector;
pub mod analytics;         // Pool analytics and pattern detection
pub mod cache_free_trading;        // Phase 4: Cache-free trading engine
pub mod real_time_blockchain;      // Phase 5: Real-time blockchain integration
pub mod real_time_trading;         // Phase 5A: Real-time trading with live blockchain integration
pub mod alternative_apis;          // NEW: Alternative API sources for resilience
pub mod mainnet_trading;           // Phase 5B: MainNet real trading with minimal capital
pub mod transaction_monitor;     // Transaction monitoring for trading safety
pub mod real_trading_engine;       // NEW: Real trading execution engine with Jupiter integration
pub mod cache_free_trader_simple;   // Simplified cache-free trading

use anyhow::Result;
use std::sync::Arc;
use tokio::sync::RwLock;
use tracing::{info, error};

use crate::config::Config;
use crate::types::HealthStatus;

use rpc_pool::RpcConnectionPool;
use wallet_manager::WalletManager;
use data_feeds::MarketDataFeeds;
use monitoring::MonitoringSystem;
use jupiter::Jupiter;
use websocket_manager::WebSocketManager;

pub struct SharedServices {
    rpc_pool: Arc<RpcConnectionPool>,
    wallet_manager: Arc<WalletManager>,
    data_feeds: Arc<MarketDataFeeds>,
    monitoring: Arc<MonitoringSystem>,
    jupiter: Arc<Jupiter>,
    websocket_manager: Arc<WebSocketManager>,
    is_running: Arc<RwLock<bool>>,
}

impl SharedServices {
    pub async fn new(config: &Config) -> Result<Self> {
        info!("🔧 Initializing shared services");
        
        // Initialize RPC connection pool
        let rpc_pool = Arc::new(RpcConnectionPool::new(config).await?);
        
        // Initialize wallet manager
        let wallet_manager = Arc::new(WalletManager::new(config).await?);
        
        // Initialize market data feeds
        let data_feeds = Arc::new(MarketDataFeeds::new(config, rpc_pool.clone()).await?);
          // Initialize monitoring system
        let monitoring = Arc::new(MonitoringSystem::new(config)?);
          // Initialize Jupiter integration
        let jupiter_config = jupiter::JupiterConfig::default();
        let jupiter = Arc::new(Jupiter::new(&jupiter_config).await?);
        
        // Initialize WebSocket manager for real-time updates (with premium RPC pool)
        let websocket_manager = Arc::new(WebSocketManager::new_with_rpc_pool(config, Some(rpc_pool.clone())).await?);
        
        Ok(Self {
            rpc_pool,
            wallet_manager,
            data_feeds,
            monitoring,
            jupiter,
            websocket_manager,
            is_running: Arc::new(RwLock::new(false)),
        })
    }
    
    pub async fn start(&self) -> Result<()> {
        info!("🚀 Starting shared services");
        
        *self.is_running.write().await = true;
        
        // Start RPC pool
        self.rpc_pool.start().await?;
        
        // Start wallet manager
        self.wallet_manager.start().await?;
        
        // Start data feeds
        self.data_feeds.start().await?;
        
        // Start monitoring
        self.monitoring.start().await?;
        
        info!("✅ Shared services started");
        Ok(())
    }
    
    pub async fn stop(&self) -> Result<()> {
        info!("🛑 Stopping shared services");
        
        *self.is_running.write().await = false;
        
        // Stop in reverse order
        if let Err(e) = self.monitoring.stop().await {
            error!("Error stopping monitoring: {}", e);
        }
        
        if let Err(e) = self.data_feeds.stop().await {
            error!("Error stopping data feeds: {}", e);
        }
        
        if let Err(e) = self.wallet_manager.stop().await {
            error!("Error stopping wallet manager: {}", e);
        }
        
        if let Err(e) = self.rpc_pool.stop().await {
            error!("Error stopping RPC pool: {}", e);
        }
        
        info!("✅ Shared services stopped");
        Ok(())
    }
    
    pub async fn health_check(&self) -> Result<HealthStatus> {
        let is_running = *self.is_running.read().await;
        
        if !is_running {
            return Ok(HealthStatus {
                is_healthy: false,
                component: "SharedServices".to_string(),
                message: Some("Services not running".to_string()),
                checked_at: chrono::Utc::now(),
                metrics: std::collections::HashMap::new(),
            });
        }
        
        // Check all components
        let rpc_health = self.rpc_pool.health_check().await?;
        let wallet_health = self.wallet_manager.health_check().await?;
        let data_health = self.data_feeds.health_check().await?;
        let monitoring_health = self.monitoring.health_check().await?;
        
        let is_healthy = rpc_health.is_healthy 
            && wallet_health.is_healthy 
            && data_health.is_healthy 
            && monitoring_health.is_healthy;
        
        Ok(HealthStatus {
            is_healthy,
            component: "SharedServices".to_string(),
            message: if is_healthy {
                None
            } else {
                Some("One or more components unhealthy".to_string())
            },
            checked_at: chrono::Utc::now(),
            metrics: std::collections::HashMap::new(),
        })
    }
    
    /// Get access to the RPC pool
    pub fn rpc_pool(&self) -> Arc<RpcConnectionPool> {
        self.rpc_pool.clone()
    }

    /// Get access to the wallet manager
    pub fn wallet_manager(&self) -> Arc<WalletManager> {
        self.wallet_manager.clone()
    }

    /// Get access to the data feeds
    pub fn data_feeds(&self) -> Arc<MarketDataFeeds> {
        self.data_feeds.clone()
    }

    /// Get access to the monitoring system
    pub fn monitoring(&self) -> Arc<MonitoringSystem> {
        self.monitoring.clone()
    }    /// Get access to Jupiter API integration
    pub fn jupiter(&self) -> Arc<Jupiter> {
        self.jupiter.clone()
    }

    /// Get access to the WebSocket manager
    pub fn websocket_manager(&self) -> Arc<WebSocketManager> {
        self.websocket_manager.clone()
    }

    /// Get shared services metrics
    pub async fn get_metrics(&self) -> Result<SharedServicesMetrics> {
        // Get metrics from each service
        let rpc_stats = self.rpc_pool.get_stats().await;
        let wallet_count = self.wallet_manager.list_wallets().await.len();
        let data_stats = self.data_feeds.get_stats().await;
        let _monitoring_stats = self.monitoring.get_stats().await;
        
        // Get system metrics if available
        let (cpu_usage, memory_usage, uptime) = if let Some(system_metrics) = self.monitoring.get_latest_system_metrics().await {
            (system_metrics.cpu_usage_percent, system_metrics.memory_usage_mb, system_metrics.uptime_seconds)
        } else {
            (0.0, 0, 0)
        };
        
        Ok(SharedServicesMetrics {
            rpc_connections: rpc_stats.active_connections,
            active_wallets: wallet_count,
            data_feed_subscriptions: data_stats.price_subscriptions + data_stats.pool_subscriptions,
            cpu_usage_percent: cpu_usage,
            memory_usage_mb: memory_usage,
            uptime_seconds: uptime,
        })
    }
}

#[derive(Debug, Clone)]
pub struct SharedServicesMetrics {
    pub rpc_connections: usize,
    pub active_wallets: usize,
    pub data_feed_subscriptions: usize,
    pub cpu_usage_percent: f64,
    pub memory_usage_mb: u64,
    pub uptime_seconds: u64,
}
