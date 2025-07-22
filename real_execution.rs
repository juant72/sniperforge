// ===== REAL EXECUTION MODULE =====
// Sistema de ejecución real en mainnet con protocolo empresarial

use anyhow::Result;
use tracing::{info, warn};
use solana_sdk::signature::{Keypair, Signature};
use solana_sdk::pubkey::Pubkey;
use std::time::Instant;

use crate::types::*;
use crate::risk_manager::EnterpriseRiskManager;
use crate::jupiter_integration::JupiterIntegration;
use crate::transaction_executor::TransactionExecutor;

// Constants for mainnet execution
const MAINNET_MIN_PROFIT_SOL: f64 = 0.01;

#[derive(Debug, Clone)]
pub struct JupiterSwapResult {
    pub signature: Signature,
    pub input_amount: u64,
    pub output_amount: u64,
    pub price_impact: f64,
}

pub struct RealExecutionEngine;

impl RealExecutionEngine {
    /// EXECUTE REAL ARBITRAGE ON MAINNET - Main execution function
    pub async fn execute_real_arbitrage_mainnet(
        engine: &ProfessionalArbitrageEngine,
        opportunity: &DirectOpportunity, 
        wallet: &Keypair
    ) -> Result<String> {
        let start_time = Instant::now();
        info!("⚡ EXECUTING REAL ARBITRAGE ON MAINNET");
        info!("🎯 OPPORTUNITY: {} SOL profit expected", opportunity.profit_lamports as f64 / 1e9);
        
        // STEP 1: Pre-execution validation
        Self::validate_execution(opportunity)?;
        
        // STEP 2: Execute first swap
        info!("🔄 STEP 1: Executing first swap");
        let swap_a_result = Self::execute_jupiter_swap_mainnet(
            engine,
            &opportunity.token_in,
            &opportunity.token_out,
            opportunity.amount_in,
            wallet,
        ).await?;
        
        info!("✅ First swap completed: {} -> {}", 
              opportunity.amount_in, swap_a_result.output_amount);
        
        // STEP 3: Execute second swap
        info!("🔄 STEP 2: Executing second swap");
        let swap_b_result = Self::execute_jupiter_swap_mainnet(
            engine,
            &opportunity.token_out,
            &opportunity.token_in,
            swap_a_result.output_amount,
            wallet,
        ).await?;
        
        info!("✅ Second swap completed: {} -> {}", 
              swap_a_result.output_amount, swap_b_result.output_amount);
        
        // STEP 4: Calculate and report results
        Self::process_execution_results(opportunity, swap_b_result, start_time).await
    }
    
    /// PRE-EXECUTION VALIDATION
    fn validate_execution(opportunity: &DirectOpportunity) -> Result<()> {
        EnterpriseRiskManager::validate_execution(opportunity, MAINNET_MIN_PROFIT_SOL)
    }
    
    /// EXECUTE JUPITER SWAP ON MAINNET
    async fn execute_jupiter_swap_mainnet(
        engine: &ProfessionalArbitrageEngine,
        input_mint: &Pubkey,
        output_mint: &Pubkey,
        amount: u64,
        wallet: &Keypair,
    ) -> Result<JupiterSwapResult> {
        info!("🚀 EXECUTING JUPITER SWAP ON MAINNET");
        info!("📊 {} -> {} | Amount: {}", 
              input_mint.to_string()[..8].to_uppercase(),
              output_mint.to_string()[..8].to_uppercase(),
              amount);
        
        // Get quote from Jupiter
        let quote = JupiterIntegration::get_jupiter_quote_mainnet(engine, input_mint, output_mint, amount).await?;
        info!("✅ Quote received: {} -> {} (impact: {:.4}%)", 
              amount, quote.out_amount, quote.price_impact_pct);
        
        // Get swap transaction
        let swap_tx = JupiterIntegration::get_jupiter_swap_transaction(engine, &quote, wallet).await?;
        
        // Sign and send transaction
        let signature = TransactionExecutor::sign_and_send_transaction(engine, swap_tx, wallet).await?;
        
        info!("✅ SWAP EXECUTED - Signature: {}", signature);
        
        Ok(JupiterSwapResult {
            signature,
            input_amount: amount,
            output_amount: quote.out_amount,
            price_impact: quote.price_impact_pct,
        })
    }
    
    /// PROCESS EXECUTION RESULTS
    async fn process_execution_results(
        opportunity: &DirectOpportunity,
        swap_b_result: JupiterSwapResult,
        start_time: Instant,
    ) -> Result<String> {
        let execution_time_ms = start_time.elapsed().as_millis() as u64;
        let actual_profit = if swap_b_result.output_amount > opportunity.amount_in {
            swap_b_result.output_amount - opportunity.amount_in
        } else {
            0
        };
        
        let actual_profit_sol = actual_profit as f64 / 1e9;
        
        if actual_profit > 0 {
            info!("🎉 ARBITRAGE EXECUTION SUCCESSFUL!");
            info!("💰 ACTUAL PROFIT: {:.6} SOL", actual_profit_sol);
            info!("⏱️  EXECUTION TIME: {}ms", execution_time_ms);
            
            Ok(format!("EXECUTED_{}_{}_PROFIT_{:.6}_SOL", 
                opportunity.pool_a.address.to_string()[..8].to_uppercase(),
                opportunity.pool_b.address.to_string()[..8].to_uppercase(),
                actual_profit_sol))
        } else {
            warn!("⚠️  ARBITRAGE EXECUTION COMPLETED BUT NO PROFIT");
            warn!("📊 Expected: {:.6} SOL, Actual: {:.6} SOL", 
                  opportunity.profit_lamports as f64 / 1e9, actual_profit_sol);
            
            Ok(format!("EXECUTED_{}_{}_NO_PROFIT", 
                opportunity.pool_a.address.to_string()[..8].to_uppercase(),
                opportunity.pool_b.address.to_string()[..8].to_uppercase()))
        }
    }
}
