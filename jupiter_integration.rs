// ===== JUPITER INTEGRATION MODULE =====
// Integración con Jupiter API v6 para swaps reales en mainnet

use anyhow::{Result, anyhow};
use serde_json::Value;
use solana_sdk::signature::{Keypair, Signer};
use solana_sdk::pubkey::Pubkey;
use solana_sdk::transaction::Transaction;
use std::time::Duration;

use crate::types::*;

// Mainnet Jupiter API endpoints
const MAINNET_JUPITER_API: &str = "https://quote-api.jup.ag/v6";
const MAINNET_JUPITER_SWAP_API: &str = "https://quote-api.jup.ag/v6/swap";
const MAINNET_MAX_SLIPPAGE_BPS: u16 = 150;
const MAINNET_EXECUTION_TIMEOUT: u64 = 30;

pub struct JupiterIntegration;

impl JupiterIntegration {
    /// GET JUPITER QUOTE FOR MAINNET
    pub async fn get_jupiter_quote_mainnet(
        engine: &ProfessionalArbitrageEngine,
        input_mint: &Pubkey, 
        output_mint: &Pubkey, 
        amount: u64
    ) -> Result<JupiterQuote> {
        let url = format!(
            "{}/quote?inputMint={}&outputMint={}&amount={}&slippageBps={}",
            MAINNET_JUPITER_API, input_mint, output_mint, amount, MAINNET_MAX_SLIPPAGE_BPS
        );
        
        let response = engine.jupiter_client
            .get(&url)
            .timeout(Duration::from_secs(MAINNET_EXECUTION_TIMEOUT))
            .send()
            .await?;
        
        if !response.status().is_success() {
            let error_text = response.text().await?;
            return Err(anyhow!("Jupiter quote API error: {}", error_text));
        }
        
        let quote_json: Value = response.json().await?;
        
        let out_amount = quote_json["outAmount"]
            .as_str()
            .ok_or_else(|| anyhow!("Missing outAmount"))?
            .parse::<u64>()?;
        
        let price_impact_pct = quote_json["priceImpactPct"]
            .as_str()
            .unwrap_or("0")
            .parse::<f64>()
            .unwrap_or(0.0);
        
        Ok(JupiterQuote {
            out_amount,
            price_impact_pct,
            route_plan: vec!["MAINNET_REAL".to_string()],
        })
    }
    
    /// GET JUPITER SWAP TRANSACTION
    pub async fn get_jupiter_swap_transaction(
        engine: &ProfessionalArbitrageEngine,
        quote: &JupiterQuote, 
        wallet: &Keypair
    ) -> Result<Transaction> {
        let swap_request = serde_json::json!({
            "quoteResponse": {
                "outAmount": quote.out_amount.to_string(),
                "priceImpactPct": quote.price_impact_pct.to_string(),
            },
            "userPublicKey": wallet.pubkey().to_string(),
            "wrapAndUnwrapSol": true,
            "useSharedAccounts": true,
            "computeUnitPriceMicroLamports": 5000,
            "prioritizationFeeLamports": 5000,
        });
        
        let response = engine.jupiter_client
            .post(MAINNET_JUPITER_SWAP_API)
            .json(&swap_request)
            .timeout(Duration::from_secs(MAINNET_EXECUTION_TIMEOUT))
            .send()
            .await?;
        
        if !response.status().is_success() {
            let error_text = response.text().await?;
            return Err(anyhow!("Jupiter swap API error: {}", error_text));
        }
        
        let swap_response: Value = response.json().await?;
        let transaction_b64 = swap_response["swapTransaction"]
            .as_str()
            .ok_or_else(|| anyhow!("Missing swapTransaction in response"))?;
        
        // Decode base64 transaction
        use base64::Engine;
        let transaction_bytes = base64::engine::general_purpose::STANDARD.decode(transaction_b64)?;
        let transaction: Transaction = bincode::deserialize(&transaction_bytes)?;
        
        Ok(transaction)
    }
}
