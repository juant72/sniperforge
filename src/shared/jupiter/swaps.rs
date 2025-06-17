use anyhow::{Result, anyhow};
use base64::{engine::general_purpose, Engine as _};
use serde_json::Value;
use solana_client::rpc_client::RpcClient;
use solana_sdk::{signature::Keypair, signer::Signer, transaction::Transaction};
use tracing::{info, warn, error, debug};
use rand::Rng;
use bincode;


use super::JupiterClient;
use crate::shared::jupiter::JupiterQuote;

/// Result of a swap execution
#[derive(Debug, Clone)]
pub struct SwapResult {
    pub success: bool,
    pub transaction_signature: Option<String>,
    pub input_amount: u64,
    pub output_amount: u64,
    pub price_impact: f64,
    pub slippage: f64,
    pub gas_fee: f64,
    pub error_message: Option<String>,
    pub is_paper_trade: bool,
    pub execution_time_ms: u64,
    pub route_info: RouteInfo,
}

#[derive(Debug, Clone)]
pub struct RouteInfo {
    pub dexes_used: Vec<String>,
    pub number_of_trades: usize,
}

/// Jupiter Swap Service
/// Handles actual swap execution on Solana
pub struct JupiterSwapService {
    client: JupiterClient,
    rpc_client: RpcClient,
}

impl JupiterSwapService {
    pub fn new(client: JupiterClient, rpc_url: &str) -> Self {
        let rpc_client = RpcClient::new(rpc_url.to_string());
        
        Self {
            client,
            rpc_client,
        }
    }

    /// Execute swap on DevNet
    pub async fn execute_devnet_swap(
        &self,
        quote: &JupiterQuote,
        user_keypair: &Keypair,
    ) -> Result<SwapResult> {
        let start_time = std::time::Instant::now();
        
        info!("🔄 Executing REAL swap on DevNet");
        info!("   Route: {} -> {}", quote.input_mint, quote.output_mint);
        info!("   Amount: {} -> {}", quote.in_amount, quote.out_amount);
        info!("   Price Impact: {}%", quote.price_impact_pct);

        // Get swap transaction from Jupiter
        let swap_response = self.client.get_swap_transaction(
            &serde_json::to_value(quote)?,
            &user_keypair.pubkey().to_string(),
        ).await?;

        // Decode the transaction from Jupiter response
        // Jupiter API returns: { "swapTransaction": "base64_encoded_transaction" }
        let tx_base64 = swap_response["swapTransaction"]
            .as_str()
            .ok_or_else(|| anyhow::anyhow!("Missing swapTransaction field in response"))?;
        let tx_bytes = general_purpose::STANDARD.decode(tx_base64)?;
        let mut transaction: Transaction = bincode::deserialize(&tx_bytes)?;
        
        // Sign the transaction
        transaction.try_sign(&[user_keypair], self.rpc_client.get_latest_blockhash()?)?;

        // Submit transaction
        debug!("📡 Submitting transaction to DevNet...");
        
        match self.rpc_client.send_and_confirm_transaction(&transaction) {
            Ok(signature) => {
                let execution_time = start_time.elapsed().as_millis() as u64;
                
                info!("✅ DevNet swap executed successfully!");
                info!("   Signature: {}", signature);
                info!("   Execution time: {} ms", execution_time);

                Ok(SwapResult {
                    success: true,
                    transaction_signature: Some(signature.to_string()),
                    input_amount: quote.in_amount.parse().unwrap_or(0),
                    output_amount: quote.out_amount.parse().unwrap_or(0),
                    price_impact: quote.price_impact_pct.parse().unwrap_or(0.0),
                    slippage: quote.slippage_bps as f64 / 100.0,
                    route_info: RouteInfo {
                        dexes_used: quote.route_plan.iter()
                            .map(|r| r.swap_info.label.clone())
                            .collect(),
                        number_of_trades: quote.route_plan.len(),
                    },
                    gas_fee: 0.005, // Actual gas fee would be calculated
                    error_message: None,
                    is_paper_trade: false,
                    execution_time_ms: execution_time,
                })
            }
            Err(e) => {
                let execution_time = start_time.elapsed().as_millis() as u64;
                
                error!("❌ DevNet swap failed: {}", e);
                
                Ok(SwapResult {
                    success: false,
                    transaction_signature: None,
                    input_amount: quote.in_amount.parse().unwrap_or(0),
                    output_amount: 0,
                    price_impact: quote.price_impact_pct.parse().unwrap_or(0.0),
                    slippage: quote.slippage_bps as f64 / 100.0,
                    route_info: RouteInfo {
                        dexes_used: quote.route_plan.iter()
                            .map(|r| r.swap_info.label.clone())
                            .collect(),
                        number_of_trades: quote.route_plan.len(),
                    },
                    gas_fee: 0.0,
                    error_message: Some(e.to_string()),
                    is_paper_trade: false,
                    execution_time_ms: execution_time,
                })
            }
        }
    }

