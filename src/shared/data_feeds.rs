use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::{RwLock, mpsc, broadcast};
use anyhow::Result;
use tracing::{info, warn, error, debug};
use serde::{Serialize, Deserialize};
use solana_sdk::pubkey::Pubkey;
use reqwest::Client;
use rand::Rng;

use crate::config::Config;
use crate::shared::rpc_pool::RpcConnectionPool;
use crate::types::{PlatformError, HealthStatus, PoolInfo, TokenInfo, PriceData};

/// Market data feed types
#[derive(Debug, Clone, Hash, Eq, PartialEq, Serialize, Deserialize)]
pub enum DataFeedType {
    RaydiumPools,
    TokenPrices,
    VolumeData,
    LiquidityData,
    Custom(String),
}

/// Market data update event
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MarketDataUpdate {
    pub feed_type: DataFeedType,
    pub data: serde_json::Value,
    pub timestamp: chrono::DateTime<chrono::Utc>,
    pub source: String,
}

/// Price feed subscription
#[derive(Debug, Clone)]
pub struct PriceFeedSubscription {
    pub id: uuid::Uuid,
    pub tokens: Vec<Pubkey>,
    pub update_interval: std::time::Duration,
    pub sender: broadcast::Sender<PriceData>,
}

/// Pool monitoring subscription
#[derive(Debug, Clone)]
pub struct PoolMonitoringSubscription {
    pub id: uuid::Uuid,
    pub pool_addresses: Vec<Pubkey>,
    pub sender: broadcast::Sender<PoolInfo>,
}

/// Market data feeds manager
pub struct MarketDataFeeds {
    config: Config,
    rpc_pool: Arc<RpcConnectionPool>,
    price_subscriptions: Arc<RwLock<HashMap<uuid::Uuid, PriceFeedSubscription>>>,
    pool_subscriptions: Arc<RwLock<HashMap<uuid::Uuid, PoolMonitoringSubscription>>>,
    cached_prices: Arc<RwLock<HashMap<Pubkey, PriceData>>>,
    cached_pools: Arc<RwLock<HashMap<Pubkey, PoolInfo>>>,
    http_client: Client,
    shutdown_tx: mpsc::Sender<()>,
    is_running: Arc<RwLock<bool>>,
}

impl MarketDataFeeds {
    pub async fn new(config: &Config, rpc_pool: Arc<RpcConnectionPool>) -> Result<Self> {
        info!("📊 Initializing Market Data Feeds");
        
        let http_client = Client::builder()
            .timeout(std::time::Duration::from_secs(10))
            .build()?;
            
        let (shutdown_tx, _) = mpsc::channel(1);
        
        Ok(Self {
            config: config.clone(),
            rpc_pool,
            price_subscriptions: Arc::new(RwLock::new(HashMap::new())),
            pool_subscriptions: Arc::new(RwLock::new(HashMap::new())),
            cached_prices: Arc::new(RwLock::new(HashMap::new())),
            cached_pools: Arc::new(RwLock::new(HashMap::new())),
            http_client,
            shutdown_tx,
            is_running: Arc::new(RwLock::new(false)),
        })
    }

    /// Start the market data feeds
    pub async fn start(&self) -> Result<()> {
        info!("🚀 Starting Market Data Feeds");
        
        *self.is_running.write().await = true;
        
        // Start price feed updater
        self.start_price_feed_updater().await;
        
        // Start pool monitoring
        self.start_pool_monitoring().await;
        
        // Start cache cleanup task
        self.start_cache_cleanup().await;
        
        info!("✅ Market Data Feeds started successfully");
        Ok(())
    }

    /// Stop the market data feeds
    pub async fn stop(&self) -> Result<()> {
        info!("🛑 Stopping Market Data Feeds");
        
        *self.is_running.write().await = false;
        
        // Clear subscriptions
        {
            let mut price_subs = self.price_subscriptions.write().await;
            price_subs.clear();
        }
        {
            let mut pool_subs = self.pool_subscriptions.write().await;
            pool_subs.clear();
        }
        
        // Send shutdown signal
        let _ = self.shutdown_tx.send(()).await;
        
        Ok(())
    }

