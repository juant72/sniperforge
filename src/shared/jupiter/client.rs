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

use super::JupiterConfig;

/// HTTP client for Jupiter API
#[derive(Debug, Clone)]
pub struct JupiterClient {
    config: JupiterConfig,
    http_client: Client,
    base_url: Url,
}

impl JupiterClient {
    /// Create new Jupiter client
    pub async fn new(config: &JupiterConfig) -> Result<Self> {
        debug!("🌐 Setting up Jupiter HTTP client");

        let http_client = Client::builder()
            .timeout(Duration::from_secs(config.timeout_seconds))
            .user_agent("SniperForge/0.1.0")
            .build()?;

        let base_url = Url::parse(&config.api_base_url)?;

        let client = Self {
            config: config.clone(),
            http_client,
            base_url,
        };

        // Test initial connection
        client.health_check().await?;

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
    }

    /// Get quote for token swap
    pub async fn get_quote(
        &self,
        input_mint: &str,
        output_mint: &str,
        amount: u64,
        slippage_bps: Option<u16>,
    ) -> Result<Value> {
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
        let quote: Value = response.json().await?;

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
    }

    /// Get price for a specific token
    pub async fn get_price(&self, token_mint: &str) -> Result<Option<f64>> {
        debug!("💵 Getting price for token: {}", token_mint);

        let url = format!("https://price.jup.ag/v4/price?ids={}", token_mint);
        let response = self.http_client.get(&url).send().await?;

        if response.status().is_success() {
            let price_data: Value = response.json().await?;
            
            if let Some(data) = price_data.get("data") {
                if let Some(token_data) = data.get(token_mint) {
                    if let Some(price) = token_data.get("price") {
                        if let Some(price_num) = price.as_f64() {
                            debug!("✅ Price retrieved: ${}", price_num);
                            return Ok(Some(price_num));
                        }
                    }
                }
            }
        }

        debug!("⚠️  Price not found for token: {}", token_mint);
        Ok(None)
    }

    /// Execute HTTP request with retry logic
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
                        tokio::time::sleep(Duration::from_millis(1000 * attempts as u64)).await;
                        continue;
                    } else {
                        let error_text = response.text().await.unwrap_or_default();
                        return Err(anyhow!("HTTP error {}: {}", response.status(), error_text));
                    }
                }
                Err(e) if attempts < max_attempts => {
                    warn!("Request failed (attempt {}/{}): {}", attempts, max_attempts, e);
                    tokio::time::sleep(Duration::from_millis(1000 * attempts as u64)).await;
                    continue;
                }
                Err(e) => {
                    return Err(anyhow!("Request failed after {} attempts: {}", max_attempts, e));
                }
            }
        }
    }
}
