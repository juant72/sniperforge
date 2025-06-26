//! Jupiter API Integration - Real Implementation
//! 
//! This module provides real integration with Jupiter API for:
//! - Real-time price feeds
//! - Token quotes and swaps
//! - Route optimization
//! - Slippage calculation

use anyhow::{Result, anyhow};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use reqwest::Client;
use tokio::time::Duration;
use tracing::{info, warn, error};

/// Jupiter API client for real price and swap data
#[derive(Debug, Clone)]
pub struct JupiterClient {
    client: Client,
    base_url: String,
    api_key: Option<String>,
}

/// Main Jupiter wrapper for easier integration
#[derive(Debug)]
pub struct Jupiter {
    client: JupiterClient,
}

/// Jupiter price response structure
#[derive(Debug, Deserialize)]
pub struct JupiterPriceResponse {
    pub data: HashMap<String, PriceData>,
}

#[derive(Debug, Deserialize)]
#[allow(non_snake_case)] // Jupiter API uses camelCase field names
pub struct PriceData {
    pub id: String,
    pub mintSymbol: String,
    pub vsToken: String,
    pub vsTokenSymbol: String,
    pub price: f64,
}

/// Jupiter quote request
#[derive(Debug, Serialize)]
#[allow(non_snake_case)] // Jupiter API uses camelCase field names
pub struct QuoteRequest {
    pub inputMint: String,
    pub outputMint: String,
    pub amount: u64,
    pub slippageBps: u16,
}

/// Jupiter quote response
#[derive(Debug, Deserialize, Clone, Serialize)]
#[allow(non_snake_case)] // Jupiter API uses camelCase field names
pub struct QuoteResponse {
    pub inputMint: String,
    pub inAmount: String,
    pub outputMint: String,
    pub outAmount: String,
    pub otherAmountThreshold: String,
    pub swapMode: String,
    pub slippageBps: u16,
    pub platformFee: Option<PlatformFee>,
    pub priceImpactPct: String,
    pub routePlan: Vec<RoutePlan>,
    // Additional fields for compatibility
    pub in_amount: f64,
    pub out_amount: f64,
    pub price_impact_pct: f64,
    pub route_plan: Vec<RoutePlan>,
}

#[derive(Debug, Deserialize, Clone, Serialize)]
#[allow(non_snake_case)] // Jupiter API uses camelCase field names
pub struct PlatformFee {
    pub amount: String,
    pub feeBps: u16,
}

#[derive(Debug, Deserialize, Clone, Serialize)]
#[allow(non_snake_case)] // Jupiter API uses camelCase field names
pub struct RoutePlan {
    pub swapInfo: SwapInfo,
    pub percent: u8,
}

#[derive(Debug, Deserialize, Clone, Serialize)]
#[allow(non_snake_case)] // Jupiter API uses camelCase field names
pub struct SwapInfo {
    pub ammKey: String,
    pub label: String,
    pub inputMint: String,
    pub outputMint: String,
    pub inAmount: String,
    pub outAmount: String,
    pub feeAmount: String,
    pub feeMint: String,
}

/// Jupiter configuration
#[derive(Debug, Clone)]
pub struct JupiterConfig {
    pub base_url: String,
    pub api_key: Option<String>,
    pub timeout_seconds: u64,
    pub max_retries: u32,
}

impl JupiterConfig {
    /// Create mainnet configuration
    pub fn mainnet() -> Self {
        Self {
            base_url: "https://price.jup.ag".to_string(),
            api_key: None,
            timeout_seconds: 10,
            max_retries: 3,
        }
    }

    /// Create devnet configuration
    pub fn devnet() -> Self {
        Self {
            base_url: "https://price.jup.ag".to_string(), // Jupiter uses same endpoint for both
            api_key: None,
            timeout_seconds: 15, // Slightly longer timeout for devnet
            max_retries: 5,
        }
    }
}

impl Default for JupiterConfig {
    fn default() -> Self {
        Self {
            base_url: "https://price.jup.ag".to_string(),
            api_key: None,
            timeout_seconds: 10,
            max_retries: 3,
        }
    }
}

