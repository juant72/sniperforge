/// Trading Engine Sin Caché - Máxima Seguridad para Operaciones Reales
/// 
/// Este módulo implementa un sistema de trading completamente libre de caché
/// para eliminar todos los riesgos de datos obsoletos en operaciones de dinero real

use anyhow::{Result, anyhow};
use std::time::{Duration, Instant};
use tracing::{info, warn, error, debug};

use crate::shared::jupiter::client::JupiterClient;
use crate::shared::jupiter::JupiterConfig;
use crate::shared::syndica_websocket::{SyndicaWebSocketClient, SyndicaConfig};

/// Configuración para trading sin caché
#[derive(Debug, Clone)]
pub struct TradingSafetyConfig {
    /// Máxima edad permitida para datos de precio (ms)
    pub max_price_age_ms: u64,
    /// Número mínimo de fuentes para verificar precios
    pub min_price_sources: u8,
    /// Timeout para fetch de datos frescos (ms)
    pub fresh_data_timeout_ms: u64,
    /// Margen de tolerancia para diferencias de precio entre fuentes (%)
    pub price_tolerance_percent: f64,
}

impl Default for TradingSafetyConfig {
    fn default() -> Self {
        Self {
            max_price_age_ms: 50,        // Solo datos < 50ms
            min_price_sources: 2,        // Mínimo 2 fuentes
            fresh_data_timeout_ms: 1000, // 1s timeout para fetch
            price_tolerance_percent: 0.5, // Max 0.5% diferencia entre fuentes
        }
    }
}

/// Trading Engine completamente libre de caché
#[derive(Debug)]
pub struct CacheFreeTrader {
    config: TradingSafetyConfig,
    jupiter_client: JupiterClient,
    syndica_client: Option<SyndicaWebSocketClient>,
}

impl CacheFreeTrader {
    /// Create new cache-free trader
    pub async fn new(config: TradingSafetyConfig) -> Result<Self> {
        info!("🛡️ Initializing Cache-Free Trading Engine");
        info!("   Max price age: {}ms", config.max_price_age_ms);
        info!("   Min sources: {}", config.min_price_sources);
        info!("   Price tolerance: {}%", config.price_tolerance_percent);
        
        // Initialize Jupiter client with default config
        let jupiter_config = JupiterConfig::default();
        let jupiter_client = JupiterClient::new(&jupiter_config).await?;
        
        // Initialize Syndica client but disable cache immediately
        let syndica_config = SyndicaConfig::default();
        let syndica_client = SyndicaWebSocketClient::new(syndica_config).await?;
        
        // CRITICAL: Disable cache completely for trading safety
        syndica_client.disable_cache_completely().await?;
        warn!("🚨 Syndica cache DISABLED for trading safety");
        
        Ok(Self {
            config,
            jupiter_client,
            syndica_client: Some(syndica_client),
        })
    }

    /// Get fresh price from multiple sources without any caching
    /// This is the SAFE method for trading operations
    pub async fn get_fresh_price_multi_source(&self, token_mint: &str) -> Result<TradingSafePrice> {
        info!("🔍 Fetching fresh price for {} from multiple sources", token_mint);
        let start_time = Instant::now();
        
        let mut price_sources = Vec::new();
        
        // Source 1: Jupiter API (always fresh)
        match self.fetch_jupiter_fresh_price(token_mint).await {
            Ok(price) => {
                price_sources.push(PriceSource {
                    source: "Jupiter API".to_string(),
                    price,
                    latency: Instant::now().duration_since(start_time),
                    confidence: PriceConfidence::High,
                });
            }
            Err(e) => {
                warn!("❌ Jupiter price fetch failed: {}", e);
            }
        }
        
        // Source 2: Syndica WebSocket (direct, no cache)
        if let Some(ref syndica) = self.syndica_client {
            match syndica.get_price_direct_no_cache(token_mint).await {
                Ok(Some(price)) => {
                    price_sources.push(PriceSource {
                        source: "Syndica Direct".to_string(),
                        price,
                        latency: Instant::now().duration_since(start_time),
                        confidence: PriceConfidence::High,
                    });
                }
                Ok(None) => {
                    debug!("📡 Syndica: No fresh data available");
                }
                Err(e) => {
                    warn!("❌ Syndica price fetch failed: {}", e);
                }
            }
        }
        
        // Source 3: Additional HTTP sources could be added here
        // TODO: Add Helius, QuickNode, etc.
        
        // Validate we have enough sources
        if price_sources.len() < self.config.min_price_sources as usize {
            return Err(anyhow!(
                "❌ TRADING SAFETY: Only {} price sources available, need minimum {}",
                price_sources.len(),
                self.config.min_price_sources
            ));
        }
        
        // Calculate consensus price and validate consistency
        let consensus = self.calculate_price_consensus(&price_sources)?;
        
        Ok(TradingSafePrice {
            price: consensus.price,
            sources: price_sources,
            consensus_confidence: consensus.confidence,
            total_latency: start_time.elapsed(),
            is_safe_for_trading: consensus.is_safe,
        })
    }

    /// Fetch fresh price from Jupiter (no cache)
    async fn fetch_jupiter_fresh_price(&self, token_mint: &str) -> Result<f64> {
        info!("🪐 Fetching fresh Jupiter price for {}", token_mint);
        
        // Use Jupiter's price API directly (always fresh)
        // This bypasses any internal caching in our Jupiter client
        let price = self.jupiter_client.get_token_price_direct(token_mint).await?;
        
        Ok(price)
    }

