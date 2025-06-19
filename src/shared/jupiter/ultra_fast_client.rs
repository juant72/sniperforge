/// Ultra-Fast Jupiter Client para Trading de Alta Frecuencia
/// 
/// Cliente especializado con todas las optimizaciones de velocidad aplicadas

use anyhow::Result;
use reqwest::Client;
use std::time::Duration;
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;
use tracing::{debug, info, warn};
use url::Url;
use futures;
use serde_json;

use super::{JupiterConfig, types::JupiterQuote};

/// Cache ultra-rápido para precios
#[derive(Debug, Clone)]
struct UltraFastPriceCache {
    cache: Arc<RwLock<HashMap<String, (f64, std::time::Instant)>>>,
    ttl: Duration,
}

impl UltraFastPriceCache {    fn new() -> Self {
        Self {
            cache: Arc::new(RwLock::new(HashMap::new())),
            ttl: Duration::from_secs(10), // Cache agresivo de 10 segundos para trading
        }
    }
    
    async fn get_cached_price(&self, mint: &str) -> Option<f64> {
        let cache = self.cache.read().await;
        if let Some((price, timestamp)) = cache.get(mint) {
            if timestamp.elapsed() < self.ttl {
                debug!("🚀 Cache hit for {}: ${:.4}", mint, price);
                return Some(*price);
            } else {
                debug!("⏰ Cache expired for {}", mint);
            }
        }
        None
    }
    
    async fn cache_price(&self, mint: String, price: f64) {
        let mut cache = self.cache.write().await;
        cache.insert(mint, (price, std::time::Instant::now()));
        debug!("💾 Cached price: ${:.4}", price);
    }
}

/// Cliente Jupiter ultra-optimizado
#[derive(Debug)]
pub struct UltraFastJupiterClient {
    http_client: Client,
    price_cache: UltraFastPriceCache,
}

impl UltraFastJupiterClient {
    pub async fn new() -> Result<Self> {
        info!("⚡ Initializing Ultra-Fast Jupiter Client");
          // Cliente HTTP con optimizaciones balanceadas para velocidad y confiabilidad
        let http_client = Client::builder()
            .timeout(Duration::from_millis(2000)) // 2s timeout (más conservador pero confiable)
            .connect_timeout(Duration::from_millis(500)) // 500ms connect (más realista)
            .pool_idle_timeout(Duration::from_secs(120)) // Keep-alive muy largo
            .pool_max_idle_per_host(50) // Pool grande
            .tcp_keepalive(Duration::from_secs(30))
            .user_agent("SniperForge-UltraFast/1.0")
            .build()?;

        info!("✅ Ultra-Fast Jupiter Client ready");

        Ok(Self {
            http_client,
            price_cache: UltraFastPriceCache::new(),
        })
    }    /// Get price con optimizaciones extremas y fallback strategies
    pub async fn get_price_ultra_fast(&self, token_mint: &str) -> Result<Option<f64>> {
        let start = std::time::Instant::now();

        // 1. Check cache primero (sub-millisecond)
        if let Some(price) = self.price_cache.get_cached_price(token_mint).await {
            debug!("⚡ Ultra-fast cache response: {}μs", start.elapsed().as_micros());
            return Ok(Some(price));
        }

        // 2. Strategy 1: Try Jupiter price API v4
        debug!("🌐 Cache miss, trying price API v4 for {}", token_mint);
        match self.get_price_from_api_v4(token_mint).await {
            Ok(Some(price)) => {
                // Cache asíncrono (no bloquear respuesta)
                self.price_cache.cache_price(token_mint.to_string(), price).await;
                debug!("🚀 Total ultra-fast response: {}ms", start.elapsed().as_millis());
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
                    self.price_cache.cache_price(token_mint.to_string(), price).await;
                    debug!("🚀 Total ultra-fast response with fallback: {}ms", start.elapsed().as_millis());
                    return Ok(Some(price));
                }
                Err(e) => debug!("⚠️ SOL price via quote failed: {}", e),
            }
        }

