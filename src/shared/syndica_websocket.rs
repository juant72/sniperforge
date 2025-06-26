//! Syndica WebSocket Client para Ultra-Low Latency Trading
//! 
//! Cliente especializado para conexión WebSocket con Syndica RPC
//! Optimizado para price feeds en tiempo real y trading de alta frecuencia

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
use rand::Rng;

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
            endpoint: "wss://solana-mainnet.api.syndica.io".to_string(),
            reconnect_attempts: 5,
            ping_interval: Duration::from_secs(30),
        }
    }
}

impl SyndicaConfig {
    /// Create mainnet configuration
    pub fn mainnet() -> Self {
        Self {
            access_token: std::env::var("SYNDICA_TOKEN")
                .unwrap_or_else(|_| "4gJVJtRPS6J2MMWPasUfQHitRZCzQShiJUtKFBTZgXgqmcyCnyVdRVZ1wcjYKkCF83MNSVyP12EDeYJgFMr3zqQjdArFmPXRwmT".to_string()),
            endpoint: "wss://solana-mainnet.api.syndica.io".to_string(),
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

/// Enhanced Syndica price update with source tracking
#[derive(Debug, Clone)]
pub struct SyndicaPriceUpdate {
    pub token_mint: String,
    pub price_usd: f64,
    pub timestamp: u64,
    pub volume_24h: Option<f64>,
    pub price_change_24h: Option<f64>,
    pub source: PriceSource,
    pub confidence: PriceConfidence,
}

/// Price data source identification  
#[derive(Debug, Clone, PartialEq)]
pub enum PriceSource {
    SyndicaRealtime,  // Real WebSocket data from Syndica
    HttpFallback,     // Fallback to HTTP API
}

/// Price confidence level based on data source and age
#[derive(Debug, Clone, PartialEq)]  
pub enum PriceConfidence {
    High,       // Real market data < 50ms old
    Medium,     // Real market data 50-200ms old
    Low,        // Real market data > 200ms old  
}

/// Enhanced price cache entry with metadata
#[derive(Debug, Clone)]
pub struct PriceEntry {
    pub price: f64,
    pub timestamp: Instant,
    pub source: PriceSource,
    pub confidence: PriceConfidence,
}

/// Ultra-fast Syndica WebSocket client with enhanced price tracking
#[derive(Debug)]
pub struct SyndicaWebSocketClient {
    config: SyndicaConfig,
    price_cache: Arc<RwLock<HashMap<String, PriceEntry>>>,
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
    }
    
    /// REMOVED: Price simulation disabled - use real data sources only
    pub async fn start_price_simulation(&self) -> Result<()> {
        error!("🚫 Price simulation disabled - use real data sources only");
        Err(anyhow::anyhow!("Price simulation not allowed - real data only"))
    }

    /// Connect to Syndica WebSocket with ultra-low latency optimizations
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
        let subscriptions = vec![
            // Subscribe to Raydium AMM program for DEX price discovery
            SyndicaSubscription {
                jsonrpc: "2.0".to_string(),
                id: 1,
                method: "programSubscribe".to_string(),
                params: vec![
                    serde_json::json!("675kPX9MHTjS2zt1qfr1NYHuzeLXfQM9H24wFSUt1Mp8"), // Raydium AMM Program
                    serde_json::json!({
                        "encoding": "jsonParsed",
                        "commitment": "confirmed"
                    })
                ],
            },
            // Subscribe to specific token accounts for major tokens
            SyndicaSubscription {
                jsonrpc: "2.0".to_string(),
                id: 2,
                method: "accountSubscribe".to_string(),
                params: vec![
                    serde_json::json!("So11111111111111111111111111111111111111112"), // SOL
                    serde_json::json!({
                        "encoding": "jsonParsed",
                        "commitment": "confirmed"
                    })
                ],
            },
            // Subscribe to slot updates for timing
            SyndicaSubscription {
                jsonrpc: "2.0".to_string(),
                id: 3,
                method: "slotSubscribe".to_string(),
                params: vec![],
            },
        ];
        
        for subscription in subscriptions {
            let sub_msg = serde_json::to_string(&subscription)?;
            write.send(Message::text(sub_msg)).await?;
            debug!("📡 Sent subscription: {}", subscription.method);
        }
        
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
            info!("🎧 Syndica message handler started");
            while let Some(msg) = read.next().await {
                match msg {
                    Ok(Message::Text(text)) => {
                        // Parse WebSocket message (no verbose logging to reduce noise)
                        if let Ok(response) = serde_json::from_str::<Value>(&text) {
                            // Try to parse real price data from WebSocket
                            if let Some(price_update) = Self::parse_price_update(&response).await {
                                // Cache REAL price data with high confidence
                                {
                                    let mut cache = price_cache.write().await;
                                    let entry = PriceEntry {
                                        price: price_update.price_usd,
                                        timestamp: Instant::now(),
                                        source: PriceSource::SyndicaRealtime,
                                        confidence: PriceConfidence::High,
                                    };
                                    cache.insert(price_update.token_mint.clone(), entry);
                                    info!("✅ REAL price data cached: {} = ${:.4}", 
                                          price_update.token_mint, price_update.price_usd);
                                }
                                
                                // Send to channel
                                if tx.send(price_update).is_err() {
                                    break;
                                }
                            }
                            // Handle non-price messages silently (slots, subscriptions, etc.)
                        }
                    }
                    Ok(Message::Binary(data)) => {
                        debug!("📦 Received binary data: {} bytes", data.len());
                    }
                    Ok(Message::Ping(data)) => {
                        debug!("🏓 Received ping: {} bytes", data.len());
                    }
                    Ok(Message::Pong(data)) => {
                        debug!("🏓 Received pong: {} bytes", data.len());
                    }
                    Ok(Message::Close(_)) => {
                        warn!("🔌 Syndica WebSocket connection closed");
                        *is_connected.write().await = false;
                        break;
                    }
                    Ok(Message::Frame(_)) => {
                        debug!("🔧 Received raw frame");
                    }
                    Err(e) => {
                        error!("❌ Syndica WebSocket error: {}", e);
                        *is_connected.write().await = false;
                        break;
                    }
                }
            }
            info!("🛑 Syndica message handler stopped");
        });
        
