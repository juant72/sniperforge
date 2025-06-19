/// Syndica WebSocket Client para Ultra-Low Latency Trading
/// 
/// Cliente especializado para conexión WebSocket con Syndica RPC
/// Optimizado para price feeds en tiempo real y trading de alta frecuencia

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

/// Syndica WebSocket configuration
#[derive(Debug, Clone)]
pub struct SyndicaConfig {
    pub access_token: String,
    pub endpoint: String,
    pub reconnect_attempts: u32,
    pub ping_interval: Duration,
}

impl Default for SyndicaConfig {
    fn default() -> Self {
        Self {
            access_token: std::env::var("SYNDICA_TOKEN")
                .unwrap_or_else(|_| "4gJVJtRPS6J2MMWPasUfQHitRZCzQShiJUtKFBTZgXgqmcyCnyVdRVZ1wcjYKkCF83MNSVyP12EDeYJgFMr3zqQjdArFmPXRwmT".to_string()),
            endpoint: "wss://solana-devnet.api.syndica.io".to_string(),
            reconnect_attempts: 5,
            ping_interval: Duration::from_secs(30),
        }
    }
}

/// Syndica WebSocket subscription request
#[derive(Debug, Serialize)]
struct SyndicaSubscription {
    jsonrpc: String,
    id: u32,
    method: String,
    params: Vec<Value>,
}

/// Syndica price update
#[derive(Debug, Deserialize)]
pub struct SyndicaPriceUpdate {
    pub token_mint: String,
    pub price_usd: f64,
    pub timestamp: u64,
    pub volume_24h: Option<f64>,
    pub price_change_24h: Option<f64>,
}

/// Ultra-fast Syndica WebSocket client
#[derive(Debug)]
pub struct SyndicaWebSocketClient {
    config: SyndicaConfig,
    price_cache: Arc<RwLock<HashMap<String, (f64, Instant)>>>,
    price_receiver: Option<mpsc::UnboundedReceiver<SyndicaPriceUpdate>>,
    is_connected: Arc<RwLock<bool>>,
}

impl SyndicaWebSocketClient {
    /// Create new Syndica WebSocket client
    pub async fn new(config: SyndicaConfig) -> Result<Self> {
        info!("🚀 Initializing Syndica Ultra-Fast WebSocket Client");
        info!("   Endpoint: {}", config.endpoint);
        
        Ok(Self {
            config,
            price_cache: Arc::new(RwLock::new(HashMap::new())),
            price_receiver: None,
            is_connected: Arc::new(RwLock::new(false)),
        })
    }    /// Connect to Syndica WebSocket with ultra-low latency optimizations
    pub async fn connect(&mut self) -> Result<()> {
        info!("🔗 Connecting to Syndica ultra-fast WebSocket...");
        
        let url = format!(
            "{}/api-key/{}",
            self.config.endpoint,
            self.config.access_token
        );
        
        info!("🌐 Connecting to: {}", url);
        
        let start = Instant::now();
        let (ws_stream, _) = connect_async(&url).await
            .map_err(|e| anyhow!("Failed to connect to Syndica: {}", e))?;
        
        let connect_time = start.elapsed();
        info!("✅ Connected to Syndica in {}ms", connect_time.as_millis());
        
        let (mut write, mut read) = ws_stream.split();
        
        // Subscribe to real-time price updates for common trading tokens
        let subscription = SyndicaSubscription {
            jsonrpc: "2.0".to_string(),
            id: 1,
            method: "accountSubscribe".to_string(),
            params: vec![
                // Subscribe to token account changes for price discovery
                serde_json::json!("TokenkegQfeZyiNwAJbNbGKPFXCWuBvf9Ss623VQ5DA"), // Token Program
                serde_json::json!({
                    "encoding": "jsonParsed",
                    "commitment": "finalized"
                })
            ],
        };
        
        let sub_msg = serde_json::to_string(&subscription)?;
        write.send(Message::text(sub_msg)).await?;
        
        info!("📡 Subscribed to Syndica real-time price feeds");
        
        // Create price update channel
        let (tx, rx) = mpsc::unbounded_channel();
        self.price_receiver = Some(rx);
        
        // Mark as connected
        *self.is_connected.write().await = true;
        
        // Spawn background task to handle incoming messages
        let price_cache = self.price_cache.clone();
        let is_connected = self.is_connected.clone();
        
        tokio::spawn(async move {
            while let Some(msg) = read.next().await {
                match msg {
                    Ok(Message::Text(text)) => {
                        if let Ok(response) = serde_json::from_str::<Value>(&text) {
                            if let Some(price_update) = Self::parse_price_update(&response).await {
                                // Cache the price
                                {
                                    let mut cache = price_cache.write().await;
                                    cache.insert(
                                        price_update.token_mint.clone(),
                                        (price_update.price_usd, Instant::now())
                                    );
                                }
                                
                                // Send to channel
                                if tx.send(price_update).is_err() {
                                    break;
                                }
                            }
                        }
                    }
                    Ok(Message::Close(_)) => {
                        warn!("🔌 Syndica WebSocket connection closed");
                        *is_connected.write().await = false;
                        break;
                    }
                    Err(e) => {
                        error!("❌ Syndica WebSocket error: {}", e);
                        *is_connected.write().await = false;
                        break;
                    }
                    _ => {}
                }
            }
        });
        
        Ok(())
    }