        debug!("⚠️ All ultra-fast price strategies failed for token: {}", token_mint);
        Ok(None)
    }

    /// Get price from Jupiter price API v4 (same as standard client)
    async fn get_price_from_api_v4(&self, token_mint: &str) -> Result<Option<f64>> {
        let url = format!("https://price.jup.ag/v4/price?ids={}", token_mint);
        debug!("🌐 Making request to: {}", url);

        let api_start = std::time::Instant::now();
        
        let response = self.http_client.get(&url).send().await?;
        
        if response.status().is_success() {
            let price_data: serde_json::Value = response.json().await?;
            
            if let Some(data) = price_data.get("data") {
                if let Some(token_data) = data.get(token_mint) {
                    if let Some(price) = token_data.get("price") {
                        if let Some(price_num) = price.as_f64() {
                            let api_time = api_start.elapsed();
                            debug!("✅ Price from API v4: ${} in {}ms", price_num, api_time.as_millis());
                            return Ok(Some(price_num));
                        }
                    }
                }
            }
        }
        
        Ok(None)
    }

    /// Get SOL price by getting a quote from SOL to USDC (ultra-fast version)
    async fn get_sol_price_via_usdc_quote(&self) -> Result<f64> {
        debug!("💰 Getting SOL price via USDC quote (ultra-fast)");
        
        // Get quote for 1 SOL to USDC using ultra-fast client
        let quote_url = "https://quote-api.jup.ag/v6/quote";
        
        let mut url = reqwest::Url::parse(quote_url)?;
        url.query_pairs_mut()
            .append_pair("inputMint", "So11111111111111111111111111111111111111112") // SOL
            .append_pair("outputMint", "EPjFWdd5AufqSSqeM2qN1xzybapC8G4wEGGkZwyTDt1v") // USDC
            .append_pair("amount", "1000000000") // 1 SOL (9 decimals)
            .append_pair("slippageBps", "50"); // 0.5% slippage

        let response = self.http_client.get(url).send().await?;
        
        if response.status().is_success() {
            let quote: serde_json::Value = response.json().await?;
            
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

        Err(anyhow::anyhow!("Failed to calculate SOL price from quote"))
    }

    /// Warmer de conexiones para trading de alta frecuencia
    pub async fn warm_connections(&self) -> Result<()> {
        info!("🔥 Warming up connections for ultra-fast trading...");
        
        // Hacer una llamada dummy para establecer conexiones HTTP
        let warm_start = std::time::Instant::now();
        let _ = self.get_price_ultra_fast("So11111111111111111111111111111111111111112").await;
        
        info!("✅ Connection warmed in {}ms", warm_start.elapsed().as_millis());
        Ok(())
    }

    /// Precarga precios de tokens comunes para trading instantáneo
    pub async fn preload_common_prices(&self) -> Result<()> {
        info!("🔄 Preloading ultra-common trading pairs...");
        
        let common_tokens = [
            "So11111111111111111111111111111111111111112", // SOL
            "EPjFWdd5AufqSSqeM2qN1xzybapC8G4wEGGkZwyTDt1v", // USDC
            "Es9vMFrzaCERmJfrF4H2FYD4KCoNkY11McCe8BenwNYB", // USDT
        ];
        
        let preload_start = std::time::Instant::now();
        
        // Cargar en paralelo para velocidad máxima
        let tasks: Vec<_> = common_tokens.iter().map(|&token| {
            async move {
                let _ = self.get_price_ultra_fast(token).await;
            }
        }).collect();
        
        // Esperar a que todas las cargas terminen
        futures::future::join_all(tasks).await;
        
        info!("✅ Common prices preloaded in {}ms", preload_start.elapsed().as_millis());
        Ok(())
    }
}

/// Test de performance para el cliente ultra-rápido
pub async fn test_ultra_fast_client() -> Result<()> {
    println!("⚡ Testing Ultra-Fast Jupiter Client");
    println!("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━");

    let client = UltraFastJupiterClient::new().await?;
    
    // Test de velocidad directo (sin preload problemático)
    println!("\n🚀 Speed test:");
    for i in 1..=3 {
        let start = std::time::Instant::now();
        match client.get_price_ultra_fast("So11111111111111111111111111111111111111112").await? {
            Some(price) => {
                let time = start.elapsed();
                println!("  Call {}: {}ms (${:.2})", i, time.as_millis(), price);
            }
            None => println!("  Call {}: No price available", i),
        }
    }

    println!("✅ Ultra-fast client test completed");
    Ok(())
}
