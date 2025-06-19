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
        
        Ok(Self {
            price_cache: Arc::new(RwLock::new(HashMap::new())),
            price_receiver: None,
            is_connected: Arc::new(RwLock::new(false)),
            last_update: Arc::new(RwLock::new(Instant::now())),
        })
    }
      /// Conectar a Solana WebSocket y suscribirse a pools de Raydium
    pub async fn connect_solana_pools(&mut self) -> Result<()> {
        info!("🔗 Connecting to Solana WebSocket for pool monitoring");
        
        let url = "wss://api.devnet.solana.com/";
        let (ws_stream, _) = connect_async(url).await?;
        let (mut write, mut read) = ws_stream.split();
        
        // Subscribe to Raydium AMM pools
        let subscribe_request = SolanaSubscribeRequest {
            jsonrpc: "2.0".to_string(),
            id: 1,
            method: "programSubscribe".to_string(),
            params: vec![
                Value::String("675kPX9MHTjS2zt1qfr1NYHuzeLXfQM9H24wFSUt1Mp8".to_string()), // Raydium AMM
                Value::Object(serde_json::Map::new()),
            ],
        };        let subscribe_msg = serde_json::to_string(&subscribe_request)
            .map_err(|e| anyhow::anyhow!("Failed to serialize subscribe request: {}", e))?;
        let message = Message::text(subscribe_msg);
        write.send(message).await?;
        
        info!("📡 Subscribed to Raydium pool updates");
        
        let (tx, rx) = mpsc::unbounded_channel();
        self.price_receiver = Some(rx);
        
        let price_cache = self.price_cache.clone();
        let is_connected = self.is_connected.clone();
        let last_update = self.last_update.clone();
        
        // Background task para procesar mensajes WebSocket
        tokio::spawn(async move {
            *is_connected.write().await = true;
            
            while let Some(msg) = read.next().await {
                match msg {
                    Ok(Message::Text(text)) => {
                        if let Ok(response) = serde_json::from_str::<SolanaResponse>(&text) {
                            if let Some(params) = response.params {
                                if let Some(result) = params.get("result") {
                                    // Parse pool data and extract price info
                                    if let Some(price_update) = Self::parse_pool_update(result).await {
                                        // Update cache
                                        let mut cache = price_cache.write().await;
                                        cache.insert(price_update.token_mint.clone(), PriceData {
                                            price: price_update.price,
                                            timestamp: Instant::now(),
                                            source: price_update.source.clone(),
                                        });
                                        
                                        *last_update.write().await = Instant::now();
                                        
                                        // Send update via channel
                                        let _ = tx.send(price_update);
                                        
                                        debug!("💰 Price update received via WebSocket");
                                    }
                                }
                            }
                        }
                    }
                    Ok(Message::Close(_)) => {
                        warn!("🔌 WebSocket connection closed");
                        *is_connected.write().await = false;
                        break;
                    }
                    Err(e) => {
                        error!("❌ WebSocket error: {}", e);
                        *is_connected.write().await = false;
                        break;
                    }
                    _ => {}
                }
            }
        });
        
        Ok(())
    }
    
    /// Obtener precio de token ultra-rápido desde cache WebSocket
    pub async fn get_price_realtime(&self, token_mint: &str) -> Option<f64> {
        let cache = self.price_cache.read().await;
        if let Some(price_data) = cache.get(token_mint) {
            // Data es válida si es menos de 30 segundos
            if price_data.timestamp.elapsed() < Duration::from_secs(30) {
                debug!("⚡ WebSocket price hit: {} = ${:.6}", token_mint, price_data.price);
                return Some(price_data.price);
            }
        }
        None
    }
    
    /// Obtener precio con fallback HTTP si WebSocket no tiene data
    pub async fn get_price_hybrid(&self, token_mint: &str) -> Result<Option<f64>> {
        // 1. Intentar WebSocket primero (ultra-rápido)
        if let Some(price) = self.get_price_realtime(token_mint).await {
            return Ok(Some(price));
        }
        
        // 2. Fallback a HTTP si no hay data WebSocket
        debug!("🌐 WebSocket miss, falling back to HTTP for {}", token_mint);
        self.get_price_http_fallback(token_mint).await
    }
    
    async fn get_price_http_fallback(&self, token_mint: &str) -> Result<Option<f64>> {
        // Usar Jupiter API como fallback
        let url = format!("https://price.jup.ag/v4/price?ids={}", token_mint);
        
        let client = reqwest::Client::builder()
            .timeout(Duration::from_millis(1000)) // Timeout agresivo
            .build()?;
        
        let response = client.get(&url).send().await?;
        
        if response.status().is_success() {
            let price_data: Value = response.json().await?;
            
            if let Some(data) = price_data.get("data") {
                if let Some(token_data) = data.get(token_mint) {
                    if let Some(price) = token_data.get("price") {
                        if let Some(price_num) = price.as_f64() {
                            // Cache en WebSocket store también
                            let mut cache = self.price_cache.write().await;
                            cache.insert(token_mint.to_string(), PriceData {
                                price: price_num,
                                timestamp: Instant::now(),
                                source: "http_fallback".to_string(),
                            });
                            
                            return Ok(Some(price_num));
                        }
                    }
                }
            }
        }
        
        Ok(None)
    }
      async fn parse_pool_update(_data: &Value) -> Option<PriceUpdate> {
        // TODO: Implementar parsing de Raydium pool data
        // Por ahora retornamos None, pero aquí iría la lógica
        // para extraer precios de los pools de Raydium
        None
    }
    
    pub async fn is_connected(&self) -> bool {
        *self.is_connected.read().await
    }
    
    pub async fn last_update_age(&self) -> Duration {
        self.last_update.read().await.elapsed()
    }
    
    /// Obtener estadísticas del WebSocket price feed
    pub async fn get_stats(&self) -> PriceFeedStats {
        let cache = self.price_cache.read().await;
        let is_connected = *self.is_connected.read().await;
        let last_update = self.last_update.read().await.elapsed();
        
        PriceFeedStats {
            cached_tokens: cache.len(),
            is_connected,
            last_update_age: last_update,
            oldest_price_age: cache.values()
                .map(|data| data.timestamp.elapsed())
                .max()
                .unwrap_or(Duration::from_secs(0)),
        }
    }
}

#[derive(Debug)]
pub struct PriceFeedStats {
    pub cached_tokens: usize,
    pub is_connected: bool,
    pub last_update_age: Duration,
    pub oldest_price_age: Duration,
}
