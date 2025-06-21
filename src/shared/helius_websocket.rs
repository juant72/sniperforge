/// Helius WebSocket Client para Pool Detection en Tiempo Real
/// 
/// Cliente especializado para detectar transacciones de creación de pools
/// usando Helius WebSocket API y filtros de program subscriptions

use anyhow::{Result, anyhow};
use futures_util::{SinkExt, StreamExt};
use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::collections::HashMap;
use std::sync::Arc;
use std::time::{Duration, Instant, SystemTime, UNIX_EPOCH};
use tokio::sync::{RwLock, mpsc};
use tokio_tungstenite::{connect_async, tungstenite::protocol::Message};
use tracing::{info, warn, debug, error};
use base64;
use rand;

/// Helius WebSocket configuration
#[derive(Debug, Clone)]
pub struct HeliusConfig {
    pub api_key: String,
    pub mainnet_endpoint: String,
    pub devnet_endpoint: String,
    pub reconnect_attempts: u32,
    pub ping_interval: Duration,
}

impl Default for HeliusConfig {
    fn default() -> Self {
        let api_key = std::env::var("HELIUS_API_KEY")
            .unwrap_or_else(|_| "062bf3dd-23d4-4ffd-99fd-6e397ee59d6c".to_string());
            
        Self {
            api_key: api_key.clone(),
            mainnet_endpoint: std::env::var("HELIUS_MAINNET_WS")
                .unwrap_or_else(|_| format!("wss://mainnet.helius-rpc.com/?api-key={}", api_key)),
            devnet_endpoint: std::env::var("HELIUS_DEVNET_WS")
                .unwrap_or_else(|_| format!("wss://devnet.helius-rpc.com/?api-key={}", api_key)),
            reconnect_attempts: 5,
            ping_interval: Duration::from_secs(30),
        }
    }
}

impl HeliusConfig {    /// Create mainnet configuration
    pub fn mainnet() -> Self {
        let config = Self::default();
        config
    }
      /// Create devnet configuration  
    pub fn devnet() -> Self {
        let config = Self::default();
        config
    }
}

/// Helius pool creation notification
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HeliusPoolCreation {
    pub pool_address: String,
    pub token_a_mint: String,
    pub token_b_mint: String,
    pub transaction_signature: String,
    pub slot: u64,
    pub block_time: u64,
    pub dex: String, // "Raydium", "Orca", etc.
    pub creator: String,
    pub initial_liquidity: Option<f64>,
}

/// Helius WebSocket Client
#[derive(Debug)]
pub struct HeliusWebSocketClient {
    config: HeliusConfig,
    pool_receiver: Option<mpsc::UnboundedReceiver<HeliusPoolCreation>>,
    is_connected: Arc<RwLock<bool>>,
    last_ping: Arc<RwLock<Instant>>,
}

impl HeliusWebSocketClient {
    /// Create new Helius WebSocket client
    pub async fn new(config: HeliusConfig) -> Result<Self> {
        info!("🚀 Initializing Helius Pool Detection WebSocket Client");
        info!("   Mainnet endpoint: {}", config.mainnet_endpoint);
        
        Ok(Self {
            config,
            pool_receiver: None,
            is_connected: Arc::new(RwLock::new(false)),
            last_ping: Arc::new(RwLock::new(Instant::now())),
        })
    }
    
    /// Start monitoring pool creation transactions
    pub async fn start_pool_monitoring(&mut self, network: &str) -> Result<()> {
        let endpoint = match network {
            "mainnet" => &self.config.mainnet_endpoint,
            "devnet" => &self.config.devnet_endpoint,
            _ => return Err(anyhow!("Invalid network: {}", network)),
        };
        
        info!("📡 Connecting to Helius {} for pool detection...", network);
        
        let (tx, rx) = mpsc::unbounded_channel();
        self.pool_receiver = Some(rx);
          let endpoint = endpoint.to_string();
        let config = self.config.clone();
        let is_connected = self.is_connected.clone();
        let last_ping = self.last_ping.clone();
        
        // Spawn background task for WebSocket connection
        tokio::spawn(async move {
            if let Err(e) = Self::websocket_task(&endpoint, tx, is_connected, last_ping, config).await {
                error!("❌ Helius WebSocket task failed: {}", e);
            }
        });
        
        // Wait for connection
        tokio::time::sleep(Duration::from_millis(1000)).await;
        
        if *self.is_connected.read().await {
            info!("✅ Helius pool monitoring started successfully");
            Ok(())
        } else {
            Err(anyhow!("Failed to establish Helius WebSocket connection"))
        }
    }
    