impl JupiterClient {
    /// Create new Jupiter client with real API integration
    pub async fn new(config: &JupiterConfig) -> Result<Self> {
        let client = Client::builder()
            .timeout(Duration::from_secs(config.timeout_seconds))
            .build()?;

        let jupiter_client = Self {
            client,
            base_url: config.base_url.clone(),
            api_key: config.api_key.clone(),
        };

        // Test connectivity
        jupiter_client.test_connectivity().await?;
        
        Ok(jupiter_client)
    }

    /// Test Jupiter API connectivity
    async fn test_connectivity(&self) -> Result<()> {
        info!("Testing Jupiter API connectivity...");
        
        // Test with SOL price
        let sol_mint = "So11111111111111111111111111111111111111112";
        match self.get_price(sol_mint).await {
            Ok(Some(price)) => {
                info!("✅ Jupiter API connected successfully. SOL price: ${:.2}", price);
                Ok(())
            },
            Ok(None) => {
                warn!("⚠️ Jupiter API connected but no price data");
                Ok(())
            },
            Err(e) => {
                error!("❌ Jupiter API connection failed: {}", e);
                Err(anyhow!("Jupiter API connection test failed: {}", e))
            }
        }
    }

    /// Get real-time price for a token
    pub async fn get_price(&self, mint: &str) -> Result<Option<f64>> {
        let url = format!("{}/v4/price?ids={}", self.base_url, mint);
        
        let mut request = self.client.get(&url);
        
        if let Some(api_key) = &self.api_key {
            request = request.header("Authorization", format!("Bearer {}", api_key));
        }

        let response = request
            .send()
            .await
            .map_err(|e| anyhow!("Failed to fetch price from Jupiter: {}", e))?;

        if !response.status().is_success() {
            return Err(anyhow!("Jupiter API returned error: {}", response.status()));
        }

        let price_response: JupiterPriceResponse = response
            .json()
            .await
            .map_err(|e| anyhow!("Failed to parse Jupiter price response: {}", e))?;

        // Extract price from response
        if let Some(price_data) = price_response.data.get(mint) {
            Ok(Some(price_data.price))
        } else {
            warn!("No price data found for mint: {}", mint);
            Ok(None)
        }
    }

    /// Get multiple token prices in one request
    pub async fn get_prices(&self, mints: &[String]) -> Result<HashMap<String, f64>> {
        let ids = mints.join(",");
        let url = format!("{}/v4/price?ids={}", self.base_url, ids);
        
        let mut request = self.client.get(&url);
        
        if let Some(api_key) = &self.api_key {
            request = request.header("Authorization", format!("Bearer {}", api_key));
        }

        let response = request
            .send()
            .await
            .map_err(|e| anyhow!("Failed to fetch prices from Jupiter: {}", e))?;

        if !response.status().is_success() {
            return Err(anyhow!("Jupiter API returned error: {}", response.status()));
        }

        let price_response: JupiterPriceResponse = response
            .json()
            .await
            .map_err(|e| anyhow!("Failed to parse Jupiter price response: {}", e))?;

        let mut prices = HashMap::new();
        for (mint, price_data) in price_response.data {
            prices.insert(mint, price_data.price);
        }

        Ok(prices)
    }

    /// Get quote for a potential swap
    pub async fn get_quote(&self, request: QuoteRequest) -> Result<QuoteResponse> {
        let url = format!("{}/v6/quote", self.base_url.replace("price.jup.ag", "quote-api.jup.ag"));
        
        let query_params = [
            ("inputMint", request.inputMint.as_str()),
            ("outputMint", request.outputMint.as_str()),
            ("amount", &request.amount.to_string()),
            ("slippageBps", &request.slippageBps.to_string()),
        ];

        let mut req = self.client.get(&url).query(&query_params);
        
        if let Some(api_key) = &self.api_key {
            req = req.header("Authorization", format!("Bearer {}", api_key));
        }

        let response = req
            .send()
            .await
            .map_err(|e| anyhow!("Failed to fetch quote from Jupiter: {}", e))?;

        if !response.status().is_success() {
            let status = response.status();
            let error_text = response.text().await.unwrap_or_default();
            return Err(anyhow!("Jupiter quote API error: {} - {}", status, error_text));
        }

        let quote: QuoteResponse = response
            .json()
            .await
            .map_err(|e| anyhow!("Failed to parse Jupiter quote response: {}", e))?;

        Ok(quote)
    }

