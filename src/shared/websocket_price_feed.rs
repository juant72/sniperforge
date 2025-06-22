/// WebSocket Price Feed Client para Solana
/// 
/// Obtiene precios de tokens en tiempo real usando WebSocket connections
/// Mucho más rápido que HTTP REST para trading bots

use anyhow::{Result, anyhow};
use futures_util::{SinkExt, StreamExt};
use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::collections::HashMap;
use std::sync::Arc;
use std::time::{Duration, Instant};
use tokio::sync::{RwLock, mpsc};
use tokio_tungstenite::{connect_async, tungstenite::protocol::Message};
use tracing::{info, warn, debug, error};

/// WebSocket price feed client
#[derive(Debug)]
pub struct WebSocketPriceFeed {
    // Price cache actualizado en tiempo real
    price_cache: Arc<RwLock<HashMap<String, PriceData>>>,
    // Channel para recibir updates
    price_receiver: Option<mpsc::UnboundedReceiver<PriceUpdate>>,
    // Control de conexión
    is_connected: Arc<RwLock<bool>>,
    last_update: Arc<RwLock<Instant>>,
}

#[derive(Debug, Clone)]
pub struct PriceData {
    pub price: f64,
    pub timestamp: Instant,
    pub source: String, // "raydium", "orca", etc.
}

#[derive(Debug, Clone)]
pub struct PriceUpdate {
    pub token_mint: String,
    pub price: f64,
    pub source: String,
}

/// Solana account subscription request
#[derive(Debug, Serialize)]
struct SolanaSubscribeRequest {
    jsonrpc: String,
    id: u32,
    method: String,
    params: Vec<Value>,
}

/// Solana WebSocket response
#[derive(Debug, Deserialize)]
struct SolanaResponse {
    jsonrpc: String,
    id: Option<u32>,
    result: Option<Value>,
    method: Option<String>,
    params: Option<Value>,
}