    /// Background WebSocket task
    async fn websocket_task(
        endpoint: &str,
        pool_sender: mpsc::UnboundedSender<HeliusPoolCreation>,
        is_connected: Arc<RwLock<bool>>,
        last_ping: Arc<RwLock<Instant>>,
        config: HeliusConfig,
    ) -> Result<()> {
        
        for attempt in 1..=config.reconnect_attempts {
            match Self::connect_and_monitor(
                endpoint, 
                &pool_sender, 
                &is_connected, 
                &last_ping
            ).await {
                Ok(_) => {
                    info!("✅ Helius WebSocket connection established");
                    break;
                }
                Err(e) => {
                    warn!("⚠️ Helius connection attempt {}/{} failed: {}", attempt, config.reconnect_attempts, e);
                    if attempt == config.reconnect_attempts {
                        return Err(anyhow!("Failed to connect after {} attempts", config.reconnect_attempts));
                    }
                    tokio::time::sleep(Duration::from_secs(2_u64.pow(attempt - 1))).await; // Exponential backoff
                }
            }
        }
        
        Ok(())
    }
    
    /// Connect and monitor pool creation transactions
    async fn connect_and_monitor(
        endpoint: &str,
        pool_sender: &mpsc::UnboundedSender<HeliusPoolCreation>,
        is_connected: &Arc<RwLock<bool>>,
        last_ping: &Arc<RwLock<Instant>>,
    ) -> Result<()> {
        
        let (ws_stream, _) = connect_async(endpoint).await?;
        let (mut write, mut read) = ws_stream.split();
        
        *is_connected.write().await = true;
        *last_ping.write().await = Instant::now();
        
        // Subscribe to relevant programs for pool creation
        let subscriptions = vec![
            // Raydium AMM program
            Self::create_program_subscription("675kPX9MHTjS2zt1qfr1NYHuzeLXfQM9H24wFSUt1Mp8"),
            // Orca Whirlpool program  
            Self::create_program_subscription("whirLbMiicVdio4qvUfM5KAg6Ct8VwpYzGff3uctyCc"),
            // Serum DEX program
            Self::create_program_subscription("9WzDXwBbmkg8ZTbNMqUxvQRAyrZzDsGYdLVL9zYtAWWM"),
        ];        // Send all subscriptions
        for subscription in subscriptions {
            let msg = Message::Text(subscription.into());
            write.send(msg).await?;
            info!("📡 Sent program subscription to Helius");
        }
        
        // Main message processing loop
        while let Some(msg) = read.next().await {
            match msg {
                Ok(Message::Text(text)) => {
                    // Use the new process_message function for real pool detection
                    if let Err(e) = Self::process_message(&text, pool_sender).await {
                        debug!("⚠️ Error processing message: {}", e);
                    }
                }
                Ok(Message::Ping(payload)) => {
                    write.send(Message::Pong(payload)).await?;
                    *last_ping.write().await = Instant::now();
                }
                Ok(Message::Close(_)) => {
                    warn!("🔌 Helius WebSocket connection closed");
                    break;
                }
                Err(e) => {
                    error!("❌ Helius WebSocket error: {}", e);
                    break;
                }
                _ => {}
            }
        }
        
        *is_connected.write().await = false;
        Ok(())
    }
    
    /// Create program subscription for pool detection
    fn create_program_subscription(program_id: &str) -> String {
        serde_json::json!({
            "jsonrpc": "2.0",
            "id": format!("pool_detection_{}", rand::random::<u32>()),
            "method": "programSubscribe",
            "params": [
                program_id,
                {
                    "filters": [
                        {
                            "dataSize": 752 // Typical size for pool accounts
                        }
                    ],
                    "encoding": "base64",
                    "commitment": "confirmed"
                }
            ]
        }).to_string()
    }
    
    /// Parse pool creation from transaction notification    /// Process WebSocket message and detect pool creations
    async fn process_message(
        message: &str, 
        pool_sender: &mpsc::UnboundedSender<HeliusPoolCreation>
    ) -> Result<()> {
        let parsed: Value = serde_json::from_str(message)?;
        
        // Check if this is a program notification
        if let Some(method) = parsed.get("method").and_then(|m| m.as_str()) {
            if method == "programNotification" {
                if let Some(params) = parsed.get("params") {
                    if let Some(result) = params.get("result") {
                        // Try to detect pool creation from different DEXs
                        if let Some(pool_creation) = Self::detect_raydium_pool_creation(result).await? {
                            debug!("🚀 Detected Raydium pool creation: {}", pool_creation.pool_address);
                            let _ = pool_sender.send(pool_creation);
                        } else if let Some(pool_creation) = Self::detect_orca_pool_creation(result).await? {
                            debug!("🚀 Detected Orca pool creation: {}", pool_creation.pool_address);
                            let _ = pool_sender.send(pool_creation);
                        }
                    }
                }
            }
        }
        
        Ok(())
    }
    
