use anyhow::Result;
use serde::{Deserialize, Serialize};
use tracing::{info, warn, debug};

#[derive(Debug, Clone)]
pub struct OrcaClient {
    network: String,
    rpc_url: String,
}

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
        let rpc_url = match network {
            "mainnet" => "https://api.mainnet-beta.solana.com",
            "devnet" => "https://api.devnet.solana.com",
            "localhost" => "http://localhost:8899",
            _ => "https://api.devnet.solana.com",
        }.to_string();
        
        info!("Initializing Orca client for network: {} using RPC: {}", network, rpc_url);
        
        Self {
            rpc_url,
            network: network.to_string(),
        }
    }
    
    pub async fn get_quote(&self, request: &OrcaQuoteRequest) -> Result<OrcaQuoteResponse> {
        // TODO: Implement actual Orca Whirlpool SDK integration
        // For now, return a detailed error explaining the proper approach
        
        warn!("❌ Orca client needs proper Whirlpool SDK integration");
        warn!("🔍 DISCOVERY: Orca doesn't have REST API like Jupiter!");
        warn!("✅ SOLUTION: Orca uses on-chain program calls via Solana RPC");
        warn!("📋 Required: 1) Whirlpool SDK, 2) Direct Solana RPC calls, 3) On-chain quote calculation");
        
        info!("💡 This is why we got 403 error - the endpoint doesn't exist!");
        info!("🎯 Next steps: Integrate Orca Whirlpool Rust SDK instead of REST calls");
        
        Err(anyhow::anyhow!(
            "Orca API requires Whirlpool SDK integration, not REST calls. \
            Orca calculates quotes on-chain via Solana RPC ({}), not through REST endpoints. \
            The 403 error was caused by non-existent API endpoints. \
            Network: {}", 
            self.rpc_url,
            self.network
        ))
    }
    
    pub async fn get_swap_transaction(
        &self, 
        request: &OrcaSwapRequest
    ) -> Result<OrcaSwapResponse> {
        warn!("❌ Orca swap transaction requires Whirlpool SDK integration");
        warn!("📋 Required: 1) Solana wallet integration, 2) Whirlpool program calls, 3) Transaction building");
        
        Err(anyhow::anyhow!(
            "Orca swap transaction requires proper Whirlpool SDK integration. \
            Network: {}, RPC: {}", 
            self.network, 
            self.rpc_url
        ))
    }
    
    /// Check if Orca Whirlpool program is accessible on the network
    pub async fn health_check(&self) -> Result<bool> {
        info!("Checking Orca Whirlpool program accessibility on {}", self.network);
        
        // For Orca, we should check if we can access the Whirlpool program
        // and if Solana RPC is responding, not a REST API endpoint
        warn!("❌ Orca health check needs proper RPC client integration");
        warn!("📋 Should check: 1) Solana RPC connectivity, 2) Whirlpool program accessibility");
        
        // For now, return false to indicate this needs proper implementation
        info!("💡 Once properly implemented, this will check Whirlpool program via RPC: {}", self.rpc_url);
        Ok(false)
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
            "USDC-Dev" => Some("4zMMC9srt5Ri5X14GAgXhaHii3GnPAEERYPJgZJDncDU"),
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
    
    /// Get recommended RPC endpoint for the network
    pub fn get_rpc_url(&self) -> &str {
        &self.rpc_url
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
            Some("4zMMC9srt5Ri5X14GAgXhaHii3GnPAEERYPJgZJDncDU")
        );
        assert_eq!(OrcaClient::get_devnet_token_mint("INVALID"), None);
    }
    
    #[tokio::test]
    async fn test_orca_client_creation() {
        let client = OrcaClient::new("devnet");
        assert!(client.rpc_url.contains("devnet"));
        assert_eq!(client.network, "devnet");
    }
    
    #[tokio::test]
    async fn test_orca_get_quote_explains_issue() {
        let client = OrcaClient::new("devnet");
        let request = OrcaQuoteRequest {
            input_mint: "So11111111111111111111111111111111111111112".to_string(),
            output_mint: "4zMMC9srt5Ri5X14GAgXhaHii3GnPAEERYPJgZJDncDU".to_string(),
            amount: "1000000".to_string(),
            slippage_bps: 100,
        };
        
        let result = client.get_quote(&request).await;
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("Whirlpool SDK integration"));
    }
}