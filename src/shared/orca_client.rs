// Orca Client for SniperForge - INTEGRATED VERSION
// Now using sync wrapper to avoid async/Send issues with Orca SDK

use crate::shared::orca_sync_wrapper::{OrcaSyncWrapper, OrcaQuoteRequest as SyncQuoteRequest, OrcaQuoteResponse as SyncQuoteResponse};
use anyhow::Result;
use serde::{Deserialize, Serialize};
use tracing::{info, warn, error};
use solana_sdk::pubkey::Pubkey;
use std::str::FromStr;

#[derive(Debug, Clone)]
pub struct OrcaClient {
    sync_wrapper: OrcaSyncWrapper,
    network: String,
}

// Legacy types for backward compatibility with the rest of the system
#[derive(Debug, Serialize)]
pub struct OrcaQuoteRequest {
    pub input_mint: String,
    pub output_mint: String,
    pub amount: String,
    pub slippage_bps: u16, // slippage in basis points (1% = 100 bps)
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OrcaQuoteResponse {
    pub input_amount: String,
    pub output_amount: String,
    pub price_impact_pct: Option<f64>,
    pub route: Vec<OrcaRouteStep>,
    pub fees: OrcaFees,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OrcaRouteStep {
    pub pool_id: String,
    pub token_in: String,
    pub token_out: String,
    pub fee_rate: f64,
    pub amm_type: String, // "CLMM" or "CPMM"
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OrcaFees {
    pub trading_fee: String,
    pub total_fee: String,
}

#[derive(Debug, Serialize)]
pub struct OrcaSwapRequest {
    pub quote: OrcaQuoteResponse,
    pub user_public_key: String,
    pub wrap_unwrap_sol: bool,
}

#[derive(Debug, Deserialize)]
pub struct OrcaSwapResponse {
    pub transaction: String,
    pub last_valid_block_height: u64,
}

impl OrcaClient {
    pub fn new(network: &str) -> Self {
        info!("🌊 Initializing Orca client for network: {} (using sync wrapper)", network);
        
        Self {
            sync_wrapper: OrcaSyncWrapper::new(network),
            network: network.to_string(),
        }
    }
    
    /// Get quote from Orca using the sync wrapper (main interface for the rest of the system)
    pub async fn get_quote(&self, request: &OrcaQuoteRequest) -> Result<OrcaQuoteResponse> {
        info!("🌊 Getting quote using Orca Whirlpool SDK (via sync wrapper)");
        info!("   Input: {} {} ({})", request.amount, request.input_mint, request.input_mint);
        info!("   Output: {} ({})", request.output_mint, request.output_mint);
        info!("   Slippage: {}bps", request.slippage_bps);
        
        // Convert legacy request to sync wrapper format
        let input_mint = Pubkey::from_str(&request.input_mint)
            .map_err(|e| anyhow::anyhow!("Invalid input mint: {}", e))?;
        let output_mint = Pubkey::from_str(&request.output_mint)
            .map_err(|e| anyhow::anyhow!("Invalid output mint: {}", e))?;
        let amount = request.amount.parse::<u64>()
            .map_err(|e| anyhow::anyhow!("Invalid amount: {}", e))?;
        
        let sync_request = SyncQuoteRequest {
            input_mint,
            output_mint,
            amount,
            slippage_bps: request.slippage_bps,
        };
        
        match self.sync_wrapper.get_quote(sync_request).await {
            Ok(sync_response) => {
                info!("✅ Orca quote successful:");
                info!("   Input amount: {}", sync_response.input_amount);
                info!("   Output amount: {}", sync_response.output_amount);
                info!("   Price impact: {:.2}%", sync_response.price_impact_pct);
                info!("   Estimated fee: {}", sync_response.estimated_fee);
                info!("   Route: {}", sync_response.route);
                
                // Convert sync response to legacy format
                Ok(self.convert_sync_response_to_legacy(sync_response))
            }
            Err(e) => {
                error!("❌ Orca quote failed: {}", e);
                Err(e)
            }
        }
    }
    
    /// Convert sync wrapper response to legacy format for backward compatibility
    fn convert_sync_response_to_legacy(&self, sync_response: SyncQuoteResponse) -> OrcaQuoteResponse {
        OrcaQuoteResponse {
            input_amount: sync_response.input_amount.to_string(),
            output_amount: sync_response.output_amount.to_string(),
            price_impact_pct: Some(sync_response.price_impact_pct),
            route: vec![OrcaRouteStep {
                pool_id: sync_response.route.clone(),
                token_in: "input".to_string(),
                token_out: "output".to_string(),
                fee_rate: 0.0, // TODO: Extract from sync response
                amm_type: "CLMM".to_string(),
            }],
            fees: OrcaFees {
                trading_fee: sync_response.estimated_fee.to_string(),
                total_fee: sync_response.estimated_fee.to_string(),
            },
        }
    }
    
    /// Build swap transaction using the sync wrapper
    pub async fn get_swap_transaction(&self, request: &OrcaSwapRequest) -> Result<OrcaSwapResponse> {
        self.execute_swap_internal(request, true).await
    }
    
    /// Execute a real swap (non-simulated) using the sync wrapper
    pub async fn execute_real_swap(&self, request: &OrcaSwapRequest) -> Result<OrcaSwapResponse> {
        self.execute_swap_internal(request, false).await
    }
    
    /// Internal method to handle both simulated and real swaps
    async fn execute_swap_internal(&self, request: &OrcaSwapRequest, simulate_only: bool) -> Result<OrcaSwapResponse> {
        let action_type = if simulate_only { "Building swap transaction" } else { "Executing REAL swap" };
        info!("🌊 {} using Orca Whirlpool SDK (via sync wrapper)", action_type);
        
        // Convert the request to sync wrapper format
        let user_pubkey = Pubkey::from_str(&request.user_public_key)
            .map_err(|e| anyhow::anyhow!("Invalid user public key: {}", e))?;
        
        let sync_swap_request = crate::shared::orca_sync_wrapper::OrcaSwapRequest {
            quote: crate::shared::orca_sync_wrapper::OrcaQuoteResponse {
                input_amount: request.quote.input_amount.parse().unwrap_or(0),
                output_amount: request.quote.output_amount.parse().unwrap_or(0),
                price_impact_pct: request.quote.price_impact_pct.unwrap_or(0.0),
                estimated_fee: request.quote.fees.trading_fee.parse().unwrap_or(0),
                route: request.quote.route.first().map(|r| r.pool_id.clone()).unwrap_or_default(),
            },
            user_pubkey,
            slippage_bps: 50, // Default 0.5% slippage
            simulate_only,
        };
        
        match self.sync_wrapper.execute_swap(sync_swap_request).await {
            Ok(sync_response) => {
                if sync_response.success {
                    let action_result = if sync_response.was_simulated { "simulated" } else { "executed" };
                    info!("✅ Orca swap {} successfully:", action_result);
                    info!("   Transaction signature: {}", sync_response.transaction_signature);
                    info!("   Was simulated: {}", sync_response.was_simulated);
                } else {
                    error!("❌ Orca swap failed: {}", sync_response.error_message.unwrap_or_default());
                }
                
                // Convert to legacy format
                Ok(OrcaSwapResponse {
                    transaction: sync_response.transaction_signature,
                    last_valid_block_height: 0, // Not provided by sync wrapper
                })
            }
            Err(e) => {
                error!("❌ Orca swap failed: {}", e);
                Err(e)
            }
        }
    }
    
    /// Health check for Orca connectivity using sync wrapper
    pub async fn health_check(&self) -> Result<bool> {
        info!("🔍 Checking Orca Whirlpool program accessibility on {} (via sync wrapper)", self.network);
        
        // Use sync wrapper for health check
        match self.sync_wrapper.health_check().await {
            Ok(is_healthy) => {
                if is_healthy {
                    info!("✅ Orca Whirlpool health check PASSED");
                } else {
                    warn!("⚠️ Orca Whirlpool health check FAILED");
                }
                Ok(is_healthy)
            }
            Err(e) => {
                error!("❌ Orca health check error: {}", e);
                Ok(false)
            }
        }
    }
    
    /// Convert slippage percentage to basis points
    pub fn slippage_to_bps(slippage_pct: f64) -> u16 {
        (slippage_pct * 100.0) as u16
    }
    
    /// Get supported token pairs for DevNet
    pub fn get_devnet_token_pairs() -> Vec<(&'static str, &'static str)> {
        vec![
            ("SOL", "USDC-Dev"),
            ("SOL", "USDT-Dev"),
            ("USDC-Dev", "USDT-Dev"),
        ]
    }
    
    /// Get DevNet token mints
    pub fn get_devnet_token_mint(symbol: &str) -> Option<&'static str> {
        match symbol {
            "SOL" => Some("So11111111111111111111111111111111111111112"),
            "USDC-Dev" => Some("EPjFWdd5AufqSSqeM2qN1xzybapC8G4wEGGkZwyTDt1v"),
            "USDT-Dev" => Some("Es9vMFrzaCERmJfrF4H2FYD4KCoNkY11McCe8BenwNYB"),
            "WSOL" => Some("So11111111111111111111111111111111111111112"),
            _ => None,
        }
    }
    
    /// Get the Whirlpool program ID for the network
    pub fn get_whirlpool_program_id(&self) -> &'static str {
        // Whirlpool program ID is the same on all networks
        "whirLbMiicVdio4qvUfM5KAg6Ct8VwpYzGff3uctyCc"
    }
    
    /// Get network identifier
    pub fn get_network(&self) -> &str {
        &self.network
    }
    
    /// Get token price from Orca using quotes
    /// This method gets the price by requesting a quote for a small amount and calculating the rate
    pub async fn get_price(&self, token_mint: &str) -> Result<Option<f64>> {
        info!("🌊 Getting price for token {} from Orca", token_mint);
        
        // Use SOL as reference token if the requested token is not SOL
        let (input_mint, output_mint, amount) = if token_mint == "So11111111111111111111111111111111111111112" {
            // For SOL, we'll quote SOL -> USDC to get SOL price in USD
            (
                "So11111111111111111111111111111111111111112", // SOL
                "EPjFWdd5AufqSSqeM2qN1xzybapC8G4wEGGkZwyTDt1v", // USDC-Dev
                "1000000" // 0.001 SOL (1M lamports)
            )
        } else {
            // For other tokens, quote token -> SOL
            (
                token_mint,
                "So11111111111111111111111111111111111111112", // SOL
                "1000000" // Small amount
            )
        };
        
        let quote_request = OrcaQuoteRequest {
            input_mint: input_mint.to_string(),
            output_mint: output_mint.to_string(),
            amount: amount.to_string(),
            slippage_bps: 100, // 1% slippage for price quotes
        };
        
        match self.get_quote(&quote_request).await {
            Ok(quote) => {
                let input_amount = quote.input_amount.parse::<f64>().unwrap_or(0.0);
                let output_amount = quote.output_amount.parse::<f64>().unwrap_or(0.0);
                
                if input_amount > 0.0 && output_amount > 0.0 {
                    if token_mint == "So11111111111111111111111111111111111111112" {
                        // SOL -> USDC quote, so price is output/input (USDC per SOL)
                        // Convert from lamports to SOL: divide by 1e9
                        // Convert from USDC micro-units to USDC: divide by 1e6  
                        let sol_amount = input_amount / 1e9;
                        let usdc_amount = output_amount / 1e6;
                        let price = usdc_amount / sol_amount;
                        info!("✅ SOL price from Orca: ${:.6}", price);
                        Ok(Some(price))
                    } else {
                        // Token -> SOL quote, calculate token price in USD
                        // We'd need SOL price in USD to calculate this properly
                        // For now, return the exchange rate
                        let rate = output_amount / input_amount;
                        info!("✅ Token exchange rate from Orca: {:.6} SOL per token", rate);
                        Ok(Some(rate))
                    }
                } else {
                    warn!("⚠️ Invalid quote amounts from Orca: input={}, output={}", input_amount, output_amount);
                    Ok(None)
                }
            }
            Err(e) => {
                warn!("⚠️ Failed to get price from Orca: {}", e);
                // Don't return error, just return None to indicate price unavailable
                Ok(None)
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_slippage_conversion() {
        assert_eq!(OrcaClient::slippage_to_bps(1.0), 100);
        assert_eq!(OrcaClient::slippage_to_bps(0.5), 50);
        assert_eq!(OrcaClient::slippage_to_bps(2.5), 250);
    }
    
    #[test]
    fn test_devnet_token_pairs() {
        let pairs = OrcaClient::get_devnet_token_pairs();
        assert!(!pairs.is_empty());
        assert!(pairs.contains(&("SOL", "USDC-Dev")));
    }
    
    #[test]
    fn test_devnet_token_mints() {
        assert_eq!(
            OrcaClient::get_devnet_token_mint("SOL"),
            Some("So11111111111111111111111111111111111111112")
        );
        assert_eq!(
            OrcaClient::get_devnet_token_mint("USDC-Dev"),
            Some("EPjFWdd5AufqSSqeM2qN1xzybapC8G4wEGGkZwyTDt1v")
        );
        assert_eq!(OrcaClient::get_devnet_token_mint("INVALID"), None);
    }
    
    #[tokio::test]
    async fn test_orca_client_creation() {
        let client = OrcaClient::new("devnet");
        assert_eq!(client.network, "devnet");
    }
    
    #[tokio::test]
    async fn test_orca_get_quote_with_sync_wrapper() {
        let client = OrcaClient::new("devnet");
        let request = OrcaQuoteRequest {
            input_mint: "So11111111111111111111111111111111111111112".to_string(),
            output_mint: "EPjFWdd5AufqSSqeM2qN1xzybapC8G4wEGGkZwyTDt1v".to_string(),
            amount: "1000000".to_string(),
            slippage_bps: 100,
        };
        
        let result = client.get_quote(&request).await;
        // The result can be either success (if pools found) 
        // or error (if no pools available) - both are valid in DevNet
        match result {
            Ok(quote) => {
                // If successful, verify it's a valid quote
                assert!(!quote.input_amount.is_empty());
                assert!(!quote.output_amount.is_empty());
            }
            Err(e) => {
                // If failed, should mention pools or connectivity
                let error_msg = e.to_string();
                assert!(error_msg.contains("Whirlpool") || error_msg.contains("pool") || error_msg.contains("connectivity"));
            }
        }
    }
}