    /// Subscribe to price updates for specific tokens
    pub async fn subscribe_to_prices(
        &self,
        tokens: Vec<Pubkey>,
        update_interval: std::time::Duration,
    ) -> Result<(uuid::Uuid, broadcast::Receiver<PriceData>)> {
        let subscription_id = uuid::Uuid::new_v4();
        let (tx, rx) = broadcast::channel(1000);
        
        let subscription = PriceFeedSubscription {
            id: subscription_id,
            tokens: tokens.clone(),
            update_interval,
            sender: tx,
        };

        let mut subscriptions = self.price_subscriptions.write().await;
        subscriptions.insert(subscription_id, subscription);
        
        info!("📈 Created price subscription for {} tokens", tokens.len());
        Ok((subscription_id, rx))
    }

    /// Subscribe to pool updates
    pub async fn subscribe_to_pools(
        &self,
        pool_addresses: Vec<Pubkey>,
    ) -> Result<(uuid::Uuid, broadcast::Receiver<PoolInfo>)> {
        let subscription_id = uuid::Uuid::new_v4();
        let (tx, rx) = broadcast::channel(1000);
        
        let subscription = PoolMonitoringSubscription {
            id: subscription_id,
            pool_addresses: pool_addresses.clone(),
            sender: tx,
        };

        let mut subscriptions = self.pool_subscriptions.write().await;
        subscriptions.insert(subscription_id, subscription);
        
        info!("🏊 Created pool subscription for {} pools", pool_addresses.len());
        Ok((subscription_id, rx))
    }

    /// Unsubscribe from price updates
    pub async fn unsubscribe_prices(&self, subscription_id: uuid::Uuid) -> Result<()> {
        let mut subscriptions = self.price_subscriptions.write().await;
        if subscriptions.remove(&subscription_id).is_some() {
            info!("📈 Removed price subscription: {}", subscription_id);
            Ok(())
        } else {
            Err(PlatformError::DataFeed("Subscription not found".to_string()).into())
        }
    }

    /// Unsubscribe from pool updates
    pub async fn unsubscribe_pools(&self, subscription_id: uuid::Uuid) -> Result<()> {
        let mut subscriptions = self.pool_subscriptions.write().await;
        if subscriptions.remove(&subscription_id).is_some() {
            info!("🏊 Removed pool subscription: {}", subscription_id);
            Ok(())
        } else {
            Err(PlatformError::DataFeed("Subscription not found".to_string()).into())
        }
    }

    /// Get current price for a token
    pub async fn get_token_price(&self, token: &Pubkey) -> Option<PriceData> {
        let cached_prices = self.cached_prices.read().await;
        cached_prices.get(token).cloned()
    }

    /// Get current pool information
    pub async fn get_pool_info(&self, pool_address: &Pubkey) -> Option<PoolInfo> {
        let cached_pools = self.cached_pools.read().await;
        cached_pools.get(pool_address).cloned()
    }

    /// Fetch latest prices from external API
    pub async fn fetch_token_prices(&self, tokens: &[Pubkey]) -> Result<HashMap<Pubkey, PriceData>> {
        debug!("📊 Fetching prices for {} tokens", tokens.len());
        
        // For now, we'll simulate price data
        // In a real implementation, this would call Jupiter, CoinGecko, or other price APIs
        let mut prices = HashMap::new();
        
        for token in tokens {            let price_data = PriceData {
                token: *token,
                price_usd: self.simulate_price(token).await,
                price_sol: Some(self.simulate_price(token).await / 100.0), // Simulated SOL price
                volume_24h: 100000.0, // Simulated
                price_change_24h: -2.5 + (rand::rng().random::<f64>() * 5.0), // -2.5% to +2.5%
                market_cap: Some(1000000.0), // $1M market cap
                timestamp: chrono::Utc::now(),
            };
            
            prices.insert(*token, price_data);
        }
        
        // Update cache
        {
            let mut cached_prices = self.cached_prices.write().await;
            for (token, price_data) in &prices {
                cached_prices.insert(*token, price_data.clone());
            }
        }
        
        Ok(prices)
    }

