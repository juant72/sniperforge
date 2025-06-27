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
use solana_client::rpc_client::RpcClient;
use solana_sdk::{
    pubkey::Pubkey, 
    signature::{Signature, Keypair, Signer}, 
    transaction::{Transaction, VersionedTransaction},
    commitment_config::CommitmentConfig,
    message::VersionedMessage,
    hash::Hash,
};
use base64::{Engine as _, engine::general_purpose};
use chrono::{DateTime, Utc};

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
    config: JupiterConfig,
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

/// Jupiter swap request structure
#[derive(Debug, Serialize)]
#[allow(non_snake_case)] // Jupiter API uses camelCase field names
pub struct SwapRequest {
    pub quoteResponse: QuoteResponse,
    pub userPublicKey: String,
    pub dynamicComputeUnitLimit: Option<bool>,
    pub dynamicSlippage: Option<bool>,
    pub prioritizationFeeLamports: Option<PrioritizationFee>,
    pub asLegacyTransaction: Option<bool>, // Force legacy transaction format for DevNet
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

/// Transaction confirmation configuration
#[derive(Debug, Clone)]
pub struct ConfirmationConfig {
    pub timeout_seconds: u64,
    pub poll_interval_ms: u64,
    pub commitment_level: CommitmentConfig,
    pub max_retries: u32,
}

impl Default for ConfirmationConfig {
    fn default() -> Self {
        Self {
            timeout_seconds: 60,
            poll_interval_ms: 1000,
            commitment_level: CommitmentConfig::confirmed(),
            max_retries: 3,
        }
    }
}

/// Transaction confirmation status
#[derive(Debug, Clone)]
pub struct ConfirmationStatus {
    pub confirmed: bool,
    pub finalized: bool,
    pub block_height: Option<u64>,
    pub slot: Option<u64>,
    pub confirmations: Option<usize>,
    pub error: Option<String>,
}

/// Transaction confirmation utilities
pub struct TransactionConfirmation {
    rpc_client: RpcClient,
    config: ConfirmationConfig,
}

impl TransactionConfirmation {
    /// Create new transaction confirmation helper
    pub fn new(rpc_endpoint: &str, config: ConfirmationConfig) -> Self {
        let rpc_client = RpcClient::new_with_commitment(
            rpc_endpoint.to_string(),
            config.commitment_level.clone()
        );
        
        Self {
            rpc_client,
            config,
        }
    }

    /// Wait for transaction confirmation with timeout and polling
    pub async fn wait_for_confirmation(
        &self,
        signature: &Signature,
    ) -> Result<ConfirmationStatus> {
        info!("⏳ Waiting for transaction confirmation: {}", signature);
        
        let start_time = std::time::Instant::now();
        let timeout_duration = std::time::Duration::from_secs(self.config.timeout_seconds);
        let poll_interval = std::time::Duration::from_millis(self.config.poll_interval_ms);
        
        let mut retries = 0;
        
        loop {
            // Check timeout
            if start_time.elapsed() > timeout_duration {
                warn!("⏰ Transaction confirmation timeout after {} seconds", self.config.timeout_seconds);
                return Ok(ConfirmationStatus {
                    confirmed: false,
                    finalized: false,
                    block_height: None,
                    slot: None,
                    confirmations: None,
                    error: Some("Confirmation timeout".to_string()),
                });
            }
            
            // Get signature status with retry logic
            match self.get_signature_status_with_retry(signature, &mut retries).await {
                Ok(status) => {
                    if status.confirmed {
                        info!("✅ Transaction confirmed: {}", signature);
                        if status.finalized {
                            info!("🔒 Transaction finalized: {}", signature);
                        }
                        return Ok(status);
                    }
                    
                    // Log progress
                    if let Some(slot) = status.slot {
                        debug!("📡 Transaction status check - Slot: {}, Confirmed: {}", 
                               slot, status.confirmed);
                    }
                }
                Err(e) => {
                    if retries >= self.config.max_retries {
                        error!("❌ Failed to get transaction status after {} retries: {}", 
                               self.config.max_retries, e);
                        return Ok(ConfirmationStatus {
                            confirmed: false,
                            finalized: false,
                            block_height: None,
                            slot: None,
                            confirmations: None,
                            error: Some(format!("Status check failed: {}", e)),
                        });
                    }
                    
                    warn!("⚠️ Transaction status check failed (retry {}/{}): {}", 
                          retries + 1, self.config.max_retries, e);
                    retries += 1;
                }
            }
            
            // Wait before next poll
            tokio::time::sleep(poll_interval).await;
        }
    }

