use crate::{
    config::SimpleConfig,
    types::ApiResult as Result,
    apis::rate_limiter::RateLimiter,
};
use reqwest::Client;
use serde::{Deserialize, Serialize};
use std::{
    collections::HashMap,
    sync::Arc,
    time::{Duration, Instant},
};
use tokio::sync::RwLock;
use tracing::debug;

/// DexScreener API client for market data
#[derive(Clone)]
pub struct DexScreenerClient {
    client: Client,
    base_url: String,
    rate_limiter: Arc<RateLimiter>,
    cache: Arc<RwLock<DataCache>>,
}

impl DexScreenerClient {
    /// Create a new DexScreener client
    pub fn new(config: &SimpleConfig) -> Self {
        let client = Client::builder()
            .timeout(Duration::from_secs(30))
            .user_agent("SniperForge/1.0")
            .build()
            .expect("Failed to create HTTP client");
        
        let rate_limiter = Arc::new(RateLimiter::new(
            config.max_requests_per_second,
            Duration::from_millis(config.cooldown_period_ms),
        ));
        
        Self {
            client,
            base_url: config.dexscreener_base_url.clone(),
            rate_limiter,
            cache: Arc::new(RwLock::new(DataCache::new())),
        }
    }
    
    /// Get token information by address
    pub async fn get_token_info(&self, token_address: &str) -> Result<TokenInfo> {
        // Check cache first
        if let Some(cached) = self.check_cache("token_info", token_address).await {
            if let Ok(token_info) = serde_json::from_str::<TokenInfo>(&cached) {
                return Ok(token_info);
            }
        }
        
        self.rate_limiter.wait().await;
        
        let url = format!("{}/latest/dex/tokens/{}", self.base_url, token_address);
        
        debug!("Fetching token info from: {}", url);
        
        let response = self.client
            .get(&url)
            .send()
            .await
            .map_err(|e| format!("Request failed: {}", e))?;
        
        if !response.status().is_success() {
            return Err(format!("API request failed with status: {}", response.status()));
        }
        
        let api_response: DexScreenerTokenResponse = response
            .json()
            .await
            .map_err(|e| format!("Failed to parse response: {}", e))?;
        
        if let Some(pair) = api_response.pairs.first() {
            let token_info = TokenInfo {
                address: token_address.to_string(),
                name: pair.base_token.name.clone(),
                symbol: pair.base_token.symbol.clone(),
                price_usd: pair.price_usd.parse().unwrap_or(0.0),
                price_change_24h: pair.price_change.h24,
                volume_24h: pair.volume.h24,
                liquidity: pair.liquidity.usd,
                market_cap: pair.fdv.unwrap_or(0.0),
                last_updated: Some(Instant::now()),
            };
            
            // Cache the result
            self.update_cache("token_info", token_address, &serde_json::to_string(&token_info).unwrap()).await;
            
            Ok(token_info)
        } else {
            Err("No pairs found for token".to_string())
        }
    }
    
    /// Get trending tokens
    pub async fn get_trending_tokens(&self, chain: &str) -> Result<Vec<TrendingToken>> {
        let cache_key = format!("trending_{}", chain);
        
        // Check cache first (trending data changes frequently, so shorter cache)
        if let Some(cached) = self.check_cache_with_ttl("trending", &cache_key, Duration::from_secs(60)).await {
            if let Ok(tokens) = serde_json::from_str::<Vec<TrendingToken>>(&cached) {
                return Ok(tokens);
            }
        }
        
        self.rate_limiter.wait().await;
        
        let url = format!("{}/latest/dex/search/?q={}", self.base_url, chain);
        
        debug!("Fetching trending tokens from: {}", url);
        
        let response = self.client
            .get(&url)
            .send()
            .await
            .map_err(|e| format!("Request failed: {}", e))?;
        
        if !response.status().is_success() {
            return Err(format!("API request failed with status: {}", response.status()));
        }
        
        let api_response: DexScreenerSearchResponse = response
            .json()
            .await
            .map_err(|e| format!("Failed to parse response: {}", e))?;
        
        let trending_tokens: Vec<TrendingToken> = api_response.pairs
            .into_iter()
            .take(20) // Limit to top 20
            .map(|pair| TrendingToken {
                address: pair.base_token.address,
                symbol: pair.base_token.symbol,
                name: pair.base_token.name,
                price_usd: pair.price_usd.parse().unwrap_or(0.0),
                price_change_24h: pair.price_change.h24,
                volume_24h: pair.volume.h24,
                liquidity: pair.liquidity.usd,
                dex: pair.dex_id,
            })
            .collect();
        
        // Cache the result
        self.update_cache("trending", &cache_key, &serde_json::to_string(&trending_tokens).unwrap()).await;
        
        Ok(trending_tokens)
    }
    