    /// Fetch pool information from Raydium
    pub async fn fetch_pool_info(&self, pool_addresses: &[Pubkey]) -> Result<HashMap<Pubkey, PoolInfo>> {
        debug!("🏊 Fetching pool info for {} pools", pool_addresses.len());
        
        // For now, we'll simulate pool data
        // In a real implementation, this would call Raydium API or parse on-chain data
        let mut pools = HashMap::new();
          for pool_address in pool_addresses {
            let pool_info = PoolInfo {
                pool_id: *pool_address,
                dex: crate::types::DexType::Raydium,
                token_a: crate::types::TokenInfo {
                    mint: Pubkey::new_unique(),
                    symbol: "SOL".to_string(),
                    name: "Solana".to_string(),
                    decimals: 9,
                    supply: Some(1000000),
                    is_verified: true,
                },
                token_b: crate::types::TokenInfo {
                    mint: Pubkey::new_unique(),
                    symbol: "TOKEN".to_string(),
                    name: "New Token".to_string(),
                    decimals: 6,
                    supply: Some(100000),
                    is_verified: false,
                },
                liquidity_usd: 50000.0 + (rand::rng().random::<f64>() * 100000.0), // $50k-$150k
                volume_24h_usd: Some(10000.0 + (rand::rng().random::<f64>() * 50000.0)), // $10k-$60k
                created_at: chrono::Utc::now(),
                detected_at: chrono::Utc::now(),
                is_new: true,
            };
            
            pools.insert(*pool_address, pool_info);
        }
        
        // Update cache
        {
            let mut cached_pools = self.cached_pools.write().await;
            for (pool_address, pool_info) in &pools {
                cached_pools.insert(*pool_address, pool_info.clone());
            }
        }
        
        Ok(pools)
    }

    /// Get market data statistics
    pub async fn get_stats(&self) -> MarketDataStats {
        let price_subs = self.price_subscriptions.read().await;
        let pool_subs = self.pool_subscriptions.read().await;
        let cached_prices = self.cached_prices.read().await;
        let cached_pools = self.cached_pools.read().await;
        
        MarketDataStats {
            price_subscriptions: price_subs.len(),
            pool_subscriptions: pool_subs.len(),
            cached_prices: cached_prices.len(),
            cached_pools: cached_pools.len(),
            is_running: *self.is_running.read().await,
        }
    }

    /// Health check
    pub async fn health_check(&self) -> Result<HealthStatus> {
        let is_running = *self.is_running.read().await;
          if !is_running {
            return Ok(HealthStatus {
                is_healthy: false,
                component: "market_data_feeds".to_string(),
                message: Some("Market data feeds not running".to_string()),
                checked_at: chrono::Utc::now(),
                metrics: HashMap::new(),
            });
        }

        // Check if we have recent data
        let cached_prices = self.cached_prices.read().await;
        let now = chrono::Utc::now();
        let stale_threshold = chrono::Duration::minutes(5);
          let stale_prices = cached_prices.values()
            .filter(|price| now.signed_duration_since(price.timestamp) > stale_threshold)
            .count();
          if stale_prices > 0 && cached_prices.len() > 0 {
            Ok(HealthStatus {
                is_healthy: true,
                component: "market_data_feeds".to_string(),
                message: Some(format!("{} stale prices detected", stale_prices)),
                checked_at: now,
                metrics: HashMap::new(),
            })
        } else {
            Ok(HealthStatus {
                is_healthy: true,
                component: "market_data_feeds".to_string(),
                message: None,
                checked_at: now,
                metrics: HashMap::new(),
            })
        }
    }