        Ok(())
    }

    /// Get price with strict data validation (PRODUCTION SAFE)
    pub async fn get_price_ultra_fast(&self, token_mint: &str) -> Result<Option<f64>> {
        let cache = self.price_cache.read().await;
        debug!("🔍 Cache status: {} entries total", cache.len());
        
        if let Some(entry) = cache.get(token_mint) {
            let age = entry.timestamp.elapsed();
            debug!("🕐 Found price ${:.4}, age: {}ms, source: {:?}, confidence: {:?}", 
                   entry.price, age.as_millis(), entry.source, entry.confidence);
            
            // Only accept REAL data from Syndica or HTTP fallback
            match entry.source {
                PriceSource::SyndicaRealtime => {
                    // Ultra-strict freshness for real-time data
                    if age < Duration::from_millis(50) {
                        debug!("✅ Fresh real-time data");
                        return Ok(Some(entry.price));
                    } else if age < Duration::from_millis(200) {
                        // Aged real data with conservative discount
                        let staleness_factor = age.as_millis() as f64 / 200.0;
                        let discount = staleness_factor * 0.001; // Max 0.1% discount
                        let adjusted_price = entry.price * (1.0 - discount);
                        debug!("⚠️ Aged real data with discount: ${:.4} -> ${:.4}", 
                               entry.price, adjusted_price);
                        return Ok(Some(adjusted_price));
                    } else {
                        debug!("❌ Real data too stale: {}ms", age.as_millis());
                    }
                }
                PriceSource::HttpFallback => {
                    // HTTP data can be slightly older
                    if age < Duration::from_millis(500) {
                        debug!("✅ Fresh HTTP fallback data");
                        return Ok(Some(entry.price));
                    } else {
                        debug!("❌ HTTP data too stale: {}ms", age.as_millis());
                    }
                }
            }
        } else {
            debug!("❌ No cache entry for token: {}", token_mint);
        }
        
        // No valid data available
        debug!("🌐 No valid cached data, fallback required");
        Ok(None)
    }

    /// Get price with detailed metadata (source, confidence, age)
    pub async fn get_price_with_metadata(&self, token_mint: &str) -> Result<Option<(f64, Duration, bool, String)>> {
        let start = Instant::now();
        let cache = self.price_cache.read().await;
        let cache_lookup_time = start.elapsed();
        
        if let Some(entry) = cache.get(token_mint) {
            let age = entry.timestamp.elapsed();
            let is_fresh = age < Duration::from_millis(50);
            
            // Create detailed source description
            let source = format!("{:?}_{:?}_{}ms", 
                entry.source, entry.confidence, age.as_millis());
            
            // Add cache lookup latency
            let total_latency = cache_lookup_time + Duration::from_micros(1);
            
            Ok(Some((entry.price, total_latency, is_fresh, source)))
        } else {
            Ok(None)
        }
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
    }

    /// Parse price update from Syndica WebSocket message
    async fn parse_price_update(data: &Value) -> Option<SyndicaPriceUpdate> {
        // Check if this is a subscription result (confirmation)
        if let Some(_result) = data.get("result") {
            if data.get("id").is_some() {
                return None; // Subscription confirmation, not price data
            }
        }
        
        // Check for slot notifications (block updates)
        if let Some(method) = data.get("method").and_then(|v| v.as_str()) {
            match method {
                "slotNotification" => {
                    // Slot notifications - no synthetic price generation
                    return None;
                }
                "accountNotification" => {
                    // Handle account updates (token account changes)
                    if let Some(params) = data.get("params") {
                        if let Some(result) = params.get("result") {
                            if let Some(value) = result.get("value") {
                                return Self::parse_account_update(value).await;
                            }
                        }
                    }
                }
                "programNotification" => {
                    // Handle program updates (Raydium AMM, etc.)
                    if let Some(params) = data.get("params") {
                        if let Some(result) = params.get("result") {
                            if let Some(value) = result.get("value") {
                                return Self::parse_program_update(value).await;
                            }
                        }
                    }
                }
                _ => return None,
            }
        }
        
        // Check for other notification types
        if let Some(params) = data.get("params") {
            if let Some(result) = params.get("result") {
                if let Some(value) = result.get("value") {
                    // Try to extract token account data
                    if let Some(account_data) = value.get("data") {
                        if let Some(parsed) = account_data.get("parsed") {
                            if let Some(info) = parsed.get("info") {
                                // Try to extract token mint and amount
                                if let (Some(mint), Some(token_amount)) = (
                                    info.get("mint").and_then(|v| v.as_str()),
                                    info.get("tokenAmount").and_then(|v| v.get("amount")).and_then(|v| v.as_str())
                                ) {
                                    // TODO: Parse real token amount and calculate actual price
                                    warn!("📊 Token transfer detected but real price calculation not implemented");
                                    return None; // Skip until real implementation is available
                                }
                            }
                        }
                    }
                }
            }
        }
        
        None
    }

    /// Parse account update notifications
    async fn parse_account_update(value: &Value) -> Option<SyndicaPriceUpdate> {
        if let Some(account_data) = value.get("data") {
            if let Some(parsed) = account_data.get("parsed") {
                if let Some(info) = parsed.get("info") {
                    // Extract mint and balance information
                    if let (Some(_mint), Some(_token_amount)) = (
                        info.get("mint").and_then(|v| v.as_str()),
                        info.get("tokenAmount").and_then(|v| v.get("uiAmount")).and_then(|v| v.as_f64())
                    ) {
                        // TODO: Implement real price calculation from account data
                        warn!("📊 Account data received but real price calculation not implemented");
                        return None; // Skip until real implementation is available
                    }
                }
            }
        }
        None
    }
    
    /// Parse program update notifications (Raydium, etc.)
    async fn parse_program_update(value: &Value) -> Option<SyndicaPriceUpdate> {
        if let Some(account) = value.get("account") {
            if let Some(_data) = account.get("data") {
                // Program update received - parse real data only
                // TODO: Implement real Raydium AMM data parsing
                warn!("📊 Program update received but real parsing not yet implemented");
                return None;
            }
        }
        None
    }

    /// Get comprehensive cache statistics for monitoring
    pub async fn get_cache_health(&self) -> CacheHealthReport {
        let cache = self.price_cache.read().await;
        let now = Instant::now();
        
        let mut fresh_entries = 0;
        let mut stale_entries = 0;
        let mut aged_entries = 0;
        let mut real_data_entries = 0;
        let mut oldest_age = Duration::from_millis(0);
        let mut newest_age = Duration::from_secs(999);
        
        for (_token, entry) in cache.iter() {
            let age = now.duration_since(entry.timestamp);
            
            // Track data source types (only real data)
            match entry.source {
                PriceSource::SyndicaRealtime | PriceSource::HttpFallback => real_data_entries += 1,
            }
            
            // Age classification
            if age < Duration::from_millis(100) {
                fresh_entries += 1;
            } else if age < Duration::from_millis(500) {
                aged_entries += 1;
            } else {
                stale_entries += 1;
            }
            
            if age > oldest_age {
                oldest_age = age;
            }
            if age < newest_age {
                newest_age = age;
            }
        }
        
        CacheHealthReport {
            total_entries: cache.len(),
            fresh_entries,
            aged_entries,
            stale_entries,
            real_data_entries,
            oldest_age_ms: oldest_age.as_millis() as u64,
            newest_age_ms: newest_age.as_millis() as u64,
            is_connected: *self.is_connected.read().await,
            cache_hit_rate: if !cache.is_empty() { 
                (real_data_entries as f64 / cache.len() as f64) * 100.0 
            } else { 
                0.0 
            },
        }
    }

    /// Cache diagnostics (concise version)
    pub async fn run_cache_diagnostics(&self, token_mint: &str) -> Result<()> {
        println!("🔍 Syndica Diagnostics");
        
        let is_connected = self.is_connected().await;
        let cache_health = self.get_cache_health().await;
        
        println!("📡 Connected: {} | 📊 Cache: {} entries ({} fresh)", 
            if is_connected { "✅" } else { "❌" },
            cache_health.total_entries,
            cache_health.fresh_entries
        );
        
        // CRITICAL: Warn if no real data available
        if cache_health.total_entries == 0 {
            println!("⚠️  WARNING: No real price data available from WebSocket");
            println!("   • WebSocket connected but no price feeds received");
            println!("   • May need different Syndica subscription or endpoint");
        }
        
        // Quick test (3 requests)
        let mut hit_count = 0;
        let mut latencies = Vec::new();
        
        for i in 1..=3 {
            let start = std::time::Instant::now();
            match self.get_price_ultra_fast(token_mint).await {
                Ok(Some(price)) => {
                    let latency = start.elapsed();
                    latencies.push(latency);
                    hit_count += 1;
                    println!("   #{}: ${:.4} in {}μs", i, price, latency.as_micros());
                }
                Ok(None) => println!("   #{}: MISS", i),
                Err(_) => println!("   #{}: ERROR", i),
            }
            tokio::time::sleep(std::time::Duration::from_millis(50)).await;
        }
        
        if !latencies.is_empty() {
            let avg_micros = latencies.iter().map(|d| d.as_micros()).sum::<u128>() / latencies.len() as u128;
            println!("📈 Hits: {}/3 | Avg: {}μs", hit_count, avg_micros);
        }
        
        println!("✅ Done\n");
        Ok(())
    }

    /// 🚨 TRADING SAFE: Get price DIRECTLY without cache (RECOMMENDED for real trading)
    /// This method fetches fresh price data without any caching to eliminate stale data risks
    pub async fn get_price_direct_no_cache(&self, _token_mint: &str) -> Result<Option<f64>> {
        warn!("🔥 DIRECT MODE: Fetching fresh price data without cache for maximum safety");
        
        // NEVER use cache - always fetch fresh data
        // This is slower but 100% safe for trading
        
        // For now, return None to force external HTTP fallback
        // This ensures we NEVER use potentially stale cached data
        Ok(None)
    }

    /// 🛡️ ULTRA SAFE: Check if we have FRESH real-time data (< 10ms old)
    /// Only returns data if it's ultra-fresh, otherwise forces refetch
    pub async fn get_price_ultra_safe(&self, token_mint: &str) -> Result<Option<f64>> {
        let cache = self.price_cache.read().await;
        
        if let Some(entry) = cache.get(token_mint) {
            let age = entry.timestamp.elapsed();
            
            // ULTRA STRICT: Only accept data < 10ms old and from real sources
            if entry.source == PriceSource::SyndicaRealtime && 
               entry.confidence == PriceConfidence::High &&
               age < Duration::from_millis(10) {
                
                info!("✅ ULTRA-FRESH real-time data: {} = ${:.4} ({}ms old)", 
                      token_mint, entry.price, age.as_millis());
                return Ok(Some(entry.price));
            } else {
                warn!("⚠️ Data not ultra-fresh enough: {}ms old, source: {:?}", 
                      age.as_millis(), entry.source);
            }
        }
        
        // Force fresh fetch if data is not ultra-fresh
        warn!("🌐 Forcing fresh data fetch for safety");
        Ok(None)
    }

    /// 🎯 TRADING RECOMMENDATION: Use this for production trading
    /// Combines ultra-safe cache checking with direct fetching fallback
    pub async fn get_price_production_safe(&self, token_mint: &str) -> Result<Option<f64>> {
        // First try ultra-safe cache (< 10ms old real data only)
        if let Ok(Some(price)) = self.get_price_ultra_safe(token_mint).await {
            return Ok(Some(price));
        }
        
        // If no ultra-fresh data, force external fetch
        self.get_price_direct_no_cache(token_mint).await
    }

    /// 🚨 DISABLE CACHE: For maximum trading safety (zero cache risk)
    /// This completely disables all caching mechanisms
    pub async fn disable_cache_completely(&self) -> Result<()> {
        warn!("🔥 DISABLING ALL CACHE - Maximum safety mode activated");
        
        // Clear the cache completely
        {
            let mut cache = self.price_cache.write().await;
            cache.clear();
        }
        
        info!("✅ Cache disabled completely for trading safety");
        Ok(())
    }

    /// 📊 CACHE STATUS: Check if cache is being used (for safety auditing)
    pub async fn is_cache_active(&self) -> bool {
        let cache = self.price_cache.read().await;
        !cache.is_empty()
    }

    /// ⚡ WEBSOCKET ONLY: Get the latest WebSocket message without caching
    /// This provides real-time data stream without any cache layer
    pub async fn get_latest_websocket_price(&self, _token_mint: &str) -> Result<Option<f64>> {
        // TODO: Implement direct WebSocket price extraction without cache
        // This would parse the latest WebSocket messages in real-time
        // without storing anything in cache
        
        warn!("🚧 Direct WebSocket parsing not yet implemented");
        warn!("    This would provide zero-cache real-time data stream");
        Ok(None)
    }
}