    /// Get pair information
    pub async fn get_pair_info(&self, pair_address: &str) -> Result<PairInfo> {
        // Check cache first
        if let Some(cached) = self.check_cache("pair_info", pair_address).await {
            if let Ok(pair_info) = serde_json::from_str::<PairInfo>(&cached) {
                return Ok(pair_info);
            }
        }
        
        self.rate_limiter.wait().await;
        
        let url = format!("{}/latest/dex/pairs/{}", self.base_url, pair_address);
        
        debug!("Fetching pair info from: {}", url);
        
        let response = self.client
            .get(&url)
            .send()
            .await
            .map_err(|e| format!("Request failed: {}", e))?;
        
        if !response.status().is_success() {
            return Err(format!("API request failed with status: {}", response.status()));
        }
        
        let api_response: DexScreenerPairResponse = response
            .json()
            .await
            .map_err(|e| format!("Failed to parse response: {}", e))?;
        
        if let Some(pair) = api_response.pair {
            let pair_info = PairInfo {
                address: pair_address.to_string(),
                dex: pair.dex_id,
                base_token: TokenBasic {
                    address: pair.base_token.address,
                    symbol: pair.base_token.symbol,
                    name: pair.base_token.name,
                },
                quote_token: TokenBasic {
                    address: pair.quote_token.address,
                    symbol: pair.quote_token.symbol,
                    name: pair.quote_token.name,
                },
                price_usd: pair.price_usd.parse().unwrap_or(0.0),
                liquidity: pair.liquidity.usd,
                volume_24h: pair.volume.h24,
                last_updated: Some(Instant::now()),
            };
            
            // Cache the result
            self.update_cache("pair_info", pair_address, &serde_json::to_string(&pair_info).unwrap()).await;
            
            Ok(pair_info)
        } else {
            Err("Pair not found".to_string())
        }
    }
    
    /// Check cache for data
    async fn check_cache(&self, category: &str, key: &str) -> Option<String> {
        self.check_cache_with_ttl(category, key, Duration::from_secs(300)).await
    }
    
    /// Check cache with custom TTL
    async fn check_cache_with_ttl(&self, category: &str, key: &str, ttl: Duration) -> Option<String> {
        let cache = self.cache.read().await;
        let cache_key = format!("{}:{}", category, key);
        
        if let Some(entry) = cache.data.get(&cache_key) {
            if entry.timestamp.elapsed() < ttl {
                debug!("Cache hit for: {}", cache_key);
                return Some(entry.data.clone());
            }
        }
        
        None
    }
    
    /// Update cache with data
    async fn update_cache(&self, category: &str, key: &str, data: &str) {
        let mut cache = self.cache.write().await;
        let cache_key = format!("{}:{}", category, key);
        
        cache.data.insert(cache_key.clone(), CacheEntry {
            data: data.to_string(),
            timestamp: Instant::now(),
        });
        
        // Clean old entries if cache is too large
        if cache.data.len() > 1000 {
            cache.cleanup();
        }
        
        debug!("Cached data for: {}", cache_key);
    }
    
    /// Get cache statistics
    pub async fn get_cache_stats(&self) -> CacheStats {
        let cache = self.cache.read().await;
        CacheStats {
            entries: cache.data.len(),
            memory_usage: cache.estimate_memory_usage(),
        }
    }
}

/// Data cache for API responses
#[derive(Debug)]
struct DataCache {
    data: HashMap<String, CacheEntry>,
}

impl DataCache {
    fn new() -> Self {
        Self {
            data: HashMap::new(),
        }
    }
    
    fn cleanup(&mut self) {
        let cutoff = Instant::now() - Duration::from_secs(3600); // 1 hour
        self.data.retain(|_, entry| entry.timestamp > cutoff);
    }
    
    fn estimate_memory_usage(&self) -> usize {
        self.data.len() * 256 // Rough estimate
    }
}

#[derive(Debug, Clone)]
struct CacheEntry {
    data: String,
    timestamp: Instant,
}

/// Cache statistics
#[derive(Debug, Clone)]
pub struct CacheStats {
    pub entries: usize,
    pub memory_usage: usize,
}

/// Token information from DexScreener
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TokenInfo {
    pub address: String,
    pub name: String,
    pub symbol: String,
    pub price_usd: f64,
    pub price_change_24h: f64,
    pub volume_24h: f64,
    pub liquidity: f64,
    pub market_cap: f64,
    #[serde(skip)]
    pub last_updated: Option<Instant>,
}