    /// Get historical prices (if supported by Jupiter)
    pub async fn get_historical_prices(
        &self, 
        _mint: &str, 
        _from_timestamp: i64, 
        _to_timestamp: i64
    ) -> Result<Vec<HistoricalPrice>> {
        // Note: This may need to be implemented based on Jupiter's historical data API
        // For now, return error indicating not supported
        Err(anyhow!("Historical price data not yet implemented. Use external sources like Birdeye or DexScreener for historical data."))
    }

    /// Calculate real slippage for a trade
    pub async fn calculate_slippage(
        &self,
        input_mint: &str,
        output_mint: &str,
        amount: u64
    ) -> Result<SlippageInfo> {
        // Get quotes with different slippage tolerances
        let slippage_levels = vec![10, 50, 100, 300, 500]; // 0.1%, 0.5%, 1%, 3%, 5%
        let mut slippage_data = Vec::new();

        for slippage_bps in slippage_levels {
            let quote_request = QuoteRequest {
                inputMint: input_mint.to_string(),
                outputMint: output_mint.to_string(),
                amount,
                slippageBps: slippage_bps,
            };

            match self.get_quote(quote_request).await {
                Ok(quote) => {
                    let output_amount: u64 = quote.outAmount.parse().unwrap_or(0);
                    slippage_data.push(SlippagePoint {
                        slippage_bps,
                        output_amount,
                        price_impact: quote.priceImpactPct.parse().unwrap_or(0.0),
                    });
                },
                Err(e) => {
                    warn!("Failed to get quote for slippage {}: {}", slippage_bps, e);
                }
            }
        }

        Ok(SlippageInfo {
            input_mint: input_mint.to_string(),
            output_mint: output_mint.to_string(),
            input_amount: amount,
            slippage_data,
        })
    }

    /// Health check for Jupiter API
    pub async fn health_check(&self) -> Result<()> {
        self.test_connectivity().await
    }

    /// Ultra-fast price retrieval (optimized version)
    pub async fn get_price_ultra_fast(&self, mint: &str) -> Result<Option<f64>> {
        // For now, delegate to regular get_price method
        // Could be optimized with caching, connection pooling, etc.
        self.get_price(mint).await
    }
}

/// Historical price data point
#[derive(Debug, Clone)]
pub struct HistoricalPrice {
    pub timestamp: i64,
    pub price: f64,
    pub volume: Option<f64>,
}

/// Slippage analysis information
#[derive(Debug)]
pub struct SlippageInfo {
    pub input_mint: String,
    pub output_mint: String,
    pub input_amount: u64,
    pub slippage_data: Vec<SlippagePoint>,
}

#[derive(Debug)]
pub struct SlippagePoint {
    pub slippage_bps: u16,
    pub output_amount: u64,
    pub price_impact: f64,
}

/// Common Solana token mints for convenience
pub mod tokens {
    pub const SOL: &str = "So11111111111111111111111111111111111111112";
    pub const USDC: &str = "EPjFWdd5AufqSSqeM2qN1xzybapC8G4wEGGkZwyTDt1v";
    pub const USDT: &str = "Es9vMFrzaCERmJfrF4H2FYD4KCoNkY11McCe8BenwNYB";
    pub const RAY: &str = "4k3Dyjzvzp8eMZWUXbBCjEvwSkkk59S5iCNLY3QrkX6R";
    pub const SRM: &str = "SRMuApVNdxXokk5GT7XD5cUUgXMBCoAz2LHeuAoKWRt";
    pub const ORCA: &str = "orcaEKTdK7LKz57vaAYr9QeNsVEPfiu6QeMU1kektZE";
    pub const MNGO: &str = "MangoCzJ36AjZyKwVj3VnYU4GTonjfVEnJmvvWaxLac";
}