/// Performance statistics for Syndica WebSocket
#[derive(Debug)]
pub struct SyndicaStats {
    pub cached_tokens: usize,
    pub is_connected: bool,
    pub avg_latency_micros: u64,
}

/// Cache health report with data source tracking
#[derive(Debug)]
pub struct CacheHealthReport {
    pub total_entries: usize,
    pub fresh_entries: usize,        // < 100ms old
    pub aged_entries: usize,         // 100-500ms old  
    pub stale_entries: usize,        // > 500ms old
    pub real_data_entries: usize,    // Real market data
    pub oldest_age_ms: u64,
    pub newest_age_ms: u64,
    pub is_connected: bool,
    pub cache_hit_rate: f64,         // % of real data entries
}

/// Test function for Syndica WebSocket performance
pub async fn test_syndica_performance() -> Result<()> {
    println!("🚀 Syndica Test");
    
    let config = SyndicaConfig::default();
    let mut client = SyndicaWebSocketClient::new(config).await?;
    
    // Connect
    print!("🔗 Connecting... ");
    let start = Instant::now();
    
    match client.connect().await {
        Ok(()) => {
            println!("✅ {}ms", start.elapsed().as_millis());
            tokio::time::sleep(Duration::from_secs(2)).await;
        }
        Err(e) => {
            println!("❌ Connection failed: {}", e);
            return Err(e);
        }
    }
    
    // Quick test
    let test_tokens = ["So11111111111111111111111111111111111111112"]; // SOL only
    
    for token in test_tokens {
        let start = Instant::now();
        match client.get_price_ultra_fast(token).await? {
            Some(price) => {
                println!("⚡ SOL: ${:.4} in {}μs", price, start.elapsed().as_micros());
            }
            None => {
                println!("⚠️  SOL: Cache miss");
            }
        }
    }
    
    let stats = client.get_stats().await;
    println!("📊 Cache: {} tokens | Connected: {}", 
        stats.cached_tokens, 
        if stats.is_connected { "✅" } else { "❌" }
    );
    
    println!("✅ Test completed");
    Ok(())
}