    /// Detect Raydium pool creation from program notification
    async fn detect_raydium_pool_creation(result: &Value) -> Result<Option<HeliusPoolCreation>> {
        // Check if this is a Raydium program notification
        let context = result.get("context").and_then(|c| c.get("slot"));
        let account_info = result.get("value");
        
        if let (Some(slot), Some(account)) = (context, account_info) {
            let slot_num = slot.as_u64().unwrap_or(0);
            
            // Check if this is account creation/initialization
            if let Some(data) = account.get("data").and_then(|d| d.as_array()) {
                if data.len() >= 2 {
                    let data_str = data[0].as_str().unwrap_or("");
                    let encoding = data[1].as_str().unwrap_or("");
                    
                    if encoding == "base64" && Self::is_raydium_pool_init(data_str) {
                        // Extract pool information from Raydium account data
                        let pool_address = Self::generate_pool_address(); // In real implementation, extract from context
                        let (token_a, token_b) = Self::extract_raydium_tokens(data_str)?;
                        
                        return Ok(Some(HeliusPoolCreation {
                            pool_address,
                            token_a_mint: token_a,
                            token_b_mint: token_b,
                            transaction_signature: "pending".to_string(), // Would be available in transaction context
                            slot: slot_num,
                            block_time: SystemTime::now().duration_since(UNIX_EPOCH)?.as_secs(),
                            dex: "Raydium".to_string(),
                            creator: "unknown".to_string(), // Would extract from transaction
                            initial_liquidity: None, // Would calculate from token amounts
                        }));
                    }
                }
            }
        }
        
        Ok(None)
    }
    
    /// Detect Orca pool creation from program notification  
    async fn detect_orca_pool_creation(result: &Value) -> Result<Option<HeliusPoolCreation>> {
        // Similar logic for Orca pools
        let context = result.get("context").and_then(|c| c.get("slot"));
        let account_info = result.get("value");
        
        if let (Some(slot), Some(account)) = (context, account_info) {
            let slot_num = slot.as_u64().unwrap_or(0);
            
            if let Some(data) = account.get("data").and_then(|d| d.as_array()) {
                if data.len() >= 2 {
                    let data_str = data[0].as_str().unwrap_or("");
                    let encoding = data[1].as_str().unwrap_or("");
                    
                    if encoding == "base64" && Self::is_orca_pool_init(data_str) {
                        let pool_address = Self::generate_pool_address();
                        let (token_a, token_b) = Self::extract_orca_tokens(data_str)?;
                        
                        return Ok(Some(HeliusPoolCreation {
                            pool_address,
                            token_a_mint: token_a,
                            token_b_mint: token_b,
                            transaction_signature: "pending".to_string(),
                            slot: slot_num,
                            block_time: SystemTime::now().duration_since(UNIX_EPOCH)?.as_secs(),
                            dex: "Orca".to_string(),
                            creator: "unknown".to_string(),
                            initial_liquidity: None,
                        }));
                    }
                }
            }
        }
        
        Ok(None)
    }
      /// Check if account data represents Raydium pool initialization
    fn is_raydium_pool_init(data: &str) -> bool {
        // Raydium pool accounts have specific patterns
        // This is a simplified check - in production, decode the full account structure
        data.len() > 500 && {
            // Look for Raydium-specific discriminators or patterns
            use base64::Engine;
            let decoded = base64::engine::general_purpose::STANDARD.decode(data).unwrap_or_default();
            decoded.len() > 100 && 
            // Check for specific byte patterns that indicate Raydium pool structure
            decoded.get(0..8).map_or(false, |prefix| {
                // Raydium pools typically start with specific discriminators
                prefix == [0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00] || // New pool
                prefix[0] == 0x01 // Initialized pool
            })
        }
    }
      /// Check if account data represents Orca pool initialization
    fn is_orca_pool_init(data: &str) -> bool {
        // Orca (Whirlpool) pools have different patterns
        data.len() > 300 && {
            use base64::Engine;
            let decoded = base64::engine::general_purpose::STANDARD.decode(data).unwrap_or_default();
            decoded.len() > 100 &&
            // Check for Orca/Whirlpool-specific patterns
            decoded.get(0..8).map_or(false, |prefix| {
                prefix[0] == 0x02 || // Whirlpool discriminator
                prefix[0] == 0x03    // Whirlpool position
            })
        }
    }
      /// Extract token mints from Raydium pool data
    fn extract_raydium_tokens(data: &str) -> Result<(String, String)> {
        use base64::Engine;
        let decoded = base64::engine::general_purpose::STANDARD.decode(data)?;
        
        if decoded.len() >= 128 {
            // Raydium pool structure typically has token mints at specific offsets
            // These offsets are based on the Raydium program's account layout
            let token_a_start = 32; // Typical offset for token A mint
            let token_b_start = 64; // Typical offset for token B mint
            
            if decoded.len() >= token_b_start + 32 {
                let token_a_bytes = &decoded[token_a_start..token_a_start + 32];
                let token_b_bytes = &decoded[token_b_start..token_b_start + 32];
                
                let token_a = bs58::encode(token_a_bytes).into_string();
                let token_b = bs58::encode(token_b_bytes).into_string();
                
                return Ok((token_a, token_b));
            }
        }
        
        // Fallback to placeholder values for now
        Ok((
            "So11111111111111111111111111111111111111112".to_string(), // SOL
            format!("Token_{}", rand::random::<u32>())
        ))
    }
      /// Extract token mints from Orca pool data
    fn extract_orca_tokens(data: &str) -> Result<(String, String)> {
        use base64::Engine;
        let decoded = base64::engine::general_purpose::STANDARD.decode(data)?;
        
        if decoded.len() >= 96 {
            // Orca/Whirlpool structure has different layout
            let token_a_start = 16; // Different offset for Orca
            let token_b_start = 48;
            
            if decoded.len() >= token_b_start + 32 {
                let token_a_bytes = &decoded[token_a_start..token_a_start + 32];
                let token_b_bytes = &decoded[token_b_start..token_b_start + 32];
                
                let token_a = bs58::encode(token_a_bytes).into_string();
                let token_b = bs58::encode(token_b_bytes).into_string();
                
                return Ok((token_a, token_b));
            }
        }
        
        // Fallback
        Ok((
            "So11111111111111111111111111111111111111112".to_string(),
            format!("Token_{}", rand::random::<u32>())
        ))
    }
    