/// Jupiter wrapper for easier integration
impl Jupiter {
    /// Create new Jupiter instance
    pub async fn new(config: &JupiterConfig) -> Result<Self> {
        let client = JupiterClient::new(config).await?;
        Ok(Self { client })
    }

    /// Get token price using the client
    pub async fn get_token_price(&self, mint: &str) -> Result<TokenPrice> {
        match self.client.get_price(mint).await? {
            Some(price) => Ok(TokenPrice {
                price,
                volume_24h: None,
                market_cap: None,
            }),
            None => Err(anyhow!("No price data found for token: {}", mint))
        }
    }

    /// Get quote for a swap
    pub async fn get_quote(
        &self,
        input_mint: &str,
        output_mint: &str,
        amount: f64,
        slippage_bps: u16,
    ) -> Result<QuoteResponse> {
        let quote_request = QuoteRequest {
            inputMint: input_mint.to_string(),
            outputMint: output_mint.to_string(),
            amount: (amount * 1_000_000_000.0) as u64, // Convert to lamports
            slippageBps: slippage_bps,
        };

        let mut quote = self.client.get_quote(quote_request).await?;
        
        // Convert string fields to numeric for compatibility
        quote.in_amount = quote.inAmount.parse().unwrap_or(0.0);
        quote.out_amount = quote.outAmount.parse().unwrap_or(0.0);
        quote.price_impact_pct = quote.priceImpactPct.parse().unwrap_or(0.0);
        quote.route_plan = quote.routePlan.clone();

        Ok(quote)
    }

    /// Execute swap (placeholder for real implementation)
    pub async fn execute_swap(&self, _quote: &QuoteResponse, _wallet_name: &str) -> Result<SwapResult> {
        // This would implement the actual swap execution
        // For now, return a placeholder result
        warn!("⚠️ Swap execution not fully implemented - safety mode");
        
        Ok(SwapResult {
            success: false,
            transaction_signature: None,
            output_amount: 0.0,
            actual_slippage: 0.0,
            fee_amount: 0.0,
        })
    }

    /// Check if Jupiter is properly configured
    pub fn is_configured(&self) -> bool {
        true // Since we got here, it's configured
    }
}

/// Token price response
#[derive(Debug)]
pub struct TokenPrice {
    pub price: f64,
    pub volume_24h: Option<f64>,
    pub market_cap: Option<f64>,
}

/// Swap execution result
#[derive(Debug)]
pub struct SwapResult {
    pub success: bool,
    pub transaction_signature: Option<String>,
    pub output_amount: f64,
    pub actual_slippage: f64,
    pub fee_amount: f64,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_jupiter_price_fetch() {
        let config = JupiterConfig::default();
        let client = JupiterClient::new(&config).await.unwrap();
        
        // Test SOL price
        let sol_price = client.get_price(tokens::SOL).await.unwrap();
        assert!(sol_price.is_some());
        assert!(sol_price.unwrap() > 0.0);
    }

    #[tokio::test]
    async fn test_multiple_prices() {
        let config = JupiterConfig::default();
        let client = JupiterClient::new(&config).await.unwrap();
        
        let mints = vec![
            tokens::SOL.to_string(),
            tokens::USDC.to_string(),
            tokens::RAY.to_string(),
        ];
        
        let prices = client.get_prices(&mints).await.unwrap();
        assert!(!prices.is_empty());
        assert!(prices.contains_key(tokens::SOL));
    }

    #[tokio::test]
    async fn test_quote_request() {
        let config = JupiterConfig::default();
        let client = JupiterClient::new(&config).await.unwrap();
        
        let quote_request = QuoteRequest {
            inputMint: tokens::SOL.to_string(),
            outputMint: tokens::USDC.to_string(),
            amount: 1_000_000_000, // 1 SOL
            slippageBps: 50, // 0.5%
        };
        
        let quote = client.get_quote(quote_request).await;
        assert!(quote.is_ok());
    }
}