    /// Execute paper trade (simulation)
    pub async fn execute_paper_trade(
        &self,
        quote: &JupiterQuote,
        slippage: f64,
    ) -> Result<SwapResult> {
        let start_time = std::time::Instant::now();
        
        info!("📝 Executing paper trade (simulation)");
        info!("   Route: {} -> {}", quote.input_mint, quote.output_mint);
        info!("   Amount: {} -> {}", quote.in_amount, quote.out_amount);
        info!("   Price Impact: {}%", quote.price_impact_pct);

        // Simulate execution delay
        tokio::time::sleep(tokio::time::Duration::from_millis(50 + rand::rng().random_range(0..100))).await;
        
        // Simulate success/failure
        let success = rand::rng().random_range(0..100) < 95; // 95% success rate in simulation
        let execution_time = start_time.elapsed().as_millis() as u64;
        
        if success {
            info!("✅ Paper trade executed successfully!");
            info!("   Simulated execution time: {} ms", execution_time);
            
            Ok(SwapResult {
                success: true,
                transaction_signature: Some(format!("SIMULATED_{}", rand::rng().random_range(1000000..9999999))),
                input_amount: quote.in_amount.parse().unwrap_or(0),
                output_amount: quote.out_amount.parse().unwrap_or(0),
                price_impact: quote.price_impact_pct.parse().unwrap_or(0.0),
                slippage,
                route_info: RouteInfo {
                    dexes_used: quote.route_plan.iter()
                        .map(|r| r.swap_info.label.clone())
                        .collect(),
                    number_of_trades: quote.route_plan.len(),
                },
                gas_fee: 0.005,
                error_message: None,
                is_paper_trade: true,
                execution_time_ms: execution_time,
            })
        } else {
            warn!("❌ Paper trade simulation failed");
            
            Ok(SwapResult {
                success: false,
                transaction_signature: None,
                input_amount: quote.in_amount.parse().unwrap_or(0),
                output_amount: 0,
                price_impact: quote.price_impact_pct.parse().unwrap_or(0.0),
                slippage,
                route_info: RouteInfo {
                    dexes_used: quote.route_plan.iter()
                        .map(|r| r.swap_info.label.clone())
                        .collect(),
                    number_of_trades: quote.route_plan.len(),
                },
                gas_fee: 0.0,
                error_message: Some("Simulated failure for testing".to_string()),
                is_paper_trade: true,
                execution_time_ms: execution_time,
            })
        }
    }

    /// Validate quote before execution
    pub async fn validate_quote(&self, quote: &JupiterQuote) -> Result<bool> {
        debug!("🔍 Validating quote executability");
        
        // Basic validation checks
        if quote.in_amount.is_empty() || quote.out_amount.is_empty() {
            return Ok(false);
        }
        
        if quote.route_plan.is_empty() {
            return Ok(false);
        }
        
        // TODO: Add more sophisticated validation:
        // - Check token balances
        // - Verify route availability
        // - Check slippage tolerance
        // - Validate minimum output amount
        
        Ok(true)
    }

    /// Get supported DEXes
    pub fn get_supported_dexes(&self) -> Vec<String> {
        vec![
            "Jupiter".to_string(),
            "Raydium".to_string(),
            "Orca".to_string(),
            "Serum".to_string(),
            "Saber".to_string(),
            "Mercurial".to_string(),
            "Aldrin".to_string(),
            "Crema".to_string(),
            "Lifinity".to_string(),
            "Whirlpool".to_string(),
        ]
    }
}





