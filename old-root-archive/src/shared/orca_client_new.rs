// Orca Client for SniperForge - FIXED VERSION
// Now using sync wrapper to avoid async/Send issues with Orca SDK

use crate::shared::orca_sync_wrapper::{OrcaSyncWrapper, OrcaQuoteRequest, OrcaQuoteResponse};
use anyhow::Result;
use tracing::{info, warn, error};
use solana_sdk::pubkey::Pubkey;
use std::str::FromStr;

#[derive(Debug, Clone)]
pub struct OrcaClient {
    sync_wrapper: OrcaSyncWrapper,
    network: String,
}

impl OrcaClient {
    pub fn new(network: &str) -> Self {
        info!("🌊 Initializing Orca client for network: {}", network);
        
        Self {
            sync_wrapper: OrcaSyncWrapper::new(network),
            network: network.to_string(),
        }
    }
    
    /// Get quote from Orca using the sync wrapper
    pub async fn get_quote(&self, request: &OrcaQuoteRequest) -> Result<OrcaQuoteResponse> {
        info!("🌊 Getting quote using Orca Whirlpool SDK (via sync wrapper)");
        info!("   Input: {} {} ({})", request.amount, request.input_mint, request.input_mint);
        info!("   Output: {} ({})", request.output_mint, request.output_mint);
        info!("   Slippage: {}bps", request.slippage_bps);
        
        match self.sync_wrapper.get_quote(request.clone()).await {
            Ok(response) => {
                info!("✅ Orca quote successful:");
                info!("   Input amount: {}", response.input_amount);
                info!("   Output amount: {}", response.output_amount);
                info!("   Price impact: {:.2}%", response.price_impact_pct);
                info!("   Estimated fee: {}", response.estimated_fee);
                info!("   Route: {}", response.route);
                Ok(response)
            }
            Err(e) => {
                error!("❌ Orca quote failed: {}", e);
                Err(e)
            }
        }
    }
    
    /// Health check for Orca connectivity
    pub async fn health_check(&self) -> Result<String> {
        info!("🏥 Performing Orca health check...");
        
        // Test with a minimal SOL/USDC quote
        let sol_mint = Pubkey::from_str("So11111111111111111111111111111111111111112")
            .map_err(|e| anyhow::anyhow!("Invalid SOL mint: {}", e))?;
        let usdc_mint = Pubkey::from_str("4zMMC9srt5Ri5X14GAgXhaHii3GnPAEERYPJgZJDncDU")
            .map_err(|e| anyhow::anyhow!("Invalid USDC mint: {}", e))?;
        
        let test_request = OrcaQuoteRequest {
            input_mint: sol_mint,
            output_mint: usdc_mint,
            amount: 1_000_000, // 0.001 SOL
            slippage_bps: 50,   // 0.5%
        };
        
        match self.get_quote(&test_request).await {
            Ok(_) => {
                let status = format!("✅ Orca {} operational", self.network);
                info!("{}", status);
                Ok(status)
            }
            Err(e) => {
                let status = format!("❌ Orca {} error: {}", self.network, e);
                warn!("{}", status);
                Err(anyhow::anyhow!(status))
            }
        }
    }
    
    /// Search for available pools between two tokens
    pub async fn search_pools(
        &self,
        token_a: &str,
        token_b: &str,
    ) -> Result<Vec<String>> {
        info!("🔍 Searching for Whirlpools between {} and {}", token_a, token_b);
        
        let mint_a = Pubkey::from_str(token_a)
            .map_err(|e| anyhow::anyhow!("Invalid token A mint: {}", e))?;
        let mint_b = Pubkey::from_str(token_b)
            .map_err(|e| anyhow::anyhow!("Invalid token B mint: {}", e))?;
        
        match self.sync_wrapper.get_pools(mint_a, mint_b).await {
            Ok(pools) => {
                info!("✅ Found {} Orca pools", pools.len());
                for (i, pool) in pools.iter().enumerate() {
                    info!("   Pool {}: {}", i + 1, pool);
                }
                Ok(pools)
            }
            Err(e) => {
                warn!("⚠️ Pool search failed: {}", e);
                Err(e)
            }
        }
    }
    
    /// Get network info
    pub fn get_network(&self) -> &str {
        &self.network
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tokio;
    
    #[tokio::test]
    async fn test_orca_client_devnet() {
        let client = OrcaClient::new("devnet");
        
        // Test health check
        let health = client.health_check().await;
        println!("Health check result: {:?}", health);
        
        // Test pool search
        let pools = client.search_pools(
            "So11111111111111111111111111111111111111112", // SOL
            "4zMMC9srt5Ri5X14GAgXhaHii3GnPAEERYPJgZJDncDU"  // USDC-Dev
        ).await;
        println!("Pool search result: {:?}", pools);
    }
}
