//! Jupiter API Main Integration - Enterprise Enhanced
//!
//! High-level Jupiter API integration with business logic, wallet integration,
//! and transaction execution. Configuration-driven architecture without hardcoded values.

use anyhow::{anyhow, Context, Result};
use base64::{engine::general_purpose, Engine as _};
use serde::{Deserialize, Serialize};
use solana_client::{
    rpc_client::RpcClient,
    rpc_config::RpcSendTransactionConfig,
};
use solana_sdk::{
    commitment_config::CommitmentConfig,
    pubkey::Pubkey,
    signature::{Keypair, Signature, Signer},
    transaction::VersionedTransaction,
};
use std::collections::HashMap;
use std::fs;
use std::time::{Duration, Instant};
use tracing::{error, info, warn};

use super::client::JupiterClient;
use super::config::JupiterApiConfig;
use super::types::*;
use crate::config::network::NetworkConfig;

/// Jupiter API configuration loaded from external file
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct JupiterConfigFile {
    pub jupiter_api: JupiterApiSettings,
    pub network_mapping: HashMap<String, NetworkJupiterConfig>,
    pub rate_limiting: RateLimitingConfig,
    pub trading_parameters: TradingParameters,
    pub wallet_integration: WalletIntegrationConfig,
    pub monitoring: MonitoringConfig,
    pub fallback_configuration: FallbackConfig,
    pub advanced_features: AdvancedFeatures,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct JupiterApiSettings {
    pub base_url: String,
    pub v6_api_url: String,
    pub v4_api_url: String,
    pub price_api_url: String,
    pub tokens_api_url: String,
    pub api_version: String,
    pub timeout_seconds: u64,
    pub max_retries: u32,
    pub retry_delay_ms: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NetworkJupiterConfig {
    pub enabled: bool,
    pub jupiter_program_id: Option<String>,
    pub preferred_api_version: String,
    pub max_slippage_bps: u64,
    pub priority_fee_lamports: u64,
    pub compute_unit_limit: u32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RateLimitingConfig {
    pub requests_per_second: u32,
    pub burst_allowance: u32,
    pub cooldown_period_ms: u64,
    pub backoff_multiplier: f64,
    pub max_backoff_ms: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TradingParameters {
    pub default_slippage_bps: u64,
    pub max_slippage_bps: u64,
    pub min_output_amount_buffer: f64,
    pub price_impact_threshold: f64,
    pub enable_price_impact_warnings: bool,
    pub require_exact_out: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WalletIntegrationConfig {
    pub auto_wrap_sol: bool,
    pub auto_create_ata: bool,
    pub verify_balance_before_swap: bool,
    pub min_sol_balance_lamports: u64,
    pub max_transaction_attempts: u32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MonitoringConfig {
    pub log_requests: bool,
    pub log_responses: bool,
    pub track_performance: bool,
    pub enable_metrics: bool,
    pub alert_on_errors: bool,
    pub max_error_rate: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FallbackConfig {
    pub enable_fallback: bool,
    pub fallback_dexs: Vec<String>,
    pub fallback_delay_ms: u64,
    pub max_fallback_attempts: u32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AdvancedFeatures {
    pub enable_versioned_transactions: bool,
    pub enable_priority_fees: bool,
    pub dynamic_slippage: bool,
    pub smart_routing: bool,
    pub multi_hop_optimization: bool,
    pub jupiter_v6_features: bool,
}

/// Performance metrics for Jupiter operations
#[derive(Debug, Clone)]
pub struct JupiterMetrics {
    pub total_requests: u64,
    pub successful_requests: u64,
    pub failed_requests: u64,
    pub average_response_time: Duration,
    pub last_request_time: Option<Instant>,
    pub error_rate: f64,
}

impl Default for JupiterMetrics {
    fn default() -> Self {
        Self {
            total_requests: 0,
            successful_requests: 0,
            failed_requests: 0,
            average_response_time: Duration::from_millis(0),
            last_request_time: None,
            error_rate: 0.0,
        }
    }
}

/// Main Jupiter API wrapper with business logic and configuration-driven architecture
pub struct Jupiter {
    pub client: JupiterClient,
    config: JupiterConfigFile,
    network_config: NetworkConfig,
    network_name: String,
    metrics: JupiterMetrics,
    rpc_client: Option<RpcClient>,
}

impl Jupiter {
    /// Create new Jupiter instance from configuration files
    pub async fn from_config(network_name: &str) -> Result<Self> {
        let config = Self::load_config()?;
        let network_config = NetworkConfig::from_config(network_name)?;
        
        Self::new_with_configs(config, network_config, network_name.to_string()).await
    }

    /// Create new Jupiter instance with explicit configurations
    pub async fn new_with_configs(
        config: JupiterConfigFile,
        network_config: NetworkConfig,
        network_name: String,
    ) -> Result<Self> {
        // Validate network is supported
        let network_jupiter_config = config.network_mapping.get(&network_name)
            .context(format!("Network '{}' not found in Jupiter configuration", network_name))?;

        if !network_jupiter_config.enabled {
            return Err(anyhow!("Jupiter is not enabled for network: {}", network_name));
        }

        // Validate that Jupiter is available for this network
        if !network_config.is_dex_available("jupiter") {
            return Err(anyhow!(
                "Jupiter is not available for network: {}",
                network_config.network
            ));
        }

        // Create Jupiter client config
        let client_config = JupiterApiConfig {
            enabled: true,
            base_url: config.jupiter_api.base_url.clone(),
            api_key: None,
            timeout_seconds: config.jupiter_api.timeout_seconds,
            max_retries: config.jupiter_api.max_retries,
            rate_limit_rps: config.rate_limiting.requests_per_second,
            rpc_endpoint: network_config.rpc_endpoint.clone(),
            network_name: network_name.clone(),
        };

        let client = JupiterClient::new(client_config)?;

        info!(
            "🌐 Jupiter initialized for {} network",
            network_config.network
        );
        info!(
            "🔧 Jupiter Program ID: {:?}",
            network_config.program_ids.jupiter_program
        );

        Ok(Self {
            client,
            config,
            network_config,
            network_name,
            metrics: JupiterMetrics::default(),
            rpc_client: None,
        })
    }

    /// Load Jupiter configuration from file
    fn load_config() -> Result<JupiterConfigFile> {
        let config_path = "config/jupiter_config.json";
        let content = fs::read_to_string(config_path)
            .context(format!("Failed to read Jupiter configuration: {}", config_path))?;
        
        let config: JupiterConfigFile = serde_json::from_str(&content)
            .context("Failed to parse Jupiter configuration")?;
        
        Ok(config)
    }

    /// Set RPC client for transaction operations
    pub fn with_rpc_client(mut self, rpc_client: RpcClient) -> Self {
        self.rpc_client = Some(rpc_client);
        self
    }

    /// Get network-specific Jupiter configuration
    pub fn get_network_config(&self) -> &NetworkJupiterConfig {
        self.config.network_mapping.get(&self.network_name)
            .expect("Network config should be validated during construction")
    }

    /// Get quote for a swap
    pub async fn get_quote(&mut self, request: &QuoteRequest) -> Result<JupiterQuoteResponse> {
        let start_time = Instant::now();
        self.metrics.total_requests += 1;

        // Apply network-specific parameters
        let mut adjusted_request = request.clone();
        let network_config = self.get_network_config();
        
        // Apply max slippage if not specified
        if adjusted_request.slippage_bps.is_none() {
            adjusted_request.slippage_bps = Some(network_config.max_slippage_bps as u16);
        }

        let result = self.client.get_quote(&adjusted_request).await;

        // Update metrics
        let response_time = start_time.elapsed();
        self.update_metrics(result.is_ok(), response_time);

        match result {
            Ok(quote) => {
                if self.config.monitoring.log_requests {
                    info!("📊 Quote received: {} {} → {} {}", 
                        adjusted_request.amount, 
                        adjusted_request.input_mint,
                        quote.out_amount, 
                        adjusted_request.output_mint
                    );
                }

                // Check price impact if enabled
                if self.config.trading_parameters.enable_price_impact_warnings {
                    if let Some(route_plan) = quote.route_plan.first() {
                        let price_impact = route_plan.swap_info.price_impact_pct.unwrap_or(0.0);
                        if price_impact > self.config.trading_parameters.price_impact_threshold {
                            warn!("⚠️ High price impact detected: {:.2}%", price_impact * 100.0);
                        }
                    }
                }

                Ok(quote)
            }
            Err(e) => {
                if self.config.monitoring.alert_on_errors {
                    error!("❌ Quote request failed: {}", e);
                }
                Err(e)
            }
        }
    }

    /// Get swap transaction with full validation and configuration
    pub async fn get_swap_transaction(
        &mut self,
        quote: &JupiterQuoteResponse,
        user_public_key: &Pubkey,
    ) -> Result<String> {
        let start_time = Instant::now();
        self.metrics.total_requests += 1;

        // Validate quote is still valid
        if quote.route_plan.is_empty() {
            return Err(anyhow!("Invalid quote: no route plan available"));
        }

        // Apply network-specific parameters
        let network_config = self.get_network_config();
        
        // Create swap request with enterprise features
        let swap_request = self.build_swap_request(quote, user_public_key, network_config)?;

        let result = self.client.get_swap_transaction(&swap_request).await;

        // Update metrics
        let response_time = start_time.elapsed();
        self.update_metrics(result.is_ok(), response_time);

        match result {
            Ok(transaction_data) => {
                if self.config.monitoring.log_requests {
                    info!("🔄 Swap transaction prepared for {} -> {}", 
                        quote.input_mint, quote.output_mint);
                }

                // Validate transaction before returning
                self.validate_transaction(&transaction_data)?;
                
                Ok(transaction_data)
            }
            Err(e) => {
                if self.config.monitoring.alert_on_errors {
                    error!("❌ Swap transaction preparation failed: {}", e);
                }
                Err(e)
            }
        }
    }

    /// Execute a complete swap with wallet integration and enterprise safety checks
    pub async fn execute_swap(
        &mut self,
        input_mint: &str,
        output_mint: &str,
        amount: u64,
        wallet: &Keypair,
    ) -> Result<Signature> {
        info!("🚀 Starting enterprise swap execution: {} {} -> {}", 
            amount, input_mint, output_mint);

        // Pre-flight safety checks
        self.validate_swap_parameters(input_mint, output_mint, amount)?;
        
        // Validate wallet balance if configured
        if self.config.wallet_integration.verify_balance_before_swap {
            self.validate_wallet_balance(wallet, amount).await?;
        }

        // Get quote with retries
        let quote_request = QuoteRequest::new(
            input_mint.to_string(),
            output_mint.to_string(),
            amount
        )
        .with_slippage_bps(self.config.trading_parameters.default_slippage_bps as u16)
        .with_user_public_key(wallet.pubkey().to_string());

        let quote = self.get_quote(&quote_request).await
            .context("Failed to get swap quote")?;

        // Validate quote meets requirements
        self.validate_quote_requirements(&quote)?;

        // Get swap transaction
        let transaction_data = self.get_swap_transaction(&quote, &wallet.pubkey()).await
            .context("Failed to get swap transaction")?;

        // Execute with retry logic and monitoring
        let signature = self.execute_transaction(&transaction_data, wallet).await
            .context("Failed to execute swap transaction")?;

        info!("✅ Swap executed successfully: {}", signature);
        Ok(signature)
    }

    /// Execute transaction with enterprise retry logic and monitoring
    async fn execute_transaction(&self, transaction_data: &str, wallet: &Keypair) -> Result<Signature> {
        let rpc_client = self.rpc_client.as_ref()
            .context("RPC client not configured for transaction execution")?;

        // Decode the transaction
        let transaction_bytes = general_purpose::STANDARD
            .decode(transaction_data)
            .context("Failed to decode transaction data")?;

        let mut versioned_transaction: VersionedTransaction = bincode::deserialize(&transaction_bytes)
            .context("Failed to deserialize transaction")?;

        // Sign the transaction using the Solana SDK approach
        let message = versioned_transaction.message.clone();
        let signers = vec![wallet];
        let signatures = signers.iter().map(|signer| signer.sign_message(&message.serialize())).collect();
        versioned_transaction.signatures = signatures;

        let max_attempts = self.config.wallet_integration.max_transaction_attempts;
        let mut last_error = None;

        for attempt in 1..=max_attempts {
            info!("📡 Sending transaction attempt {}/{}", attempt, max_attempts);

            match rpc_client.send_and_confirm_transaction_with_spinner_and_config(
                &versioned_transaction,
                CommitmentConfig::confirmed(),
                RpcSendTransactionConfig {
                    skip_preflight: false,
                    preflight_commitment: Some(CommitmentConfig::processed().commitment),
                    encoding: None,
                    max_retries: Some(0), // We handle retries at higher level
                    min_context_slot: None,
                },
            ) {
                Ok(signature) => {
                    info!("✅ Transaction confirmed: {} (attempt {})", signature, attempt);
                    return Ok(signature);
                }
                Err(e) => {
                    warn!("⚠️ Transaction attempt {} failed: {}", attempt, e);
                    last_error = Some(e);

                    if attempt < max_attempts {
                        let delay = Duration::from_millis(1000 * attempt as u64);
                        info!("⏳ Waiting {:?} before retry...", delay);
                        tokio::time::sleep(delay).await;

                        // Re-sign with fresh message for retry
                        let fresh_message = versioned_transaction.message.clone();
                        let fresh_signatures = signers.iter().map(|signer| signer.sign_message(&fresh_message.serialize())).collect();
                        versioned_transaction.signatures = fresh_signatures;
                    }
                }
            }
        }

        Err(anyhow!(
            "Transaction failed after {} attempts. Last error: {:?}",
            max_attempts,
            last_error
        ))
    }

    /// Validate wallet balance before swap
    async fn validate_wallet_balance(&self, wallet: &Keypair, _amount: u64) -> Result<()> {
        let rpc_client = self.rpc_client.as_ref()
            .context("RPC client not configured for balance validation")?;

        let balance = rpc_client.get_balance(&wallet.pubkey())
            .context("Failed to get wallet balance")?;

        let min_balance = self.config.wallet_integration.min_sol_balance_lamports;
        if balance < min_balance {
            return Err(anyhow!(
                "Insufficient SOL balance: {} < {} required",
                balance,
                min_balance
            ));
        }

        Ok(())
    }

    /// Update performance metrics
    fn update_metrics(&mut self, success: bool, response_time: Duration) {
        if success {
            self.metrics.successful_requests += 1;
        } else {
            self.metrics.failed_requests += 1;
        }

        // Update average response time (simple moving average)
        // Fix: Handle the case where total_requests might be 0 or 1 safely
        if self.metrics.total_requests <= 1 {
            self.metrics.average_response_time = response_time;
        } else {
            // Safe calculation: total_requests is guaranteed to be > 1 here
            let previous_requests = self.metrics.total_requests - 1;
            let total_time = self.metrics.average_response_time.as_millis() as f64 * previous_requests as f64;
            let new_average = (total_time + response_time.as_millis() as f64) / self.metrics.total_requests as f64;
            self.metrics.average_response_time = Duration::from_millis(new_average as u64);
        }

        self.metrics.last_request_time = Some(Instant::now());
        
        // Safe division: ensure total_requests is not 0
        if self.metrics.total_requests > 0 {
            self.metrics.error_rate = self.metrics.failed_requests as f64 / self.metrics.total_requests as f64;
        } else {
            self.metrics.error_rate = 0.0;
        }

        // Alert on high error rate
        if self.config.monitoring.alert_on_errors && 
           self.metrics.error_rate > self.config.monitoring.max_error_rate &&
           self.metrics.total_requests >= 10 {
            warn!("🚨 High error rate detected: {:.1}%", self.metrics.error_rate * 100.0);
        }
    }

    /// Get current performance metrics
    pub fn get_metrics(&self) -> &JupiterMetrics {
        &self.metrics
    }

    /// Get network name - ENTERPRISE ACCESSOR
    pub fn get_network_name(&self) -> &str {
        &self.network_name
    }

    /// Get network configuration - ENTERPRISE ACCESSOR
    pub fn get_network_configuration(&self) -> &NetworkConfig {
        &self.network_config
    }

    /// Load Jupiter configuration from file - ENTERPRISE ACCESSOR
    pub fn load_jupiter_config() -> Result<JupiterConfigFile> {
        Self::load_config()
    }

    /// Update performance metrics - ENTERPRISE ACCESSOR
    pub fn update_performance_metrics(&mut self, success: bool, response_time: Duration) {
        // Increment total requests first to maintain consistency with internal flow
        self.metrics.total_requests += 1;
        self.update_metrics(success, response_time);
    }

    /// Get configuration - ENTERPRISE ACCESSOR
    pub fn get_configuration(&self) -> &JupiterConfigFile {
        &self.config
    }

    /// Check if Jupiter is healthy (simplified version)
    pub async fn health_check(&mut self) -> Result<bool> {
        // Simple health check - just verify config is loaded
        Ok(!self.config.jupiter_api.base_url.is_empty())
    }

    /// Get supported tokens for current network
    pub fn get_supported_tokens(&self) -> Vec<&crate::config::network::TokenInfo> {
        self.network_config.get_tradeable_tokens()
    }

    /// Check if token pair is supported
    pub fn is_token_pair_supported(&self, input_mint: &str, output_mint: &str) -> bool {
        let tokens = self.network_config.get_tradeable_tokens();
        let input_supported = tokens.iter().any(|token| token.address == input_mint);
        let output_supported = tokens.iter().any(|token| token.address == output_mint);
        
        input_supported && output_supported
    }

    /// Validate swap parameters before execution (enterprise safety)
    fn validate_swap_parameters(&self, input_mint: &str, output_mint: &str, amount: u64) -> Result<()> {
        // Check if tokens are supported
        if !self.is_token_pair_supported(input_mint, output_mint) {
            return Err(anyhow!(
                "Token pair not supported: {} -> {}",
                input_mint,
                output_mint
            ));
        }

        // Validate amount is reasonable
        if amount == 0 {
            return Err(anyhow!("Swap amount cannot be zero"));
        }

        // Check network configuration allows trading
        let network_config = self.get_network_config();
        if !network_config.enabled {
            return Err(anyhow!("Jupiter is disabled for network: {}", self.network_name));
        }

        Ok(())
    }

    /// Validate quote meets enterprise requirements
    fn validate_quote_requirements(&self, quote: &JupiterQuoteResponse) -> Result<()> {
        // Check if we have a valid route
        if quote.route_plan.is_empty() {
            return Err(anyhow!("Quote has no valid routes"));
        }

        // Validate price impact is acceptable
        if let Some(route_plan) = quote.route_plan.first() {
            let price_impact = route_plan.swap_info.price_impact_pct.unwrap_or(0.0);
            if price_impact > self.config.trading_parameters.price_impact_threshold {
                return Err(anyhow!(
                    "Price impact too high: {:.2}% > {:.2}%",
                    price_impact * 100.0,
                    self.config.trading_parameters.price_impact_threshold * 100.0
                ));
            }
        }

        // Validate output amount meets minimum requirements
        let out_amount = quote.out_amount.parse::<u64>()
            .map_err(|_| anyhow!("Invalid output amount in quote"))?;
        
        if out_amount == 0 {
            return Err(anyhow!("Quote output amount is zero"));
        }

        Ok(())
    }

    /// Build swap request with enterprise configuration
    fn build_swap_request(
        &self,
        quote: &JupiterQuoteResponse,
        user_public_key: &Pubkey,
        network_config: &NetworkJupiterConfig,
    ) -> Result<SwapRequest> {
        Ok(SwapRequest {
            quote_response: quote.clone(),
            user_public_key: user_public_key.to_string(),
            wrap_and_unwrap_sol: self.config.wallet_integration.auto_wrap_sol,
            compute_unit_price_micro_lamports: Some(network_config.priority_fee_lamports),
            auto_create_account_associated_tokens: self.config.wallet_integration.auto_create_ata,
            dynamic_compute_unit_limit: true,
            priority_fee_lamports: Some(network_config.priority_fee_lamports),
        })
    }

    /// Validate transaction before execution
    fn validate_transaction(&self, transaction_data: &str) -> Result<()> {
        // Basic validation - ensure transaction data is not empty
        if transaction_data.is_empty() {
            return Err(anyhow!("Transaction data is empty"));
        }

        // Validate base64 encoding
        if general_purpose::STANDARD.decode(transaction_data).is_err() {
            return Err(anyhow!("Invalid transaction data encoding"));
        }

        Ok(())
    }
}

/// Swap request structure for Jupiter API
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SwapRequest {
    #[serde(rename = "quoteResponse")]
    pub quote_response: JupiterQuoteResponse,
    #[serde(rename = "userPublicKey")]
    pub user_public_key: String,
    #[serde(rename = "wrapAndUnwrapSol")]
    pub wrap_and_unwrap_sol: bool,
    #[serde(rename = "computeUnitPriceMicroLamports")]
    pub compute_unit_price_micro_lamports: Option<u64>,
    #[serde(rename = "autoCreateAccountAssociatedTokens")]
    pub auto_create_account_associated_tokens: bool,
    #[serde(rename = "dynamicComputeUnitLimit")]
    pub dynamic_compute_unit_limit: bool,
    #[serde(rename = "priorityFeeLamports")]
    pub priority_fee_lamports: Option<u64>,
}

// Builder pattern for Jupiter
pub struct JupiterBuilder {
    network_name: Option<String>,
    config: Option<JupiterConfigFile>,
    network_config: Option<NetworkConfig>,
    rpc_client: Option<RpcClient>,
}

impl JupiterBuilder {
    pub fn new() -> Self {
        Self {
            network_name: None,
            config: None,
            network_config: None,
            rpc_client: None,
        }
    }

    pub fn network(mut self, network_name: &str) -> Self {
        self.network_name = Some(network_name.to_string());
        self
    }

    pub fn config(mut self, config: JupiterConfigFile) -> Self {
        self.config = Some(config);
        self
    }

    pub fn network_config(mut self, network_config: NetworkConfig) -> Self {
        self.network_config = Some(network_config);
        self
    }

    pub fn rpc_client(mut self, rpc_client: RpcClient) -> Self {
        self.rpc_client = Some(rpc_client);
        self
    }

    pub async fn build(self) -> Result<Jupiter> {
        let network_name = self.network_name.context("Network name is required")?;
        
        let config = match self.config {
            Some(config) => config,
            None => Jupiter::load_config()?,
        };

        let network_config = match self.network_config {
            Some(network_config) => network_config,
            None => NetworkConfig::from_config(&network_name)?,
        };

        let mut jupiter = Jupiter::new_with_configs(config, network_config, network_name).await?;

        if let Some(rpc_client) = self.rpc_client {
            jupiter = jupiter.with_rpc_client(rpc_client);
        }

        Ok(jupiter)
    }
}

impl Default for JupiterBuilder {
    fn default() -> Self {
        Self::new()
    }
}