    /// Start price feed updater task
    async fn start_price_feed_updater(&self) {
        let price_subscriptions = self.price_subscriptions.clone();
        let cached_prices = self.cached_prices.clone();
        let is_running = self.is_running.clone();
        
        tokio::spawn(async move {
            let mut interval = tokio::time::interval(std::time::Duration::from_secs(10));
            
            loop {
                interval.tick().await;
                
                if !*is_running.read().await {
                    break;
                }
                
                let subscriptions = price_subscriptions.read().await;
                
                for subscription in subscriptions.values() {
                    // For each subscription, send updates to subscribers
                    let cached = cached_prices.read().await;
                    
                    for token in &subscription.tokens {
                        if let Some(price_data) = cached.get(token) {
                            if let Err(e) = subscription.sender.send(price_data.clone()) {
                                debug!("Failed to send price update: {}", e);
                            }
                        }
                    }
                }
            }
        });
    }

    /// Start pool monitoring task
    async fn start_pool_monitoring(&self) {
        let pool_subscriptions = self.pool_subscriptions.clone();
        let cached_pools = self.cached_pools.clone();
        let is_running = self.is_running.clone();
        
        tokio::spawn(async move {
            let mut interval = tokio::time::interval(std::time::Duration::from_secs(5));
            
            loop {
                interval.tick().await;
                
                if !*is_running.read().await {
                    break;
                }
                
                let subscriptions = pool_subscriptions.read().await;
                
                for subscription in subscriptions.values() {
                    let cached = cached_pools.read().await;
                    
                    for pool_address in &subscription.pool_addresses {
                        if let Some(pool_info) = cached.get(pool_address) {
                            if let Err(e) = subscription.sender.send(pool_info.clone()) {
                                debug!("Failed to send pool update: {}", e);
                            }
                        }
                    }
                }
            }
        });
    }

    /// Start cache cleanup task
    async fn start_cache_cleanup(&self) {
        let cached_prices = self.cached_prices.clone();
        let cached_pools = self.cached_pools.clone();
        let is_running = self.is_running.clone();
        
        tokio::spawn(async move {
            let mut interval = tokio::time::interval(std::time::Duration::from_secs(300)); // 5 minutes
            
            loop {
                interval.tick().await;
                
                if !*is_running.read().await {
                    break;
                }
                
                let now = chrono::Utc::now();
                let stale_threshold = chrono::Duration::minutes(30);
                
                // Clean stale prices
                {
                    let mut prices = cached_prices.write().await;
                    prices.retain(|_, price| {
                        now.signed_duration_since(price.timestamp) <= stale_threshold
                    });
                }
                
                // Clean stale pools
                {
                    let mut pools = cached_pools.write().await;
                    pools.retain(|_, pool| {
                        now.signed_duration_since(pool.detected_at) <= stale_threshold
                    });
                }
                
                debug!("🧹 Cache cleanup completed");
            }
        });
    }

    /// Simulate price data (for testing)
    async fn simulate_price(&self, token: &Pubkey) -> f64 {
        // Generate deterministic but varying prices based on token address
        let token_str = token.to_string();
        let hash: u64 = token_str.bytes().fold(0, |acc, b| acc.wrapping_mul(31).wrapping_add(b as u64));
        
        let base_price = (hash % 10000) as f64 / 100.0; // $0.01 to $100.00
        let time_variation = (chrono::Utc::now().timestamp() % 3600) as f64 / 3600.0; // 0-1 hourly cycle
        
        base_price * (0.9 + 0.2 * time_variation) // ±10% variation
    }
}

/// Market data statistics
#[derive(Debug, Serialize)]
pub struct MarketDataStats {
    pub price_subscriptions: usize,
    pub pool_subscriptions: usize,
    pub cached_prices: usize,
    pub cached_pools: usize,
    pub is_running: bool,
}