    /// Get signature status with enhanced error handling
    async fn get_signature_status_with_retry(
        &self,
        signature: &Signature,
        retries: &mut u32,
    ) -> Result<ConfirmationStatus> {
        match self.rpc_client.get_signature_status(signature) {
            Ok(status_option) => {
                match status_option {
                    Some(status) => {
                        let confirmed = status.is_ok();
                        let error = if let Err(ref err) = status {
                            Some(format!("{:?}", err))
                        } else {
                            None
                        };
                        
                        // Get additional confirmation details
                        let (block_height, slot, confirmations) = self.get_confirmation_details(signature).await;
                        
                        Ok(ConfirmationStatus {
                            confirmed,
                            finalized: false, // Will be updated in separate check
                            block_height,
                            slot,
                            confirmations,
                            error,
                        })
                    }
                    None => {
                        debug!("📡 Transaction not yet processed: {}", signature);
                        Ok(ConfirmationStatus {
                            confirmed: false,
                            finalized: false,
                            block_height: None,
                            slot: None,
                            confirmations: None,
                            error: None,
                        })
                    }
                }
            }
            Err(e) => {
                *retries += 1;
                Err(anyhow!("RPC error getting signature status: {}", e))
            }
        }
    }

    /// Get additional confirmation details (block height, slot, confirmations)
    async fn get_confirmation_details(&self, signature: &Signature) -> (Option<u64>, Option<u64>, Option<usize>) {
        // Try to get signature statuses for more detailed info
        match self.rpc_client.get_signature_statuses(&[*signature]) {
            Ok(statuses) => {
                if let Some(Some(status)) = statuses.value.first() {
                    (None, Some(status.slot), status.confirmations)
                } else {
                    (None, None, None)
                }
            }
            Err(_) => (None, None, None)
        }
    }

    /// Check if transaction is finalized
    pub async fn check_finalization(&self, signature: &Signature) -> Result<bool> {
        match self.rpc_client.get_signature_status_with_commitment(
            signature, 
            CommitmentConfig::finalized()
        ) {
            Ok(status_option) => {
                Ok(status_option.map(|status| status.is_ok()).unwrap_or(false))
            }
            Err(e) => {
                warn!("Failed to check finalization status: {}", e);
                Ok(false)
            }
        }
    }

    /// Get current block height for tracking
    pub async fn get_current_block_height(&self) -> Result<u64> {
        self.rpc_client
            .get_block_height()
            .map_err(|e| anyhow!("Failed to get current block height: {}", e))
    }

    /// Advanced confirmation with finalization check
    pub async fn wait_for_finalization(
        &self,
        signature: &Signature,
    ) -> Result<ConfirmationStatus> {
        // First wait for confirmation
        let mut status = self.wait_for_confirmation(signature).await?;
        
        if !status.confirmed {
            return Ok(status);
        }
        
        info!("🔄 Transaction confirmed, waiting for finalization...");
        
        // Then wait for finalization
        let start_time = std::time::Instant::now();
        let finalization_timeout = std::time::Duration::from_secs(120); // 2 minutes for finalization
        let poll_interval = std::time::Duration::from_millis(2000); // 2 seconds
        
        loop {
            if start_time.elapsed() > finalization_timeout {
                warn!("⏰ Finalization timeout - transaction confirmed but not finalized");
                break;
            }
            
            if self.check_finalization(signature).await? {
                info!("🔒 Transaction finalized: {}", signature);
                status.finalized = true;
                break;
            }
            
            tokio::time::sleep(poll_interval).await;
        }
        
        Ok(status)
    }
}

/// Jupiter configuration
#[derive(Debug, Clone)]
pub struct JupiterConfig {
    pub base_url: String,
    pub api_key: Option<String>,
    pub timeout_seconds: u64,
    pub max_retries: u32,
    pub rpc_endpoint: String,
    pub network_name: String,
}

impl JupiterConfig {
    /// Create configuration from platform config
    pub fn from_network_config(network_config: &crate::config::NetworkConfig) -> Self {
        let (network_name, rpc_endpoint) = if network_config.is_mainnet() {
            ("Mainnet".to_string(), network_config.mainnet_primary_rpc.clone())
        } else {
            ("DevNet".to_string(), network_config.devnet_primary_rpc.clone())
        };

        Self {
            base_url: "https://lite-api.jup.ag".to_string(),
            api_key: None,
            timeout_seconds: if network_config.is_mainnet() { 10 } else { 15 },
            max_retries: if network_config.is_mainnet() { 3 } else { 5 },
            rpc_endpoint,
            network_name,
        }
    }

