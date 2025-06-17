/// Jupiter Quote Engine
/// 
/// High-level interface for getting price quotes and market data

use anyhow::Result;
use solana_sdk::pubkey::Pubkey;
use std::collections::HashMap;
use std::time::{Duration, Instant};
use tokio::sync::RwLock;
use tracing::{info, warn, error, debug};

use super::client::JupiterClient;
use super::types::*;

/// Cache entry for quotes
#[derive(Debug, Clone)]
struct QuoteCache {
    quote: JupiterQuote,
    timestamp: Instant,
}

/// Quote engine with caching and optimization
pub struct QuoteEngine {
    client: JupiterClient,
    quote_cache: RwLock<HashMap<String, QuoteCache>>,
    cache_duration: Duration,
}

impl QuoteEngine {
    /// Create new quote engine
    pub fn new(client: JupiterClient) -> Self {
        Self {
            client,
            quote_cache: RwLock::new(HashMap::new()),
            cache_duration: Duration::from_secs(30), // 30 second cache
        }
    }

    /// Get best quote for a swap
    pub async fn get_quote(&self, request: QuoteRequest) -> Result<JupiterQuote> {
        let cache_key = format!("{}_{}_{}_{}", 
                               request.input_mint,
                               request.output_mint, 
                               request.amount,
                               request.slippage_bps.unwrap_or(50));

        // Check cache first
        if let Some(cached) = self.get_cached_quote(&cache_key).await {
            debug!("📋 Using cached quote for {}", cache_key);
            return Ok(cached);
        }

        debug!("🔍 Fetching new quote: {} -> {} ({})", 
               request.input_mint, request.output_mint, request.amount);

        // Fetch new quote
        let quote_response = self.client.get_quote(
            &request.input_mint.to_string(),
            &request.output_mint.to_string(),
            request.amount,
            request.slippage_bps,
        ).await?;

        // Parse response
        let quote: JupiterQuote = serde_json::from_value(quote_response)?;

        // Cache the result
        self.cache_quote(cache_key, quote.clone()).await;

        info!("✅ Quote obtained: {} {} -> {} {} (impact: {}%)", 
              quote.in_amount, 
              self.format_token(&request.input_mint),
              quote.out_amount,
              self.format_token(&request.output_mint),
              quote.price_impact_pct);

        Ok(quote)
    }

    /// Get price for a token in USD
    pub async fn get_token_price_usd(&self, token_mint: &Pubkey) -> Result<Option<f64>> {
        debug!("💵 Getting USD price for token: {}", token_mint);
        
        let price = self.client.get_price(&token_mint.to_string()).await?;
        
        if let Some(price_value) = price {
            debug!("✅ USD price: ${}", price_value);
            Ok(Some(price_value))
        } else {
            debug!("⚠️  No USD price available for token: {}", token_mint);
            Ok(None)
        }
    }

    /// Get price in SOL for a token
    pub async fn get_token_price_sol(&self, token_mint: &Pubkey) -> Result<Option<f64>> {
        if *token_mint == tokens::sol() {
            return Ok(Some(1.0)); // SOL = 1 SOL
        }

        debug!("⚡ Getting SOL price for token: {}", token_mint);

        // Use 1 SOL as input to get the ratio
        let quote_request = QuoteRequest::new(
            tokens::sol(),
            *token_mint,
            1_000_000_000, // 1 SOL in lamports
        );

        match self.get_quote(quote_request).await {
            Ok(quote) => {
                let out_amount: u64 = quote.out_amount.parse().unwrap_or(0);
                if out_amount > 0 {
                    let rate = out_amount as f64 / 1_000_000_000.0;
                    debug!("✅ SOL rate: 1 SOL = {} tokens", rate);
                    Ok(Some(1.0 / rate)) // Price in SOL per token
                } else {
                    Ok(None)
                }
            }
            Err(e) => {
                warn!("Failed to get SOL price for {}: {}", token_mint, e);
                Ok(None)
            }
        }
    }

    /// Compare prices across multiple output tokens
    pub async fn compare_prices(
        &self,
        input_mint: &Pubkey,
        output_options: &[Pubkey],
        amount: u64,
    ) -> Result<Vec<(Pubkey, JupiterQuote)>> {
        debug!("📊 Comparing prices for {} output options", output_options.len());

        let mut results = Vec::new();

        for output_mint in output_options {
            let request = QuoteRequest::new(*input_mint, *output_mint, amount);
            
            match self.get_quote(request).await {
                Ok(quote) => {
                    results.push((*output_mint, quote));
                }
                Err(e) => {
                    warn!("Failed to get quote for {}: {}", output_mint, e);
                }
            }
        }

        // Sort by best output amount
        results.sort_by(|a, b| {
            let a_amount: u64 = a.1.out_amount.parse().unwrap_or(0);
            let b_amount: u64 = b.1.out_amount.parse().unwrap_or(0);
            b_amount.cmp(&a_amount) // Descending order (best first)
        });

        info!("✅ Price comparison complete: {} valid quotes", results.len());
        Ok(results)
    }

    /// Check if a quote is still valid (not expired)
    pub fn is_quote_valid(&self, quote: &JupiterQuote) -> bool {
        // Jupiter quotes are typically valid for ~30 seconds
        // We use a conservative 20 seconds
        true // For now, assume quotes are always valid
        // TODO: Implement proper quote validation based on timestamp
    }

    /// Get cached quote if available and not expired
    async fn get_cached_quote(&self, cache_key: &str) -> Option<JupiterQuote> {
        let cache = self.quote_cache.read().await;
        
        if let Some(cached) = cache.get(cache_key) {
            if cached.timestamp.elapsed() < self.cache_duration {
                return Some(cached.quote.clone());
            }
        }
        
        None
    }

    /// Cache a quote
    async fn cache_quote(&self, cache_key: String, quote: JupiterQuote) {
        let mut cache = self.quote_cache.write().await;
        cache.insert(cache_key, QuoteCache {
            quote,
            timestamp: Instant::now(),
        });

        // Clean old entries
        cache.retain(|_, entry| entry.timestamp.elapsed() < self.cache_duration * 2);
    }

    /// Format token for display
    fn format_token(&self, token_mint: &Pubkey) -> String {
        match token_mint.to_string().as_str() {
            tokens::WSOL => "SOL".to_string(),
            tokens::USDC => "USDC".to_string(),
            tokens::USDT => "USDT".to_string(),
            _ => format!("{}...", &token_mint.to_string()[0..8]),
        }
    }

    /// Clear quote cache
    pub async fn clear_cache(&self) {
        let mut cache = self.quote_cache.write().await;
        cache.clear();
        debug!("🧹 Quote cache cleared");
    }
}
