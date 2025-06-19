/// Jupiter Fallback Client - Copia exacta del cliente normal con cache
/// 
/// Para casos donde el ultra-fast client tenga problemas de conectividad

use anyhow::{Result, anyhow};
use reqwest::Client;
use serde_json::Value;
use std::collections::HashMap;
use std::sync::Arc;
use std::time::Duration;
use tracing::{info, warn, debug};
use tokio::sync::RwLock;

/// Cliente de respaldo que replica exactamente el comportamiento del cliente normal
#[derive(Debug, Clone)]
pub struct FallbackJupiterClient {
    http_client: Client,
    price_cache: Arc<RwLock<HashMap<String, (f64, std::time::Instant)>>>,
    cache_ttl: Duration,
}

impl FallbackJupiterClient {
    pub async fn new() -> Result<Self> {
        info!("🔄 Initializing Fallback Jupiter Client");
        
        // Usar exactamente la misma configuración que el cliente normal exitoso
        let http_client = Client::builder()
            .timeout(Duration::from_millis(1500))
            .connect_timeout(Duration::from_millis(300))
            .pool_idle_timeout(Duration::from_secs(60))
            .pool_max_idle_per_host(20)
            .tcp_keepalive(Duration::from_secs(60))
            .user_agent("SniperForge/0.1.0")
            .build()?;

        info!("✅ Fallback Jupiter Client ready");

        Ok(Self {
            http_client,
            price_cache: Arc::new(RwLock::new(HashMap::new())),
            cache_ttl: Duration::from_secs(5), // 5 second cache
        })
    }    pub async fn get_price(&self, token_mint: &str) -> Result<Option<f64>> {
        // 1. Check cache first
        {
            let cache = self.price_cache.read().await;
            if let Some((price, timestamp)) = cache.get(token_mint) {
                if timestamp.elapsed() < self.cache_ttl {
                    debug!("💾 Cache hit for {}: ${:.4}", token_mint, price);
                    return Ok(Some(*price));
                }
            }
        }

        // 2. Strategy 1: Try Jupiter price API v4
        debug!("🌐 Cache miss, trying price API v4 for {}", token_mint);
        match self.get_price_from_api_v4(token_mint).await {
            Ok(Some(price)) => {
                // Cache the retrieved price
                {
                    let mut cache = self.price_cache.write().await;
                    cache.insert(token_mint.to_string(), (price, std::time::Instant::now()));
                }
                return Ok(Some(price));
            },
            Ok(None) => debug!("⚠️ Price not found in API v4"),
            Err(e) => debug!("⚠️ Price API v4 failed: {}", e),
        }

        // 3. Strategy 2: For SOL, try quote-based price calculation
        if token_mint == "So11111111111111111111111111111111111111112" {
            debug!("🌐 Trying SOL price via USDC quote");
            match self.get_sol_price_via_usdc_quote().await {
                Ok(price) => {
                    debug!("✅ SOL price via quote: ${:.2}", price);
                    // Cache the retrieved price
                    {
                        let mut cache = self.price_cache.write().await;
                        cache.insert(token_mint.to_string(), (price, std::time::Instant::now()));
                    }
                    return Ok(Some(price));
                }
                Err(e) => debug!("⚠️ SOL price via quote failed: {}", e),
            }
        }

        debug!("⚠️ All fallback price strategies failed for token: {}", token_mint);
        Ok(None)
    }

    /// Get price from Jupiter price API v4 (same as standard client)
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
    }

    /// Get SOL price by getting a quote from SOL to USDC (fallback version)
    async fn get_sol_price_via_usdc_quote(&self) -> Result<f64> {
        debug!("💰 Getting SOL price via USDC quote (fallback)");
        
        // Get quote for 1 SOL to USDC using fallback client
        let quote_url = "https://quote-api.jup.ag/v6/quote";
        
        let mut url = reqwest::Url::parse(quote_url)?;
        url.query_pairs_mut()
            .append_pair("inputMint", "So11111111111111111111111111111111111111112") // SOL
            .append_pair("outputMint", "EPjFWdd5AufqSSqeM2qN1xzybapC8G4wEGGkZwyTDt1v") // USDC
            .append_pair("amount", "1000000000") // 1 SOL (9 decimals)
            .append_pair("slippageBps", "50"); // 0.5% slippage

        let response = self.http_client.get(url).send().await?;
        
        if response.status().is_success() {
            let quote: Value = response.json().await?;
            
            if let Some(out_amount) = quote.get("outAmount") {
                if let Some(out_amount_str) = out_amount.as_str() {
                    if let Ok(out_amount_num) = out_amount_str.parse::<u64>() {
                        // USDC has 6 decimals, so divide by 1_000_000
                        let usdc_amount = out_amount_num as f64 / 1_000_000.0;
                        debug!("✅ SOL price calculated: ${:.2}", usdc_amount);
                        return Ok(usdc_amount);
                    }
                }
            }
        }

        Err(anyhow!("Failed to calculate SOL price from quote"))
    }
}