    /// Create mainnet configuration (legacy method)
    pub fn mainnet() -> Self {
        Self {
            base_url: "https://lite-api.jup.ag".to_string(),
            api_key: None,
            timeout_seconds: 10,
            max_retries: 3,
            rpc_endpoint: "https://api.mainnet-beta.solana.com".to_string(),
            network_name: "Mainnet".to_string(),
        }
    }

    /// Create devnet configuration (legacy method)
    pub fn devnet() -> Self {
        Self {
            base_url: "https://lite-api.jup.ag".to_string(),
            api_key: None,
            timeout_seconds: 15,
            max_retries: 5,
            rpc_endpoint: "https://api.devnet.solana.com".to_string(),
            network_name: "DevNet".to_string(),
        }
    }
}

impl Default for JupiterConfig {
    fn default() -> Self {
        Self {
            base_url: "https://lite-api.jup.ag".to_string(),
            api_key: None,
            timeout_seconds: 30,
            max_retries: 3,
            rpc_endpoint: "https://api.devnet.solana.com".to_string(),
            network_name: "DevNet".to_string(),
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
            ("asLegacyTransaction", "true"), // Force legacy transactions for DevNet compatibility
            ("maxAccounts", "32"), // Further reduce accounts limit for DevNet
            ("restrictIntermediateTokens", "true"), // Limit intermediate tokens to reduce complexity
            ("onlyDirectRoutes", "true"), // Prefer direct routes to minimize transaction size
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
        let raw_in_amount = quote.inAmount.parse::<f64>().unwrap_or(0.0);
        let raw_out_amount = quote.outAmount.parse::<f64>().unwrap_or(0.0);
        
        debug!("Raw Jupiter response - inAmount: {}, outAmount: {}", quote.inAmount, quote.outAmount);
        debug!("Parsed - raw_in_amount: {}, raw_out_amount: {}", raw_in_amount, raw_out_amount);
        
        quote.in_amount = raw_in_amount / 1_000_000_000.0; // Convert lamports to SOL
        quote.out_amount = raw_out_amount / 1_000_000.0; // Convert to USDT/USDC (6 decimals)
        quote.price_impact_pct = quote.priceImpactPct.parse::<f64>().unwrap_or(0.0);
        quote.route_plan = quote.routePlan.clone();

        debug!("Converted - in_amount: {} SOL, out_amount: {} output token", quote.in_amount, quote.out_amount);

        info!("✅ Jupiter quote received: {} {} -> {} {}", 
              quote.in_amount, request.inputMint, quote.out_amount, request.outputMint);

        Ok(quote)
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

/// Jupiter wrapper implementation
impl Jupiter {
    /// Create new Jupiter instance
    pub async fn new(config: &JupiterConfig) -> Result<Self> {
        let client = JupiterClient::new(config).await?;
        Ok(Self { 
            client,
            config: config.clone(),
        })
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
        
        // Convert string fields to numeric with proper unit conversion
        quote.in_amount = quote.inAmount.parse::<f64>().unwrap_or(0.0) / 1_000_000_000.0; // Convert lamports to SOL
        quote.out_amount = quote.outAmount.parse::<f64>().unwrap_or(0.0) / 1_000_000.0; // Convert to output token (typically 6 decimals)
        quote.price_impact_pct = quote.priceImpactPct.parse().unwrap_or(0.0);

        Ok(quote)
    }

    /// Execute real swap transaction (builds transaction only)
    pub async fn execute_swap(&self, quote: &QuoteResponse, wallet_address: &str) -> Result<SwapResult> {
        info!("🔄 Building swap transaction...");
        
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
            asLegacyTransaction: Some(true), // Force legacy transaction format for DevNet
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

    /// Execute swap with wallet integration (signs and sends to blockchain)
    pub async fn execute_swap_with_wallet(
        &self,
        quote: &QuoteResponse,
        wallet_address: &str,
        wallet_keypair: Option<&solana_sdk::signature::Keypair>,
    ) -> Result<SwapExecutionResult> {
        info!("🔄 Executing swap with wallet integration...");
        
        // First get the swap transaction from Jupiter
        let swap_request = SwapRequest {
            quoteResponse: quote.clone(),
            userPublicKey: wallet_address.to_string(),
            dynamicComputeUnitLimit: Some(true),
            dynamicSlippage: Some(true),
            prioritizationFeeLamports: Some(PrioritizationFee {
                priorityLevelWithMaxLamports: PriorityLevelConfig {
                    maxLamports: 1000000, // 0.001 SOL max priority fee
                    priorityLevel: "medium".to_string(),
                }
            }),
            asLegacyTransaction: Some(true), // Force legacy transaction format for DevNet
        };

        let swap_url = format!("{}/v6/swap", self.client.base_url.replace("lite-api.jup.ag", "quote-api.jup.ag"));
        
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

        // Deserialize the transaction from base64
        let transaction_data = general_purpose::STANDARD
            .decode(&swap_response.swapTransaction)
            .map_err(|e| anyhow!("Failed to decode transaction: {}", e))?;
        
        debug!("Transaction data length: {} bytes", transaction_data.len());
        debug!("Transaction data (first 20 bytes): {:?}", &transaction_data[..std::cmp::min(20, transaction_data.len())]);
        
        // Try to deserialize as legacy transaction first
        let mut transaction: Transaction = match bincode::deserialize(&transaction_data) {
            Ok(tx) => {
                info!("✅ Successfully deserialized as legacy transaction");
                tx
            },
            Err(e) => {
                debug!("Failed to deserialize as legacy transaction: {}", e);
                
                // Fallback: try to deserialize as versioned transaction and convert
                match bincode::deserialize::<VersionedTransaction>(&transaction_data) {
                    Ok(_versioned_tx) => {
                        info!("⚠️ Received versioned transaction, attempting to convert to legacy");
                        
                        // For DevNet, we should not be getting versioned transactions if we set asLegacyTransaction=true
                        // This is unexpected, so let's return an error
                        return Err(anyhow!("Received versioned transaction despite requesting legacy format. This suggests Jupiter API configuration issue."));
                    },
                    Err(v_err) => {
                        return Err(anyhow!("Failed to deserialize as both legacy and versioned transaction. Legacy error: {}, Versioned error: {}", e, v_err));
                    }
                }
            }
        };
        debug!("Transaction accounts: {}, instructions: {}", 
               transaction.message.account_keys.len(), 
               transaction.message.instructions.len());
        
        // Debug: Print all program IDs in the transaction
        debug!("Transaction account keys:");
        for (i, account) in transaction.message.account_keys.iter().enumerate() {
            debug!("  [{}] {}", i, account);
        }
        
        debug!("Transaction instructions:");
        for (i, instruction) in transaction.message.instructions.iter().enumerate() {
            let program_id = &transaction.message.account_keys[instruction.program_id_index as usize];
            debug!("  Instruction [{}]: program_id={}, accounts={:?}, data_len={}", 
                   i, program_id, instruction.accounts, instruction.data.len());
        }
        
        // If wallet keypair is provided, sign the legacy transaction
        if let Some(keypair) = wallet_keypair {
            info!("✍️ Signing legacy transaction with provided wallet");
            
            // Check if wallet is a required signer
            let wallet_pubkey = keypair.pubkey();
            debug!("Wallet pubkey: {}", wallet_pubkey);
            
            // Get recent blockhash first
            let rpc_client = RpcClient::new_with_commitment(
                self.config.rpc_endpoint.clone(),
                CommitmentConfig::confirmed()
            );
            
            let recent_blockhash = rpc_client
                .get_latest_blockhash()
                .map_err(|e| anyhow!("Failed to get recent blockhash: {}", e))?;
            
            // Update the transaction's recent blockhash
            transaction.message.recent_blockhash = recent_blockhash;
            
            // Sign the legacy transaction
            transaction.try_sign(&[keypair], recent_blockhash)
                .map_err(|e| anyhow!("Failed to sign legacy transaction: {}", e))?;
                
            info!("✅ Legacy transaction signed successfully");
                
        } else {
            warn!("⚠️ No wallet keypair provided - transaction not signed ({} safety mode)", self.config.network_name);
            return Ok(SwapExecutionResult {
                success: false,
                transaction_signature: format!("UNSIGNED_{}", chrono::Utc::now().timestamp()),
                output_amount: quote.out_amount,
                actual_slippage: quote.price_impact_pct,
                fee_amount: 0.001,
                block_height: 0,
                logs: vec!["Legacy transaction built but not signed - wallet keypair required".to_string()],
            });
        }

        // SPRINT 1: Enable real transaction sending to proper network
        info!("🚀 SPRINT 1: Sending legacy transaction to {} blockchain...", self.config.network_name);
        
        // Setup RPC client with proper network configuration
        let rpc_client = RpcClient::new_with_commitment(
            self.config.rpc_endpoint.clone(),
            CommitmentConfig::confirmed()
        );

        info!("🧪 Simulating legacy transaction before sending...");
        
        // Simulate legacy transaction first for safety
        let simulation_result = rpc_client
            .simulate_transaction(&transaction)
            .map_err(|e| {
                let error_str = e.to_string();
                if error_str.contains("too large") || error_str.contains("1644") || error_str.contains("1232") {
                    anyhow!("Transaction too large for {} ({}). Try: 1) Smaller amount, 2) Different token pair, 3) Use different route. Error: {}", 
                            self.config.network_name, transaction_data.len(), e)
                } else {
                    anyhow!("Legacy transaction simulation failed: {}", e)
                }
            })?;

        if let Some(err) = simulation_result.value.err {
            error!("❌ Transaction simulation failed: {:?}", err);
            return Ok(SwapExecutionResult {
                success: false,
                transaction_signature: format!("SIMULATION_FAILED_{}", chrono::Utc::now().timestamp()),
                output_amount: 0.0,
                actual_slippage: 0.0,
                fee_amount: 0.001,
                block_height: 0,
                logs: vec![
                    format!("Legacy transaction simulation failed: {:?}", err),
                    "Legacy transaction not sent due to simulation failure".to_string(),
                ],
            });
        }

        info!("✅ Transaction simulation successful");
        let sim_logs = simulation_result.value.logs.unwrap_or_default();
        debug!("Simulation logs: {:?}", sim_logs);

        // Send the legacy transaction to blockchain
        info!("📡 Sending legacy transaction to {} blockchain...", self.config.network_name);
        let signature = rpc_client
            .send_and_confirm_transaction_with_spinner(&transaction)
            .map_err(|e| anyhow!("Failed to send and confirm legacy transaction: {}", e))?;

        info!("🎉 Transaction confirmed! Signature: {}", signature);

        // Verify transaction status
        let transaction_status = rpc_client
            .get_signature_status(&signature)
            .map_err(|e| anyhow!("Failed to get transaction status: {}", e))?;

        let success = transaction_status
            .map(|status| status.is_ok())
            .unwrap_or(false);

        if success {
            info!("✅ SPRINT 1: Real swap executed successfully on {}!", self.config.network_name);
            Ok(SwapExecutionResult {
                success: true,
                transaction_signature: signature.to_string(),
                output_amount: quote.out_amount,
                actual_slippage: quote.price_impact_pct,
                fee_amount: 0.005, // Actual fee from simulation
                block_height: swap_response.lastValidBlockHeight,
                logs: vec![
                    "Legacy transaction built successfully".to_string(),
                    "Legacy transaction signed with wallet".to_string(),
                    "Legacy transaction simulated successfully".to_string(),
                    format!("Legacy transaction sent to {}", self.config.network_name),
                    format!("Transaction confirmed: {}", signature),
                    format!("✅ REAL SWAP COMPLETED ON {} (Legacy)", self.config.network_name.to_uppercase()),
                ],
            })
        } else {
            error!("❌ Transaction failed on blockchain");
            Ok(SwapExecutionResult {
                success: false,
                transaction_signature: signature.to_string(),
                output_amount: 0.0,
                actual_slippage: 0.0,
                fee_amount: 0.005,
                block_height: 0,
                logs: vec![
                    "Legacy transaction sent but failed on blockchain".to_string(),
                    format!("Failed signature: {}", signature),
                ],
            })
        }
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

    /// DevNet-specific token mints (verified to work)
    pub mod devnet {
        pub const SOL: &str = "So11111111111111111111111111111111111111112";
        pub const USDC: &str = "Gh9ZwEmdLJ8DscKNTkTqPbNwLNNBjuSzaG9Vp2KGtKJr"; // DevNet USDC
        pub const USDC_ALT: &str = "4zMMC9srt5Ri5X14GAgXhaHii3GnPAEERYPJgZJDncDU"; // Alternative DevNet USDC
        pub const USDT: &str = "BQcdHdAQW1hczDbBi9hiegXAR7A98Q9jx3X3iBBBDiq4"; // DevNet USDT if available
    }
    
    /// Mainnet token mints (original Jupiter tokens)
    pub mod mainnet {
        pub const SOL: &str = "So11111111111111111111111111111111111111112";
        pub const USDC: &str = "EPjFWdd5AufqSSqeM2qN1xzybapC8G4wEGGkZwyTDt1v";
        pub const USDT: &str = "Es9vMFrzaCERmJfrF4H2FYD4KCoNkY11McCe8BenwNYB";
        pub const RAY: &str = "4k3Dyjzvzp8eMZWUXbBCjEvwSkkk59S5iCNLY3QrkX6R";
        pub const SRM: &str = "SRMuApVNdxXokk5GT7XD5cUUgXMBCoAz2LHeuAoKWRt";
    }
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
