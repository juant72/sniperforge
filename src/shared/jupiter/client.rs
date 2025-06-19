/// Jupiter API Client
/// 
/// Low-level HTTP client for Jupiter's REST API
/// Handles authentication, retries, and rate limiting

use anyhow::{Result, anyhow};
use reqwest::{Client, Response};
use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::time::Duration;
use tracing::{info, warn, error, debug};
use url::Url;
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;

use super::JupiterConfig;

/// HTTP client for Jupiter API
#[derive(Debug, Clone)]
pub struct JupiterClient {
    config: JupiterConfig,
    http_client: Client,
    base_url: Url,
    price_cache: PriceCache,
}

impl JupiterClient {    /// Create new Jupiter client
    pub async fn new(config: &JupiterConfig) -> Result<Self> {
        debug!("🌐 Setting up Jupiter HTTP client");        // Build HTTP client with aggressive optimizations for trading speed
        let http_client = Client::builder()
            .timeout(Duration::from_millis(1500)) // 1.5s max for ultra-fast trading
            .connect_timeout(Duration::from_millis(300)) // 300ms connect
            .pool_idle_timeout(Duration::from_secs(60)) // Keep connections alive longer
            .pool_max_idle_per_host(20) // More connection pooling
            .tcp_keepalive(Duration::from_secs(60)) // TCP keep-alive
            .user_agent("SniperForge/0.1.0")
            .build()?;

        let base_url = Url::parse(&config.api_base_url)?;

        let client = Self {
            config: config.clone(),
            http_client,
            base_url,
            price_cache: PriceCache::new(), // 5 minutes TTL for price cache
        };

        // No health check during initialization for speed
        // Connection will be tested on first real request
        
        info!("✅ Jupiter client initialized");
        Ok(client)
    }

    /// Health check endpoint
    pub async fn health_check(&self) -> Result<()> {
        debug!("🏥 Jupiter health check");
        
        let url = self.base_url.join("/health")?;
        
        let response = self.http_client
            .get(url)
            .send()
            .await?;

        if response.status().is_success() {
            debug!("✅ Jupiter API healthy");
            Ok(())
        } else {
            Err(anyhow!("Jupiter API health check failed: {}", response.status()))
        }
    }    /// Get quote for token swap
    pub async fn get_quote(
        &self,
        input_mint: &str,
        output_mint: &str,
        amount: u64,
        slippage_bps: Option<u16>,
    ) -> Result<super::types::JupiterQuote> {
        debug!("💰 Getting Jupiter quote: {} -> {} ({})", 
               input_mint, output_mint, amount);

        let mut url = self.base_url.join("/quote")?;
        
        // Add query parameters
        url.query_pairs_mut()
            .append_pair("inputMint", input_mint)
            .append_pair("outputMint", output_mint)
            .append_pair("amount", &amount.to_string())
            .append_pair("slippageBps", &slippage_bps.unwrap_or(self.config.slippage_bps).to_string());

        let response = self.execute_with_retry(url).await?;
        let quote: super::types::JupiterQuote = response.json().await?;

        debug!("✅ Quote received");
        Ok(quote)
    }

    /// Get swap transaction
    pub async fn get_swap_transaction(
        &self,
        quote: &Value,
        user_public_key: &str,
    ) -> Result<Value> {
        debug!("🔄 Getting swap transaction for user: {}", user_public_key);

        let url = self.base_url.join("/swap")?;

        let body = serde_json::json!({
            "quoteResponse": quote,
            "userPublicKey": user_public_key,
            "wrapAndUnwrapSol": true,
            "computeUnitPriceMicroLamports": "auto"
        });

        let response = self.http_client
            .post(url)
            .json(&body)
            .send()
            .await?;

        if response.status().is_success() {
            let swap_transaction: Value = response.json().await?;
            debug!("✅ Swap transaction received");
            Ok(swap_transaction)
        } else {
            let error_text = response.text().await?;
            Err(anyhow!("Failed to get swap transaction: {}", error_text))
        }
    }

    /// Get list of supported tokens
    pub async fn get_tokens(&self) -> Result<Vec<Value>> {
        debug!("📋 Getting supported tokens list");

        let url = self.base_url.join("/tokens")?;
        let response = self.execute_with_retry(url).await?;
        let tokens: Vec<Value> = response.json().await?;

        info!("✅ Retrieved {} supported tokens", tokens.len());
        Ok(tokens)
    }    /// Get price for a specific token with fallback strategies
    pub async fn get_price(&self, token_mint: &str) -> Result<Option<f64>> {
        debug!("💵 Getting price for token: {}", token_mint);

        // Check cache first
        if let Some(cached_price) = self.price_cache.get(token_mint).await {
            debug!("✅ Price found in cache: ${}", cached_price);
            return Ok(Some(cached_price));
        }        // Strategy 1: Try Jupiter price API v4
        match self.get_price_from_api_v4(token_mint).await {
            Ok(Some(price)) => {
                // Cache the retrieved price
                self.price_cache.set(token_mint, price).await;
                return Ok(Some(price));
            },
            Ok(None) => debug!("⚠️ Price not found in API v4"),
            Err(e) => debug!("⚠️ Price API v4 failed: {}", e),
        }

        // Strategy 2: For SOL, try quote-based price calculation
        if token_mint == "So11111111111111111111111111111111111111112" {
            match self.get_sol_price_via_usdc_quote().await {
                Ok(price) => {
                    debug!("✅ SOL price via quote: ${:.2}", price);
                    // Cache the retrieved price
                    self.price_cache.set(token_mint, price).await;
                    return Ok(Some(price));
                }
                Err(e) => debug!("⚠️ SOL price via quote failed: {}", e),
            }
        }

        debug!("⚠️ All price strategies failed for token: {}", token_mint);
        Ok(None)
    }

