//! Jupiter API Client - Enterprise Implementation
//!
//! Consolidated HTTP client for all Jupiter API operations
//! Includes rate limiting, retry logic, and error handling

use crate::apis::jupiter::config::JupiterApiConfig;
use crate::apis::jupiter::types::{
    JupiterPriceResponse, JupiterQuoteResponse, QuoteRequest, JupiterQuote
};
use anyhow::{anyhow, Result};
use reqwest::Client;
use std::sync::Arc;
use tokio::sync::RwLock;
use tokio::time::{sleep, Duration, Instant};
use tracing::{debug, error, warn};

/// Enterprise Jupiter API client with rate limiting and retry logic
#[derive(Debug)]
pub struct JupiterClient {
    client: Client,
    config: JupiterApiConfig,
    rate_limiter: Arc<RwLock<RateLimiter>>,
}

#[derive(Debug)]
struct RateLimiter {
    last_request: Instant,
    requests_this_second: u32,
    current_second: u64,
}

impl JupiterClient {
    /// Create a new Jupiter client with configuration
    pub fn new(config: JupiterApiConfig) -> Result<Self> {
        let client = Client::builder()
            .timeout(Duration::from_secs(config.timeout_seconds))
            .build()
            .map_err(|e| anyhow!("Failed to create HTTP client: {}", e))?;

        let rate_limiter = Arc::new(RwLock::new(RateLimiter {
            last_request: Instant::now(),
            requests_this_second: 0,
            current_second: 0,
        }));

        Ok(Self {
            client,
            config,
            rate_limiter,
        })
    }

    /// Create client with default devnet configuration
    pub fn devnet() -> Result<Self> {
        Self::new(JupiterApiConfig::devnet())
    }

    /// Create client with default mainnet configuration
    pub fn mainnet() -> Result<Self> {
        Self::new(JupiterApiConfig::mainnet())
    }

    /// Get quote for token swap with enhanced error handling
    pub async fn get_quote(&self, request: &QuoteRequest) -> Result<JupiterQuoteResponse> {
        if !self.config.enabled {
            return Err(anyhow!("Jupiter integration is disabled"));
        }

        self.enforce_rate_limit().await;

        let url = format!("{}/v6/quote", self.config.base_url);
        debug!("🔗 Jupiter quote request: {} -> {}", request.input_mint, request.output_mint);

        let mut attempt = 0;
        while attempt < self.config.max_retries {
            match self.make_quote_request(&url, request).await {
                Ok(response) => {
                    debug!("✅ Jupiter quote successful on attempt {}", attempt + 1);
                    return Ok(response);
                }
                Err(e) => {
                    attempt += 1;
                    if attempt < self.config.max_retries {
                        warn!("⚠️ Jupiter quote attempt {} failed: {}. Retrying...", attempt, e);
                        sleep(Duration::from_millis(1000 * attempt as u64)).await;
                    } else {
                        error!("❌ Jupiter quote failed after {} attempts: {}", self.config.max_retries, e);
                        return Err(e);
                    }
                }
            }
        }

        Err(anyhow!("Quote request failed after all retries"))
    }

    /// Get token prices
    pub async fn get_prices(&self, token_addresses: Vec<String>) -> Result<JupiterPriceResponse> {
        if !self.config.enabled {
            return Err(anyhow!("Jupiter integration is disabled"));
        }

        self.enforce_rate_limit().await;

        let ids = token_addresses.join(",");
        let url = format!("{}/price?ids={}", self.config.base_url, ids);
        debug!("🔗 Jupiter price request for {} tokens", token_addresses.len());

        let mut attempt = 0;
        while attempt < self.config.max_retries {
            match self.make_price_request(&url).await {
                Ok(response) => {
                    debug!("✅ Jupiter price request successful on attempt {}", attempt + 1);
                    return Ok(response);
                }
                Err(e) => {
                    attempt += 1;
                    if attempt < self.config.max_retries {
                        warn!("⚠️ Jupiter price attempt {} failed: {}. Retrying...", attempt, e);
                        sleep(Duration::from_millis(1000 * attempt as u64)).await;
                    } else {
                        error!("❌ Jupiter price failed after {} attempts: {}", self.config.max_retries, e);
                        return Err(e);
                    }
                }
            }
        }

        Err(anyhow!("Price request failed after all retries"))
    }

    /// Get quote in legacy format for backward compatibility - ENHANCED
    pub async fn get_quote_legacy(&self, request: QuoteRequest) -> Result<JupiterQuote> {
        let response = self.get_quote(&request).await?;
        Ok(JupiterQuote::from(response))
    }

    /// Health check - verify Jupiter API is accessible
    pub async fn health_check(&self) -> Result<bool> {
        let url = format!("{}/v6/quote", self.config.base_url);
        
        match self.client.head(&url).send().await {
            Ok(response) => {
                let is_healthy = response.status().is_success();
                if is_healthy {
                    debug!("✅ Jupiter API health check passed");
                } else {
                    warn!("⚠️ Jupiter API health check failed: {}", response.status());
                }
                Ok(is_healthy)
            }
            Err(e) => {
                error!("❌ Jupiter API health check error: {}", e);
                Ok(false)
            }
        }
    }