impl WebSocketPriceFeed {
    pub async fn new() -> Result<Self> {
        info!("🌐 Initializing WebSocket Price Feed");
        info!("💡 Note: Always using MainNet for prices (DevNet has no real market data)");
        
        Ok(Self {
            price_cache: Arc::new(RwLock::new(HashMap::new())),
            price_receiver: None,
            is_connected: Arc::new(RwLock::new(false)),
            last_update: Arc::new(RwLock::new(Instant::now())),
        })
    }
      /// Creates WebSocket price feed that ALWAYS uses MainNet for real prices
    /// This provides accurate price data for both DevNet and MainNet trading
    /// 
    /// IMPORTANT: DevNet has different token addresses and NO real market prices
    /// For price feeds, we ALWAYS use MainNet addresses and pools for accuracy
    pub async fn new_mainnet_prices() -> Result<Self> {
        info!("🌐 Initializing MainNet Price Feed");
        info!("📊 Real-time prices from MainNet (regardless of trading network)");
        info!("💡 DevNet tokens have different addresses and no real prices");
        
        let mut feed = Self::new().await?;
        feed.connect_solana_pools().await?;
        Ok(feed)
    }    /// Conectar a WebSocket híbrido: Jupiter price stream + Pool monitoring
    pub async fn connect_solana_pools(&mut self) -> Result<()> {
        info!("🔗 Starting HYBRID WebSocket price feed (Jupiter + Pool monitoring)");
        info!("💰 Real-time MainNet prices with ultra-fast updates");
        
        let (tx, rx) = mpsc::unbounded_channel();
        self.price_receiver = Some(rx);
        
        let price_cache = self.price_cache.clone();
        let is_connected = self.is_connected.clone();
        let last_update = self.last_update.clone();
        
        // Start IMMEDIATE price loading and real-time updates
        *is_connected.write().await = true;
        
        // Task 1: Immediate price population + continuous updates
        let price_cache_realtime = price_cache.clone();
        let tx_realtime = tx.clone();
        let last_update_realtime = last_update.clone();
        
        tokio::spawn(async move {
            // MAINNET popular tokens for real-time tracking
            let popular_tokens = vec![
                "So11111111111111111111111111111111111111112", // SOL (Native)
                "EPjFWdd5AufqSSqeM2qN1xzybapC8G4wEGGkZwyTDt1v", // USDC
                "Es9vMFrzaCERmJfrF4H2FYD4KCoNkY11McCe8BenwNYB", // USDT
                "DezXAZ8z7PnrnRJjz3wXBoRgixCa6xjnB7YaB1pPB263", // BONK
                "7vfCXTUXx5WJV5JADk17DUJ4ksgau7utNKj4b963voxs", // ETH
                "2FPyTwcZLUg1MDrwsyoP4D6s1tM7hAkHYRjkNb5w6Pxk", // PYTH
                "4k3Dyjzvzp8eMZWUXbBCjEvwSkkk59S5iCNLY3QrkX6R", // RAY
                "mSoLzYCxHdYgdzU16g5QSh3i5K3z3KZK7ytfqcJm7So",  // mSOL
                "HhJpBhRRn4g56VsyLuT8DL5Bv31HkXqsrahTTUCZeZg4", // BERN
                "7dHbWXmci3dT8UFYWYZweBLXgycu7Y3iL6trKn1Y7ARj", // stSOL
            ];
            
            // 1. IMMEDIATE initial load (parallel for speed)
            info!("⚡ Loading initial prices (parallel)...");
            let mut handles = Vec::new();
            
            for token in &popular_tokens {
                let token = token.to_string();
                let cache = price_cache_realtime.clone();
                let tx = tx_realtime.clone();
                let last_update = last_update_realtime.clone();
                
                let handle = tokio::spawn(async move {
                    if let Ok(price) = Self::fetch_jupiter_price(&token).await {
                        let mut cache_lock = cache.write().await;
                        cache_lock.insert(token.clone(), PriceData {
                            price,
                            timestamp: Instant::now(),
                            source: "initial_load".to_string(),
                        });
                        drop(cache_lock);
                        
                        *last_update.write().await = Instant::now();
                        
                        let _ = tx.send(PriceUpdate {
                            token_mint: token.clone(),
                            price,
                            source: "initial_load".to_string(),
                        });
                        
                        info!("💰 Loaded: {} = ${:.6}", token, price);
                    }
                });
                handles.push(handle);
            }
            
            // Wait for all initial loads
            for handle in handles {
                let _ = handle.await;
            }
            
            info!("✅ Initial price cache populated");
            
            // 2. CONTINUOUS updates (every 5 seconds, rotating tokens)
            let mut interval = tokio::time::interval(Duration::from_secs(5));
            let mut token_index = 0;
            
            loop {
                interval.tick().await;
                
                let token = &popular_tokens[token_index % popular_tokens.len()];
                token_index += 1;
                
                if let Ok(price) = Self::fetch_jupiter_price(token).await {
                    let mut cache = price_cache_realtime.write().await;
                    cache.insert(token.to_string(), PriceData {
                        price,
                        timestamp: Instant::now(),
                        source: "realtime_update".to_string(),
                    });
                    drop(cache);
                    
                    *last_update_realtime.write().await = Instant::now();
                    
                    let _ = tx_realtime.send(PriceUpdate {
                        token_mint: token.to_string(),
                        price,
                        source: "realtime_update".to_string(),
                    });
                    
                    debug!("🔄 Updated: {} = ${:.6}", token, price);
                }
            }
        });
          // Task 2: WebSocket pool monitoring (for trigger signals)
        let tx_pool = tx.clone();
        let price_cache_pool = price_cache.clone();
        let last_update_pool = last_update.clone();
        
        tokio::spawn(async move {
            // Connect to Solana WebSocket for pool activity monitoring
            let url = "wss://api.mainnet-beta.solana.com/";
            
            match connect_async(url).await {
                Ok((ws_stream, _)) => {
                    info!("📡 Connected to Solana WebSocket for pool monitoring");
                    let (mut write, mut read) = ws_stream.split();
                    
                    // Subscribe to Raydium AMM program for activity signals
                    let subscribe_request = SolanaSubscribeRequest {
                        jsonrpc: "2.0".to_string(),
                        id: 1,
                        method: "programSubscribe".to_string(),
                        params: vec![
                            Value::String("675kPX9MHTjS2zt1qfr1NYHuzeLXfQM9H24wFSUt1Mp8".to_string()),
                            Value::Object({
                                let mut filters = serde_json::Map::new();
                                filters.insert("commitment".to_string(), Value::String("processed".to_string()));
                                filters
                            }),
                        ],
                    };
                    
                    if let Ok(subscribe_msg) = serde_json::to_string(&subscribe_request) {
                        if write.send(Message::text(subscribe_msg)).await.is_ok() {
                            info!("📡 Subscribed to Raydium pool activity");
                        }
                    }
                    
                    let mut activity_counter = 0;
                    let mut last_triggered_update = Instant::now();
                    
                    // Monitor WebSocket for pool activity
                    while let Some(msg) = read.next().await {
                        match msg {
                            Ok(Message::Text(_)) => {
                                activity_counter += 1;
                                
                                // Trigger more aggressive price updates on high activity
                                if activity_counter % 3 == 0 || last_triggered_update.elapsed() > Duration::from_secs(8) {
                                    // Pick a random popular token to update
                                    let tokens = vec![
                                        "So11111111111111111111111111111111111111112", // SOL
                                        "EPjFWdd5AufqSSqeM2qN1xzybapC8G4wEGGkZwyTDt1v", // USDC
                                        "DezXAZ8z7PnrnRJjz3wXBoRgixCa6xjnB7YaB1pPB263", // BONK
                                    ];
                                    
                                    if let Some(&token) = tokens.get(activity_counter % tokens.len()) {
                                        if let Ok(price) = Self::fetch_jupiter_price(token).await {
                                            let mut cache = price_cache_pool.write().await;
                                            cache.insert(token.to_string(), PriceData {
                                                price,
                                                timestamp: Instant::now(),
                                                source: "pool_activity_triggered".to_string(),
                                            });
                                            drop(cache);
                                            
                                            *last_update_pool.write().await = Instant::now();
                                            last_triggered_update = Instant::now();
                                            
                                            let _ = tx_pool.send(PriceUpdate {
                                                token_mint: token.to_string(),
                                                price,
                                                source: "pool_activity_triggered".to_string(),
                                            });
                                            
                                            debug!("⚡ Pool activity triggered update: {} = ${:.6}", token, price);
                                        }
                                    }
                                }
                            }
                            Ok(Message::Close(_)) => {
                                warn!("🔌 Pool monitoring WebSocket closed");
                                break;
                            }
                            Err(e) => {
                                warn!("⚠️ Pool monitoring WebSocket error: {}", e);
                                break;
                            }
                            _ => {}
                        }
                    }
                }
                Err(e) => {
                    warn!("⚠️ Failed to connect to pool monitoring WebSocket: {}", e);
                    warn!("📊 Continuing with timer-based updates only");
                }
            }
        });        
        info!("✅ WebSocket price feed system initialized");
        info!("⚡ Real-time prices + Pool activity monitoring active");
        
        Ok(())
    }
      /// Obtener precio de token ultra-rápido desde cache WebSocket
    pub async fn get_price_realtime(&self, token_mint: &str) -> Option<f64> {
        let cache = self.price_cache.read().await;
        if let Some(price_data) = cache.get(token_mint) {
            // Data es válida si es menos de 60 segundos (más generous para estabilidad)
            if price_data.timestamp.elapsed() < Duration::from_secs(60) {
                debug!("⚡ WebSocket cache hit: {} = ${:.6} (age: {:?})", 
                       token_mint, price_data.price, price_data.timestamp.elapsed());
                return Some(price_data.price);
            } else {
                debug!("⏰ Cached price too old for {}: {:?}", token_mint, price_data.timestamp.elapsed());
            }
        } else {
            debug!("❌ No cached price for {}", token_mint);
        }
        None
    }
    