    /// Get price from Jupiter price API v4
    async fn get_price_from_api_v4(&self, token_mint: &str) -> Result<Option<f64>> {
        let url = format!("https://price.jup.ag/v4/price?ids={}", token_mint);
        
        let response = self.http_client.get(&url).send().await?;
        
        if response.status().is_success() {
            let price_data: Value = response.json().await?;
            
            if let Some(data) = price_data.get("data") {
                if let Some(token_data) = data.get(token_mint) {
                    if let Some(price) = token_data.get("price") {
                        if let Some(price_num) = price.as_f64() {
                            debug!("✅ Price from API v4: ${}", price_num);
                            return Ok(Some(price_num));
                        }
                    }
                }
            }
        }
        
        Ok(None)
    }    /// Get SOL price by getting a quote from SOL to USDC
    async fn get_sol_price_via_usdc_quote(&self) -> Result<f64> {
        debug!("💰 Getting SOL price via USDC quote");
        
        // Get quote for 1 SOL to USDC
        let quote = self.get_quote(
            "So11111111111111111111111111111111111111112", // SOL
            "EPjFWdd5AufqSSqeM2qN1xzybapC8G4wEGGkZwyTDt1v", // USDC
            1_000_000_000, // 1 SOL (9 decimals)
            Some(50), // 0.5% slippage
        ).await?;

        if let Ok(out_amount) = quote.out_amount.parse::<u64>() {
            // USDC has 6 decimals, so divide by 1_000_000
            let usdc_amount = out_amount as f64 / 1_000_000.0;
            return Ok(usdc_amount);
        }

        Err(anyhow::anyhow!("Failed to calculate SOL price from quote"))
    }    /// Execute HTTP request with retry logic
    async fn execute_with_retry(&self, url: Url) -> Result<Response> {
        let mut attempts = 0;
        let max_attempts = self.config.max_retries;

        loop {
            attempts += 1;
            
            match self.http_client.get(url.clone()).send().await {
                Ok(response) => {
                    if response.status().is_success() {
                        return Ok(response);
                    } else if response.status().is_server_error() && attempts < max_attempts {
                        warn!("Server error (attempt {}/{}): {}", 
                              attempts, max_attempts, response.status());
                        // Ultra-fast retry for trading: 50ms base, max 200ms
                        tokio::time::sleep(Duration::from_millis(50 * attempts as u64)).await;
                        continue;
                    } else {
                        let status = response.status();
                        let error_text = response.text().await.unwrap_or_default();
                        return Err(anyhow!("HTTP error {}: {}", status, error_text));
                    }
                }
                Err(e) if attempts < max_attempts => {
                    warn!("Request failed (attempt {}/{}): {}", attempts, max_attempts, e);
                    // Ultra-fast retry: 50ms, 100ms max
                    tokio::time::sleep(Duration::from_millis(50 * attempts as u64)).await;
                    continue;
                }
                Err(e) => {
                    return Err(anyhow!("Request failed after {} attempts: {}", max_attempts, e));
                }
            }
        }
    }
}

/// Price cache for ultra-fast lookups
#[derive(Debug, Clone)]
struct PriceCache {
    cache: Arc<RwLock<HashMap<String, (f64, std::time::Instant)>>>,
    ttl: Duration,
}

impl PriceCache {
    fn new() -> Self {
        Self {
            cache: Arc::new(RwLock::new(HashMap::new())),
            ttl: Duration::from_secs(5), // 5 second TTL for trading
        }
    }

    async fn get(&self, token: &str) -> Option<f64> {
        let cache = self.cache.read().await;
        if let Some((price, timestamp)) = cache.get(token) {
            if timestamp.elapsed() < self.ttl {
                debug!("💾 Cache hit for {}: ${:.6}", token, price);
                return Some(*price);
            }
        }
        None
    }

    async fn set(&self, token: &str, price: f64) {
        let mut cache = self.cache.write().await;
        cache.insert(token.to_string(), (price, std::time::Instant::now()));
        debug!("💾 Cached price for {}: ${:.6}", token, price);
    }

    async fn clear(&self) {
        let mut cache = self.cache.write().await;
        cache.clear();
        debug!("💾 Price cache cleared");
    }

    async fn size(&self) -> usize {
        let cache = self.cache.read().await;
        cache.len()
    }

    async fn cleanup_expired(&self) {
        let mut cache = self.cache.write().await;
        cache.retain(|_, (_, timestamp)| timestamp.elapsed() < self.ttl);
    }
}