    // Private helper methods

    async fn make_quote_request(&self, url: &str, request: &QuoteRequest) -> Result<JupiterQuoteResponse> {
        let query_params = self.build_quote_query_params(request)?;
        
        let response = self.client
            .get(url)
            .query(&query_params)
            .send()
            .await
            .map_err(|e| anyhow!("HTTP request failed: {}", e))?;

        if !response.status().is_success() {
            let status = response.status();
            let text = response.text().await.unwrap_or_default();
            return Err(anyhow!("Jupiter API error {}: {}", status, text));
        }

        let quote_response: JupiterQuoteResponse = response
            .json()
            .await
            .map_err(|e| anyhow!("Failed to parse quote response: {}", e))?;

        Ok(quote_response)
    }

    async fn make_price_request(&self, url: &str) -> Result<JupiterPriceResponse> {
        let response = self.client
            .get(url)
            .send()
            .await
            .map_err(|e| anyhow!("HTTP request failed: {}", e))?;

        if !response.status().is_success() {
            let status = response.status();
            let text = response.text().await.unwrap_or_default();
            return Err(anyhow!("Jupiter API error {}: {}", status, text));
        }

        let price_response: JupiterPriceResponse = response
            .json()
            .await
            .map_err(|e| anyhow!("Failed to parse price response: {}", e))?;

        Ok(price_response)
    }

    fn build_quote_query_params(&self, request: &QuoteRequest) -> Result<Vec<(String, String)>> {
        let mut params = vec![
            ("inputMint".to_string(), request.input_mint.clone()),
            ("outputMint".to_string(), request.output_mint.clone()),
            ("amount".to_string(), request.amount.to_string()),
        ];

        if let Some(slippage) = request.slippage_bps {
            params.push(("slippageBps".to_string(), slippage.to_string()));
        }

        if let Some(ref swap_mode) = request.swap_mode {
            params.push(("swapMode".to_string(), swap_mode.clone()));
        }

        if let Some(ref dexes) = request.dexes {
            params.push(("dexes".to_string(), dexes.join(",")));
        }

        if let Some(ref exclude_dexes) = request.exclude_dexes {
            params.push(("excludeDexes".to_string(), exclude_dexes.join(",")));
        }

        Ok(params)
    }

    async fn enforce_rate_limit(&self) {
        let mut limiter = self.rate_limiter.write().await;
        let now = Instant::now();
        let current_second = now.elapsed().as_secs();

        if current_second != limiter.current_second {
            limiter.current_second = current_second;
            limiter.requests_this_second = 0;
        }

        if limiter.requests_this_second >= self.config.rate_limit_rps {
            let sleep_duration = Duration::from_millis(1000 / self.config.rate_limit_rps as u64);
            drop(limiter); // Release lock before sleeping
            sleep(sleep_duration).await;
        } else {
            limiter.requests_this_second += 1;
            limiter.last_request = now;
        }
    }

    /// Get swap transaction from Jupiter API
    pub async fn get_swap_transaction(&self, swap_request: &super::jupiter::SwapRequest) -> Result<String> {
        if !self.config.enabled {
            return Err(anyhow!("Jupiter integration is disabled"));
        }

        self.enforce_rate_limit().await;

        let url = format!("{}/v6/swap", self.config.base_url);
        debug!("🔄 Jupiter swap transaction request for user: {}", swap_request.user_public_key);

        let mut attempt = 0;
        while attempt < self.config.max_retries {
            match self.make_swap_request(&url, swap_request).await {
                Ok(response) => {
                    debug!("✅ Jupiter swap transaction successful on attempt {}", attempt + 1);
                    return Ok(response);
                }
                Err(e) => {
                    attempt += 1;
                    if attempt < self.config.max_retries {
                        warn!("⚠️ Jupiter swap transaction attempt {} failed: {}. Retrying...", attempt, e);
                        sleep(Duration::from_millis(1000 * attempt as u64)).await;
                    } else {
                        error!("❌ Jupiter swap transaction failed after {} attempts: {}", self.config.max_retries, e);
                        return Err(e);
                    }
                }
            }
        }

        Err(anyhow!("Swap transaction request failed after all retries"))
    }

    /// Make swap transaction request
    async fn make_swap_request(&self, url: &str, swap_request: &super::jupiter::SwapRequest) -> Result<String> {
        let response = self.client
            .post(url)
            .header("Content-Type", "application/json")
            .json(swap_request)
            .send()
            .await
            .map_err(|e| anyhow!("Network error: {}", e))?;

        if !response.status().is_success() {
            return Err(anyhow!(
                "Jupiter swap transaction API error: {} - {}",
                response.status(),
                response.text().await.unwrap_or_default()
            ));
        }

        let swap_response: serde_json::Value = response.json().await
            .map_err(|e| anyhow!("Failed to parse swap transaction response: {}", e))?;

        // Extract the swap transaction (base64 encoded)
        swap_response
            .get("swapTransaction")
            .and_then(|tx| tx.as_str())
            .map(|tx| tx.to_string())
            .ok_or_else(|| anyhow!("No swap transaction found in response"))
    }
}