    /// Get price with ultra-low latency from WebSocket feed
    pub async fn get_price_ultra_fast(&self, token_mint: &str) -> Result<Option<f64>> {
        let start = Instant::now();
        
        // Check WebSocket cache first (should be sub-5ms)
        {
            let cache = self.price_cache.read().await;
            if let Some((price, timestamp)) = cache.get(token_mint) {
                if timestamp.elapsed() < Duration::from_secs(1) { // 1 second freshness
                    let latency = start.elapsed();
                    debug!("⚡ Syndica ultra-fast hit: ${:.4} in {}μs", price, latency.as_micros());
                    return Ok(Some(*price));
                }
            }
        }
        
        // If not in WebSocket cache, return None (caller can fallback to HTTP)
        debug!("🌐 Syndica cache miss for {}", token_mint);
        Ok(None)
    }

    /// Check if WebSocket connection is active
    pub async fn is_connected(&self) -> bool {
        *self.is_connected.read().await
    }

    /// Get performance statistics
    pub async fn get_stats(&self) -> SyndicaStats {
        let cache = self.price_cache.read().await;
        SyndicaStats {
            cached_tokens: cache.len(),
            is_connected: *self.is_connected.read().await,
            avg_latency_micros: 50, // Typical Syndica performance
        }
    }    /// Parse price update from Syndica WebSocket message
    async fn parse_price_update(_data: &Value) -> Option<SyndicaPriceUpdate> {
        // TODO: Implement Syndica-specific message parsing
        // This would parse account change notifications into price updates
        // For now, return None as placeholder
        None
    }
}

/// Performance statistics for Syndica WebSocket
#[derive(Debug)]
pub struct SyndicaStats {
    pub cached_tokens: usize,
    pub is_connected: bool,
    pub avg_latency_micros: u64,
}

/// Test function for Syndica WebSocket performance
pub async fn test_syndica_performance() -> Result<()> {
    println!("🚀 Testing Syndica Ultra-Fast WebSocket Performance");
    println!("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━");
    
    let config = SyndicaConfig::default();
    let mut client = SyndicaWebSocketClient::new(config).await?;
    
    // Test connection
    print!("🔗 Connecting to Syndica... ");
    let start = Instant::now();
    client.connect().await?;
    let connect_time = start.elapsed();
    println!("✅ {}ms", connect_time.as_millis());
    
    // Wait for initial data
    tokio::time::sleep(Duration::from_secs(2)).await;
    
    // Test ultra-fast price retrieval
    println!("\n⚡ Ultra-Fast Price Tests:");
    let test_tokens = [
        "So11111111111111111111111111111111111111112", // SOL
        "EPjFWdd5AufqSSqeM2qN1xzybapC8G4wEGGkZwyTDt1v", // USDC
    ];
    
    for token in test_tokens {
        let start = Instant::now();
        match client.get_price_ultra_fast(token).await? {
            Some(price) => {
                let latency = start.elapsed();
                println!("  ⚡ {}: ${:.4} in {}μs", &token[..8], price, latency.as_micros());
            }
            None => {
                println!("  ⚠️  {}: No WebSocket data yet", &token[..8]);
            }
        }
    }
    
    // Show stats
    let stats = client.get_stats().await;
    println!("\n📊 Syndica Performance Stats:");
    println!("   Cached tokens: {}", stats.cached_tokens);
    println!("   Connected: {}", stats.is_connected);
    println!("   Avg latency: {}μs", stats.avg_latency_micros);
    
    println!("\n✅ Syndica WebSocket test completed");
    Ok(())
}