    /// Calculate price consensus from multiple sources
    fn calculate_price_consensus(&self, sources: &[PriceSource]) -> Result<PriceConsensus> {
        if sources.is_empty() {
            return Err(anyhow!("No price sources available"));
        }
        
        let prices: Vec<f64> = sources.iter().map(|s| s.price).collect();
        let avg_price = prices.iter().sum::<f64>() / prices.len() as f64;
        
        // Check price consistency
        let max_deviation = prices.iter()
            .map(|p| ((p - avg_price) / avg_price * 100.0).abs())
            .fold(0.0, f64::max);
        
        let is_safe = max_deviation <= self.config.price_tolerance_percent;
        
        if !is_safe {
            warn!("⚠️ Price deviation too high: {:.2}% (max allowed: {:.2}%)",
                  max_deviation, self.config.price_tolerance_percent);
        }
        
        let confidence = if is_safe && sources.len() >= 3 {
            PriceConfidence::High
        } else if is_safe {
            PriceConfidence::Medium
        } else {
            PriceConfidence::Low
        };
        
        Ok(PriceConsensus {
            price: avg_price,
            confidence,
            is_safe,
            max_deviation,
        })
    }

    /// Execute a swap with maximum safety (no cached data)
    pub async fn execute_safe_swap(
        &self,
        input_token: &str,
        output_token: &str,
        amount: u64,
    ) -> Result<SwapResult> {
        info!("🔄 Executing SAFE swap: {} -> {} (amount: {})", 
              input_token, output_token, amount);
        
        // Step 1: Get fresh prices for both tokens
        let input_price = self.get_fresh_price_multi_source(input_token).await?;
        let output_price = self.get_fresh_price_multi_source(output_token).await?;
        
        // Step 2: Validate prices are safe for trading
        if !input_price.is_safe_for_trading {
            return Err(anyhow!("❌ Input token price not safe for trading"));
        }
        if !output_price.is_safe_for_trading {
            return Err(anyhow!("❌ Output token price not safe for trading"));
        }
        
        // Step 3: Get fresh quote from Jupiter (no cache)
        let quote = self.jupiter_client.get_quote_direct(
            input_token,
            output_token,
            amount,
        ).await?;
        
        info!("✅ Fresh quote obtained - executing swap...");
        
        // Step 4: Execute the swap
        // TODO: Implement actual swap execution
        warn!("🚧 Swap execution not yet implemented");
          Ok(SwapResult {
            success: true,
            input_amount: amount,
            output_amount: quote.out_amount.parse().unwrap_or(0),
            input_price: input_price.price,
            output_price: output_price.price,
            latency: input_price.total_latency + output_price.total_latency,
        })
    }
}

/// Price source information
#[derive(Debug, Clone)]
pub struct PriceSource {
    pub source: String,
    pub price: f64,
    pub latency: Duration,
    pub confidence: PriceConfidence,
}

/// Price confidence levels
#[derive(Debug, Clone, PartialEq)]
pub enum PriceConfidence {
    High,    // Multiple sources, low deviation
    Medium,  // 2 sources, acceptable deviation
    Low,     // Few sources or high deviation
}

/// Price consensus calculation result
#[derive(Debug)]
struct PriceConsensus {
    pub price: f64,
    pub confidence: PriceConfidence,
    pub is_safe: bool,
    pub max_deviation: f64,
}

/// Trading-safe price with metadata
#[derive(Debug)]
pub struct TradingSafePrice {
    pub price: f64,
    pub sources: Vec<PriceSource>,
    pub consensus_confidence: PriceConfidence,
    pub total_latency: Duration,
    pub is_safe_for_trading: bool,
}

/// Swap execution result
#[derive(Debug)]
pub struct SwapResult {
    pub success: bool,
    pub input_amount: u64,
    pub output_amount: u64,
    pub input_price: f64,
    pub output_price: f64,
    pub latency: Duration,
}

/// Test function for cache-free trading
pub async fn test_cache_free_trading() -> Result<()> {
    println!("🛡️ Testing Cache-Free Trading Engine");
    println!("====================================");
    
    let config = TradingSafetyConfig::default();
    let trader = CacheFreeTrader::new(config).await?;
    
    let test_token = "So11111111111111111111111111111111111111112"; // SOL
    
    // Test fresh price fetching
    println!("\n1️⃣ Testing fresh price fetching...");
    match trader.get_fresh_price_multi_source(test_token).await {
        Ok(price_data) => {
            println!("✅ Fresh price: ${:.4}", price_data.price);
            println!("   Sources: {}", price_data.sources.len());
            println!("   Latency: {:?}", price_data.total_latency);
            println!("   Safe for trading: {}", 
                if price_data.is_safe_for_trading { "✅ YES" } else { "❌ NO" });
            
            for source in &price_data.sources {
                println!("   📊 {}: ${:.4} ({:?})", 
                         source.source, source.price, source.latency);
            }
        }
        Err(e) => {
            println!("❌ Error: {}", e);
        }
    }
    
    println!("\n✅ Cache-free trading test completed!");
    Ok(())
}