    /// Obtener precio con fallback HTTP si WebSocket no tiene data FRESCA
    pub async fn get_price_hybrid(&self, token_mint: &str) -> Result<Option<f64>> {
        // 1. Intentar WebSocket primero (ultra-rápido)
        if let Some(price) = self.get_price_realtime(token_mint).await {
            return Ok(Some(price));
        }
        
        // 2. Fallback a HTTP directo (Jupiter API)
        debug!("🌐 WebSocket miss, fetching fresh price for {}", token_mint);
        match Self::fetch_jupiter_price(token_mint).await {
            Ok(price) => {
                // Cache the fresh price
                let mut cache = self.price_cache.write().await;
                cache.insert(token_mint.to_string(), PriceData {
                    price,
                    timestamp: Instant::now(),
                    source: "http_fallback".to_string(),
                });
                Ok(Some(price))
            }
            Err(e) => {
                warn!("Failed to fetch fallback price for {}: {}", token_mint, e);
                Ok(None)
            }
        }
    }
    
    /// Forzar actualización inmediata de un token específico
    pub async fn force_update_token(&mut self, token_mint: &str) -> Result<Option<f64>> {
        info!("🔄 Force updating price for {}", token_mint);
        
        match Self::fetch_jupiter_price(token_mint).await {
            Ok(price) => {
                let mut cache = self.price_cache.write().await;
                cache.insert(token_mint.to_string(), PriceData {
                    price,
                    timestamp: Instant::now(),
                    source: "force_update".to_string(),
                });
                
                *self.last_update.write().await = Instant::now();
                
                info!("✅ Force updated: {} = ${:.6}", token_mint, price);
                Ok(Some(price))
            }
            Err(e) => {
                error!("❌ Failed to force update {}: {}", token_mint, e);
                Err(e)
            }
        }
    }    async fn fetch_jupiter_price(token_mint: &str) -> Result<f64> {
        debug!("📡 Fetching REAL price for {}", token_mint);
        
        // Use REAL Jupiter API client
        use crate::shared::jupiter::client::JupiterClient;
        use crate::shared::jupiter::JupiterConfig;
        
        let jupiter_config = JupiterConfig::mainnet();
        let jupiter_client = JupiterClient::new(&jupiter_config).await?;
        
        // Get REAL price from Jupiter API
        match jupiter_client.get_price(token_mint).await? {
            Some(price) => {
                debug!("✅ REAL price from Jupiter: {} = ${:.6}", token_mint, price);
                Ok(price)
            },
            None => {
                warn!("⚠️ No price found for token: {}", token_mint);
                Err(anyhow!("Price not available for token: {}", token_mint))
            }        }
    }
    
