// ===== JUPITER REAL EXECUTOR - MAINNET PRODUCTION MODULE =====
// Módulo independiente para ejecución real con Jupiter API en mainnet
// PRODUCTION-READY JUPITER INTEGRATION

use anyhow::{Result, anyhow};
use tracing::{info, warn, error};
use solana_sdk::{
    pubkey::Pubkey,
    signature::{Keypair, Signature},
    transaction::Transaction,
    instruction::Instruction,
};
use solana_client::rpc_client::RpcClient;
use serde_json::Value;
use std::time::{Duration, Instant};
use std::str::FromStr;
use crate::real_execution_engine::SwapResult;

// ===== JUPITER MAINNET CONSTANTS =====
const JUPITER_V6_API_URL: &str = "https://quote-api.jup.ag/v6";
const JUPITER_SWAP_API_URL: &str = "https://quote-api.jup.ag/v6/swap";
const MAINNET_PRIORITY_FEE_LAMPORTS: u64 = 5000; // 0.000005 SOL priority fee
const JUPITER_REQUEST_TIMEOUT: Duration = Duration::from_secs(15);

// ===== JUPITER REAL EXECUTOR IMPLEMENTATION =====
pub struct JupiterRealExecutor {
    pub api_client: reqwest::Client,
    pub wallet: Keypair,
    pub rpc_client: RpcClient,
}

impl JupiterRealExecutor {
    /// Initialize Jupiter executor for mainnet production
    pub fn new(
        api_client: reqwest::Client,
        wallet: Keypair,
        rpc_client: RpcClient,
    ) -> Result<Self> {
        info!("🚀 Initializing Jupiter Real Executor for MAINNET");
        info!("💳 Wallet: {}", wallet.pubkey());
        
        Ok(Self {
            api_client,
            wallet,
            rpc_client,
        })
    }
    
    /// Execute real swap on mainnet using Jupiter aggregator
    pub async fn execute_swap(
        &self,
        input_mint: Pubkey,
        output_mint: Pubkey,
        amount: u64,
        slippage_bps: u16,
    ) -> Result<SwapResult> {
        let start_time = Instant::now();
        info!("🔄 EXECUTING REAL SWAP ON MAINNET");
        info!("📊 {} -> {} | Amount: {} | Slippage: {}bps", 
              input_mint.to_string()[..8].to_uppercase(),
              output_mint.to_string()[..8].to_uppercase(),
              amount,
              slippage_bps);
        
        // STEP 1: Get quote from Jupiter
        info!("1️⃣  Getting Jupiter quote...");
        let quote = self.get_jupiter_quote(
            &input_mint.to_string(),
            &output_mint.to_string(),
            amount,
            slippage_bps,
        ).await?;
        
        info!("✅ Quote received: {} -> {} (impact: {:.4}%)", 
              amount, quote.out_amount, quote.price_impact_pct);
        
        // STEP 2: Get swap transaction from Jupiter
        info!("2️⃣  Building swap transaction...");
        let swap_transaction = self.get_swap_transaction(&quote).await?;
        
        // STEP 3: Sign and send transaction
        info!("3️⃣  Signing and sending transaction...");
        let signature = self.sign_and_send_transaction(swap_transaction).await?;
        
        let execution_time_ms = start_time.elapsed().as_millis() as u64;
        
        info!("✅ SWAP EXECUTED SUCCESSFULLY");
        info!("📝 Signature: {}", signature);
        info!("⏱️  Execution time: {}ms", execution_time_ms);
        
        Ok(SwapResult {
            signature,
            input_amount: amount,
            output_amount: quote.out_amount,
            price_impact: quote.price_impact_pct,
            execution_time_ms,
        })
    }
    
    /// Get quote from Jupiter API
    async fn get_jupiter_quote(
        &self,
        input_mint: &str,
        output_mint: &str,
        amount: u64,
        slippage_bps: u16,
    ) -> Result<JupiterQuoteResponse> {
        let url = format!(
            "{}/quote?inputMint={}&outputMint={}&amount={}&slippageBps={}",
            JUPITER_V6_API_URL, input_mint, output_mint, amount, slippage_bps
        );
        
        let response = self.api_client
            .get(&url)
            .timeout(JUPITER_REQUEST_TIMEOUT)
            .send()
            .await?;
        
        if !response.status().is_success() {
            let error_text = response.text().await?;
            return Err(anyhow!("Jupiter quote API error: {}", error_text));
        }
        
        let quote_json: Value = response.json().await?;
        self.parse_jupiter_quote(&quote_json)
    }
    
    /// Get swap transaction from Jupiter API
    async fn get_swap_transaction(&self, quote: &JupiterQuoteResponse) -> Result<Transaction> {
        let swap_request = serde_json::json!({
            "quoteResponse": quote.raw_response,
            "userPublicKey": self.wallet.pubkey().to_string(),
            "wrapAndUnwrapSol": true,
            "useSharedAccounts": true,
            "feeAccount": null,
            "trackingAccount": null,
            "computeUnitPriceMicroLamports": MAINNET_PRIORITY_FEE_LAMPORTS,
            "prioritizationFeeLamports": MAINNET_PRIORITY_FEE_LAMPORTS,
        });
        
        let response = self.api_client
            .post(JUPITER_SWAP_API_URL)
            .json(&swap_request)
            .timeout(JUPITER_REQUEST_TIMEOUT)
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
        let transaction_bytes = base64::decode(transaction_b64)?;
        let transaction: Transaction = bincode::deserialize(&transaction_bytes)?;
        
        Ok(transaction)
    }
    
    /// Sign and send transaction to mainnet
    async fn sign_and_send_transaction(&self, mut transaction: Transaction) -> Result<Signature> {
        // Get latest blockhash
        let latest_blockhash = self.rpc_client.get_latest_blockhash()?;
        transaction.message.recent_blockhash = latest_blockhash;
        
        // Sign transaction
        transaction.sign(&[&self.wallet], latest_blockhash);
        
        // Send and confirm transaction
        let signature = self.rpc_client
            .send_and_confirm_transaction_with_spinner_and_config(
                &transaction,
                solana_client::rpc_client::RpcSendTransactionConfig {
                    skip_preflight: false,
                    preflight_commitment: Some(solana_sdk::commitment_config::CommitmentLevel::Processed),
                    encoding: Some(solana_account_decoder::UiTransactionEncoding::Base64),
                    max_retries: Some(3),
                    min_context_slot: None,
                },
            )?;
        
        Ok(signature)
    }
    
    /// Parse Jupiter quote response
    fn parse_jupiter_quote(&self, quote_json: &Value) -> Result<JupiterQuoteResponse> {
        let out_amount = quote_json["outAmount"]
            .as_str()
            .ok_or_else(|| anyhow!("Missing outAmount"))?
            .parse::<u64>()?;
        
        let price_impact_pct = quote_json["priceImpactPct"]
            .as_str()
            .unwrap_or("0")
            .parse::<f64>()
            .unwrap_or(0.0);
        
        Ok(JupiterQuoteResponse {
            out_amount,
            price_impact_pct,
            raw_response: quote_json.clone(),
        })
    }
}

// ===== JUPITER RESPONSE TYPES =====
#[derive(Debug, Clone)]
pub struct JupiterQuoteResponse {
    pub out_amount: u64,
    pub price_impact_pct: f64,
    pub raw_response: Value,
}