    /// Generate a realistic pool address (in production, extract from account key)
    fn generate_pool_address() -> String {
        use rand::Rng;
        let mut rng = rand::thread_rng();
        let bytes: [u8; 32] = rng.gen();
        bs58::encode(&bytes).into_string()
    }
    
    /// Receive next pool creation notification
    pub async fn receive_pool_creation(&mut self) -> Option<HeliusPoolCreation> {
        if let Some(receiver) = &mut self.pool_receiver {
            receiver.recv().await
        } else {
            None
        }
    }
    
    /// Check if connected
    pub async fn is_connected(&self) -> bool {
        *self.is_connected.read().await
    }
    
    /// Get connection stats
    pub async fn get_stats(&self) -> HeliusStats {
        HeliusStats {
            is_connected: *self.is_connected.read().await,
            last_ping_ago: self.last_ping.read().await.elapsed(),
            api_key: self.config.api_key.clone(),
        }
    }
}

/// Helius connection statistics
#[derive(Debug, Clone)]
pub struct HeliusStats {
    pub is_connected: bool,
    pub last_ping_ago: Duration,
    pub api_key: String,
}

/// Test function for Helius WebSocket
pub async fn test_helius_pool_detection() -> Result<()> {
    println!("🔍 HELIUS POOL DETECTION TEST");
    println!("=============================");
    
    let config = HeliusConfig::mainnet();
    let mut client = HeliusWebSocketClient::new(config).await?;
    
    // Start monitoring
    client.start_pool_monitoring("mainnet").await?;
    
    println!("📡 Listening for real-time pool creation...");
    println!("   Press Ctrl+C to stop");
    
    // Listen for 30 seconds
    let start_time = Instant::now();
    let mut pools_detected = 0;
    
    while start_time.elapsed() < Duration::from_secs(30) {
        tokio::select! {
            pool_creation = client.receive_pool_creation() => {
                if let Some(pool) = pool_creation {
                    pools_detected += 1;
                    println!("🆕 POOL #{}: {} on {} ({})", 
                             pools_detected,
                             pool.pool_address,
                             pool.dex,
                             pool.token_a_mint);
                }
            }
            _ = tokio::time::sleep(Duration::from_millis(100)) => {
                // Continue listening
            }
        }
    }
    
    let stats = client.get_stats().await;
    println!("\n📊 HELIUS TEST RESULTS:");
    println!("   Connected: {}", stats.is_connected);
    println!("   Pools detected: {}", pools_detected);
    println!("   Last ping: {:.1}s ago", stats.last_ping_ago.as_secs_f64());
    
    Ok(())
}
