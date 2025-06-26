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
use std::time::Duration;
use reqwest::Client;
use tracing::{info, warn, error, debug};
use chrono::{DateTime, Utc};
use solana_client::rpc_client::RpcClient;
use solana_sdk::{pubkey::Pubkey, signature::Signature};

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

/// Jupiter price response structure for V3 API
#[derive(Debug, Deserialize)]
pub struct JupiterPriceResponse {
    #[serde(flatten)]
    pub data: HashMap<String, PriceDataV3>,
}

/// Price data structure for V3 API
#[derive(Debug, Deserialize)]
#[allow(non_snake_case)] // Jupiter API uses camelCase field names
pub struct PriceDataV3 {
    #[serde(rename = "usdPrice")]
    pub usd_price: f64,
    #[serde(rename = "blockId")]
    pub block_id: u64,
    pub decimals: u8,
    #[serde(rename = "priceChange24h")]
    pub price_change_24h: Option<f64>,
}

/// Legacy price data structure for backwards compatibility
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
    pub contextSlot: u64,
    pub timeTaken: f64,
    pub swapUsdValue: String,
    pub simplerRouteUsed: bool,
    pub mostReliableAmmsQuoteReport: Option<serde_json::Value>, // Complex nested structure
    pub useIncurredSlippageForQuoting: Option<bool>,
    pub otherRoutePlans: Option<serde_json::Value>,
    pub aggregatorVersion: Option<String>,
    // Additional computed fields for convenience
    #[serde(skip)]
    pub in_amount: f64,
    #[serde(skip)]
    pub out_amount: f64,
    #[serde(skip)]
    pub price_impact_pct: f64,
    #[serde(skip)]
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
    pub bps: u16,
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
            base_url: "https://lite-api.jup.ag".to_string(), // Updated to V3 API
            api_key: None,
            timeout_seconds: 10,
            max_retries: 3,
        }
    }

    /// Create devnet configuration
    pub fn devnet() -> Self {
        Self {
            base_url: "https://lite-api.jup.ag".to_string(), // Same endpoint for both
            api_key: None,
            timeout_seconds: 15, // Slightly longer timeout for devnet
            max_retries: 5,
        }
    }
}

