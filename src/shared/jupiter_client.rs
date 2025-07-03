//! Jupiter HTTP Client
//! 
//! Low-level HTTP client for Jupiter API calls. Handles only HTTP requests/responses.

use anyhow::{Result, anyhow};
use reqwest::Client;
use std::time::Duration;
use tracing::{info, warn, error, debug};
use std::collections::HashMap;

use super::jupiter_config::JupiterConfig;
use super::jupiter_types::*;

/// Low-level Jupiter HTTP client
#[derive(Debug, Clone)]
pub struct JupiterClient {
    client: Client,
    base_url: String,
    api_key: Option<String>,
}

impl JupiterClient {
    /// Create new Jupiter client
    pub async fn new(config: &JupiterConfig) -> Result<Self> {
        let client = Client::builder()
            .timeout(Duration::from_secs(config.timeout_seconds))
            .build()?;
            
        Ok(Self {
            client,
            base_url: config.base_url.clone(),
            api_key: config.api_key.clone(),
        })
    }
    
    /// Get token price using Jupiter V3 API
    pub async fn get_price(&self, mint: &str) -> Result<Option<f64>> {
        let url = format!("{}/price/v2?ids={}", self.base_url, mint);
        debug!("🌐 Fetching price from: {}", url);
        
        let mut request = self.client.get(&url)
            .header("User-Agent", "SniperForge/1.0")
            .header("Accept", "application/json");
        
        if let Some(api_key) = &self.api_key {
            request = request.header("Authorization", format!("Bearer {}", api_key));
        }
        
        let response = request.send().await?;
        
        if !response.status().is_success() {
            return Err(anyhow!("Price API request failed: {}", response.status()));
        }
        
        let price_response: JupiterPriceResponse = response.json().await?;
        
        if let Some(price_data) = price_response.data.get(mint) {
            Ok(Some(price_data.price))
        } else {
            warn!("No price data found for mint: {}", mint);
            Ok(None)
        }
    }
    
    /// Get quote from Jupiter V6 API
    pub async fn get_quote(&self, request: QuoteRequest) -> Result<QuoteResponse> {
        let url = format!("{}/v6/quote", self.base_url);
        debug!("🌐 Getting quote from: {}", url);
        
        let mut http_request = self.client.get(&url)
            .header("User-Agent", "SniperForge/1.0")
            .header("Accept", "application/json")
            .query(&[
                ("inputMint", request.inputMint.as_str()),
                ("outputMint", request.outputMint.as_str()),
                ("amount", &request.amount.to_string()),
                ("slippageBps", &request.slippageBps.to_string()),
            ]);
        
        if let Some(api_key) = &self.api_key {
            http_request = http_request.header("Authorization", format!("Bearer {}", api_key));
        }
        
        let response = http_request.send().await?;
        let status = response.status();
        
        if !status.is_success() {
            let error_text = response.text().await.unwrap_or_default();
            return Err(anyhow!("Quote API request failed: {} - {}", status, error_text));
        }
        
        let quote: QuoteResponse = response.json().await?;
        Ok(quote)
    }
    
    /// Build swap transaction using Jupiter V6 API
    pub async fn build_swap_transaction(&self, swap_request: SwapRequest) -> Result<SwapResponse> {
        let swap_url = format!("{}/v6/swap", self.base_url.replace("lite-api.jup.ag", "quote-api.jup.ag"));
        debug!("🌐 Building swap transaction at: {}", swap_url);
        
        let mut request = self.client.post(&swap_url)
            .header("User-Agent", "SniperForge/1.0")
            .header("Accept", "application/json")
            .header("Content-Type", "application/json")
            .json(&swap_request);
        
        if let Some(api_key) = &self.api_key {
            request = request.header("Authorization", format!("Bearer {}", api_key));
        }
        
        let response = request.send().await?;
        let status = response.status();
        
        if !status.is_success() {
            let error_text = response.text().await.unwrap_or_default();
            return Err(anyhow!("Swap API request failed: {} - {}", status, error_text));
        }
        
        let swap_response: SwapResponse = response.json().await?;
        
        info!("✅ Swap transaction built successfully");
        debug!("Transaction base64 length: {}", swap_response.swapTransaction.len());
        debug!("Last valid block height: {}", swap_response.lastValidBlockHeight);
        
        Ok(swap_response)
    }
    
    /// Execute swap with wallet integration (delegates to Jupiter API)
    pub async fn execute_swap_with_wallet(
        &self,
        quote: &QuoteResponse,
        wallet_address: &str,
        wallet_keypair: Option<&solana_sdk::signature::Keypair>,
    ) -> Result<SwapExecutionResult> {
        use super::jupiter_api::Jupiter;
        use super::jupiter_config::JupiterConfig;
        
        // Create Jupiter wrapper with default config
        let config = JupiterConfig {
            base_url: self.base_url.clone(),
            api_key: self.api_key.clone(),
            timeout_seconds: 30,
            max_retries: 3,
            rpc_endpoint: "https://api.devnet.solana.com".to_string(),
            network_name: "DevNet".to_string(),
        };
        
        let jupiter = Jupiter::new(&config).await?;
        jupiter.execute_swap_with_wallet(quote, wallet_address, wallet_keypair).await
    }
    
    /// Get ultra-fast price (backward compatibility)
    pub async fn get_price_ultra_fast(&self, mint: &str) -> Result<Option<f64>> {
        self.get_price(mint).await
    }
    
    /// Health check (backward compatibility)
    pub async fn health_check(&self) -> Result<bool> {
        Ok(self.is_configured())
    }
    
    /// Check if client is properly configured
    pub fn is_configured(&self) -> bool {
        !self.base_url.is_empty()
    }
}
