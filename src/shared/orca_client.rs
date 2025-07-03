use reqwest::Client;
use serde::{Deserialize, Serialize};
use anyhow::Result;
use tracing::{info, warn, debug};

#[derive(Debug, Clone)]
pub struct OrcaClient {
    client: Client,
    base_url: String,
    network: String,
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
        let base_url = match network {
            "mainnet" => "https://api.orca.so",
            "devnet" => "https://api.devnet.orca.so",
            _ => "https://api.devnet.orca.so",
        }.to_string();
        
        info!("Initializing Orca client for network: {} at {}", network, base_url);
        
        Self {
            client: Client::new(),
            base_url,
            network: network.to_string(),
        }
    }
    
    pub async fn get_quote(&self, request: &OrcaQuoteRequest) -> Result<OrcaQuoteResponse> {
        let url = format!(
            "{}/v1/quote?inputMint={}&outputMint={}&amount={}&slippageBps={}",
            self.base_url, 
            request.input_mint, 
            request.output_mint, 
            request.amount,
            request.slippage_bps
        );
        
        debug!("Orca quote request: {}", url);
        
        let response = self.client
            .get(&url)
            .header("Accept", "application/json")
            .header("User-Agent", "SniperForge/1.0")
            .send()
            .await?;
            
        if !response.status().is_success() {
            let status = response.status();
            let error_text = response.text().await.unwrap_or_default();
            warn!("Orca API error: {} - {}", status, error_text);
            return Err(anyhow::anyhow!("Orca API error: {} - {}", status, error_text));
        }
        
        let quote: OrcaQuoteResponse = response.json().await?;
        debug!("Orca quote response: {:?}", quote);
        
        Ok(quote)
    }
    
    pub async fn get_swap_transaction(
        &self, 
        request: &OrcaSwapRequest
    ) -> Result<OrcaSwapResponse> {
        let url = format!("{}/v1/swap", self.base_url);
        
        debug!("Orca swap request to: {}", url);
        
        let response = self.client
            .post(&url)
            .header("Accept", "application/json")
            .header("Content-Type", "application/json")
            .header("User-Agent", "SniperForge/1.0")
            .json(request)
            .send()
            .await?;
            
        if !response.status().is_success() {
            let status = response.status();
            let error_text = response.text().await.unwrap_or_default();
            warn!("Orca swap API error: {} - {}", status, error_text);
            return Err(anyhow::anyhow!("Orca swap API error: {} - {}", status, error_text));
        }
        
        let swap_response: OrcaSwapResponse = response.json().await?;
        debug!("Orca swap response received, transaction length: {}", swap_response.transaction.len());
        
        Ok(swap_response)
    }
    
    pub async fn health_check(&self) -> Result<bool> {
        let url = format!("{}/v1/health", self.base_url);
        
        match self.client
            .get(&url)
            .header("User-Agent", "SniperForge/1.0")
            .send()
            .await
        {
            Ok(response) => {
                let is_healthy = response.status().is_success();
                info!("Orca health check for {}: {}", self.network, if is_healthy { "✅ OK" } else { "❌ Failed" });
                Ok(is_healthy)
            }
            Err(e) => {
                warn!("Orca health check failed: {}", e);
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
            "USDC-Dev" => Some("4zMMC9srt5Ri5X14GAgXhaHii3GnPAEERYPJgZJDncDU"),
            "USDT-Dev" => Some("Es9vMFrzaCERmJfrF4H2FYD4KCoNkY11McCe8BenwNYB"),
            "WSOL" => Some("So11111111111111111111111111111111111111112"),
            _ => None,
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
            Some("4zMMC9srt5Ri5X14GAgXhaHii3GnPAEERYPJgZJDncDU")
        );
        assert_eq!(OrcaClient::get_devnet_token_mint("INVALID"), None);
    }
    
    #[tokio::test]
    async fn test_orca_client_creation() {
        let client = OrcaClient::new("devnet");
        assert!(client.base_url.contains("devnet"));
        assert_eq!(client.network, "devnet");
    }
}