impl Default for TokenInfo {
    fn default() -> Self {
        Self {
            address: String::new(),
            name: String::new(),
            symbol: String::new(),
            price_usd: 0.0,
            price_change_24h: 0.0,
            volume_24h: 0.0,
            liquidity: 0.0,
            market_cap: 0.0,
            last_updated: None,
        }
    }
}

/// Trending token information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TrendingToken {
    pub address: String,
    pub symbol: String,
    pub name: String,
    pub price_usd: f64,
    pub price_change_24h: f64,
    pub volume_24h: f64,
    pub liquidity: f64,
    pub dex: String,
}

/// Pair information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PairInfo {
    pub address: String,
    pub dex: String,
    pub base_token: TokenBasic,
    pub quote_token: TokenBasic,
    pub price_usd: f64,
    pub liquidity: f64,
    pub volume_24h: f64,
    #[serde(skip)]
    pub last_updated: Option<Instant>,
}

impl Default for PairInfo {
    fn default() -> Self {
        Self {
            address: String::new(),
            dex: String::new(),
            base_token: TokenBasic::default(),
            quote_token: TokenBasic::default(),
            price_usd: 0.0,
            liquidity: 0.0,
            volume_24h: 0.0,
            last_updated: None,
        }
    }
}

/// Basic token information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TokenBasic {
    pub address: String,
    pub symbol: String,
    pub name: String,
}

impl Default for TokenBasic {
    fn default() -> Self {
        Self {
            address: String::new(),
            symbol: String::new(),
            name: String::new(),
        }
    }
}

// API Response Types
#[derive(Debug, Deserialize)]
struct DexScreenerTokenResponse {
    pairs: Vec<DexScreenerPair>,
}

#[derive(Debug, Deserialize)]
struct DexScreenerSearchResponse {
    pairs: Vec<DexScreenerPair>,
}

#[derive(Debug, Deserialize)]
struct DexScreenerPairResponse {
    pair: Option<DexScreenerPair>,
}

#[derive(Debug, Deserialize)]
struct DexScreenerPair {
    #[serde(rename = "chainId")]
    chain_id: String,
    #[serde(rename = "dexId")]
    dex_id: String,
    url: String,
    #[serde(rename = "pairAddress")]
    pair_address: String,
    #[serde(rename = "baseToken")]
    base_token: DexScreenerToken,
    #[serde(rename = "quoteToken")]
    quote_token: DexScreenerToken,
    #[serde(rename = "priceNative")]
    price_native: String,
    #[serde(rename = "priceUsd")]
    price_usd: String,
    txns: DexScreenerTxns,
    volume: DexScreenerVolume,
    #[serde(rename = "priceChange")]
    price_change: DexScreenerPriceChange,
    liquidity: DexScreenerLiquidity,
    fdv: Option<f64>,
}

#[derive(Debug, Deserialize)]
struct DexScreenerToken {
    address: String,
    name: String,
    symbol: String,
}

#[derive(Debug, Deserialize)]
struct DexScreenerTxns {
    h24: DexScreenerTxnCount,
}

#[derive(Debug, Deserialize)]
struct DexScreenerTxnCount {
    buys: u64,
    sells: u64,
}

#[derive(Debug, Deserialize)]
struct DexScreenerVolume {
    h24: f64,
}

#[derive(Debug, Deserialize)]
struct DexScreenerPriceChange {
    h24: f64,
}

#[derive(Debug, Deserialize)]
struct DexScreenerLiquidity {
    usd: f64,
}

#[cfg(test)]
mod tests {
    use super::*;
    
    fn create_test_config() -> SimpleConfig {
        SimpleConfig {
            solana_rpc_url: "test".to_string(),
            solana_ws_url: "test".to_string(),
            max_slippage: 0.005,
            min_profit_threshold: 0.001,
            max_position_size: 0.1,
            private_key_path: "test".to_string(),
            enable_simulation: true,
            log_level: "info".to_string(),
            dexscreener_base_url: "https://api.dexscreener.com".to_string(),
            max_requests_per_second: 10,
            cooldown_period_ms: 100,
            max_history_size: 1000,
        }
    }
    
    #[tokio::test]
    async fn test_dexscreener_client_creation() {
        let config = create_test_config();
        let client = DexScreenerClient::new(&config);
        
        let stats = client.get_cache_stats().await;
        assert_eq!(stats.entries, 0);
    }
    
    #[tokio::test]
    async fn test_cache_functionality() {
        let config = create_test_config();
        let client = DexScreenerClient::new(&config);
        
        // Test cache update and retrieval
        client.update_cache("test", "key1", "data1").await;
        let cached = client.check_cache("test", "key1").await;
        
        assert_eq!(cached, Some("data1".to_string()));
    }
}