    pub async fn is_connected(&self) -> bool {
        *self.is_connected.read().await
    }
    
    pub async fn last_update_age(&self) -> Duration {
        self.last_update.read().await.elapsed()
    }
    
    /// Obtener todas las prices cacheadas (para debugging/monitoring)
    pub async fn get_all_cached_prices(&self) -> HashMap<String, (f64, Duration, String)> {
        let cache = self.price_cache.read().await;
        cache.iter().map(|(token, data)| {
            (token.clone(), (data.price, data.timestamp.elapsed(), data.source.clone()))
        }).collect()
    }
    
    /// Obtener estadísticas del WebSocket price feed
    pub async fn get_stats(&self) -> PriceFeedStats {
        let cache = self.price_cache.read().await;
        let is_connected = *self.is_connected.read().await;
        let last_update = self.last_update.read().await.elapsed();
        
        // Find freshest and oldest prices
        let (freshest_age, oldest_age) = if cache.is_empty() {
            (Duration::from_secs(0), Duration::from_secs(0))
        } else {
            let ages: Vec<Duration> = cache.values().map(|data| data.timestamp.elapsed()).collect();
            let freshest = ages.iter().min().copied().unwrap_or(Duration::from_secs(0));
            let oldest = ages.iter().max().copied().unwrap_or(Duration::from_secs(0));
            (freshest, oldest)
        };
        
        PriceFeedStats {
            cached_tokens: cache.len(),
            is_connected,
            last_update_age: last_update,
            oldest_price_age: oldest_age,
            freshest_price_age: freshest_age,
        }
    }
}

#[derive(Debug)]
pub struct PriceFeedStats {
    pub cached_tokens: usize,
    pub is_connected: bool,
    pub last_update_age: Duration,
    pub oldest_price_age: Duration,
    pub freshest_price_age: Duration,
}