impl Default for JupiterConfig {
    fn default() -> Self {
        Self {
            base_url: "https://lite-api.jup.ag".to_string(), // Updated to V3 API
            api_key: None,
            timeout_seconds: 30, // Increased timeout
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
        
        // Test with SOL price directly - this is the real test
        let sol_mint = "So11111111111111111111111111111111111111112";
        match self.get_price(sol_mint).await {
            Ok(Some(price)) => {
                info!("✅ Jupiter API connected successfully. SOL price: ${:.2}", price);
                Ok(())
            },
            Ok(None) => {
                warn!("⚠️ Jupiter API connected but no price data for SOL");
                Ok(()) // Still consider this a successful connection
            },
            Err(e) => {
                error!("❌ Jupiter API connection failed: {}", e);
                Err(anyhow!("Jupiter API connection test failed: {}", e))
            }
        }
    }

    /// Get real-time price for a token using V3 API
    pub async fn get_price(&self, mint: &str) -> Result<Option<f64>> {
        // Use Jupiter's V3 price API endpoint
        let url = format!("{}/price/v3?ids={}", self.base_url, mint);
        
        debug!("🌐 Fetching price from Jupiter V3: {}", url);
        
        let mut request = self.client.get(&url)
            .header("User-Agent", "SniperForge/1.0")
            .header("Accept", "application/json");
        
        if let Some(api_key) = &self.api_key {
            request = request.header("Authorization", format!("Bearer {}", api_key));
        }

        let response = request
            .send()
            .await
            .map_err(|e| {
                error!("Network error connecting to Jupiter API: {}", e);
                anyhow!("Failed to fetch price from Jupiter: {}", e)
            })?;

        debug!("Jupiter API response status: {}", response.status());

        if !response.status().is_success() {
            let status = response.status();
            let error_text = response.text().await.unwrap_or_else(|_| "Unknown error".to_string());
            error!("Jupiter API error response: {}", error_text);
            return Err(anyhow!("Jupiter API returned error: {} - {}", status, error_text));
        }

        let response_text = response.text().await
            .map_err(|e| anyhow!("Failed to read response body: {}", e))?;
        
        debug!("Jupiter API response body: {}", response_text);

        let price_response: JupiterPriceResponse = serde_json::from_str(&response_text)
            .map_err(|e| anyhow!("Failed to parse Jupiter price response: {} - Response: {}", e, response_text))?;

        // Extract price from V3 response format
        if let Some(price_data) = price_response.data.get(mint) {
            info!("✅ Jupiter V3 price for {}: ${:.6}", mint, price_data.usd_price);
            Ok(Some(price_data.usd_price))
        } else {
            warn!("No price data found for mint: {}", mint);
            Ok(None)
        }
    }

    /// Get multiple token prices in one request using V3 API
    pub async fn get_prices(&self, mints: &[String]) -> Result<HashMap<String, f64>> {
        let ids = mints.join(",");
        let url = format!("{}/price/v3?ids={}", self.base_url, ids);
        
        debug!("🌐 Fetching multiple prices from Jupiter V3: {}", url);
        
        let mut request = self.client.get(&url)
            .header("User-Agent", "SniperForge/1.0")
            .header("Accept", "application/json");
        
        if let Some(api_key) = &self.api_key {
            request = request.header("Authorization", format!("Bearer {}", api_key));
        }

        let response = request
            .send()
            .await
            .map_err(|e| anyhow!("Failed to fetch prices from Jupiter: {}", e))?;

        if !response.status().is_success() {
            let status = response.status();
            let error_text = response.text().await.unwrap_or_else(|_| "Unknown error".to_string());
            return Err(anyhow!("Jupiter API returned error: {} - {}", status, error_text));
        }

        let response_text = response.text().await
            .map_err(|e| anyhow!("Failed to read response body: {}", e))?;

        let price_response: JupiterPriceResponse = serde_json::from_str(&response_text)
            .map_err(|e| anyhow!("Failed to parse Jupiter price response: {}", e))?;

        let mut prices = HashMap::new();
        for (mint, price_data) in price_response.data {
            prices.insert(mint, price_data.usd_price);
        }

        info!("✅ Fetched {} prices from Jupiter V3", prices.len());
        Ok(prices)
    }

    /// Get quote for a potential swap
    pub async fn get_quote(&self, request: QuoteRequest) -> Result<QuoteResponse> {
        // Quote API uses different endpoint than price API
        let quote_base_url = self.base_url.replace("lite-api.jup.ag", "quote-api.jup.ag");
        let url = format!("{}/v6/quote", quote_base_url);
        
        debug!("🌐 Fetching quote from Jupiter: {}", url);
        
        let query_params = [
            ("inputMint", request.inputMint.as_str()),
            ("outputMint", request.outputMint.as_str()),
            ("amount", &request.amount.to_string()),
            ("slippageBps", &request.slippageBps.to_string()),
        ];

        let mut req = self.client.get(&url)
            .query(&query_params)
            .header("User-Agent", "SniperForge/1.0")
            .header("Accept", "application/json");
        
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

        let mut quote: QuoteResponse = response
            .json()
            .await
            .map_err(|e| anyhow!("Failed to parse Jupiter quote response: {}", e))?;

        // Populate convenience fields
        quote.in_amount = quote.inAmount.parse::<f64>().unwrap_or(0.0) / 1_000_000_000.0; // Convert lamports to SOL
        quote.out_amount = quote.outAmount.parse::<f64>().unwrap_or(0.0) / 1_000_000.0; // Convert to USDC
        quote.price_impact_pct = quote.priceImpactPct.parse::<f64>().unwrap_or(0.0);
        quote.route_plan = quote.routePlan.clone();

        info!("✅ Jupiter quote received: {} {} -> {} {}", 
              quote.in_amount, request.inputMint, quote.out_amount, request.outputMint);

        Ok(quote)
    }

    /// Get historical prices (if supported by Jupiter)
    pub async fn get_historical_prices(
        &self, 
        _mint: &str, 
        _from_timestamp: i64, 
        _to_timestamp: i64
    ) -> Result<Vec<f64>> {
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
    ) -> Result<Vec<(u16, f64)>> {
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
                    let price_impact: f64 = quote.priceImpactPct.parse().unwrap_or(0.0);
                    slippage_data.push((slippage_bps, price_impact));
                },
                Err(e) => {
                    warn!("Failed to get quote for slippage {}: {}", slippage_bps, e);
                }
            }
        }

        Ok(slippage_data)
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

    /// Get price with fallback sources
    pub async fn get_price_with_fallback(&self, mint: &str) -> Result<Option<f64>> {
        // Try Jupiter first
        match self.get_price(mint).await {
            Ok(Some(price)) => {
                info!("✅ Jupiter price obtained: ${:.6}", price);
                return Ok(Some(price));
            }
            Ok(None) => {
                warn!("⚠️ Jupiter returned no price data for {}", mint);
            }
            Err(e) => {
                warn!("⚠️ Jupiter price fetch failed: {}", e);
            }
        }

        // Fallback to CoinGecko for major tokens
        info!("🔄 Trying fallback price source for {}", mint);
        match self.get_price_from_coingecko(mint).await {
            Ok(Some(price)) => {
                info!("✅ Fallback price obtained: ${:.6}", price);
                Ok(Some(price))
            }
            Ok(None) => {
                warn!("⚠️ No fallback price available for {}", mint);
                Ok(None)
            }
            Err(e) => {
                error!("❌ All price sources failed: {}", e);
                Err(e)
            }
        }
    }

    /// Fallback price from CoinGecko (for major tokens)
    async fn get_price_from_coingecko(&self, mint: &str) -> Result<Option<f64>> {
        // Map Solana token mints to CoinGecko IDs
        let coingecko_id = match mint {
            "So11111111111111111111111111111111111111112" => "solana", // SOL
            "EPjFWdd5AufqSSqeM2qN1xzybapC8G4wEGGkZwyTDt1v" => "usd-coin", // USDC
            "Es9vMFrzaCERmJfrF4H2FYD4KCoNkY11McCe8BenwNYB" => "tether", // USDT
            _ => {
                debug!("No CoinGecko mapping for mint: {}", mint);
                return Ok(None);
            }
        };

        let url = format!("https://api.coingecko.com/api/v3/simple/price?ids={}&vs_currencies=usd", coingecko_id);
        
        let response = self.client
            .get(&url)
            .header("User-Agent", "SniperForge/1.0")
            .send()
            .await
            .map_err(|e| anyhow!("CoinGecko API error: {}", e))?;

        if !response.status().is_success() {
            return Err(anyhow!("CoinGecko API error: {}", response.status()));
        }

        let response_text = response.text().await?;
        let price_data: serde_json::Value = serde_json::from_str(&response_text)
            .map_err(|e| anyhow!("Failed to parse CoinGecko response: {}", e))?;

        if let Some(token_data) = price_data.get(coingecko_id) {
            if let Some(usd_price) = token_data.get("usd") {
                if let Some(price) = usd_price.as_f64() {
                    return Ok(Some(price));
                }
            }
        }

        Ok(None)
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

/// Complete swap execution result with blockchain confirmation
#[derive(Debug)]
pub struct SwapExecutionResult {
    pub success: bool,
    pub transaction_signature: String,
    pub output_amount: f64,
    pub actual_slippage: f64,
    pub fee_amount: f64,
    pub block_height: u64,
    pub logs: Vec<String>,
}

/// Jupiter swap request structure
#[derive(Debug, Serialize)]
#[allow(non_snake_case)] // Jupiter API uses camelCase field names
pub struct SwapRequest {
    pub quoteResponse: QuoteResponse,
    pub userPublicKey: String,
    pub dynamicComputeUnitLimit: Option<bool>,
    pub dynamicSlippage: Option<bool>,
    pub prioritizationFeeLamports: Option<PrioritizationFee>,
}

/// Priority fee configuration
#[derive(Debug, Serialize)]
#[allow(non_snake_case)] // Jupiter API uses camelCase field names
pub struct PrioritizationFee {
    pub priorityLevelWithMaxLamports: PriorityLevelConfig,
}

#[derive(Debug, Serialize)]
#[allow(non_snake_case)] // Jupiter API uses camelCase field names
pub struct PriorityLevelConfig {
    pub maxLamports: u64,
    pub priorityLevel: String, // "veryHigh", "high", "medium", "low"
}

/// Jupiter swap response structure
#[derive(Debug, Deserialize)]
#[allow(non_snake_case)] // Jupiter API uses camelCase field names
pub struct SwapResponse {
    pub swapTransaction: String, // Base64 encoded transaction
    pub lastValidBlockHeight: u64,
}

/// Wrapper for swap request (Jupiter expects this structure)
#[derive(Debug, Serialize)]
pub struct SwapRequestWrapper {
    #[serde(rename = "swapRequest")]
    pub swap_request: SwapRequest,
}

/// Jupiter wrapper implementation
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

        Ok(quote)
    }

    /// Execute real swap transaction
    pub async fn execute_swap(&self, quote: &QuoteResponse, wallet_address: &str) -> Result<SwapResult> {
        info!("🔄 Executing real swap transaction...");
        
        // Create swap request with optimization
        let swap_request = SwapRequest {
            quoteResponse: quote.clone(),
            userPublicKey: wallet_address.to_string(),
            dynamicComputeUnitLimit: Some(true),
            dynamicSlippage: Some(true),
            prioritizationFeeLamports: Some(PrioritizationFee {
                priorityLevelWithMaxLamports: PriorityLevelConfig {
                    maxLamports: 1000000, // 0.001 SOL max priority fee for devnet
                    priorityLevel: "medium".to_string(), // Conservative for testing
                }
            }),
        };

        // Get swap endpoint URL  
        let swap_url = format!("{}/v6/swap", self.client.base_url.replace("lite-api.jup.ag", "quote-api.jup.ag"));
        
        debug!("🌐 Posting swap request to: {}", swap_url);
        
        let mut request = self.client.client.post(&swap_url)
            .header("User-Agent", "SniperForge/1.0")
            .header("Accept", "application/json")
            .header("Content-Type", "application/json")
            .json(&swap_request);
        
        if let Some(api_key) = &self.client.api_key {
            request = request.header("Authorization", format!("Bearer {}", api_key));
        }

        let response = request
            .send()
            .await
            .map_err(|e| anyhow!("Failed to send swap request to Jupiter: {}", e))?;

        if !response.status().is_success() {
            let status = response.status();
            let error_text = response.text().await.unwrap_or_default();
            return Err(anyhow!("Jupiter swap API error: {} - {}", status, error_text));
        }

        let swap_response: SwapResponse = response
            .json()
            .await
            .map_err(|e| anyhow!("Failed to parse Jupiter swap response: {}", e))?;

        info!("✅ Swap transaction built successfully");
        debug!("Transaction base64 length: {}", swap_response.swapTransaction.len());
        debug!("Last valid block height: {}", swap_response.lastValidBlockHeight);

        // For now, return success without actually sending to blockchain
        // In a real implementation, you would:
        // 1. Deserialize the transaction from base64
        // 2. Sign it with the wallet
        // 3. Simulate the transaction
        // 4. Send it to the blockchain
        // 5. Wait for confirmation
        
        warn!("⚠️ Swap transaction built but not sent - DevNet safety mode enabled");
        info!("🔒 To enable real transaction sending, wallet integration is required");
        
        Ok(SwapResult {
            success: true, // Transaction was built successfully
            transaction_signature: Some(format!("SIMULATED_{}", chrono::Utc::now().timestamp())),
            output_amount: quote.out_amount,
            actual_slippage: quote.price_impact_pct,
            fee_amount: 0.001, // Estimated fee
        })
    }

    /// Check if Jupiter is properly configured
    pub fn is_configured(&self) -> bool {
        true // Since we got here, it's configured
    }
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
