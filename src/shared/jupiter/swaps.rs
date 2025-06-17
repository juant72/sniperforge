/// Jupiter Swap Engine
/// 
/// High-level interface for executing swaps on Solana

use anyhow::{Result, anyhow};
use solana_sdk::{
    pubkey::Pubkey,
    transaction::Transaction,
    signature::{Keypair, Signature},
};
use std::time::Instant;
use tracing::{info, warn, error, debug};

use super::client::JupiterClient;
use super::types::*;

/// Swap execution engine
pub struct SwapEngine {
    client: JupiterClient,
}

impl SwapEngine {
    /// Create new swap engine
    pub fn new(client: JupiterClient) -> Self {
        Self { client }
    }

    /// Execute a swap on DevNet (real transaction)
    pub async fn execute_swap_devnet(
        &self,
        quote: &JupiterQuote,
        user_keypair: &Keypair,
        rpc_client: &solana_client::rpc_client::RpcClient,
    ) -> Result<SwapResult> {
        let start_time = Instant::now();
        
        info!("🔄 Executing REAL swap on DevNet");
        info!("   Route: {} -> {}", quote.input_mint, quote.output_mint);
        info!("   Amount: {} -> {}", quote.in_amount, quote.out_amount);
        info!("   Price Impact: {}%", quote.price_impact_pct);

        // Get swap transaction from Jupiter
        let swap_response = self.client.get_swap_transaction(
            &serde_json::to_value(quote)?,
            &user_keypair.pubkey().to_string(),
        ).await?;

        let swap_tx: JupiterSwapTransaction = serde_json::from_value(swap_response)?;

        // Decode the transaction
        let tx_bytes = base64::decode(&swap_tx.swap_transaction)?;
        let mut transaction: Transaction = bincode::deserialize(&tx_bytes)?;

        // Sign the transaction
        transaction.try_sign(&[user_keypair], rpc_client.get_latest_blockhash()?)?;

        // Submit transaction
        debug!("📡 Submitting transaction to DevNet...");
        
        match rpc_client.send_and_confirm_transaction(&transaction) {
            Ok(signature) => {
                let execution_time = start_time.elapsed().as_millis() as u64;
                
                info!("✅ DevNet swap executed successfully!");
                info!("   Signature: {}", signature);
                info!("   Execution time: {}ms", execution_time);

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
                        number_of_routes: quote.route_plan.len(),
                        best_route_label: quote.route_plan.first()
                            .map(|r| r.swap_info.label.clone())
                            .unwrap_or_default(),
                    },
                    execution_time_ms: execution_time,
                    error_message: None,
                })
            }
            Err(e) => {
                error!("❌ DevNet swap failed: {}", e);
                
                Ok(SwapResult {
                    success: false,
                    transaction_signature: None,
                    input_amount: quote.in_amount.parse().unwrap_or(0),
                    output_amount: 0,
                    price_impact: quote.price_impact_pct.parse().unwrap_or(0.0),
                    slippage: quote.slippage_bps as f64 / 100.0,
                    route_info: RouteInfo {
                        dexes_used: vec![],
                        number_of_routes: 0,
                        best_route_label: "failed".to_string(),
                    },
                    execution_time_ms: start_time.elapsed().as_millis() as u64,
                    error_message: Some(e.to_string()),
                })
            }
        }
    }

    /// Simulate a swap on MainNet (paper trading)
    pub async fn simulate_swap_mainnet(
        &self,
        quote: &JupiterQuote,
        user_pubkey: &Pubkey,
    ) -> Result<SwapResult> {
        let start_time = Instant::now();
        
        info!("📋 Simulating swap on MainNet (Paper Trading)");
        info!("   Route: {} -> {}", quote.input_mint, quote.output_mint);
        info!("   Amount: {} -> {}", quote.in_amount, quote.out_amount);
        info!("   Price Impact: {}%", quote.price_impact_pct);

        // Get the swap transaction to validate the route
        let swap_response = self.client.get_swap_transaction(
            &serde_json::to_value(quote)?,
            &user_pubkey.to_string(),
        ).await?;

        let swap_tx: JupiterSwapTransaction = serde_json::from_value(swap_response)?;
        
        // Simulate successful execution
        let execution_time = start_time.elapsed().as_millis() as u64;
        
        // Add some realistic variance to simulation
        let success_rate = 0.95; // 95% success rate for simulations
        let is_successful = rand::random::<f64>() < success_rate;

        if is_successful {
            info!("✅ MainNet swap simulation successful (Paper Trading)");
            info!("   Simulated execution time: {}ms", execution_time);

            Ok(SwapResult {
                success: true,
                transaction_signature: Some(format!("PAPER_{}", uuid::Uuid::new_v4())),
                input_amount: quote.in_amount.parse().unwrap_or(0),
                output_amount: quote.out_amount.parse().unwrap_or(0),
                price_impact: quote.price_impact_pct.parse().unwrap_or(0.0),
                slippage: quote.slippage_bps as f64 / 100.0,
                route_info: RouteInfo {
                    dexes_used: quote.route_plan.iter()
                        .map(|r| r.swap_info.label.clone())
                        .collect(),
                    number_of_routes: quote.route_plan.len(),
                    best_route_label: quote.route_plan.first()
                        .map(|r| r.swap_info.label.clone())
                        .unwrap_or_default(),
                },
                execution_time_ms: execution_time,
                error_message: None,
            })
        } else {
            warn!("❌ MainNet swap simulation failed (Paper Trading)");
            
            Ok(SwapResult {
                success: false,
                transaction_signature: None,
                input_amount: quote.in_amount.parse().unwrap_or(0),
                output_amount: 0,
                price_impact: quote.price_impact_pct.parse().unwrap_or(0.0),
                slippage: quote.slippage_bps as f64 / 100.0,
                route_info: RouteInfo {
                    dexes_used: vec![],
                    number_of_routes: 0,
                    best_route_label: "simulation_failed".to_string(),
                },
                execution_time_ms: execution_time,
                error_message: Some("Simulated failure for testing".to_string()),
            })
        }
    }

    /// Get estimated swap output without executing
    pub async fn estimate_swap_output(
        &self,
        input_mint: &Pubkey,
        output_mint: &Pubkey,
        input_amount: u64,
        slippage_bps: Option<u16>,
    ) -> Result<u64> {
        debug!("📊 Estimating swap output: {} -> {}", input_mint, output_mint);

        let quote_response = self.client.get_quote(
            &input_mint.to_string(),
            &output_mint.to_string(),
            input_amount,
            slippage_bps,
        ).await?;

        let quote: JupiterQuote = serde_json::from_value(quote_response)?;
        let output_amount: u64 = quote.out_amount.parse().unwrap_or(0);

        debug!("✅ Estimated output: {}", output_amount);
        Ok(output_amount)
    }

    /// Validate a quote is still executable
    pub async fn validate_quote(&self, quote: &JupiterQuote) -> Result<bool> {
        debug!("🔍 Validating quote executability");

        // For now, assume quotes are valid for 30 seconds
        // In production, you might want to re-fetch and compare
        
        // Simple validation: check if amounts are reasonable
        let input_amount: u64 = quote.in_amount.parse().unwrap_or(0);
        let output_amount: u64 = quote.out_amount.parse().unwrap_or(0);
        
        let is_valid = input_amount > 0 && output_amount > 0;
        
        debug!("✅ Quote validation: {}", if is_valid { "VALID" } else { "INVALID" });
        Ok(is_valid)
    }

    /// Get supported DEX list
    pub async fn get_supported_dexes(&self) -> Result<Vec<String>> {
        // This would typically come from a specific endpoint
        // For now, return known Jupiter-supported DEXes
        Ok(vec![
            "Raydium".to_string(),
            "Orca".to_string(),
            "Serum".to_string(),
            "Saber".to_string(),
            "Mercurial".to_string(),
            "Aldrin".to_string(),
            "Crema".to_string(),
            "Lifinity".to_string(),
            "Whirlpool".to_string(),
        ])
    }
}
