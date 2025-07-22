// ===== REAL EXECUTION MODULE =====
// Módulo de ejecución real basado en técnica 2C documentada exitosa
// Implementa ejecución segura con validaciones y controles de riesgo

use anyhow::{Result, anyhow};
use tracing::{info, warn, error};
use solana_sdk::{
    pubkey::Pubkey,
    signature::{Keypair, Signer},
    transaction::Transaction,
};
use solana_client::rpc_client::RpcClient;
use spl_associated_token_account::{
    get_associated_token_address,
};
use std::str::FromStr;
use std::time::{Duration, Instant};
use reqwest;
use base64::{Engine as _, engine::general_purpose};
use serde_json::Value;
use base64;

#[derive(Debug, Clone)]
pub struct ExecutionResult {
    pub success: bool,
    pub transaction_signature: Option<String>,
    pub initial_balance: f64,
    pub final_balance: f64,
    pub actual_profit: f64,
    pub execution_time_ms: u128,
    pub gas_fees_paid: f64,
    pub error_message: Option<String>,
}

#[derive(Debug, Clone)]
pub struct ExecutionConfig {
    pub max_slippage_bps: u16,
    pub timeout_seconds: u64,
    pub priority_fee_lamports: u64,
    pub max_retries: u32,
    pub confirmation_commitment: String,
}

impl Default for ExecutionConfig {
    fn default() -> Self {
        Self {
            max_slippage_bps: 150,        // 1.5% max slippage (documented safe value)
            timeout_seconds: 30,          // 30 seconds timeout
            priority_fee_lamports: 5000,  // Small priority fee for faster confirmation
            max_retries: 2,               // Maximum 2 retries on failure
            confirmation_commitment: "confirmed".to_string(),
        }
    }
}

pub struct RealExecutor {
    rpc_client: RpcClient,
    jupiter_url: String,
    config: ExecutionConfig,
}

impl RealExecutor {
    pub fn new(rpc_url: String, config: ExecutionConfig) -> Self {
        Self {
            rpc_client: RpcClient::new(rpc_url),
            jupiter_url: "https://quote-api.jup.ag/v6".to_string(),
            config,
        }
    }

    /// Execute real arbitrage using documented Technique 2C
    pub async fn execute_arbitrage(&self, 
                                 wallet_keypair: &Keypair,
                                 token_a_mint: &str, 
                                 token_b_mint: &str, 
                                 amount: f64) -> Result<ExecutionResult> {
        
        info!("🎯 INICIANDO EJECUCIÓN REAL - Técnica 2C");
        info!("   Par: {} <-> {}", self.get_token_symbol(token_a_mint), self.get_token_symbol(token_b_mint));
        info!("   Amount: {} {}", amount, self.get_token_symbol(token_a_mint));
        info!("   Wallet: {}", wallet_keypair.pubkey());
        
        let execution_start = Instant::now();
        
        // Step 1: Pre-execution validations (CRÍTICO - NO OMITIR)
        info!("🔍 Fase 1: Validaciones de seguridad");
        self.validate_execution_preconditions(wallet_keypair, token_a_mint, amount).await?;
        
        // Step 2: Get initial balance and log it
        info!("💰 Fase 2: Verificación de balance inicial");
        let initial_balance = self.get_token_balance(wallet_keypair, token_a_mint).await?;
        info!("   Balance inicial: {:.9} {}", initial_balance, self.get_token_symbol(token_a_mint));
        
        // Additional safety check: ensure we have enough balance with margin
        let required_balance = amount * 1.01; // 1% margin for safety
        if initial_balance < required_balance {
            return Ok(ExecutionResult {
                success: false,
                transaction_signature: None,
                initial_balance,
                final_balance: initial_balance,
                actual_profit: 0.0,
                execution_time_ms: execution_start.elapsed().as_millis(),
                gas_fees_paid: 0.0,
                error_message: Some(format!("Insufficient balance with safety margin: {:.9} < {:.9}", 
                                          initial_balance, required_balance)),
            });
        }
        
        // Step 3: Execute arbitrage cycle with comprehensive error handling
        info!("⚡ Fase 3: Ejecución del ciclo de arbitrage");
        let execution_result = match self.execute_arbitrage_cycle(
            wallet_keypair, token_a_mint, token_b_mint, amount
        ).await {
            Ok(signature) => {
                info!("✅ Ciclo completado, esperando confirmación final...");
                
                // Wait for final confirmation
                self.wait_for_confirmation(&signature).await?;
                
                // Get final balance with retry logic
                let final_balance = self.get_token_balance_with_retry(wallet_keypair, token_a_mint, 3).await?;
                let actual_profit = final_balance - initial_balance;
                
                ExecutionResult {
                    success: true,
                    transaction_signature: Some(signature),
                    initial_balance,
                    final_balance,
                    actual_profit,
                    execution_time_ms: execution_start.elapsed().as_millis(),
                    gas_fees_paid: 0.000015, // Approximate documented fee cost
                    error_message: None,
                }
            }
            Err(e) => {
                error!("❌ Error durante ejecución: {}", e);
                ExecutionResult {
                    success: false,
                    transaction_signature: None,
                    initial_balance,
                    final_balance: initial_balance,
                    actual_profit: 0.0,
                    execution_time_ms: execution_start.elapsed().as_millis(),
                    gas_fees_paid: 0.0,
                    error_message: Some(e.to_string()),
                }
            }
        };
        
        // Step 4: Log comprehensive results
        self.log_execution_result(&execution_result);
        
        // Step 5: Additional post-execution analysis
        if execution_result.success {
            self.analyze_execution_performance(&execution_result);
        }
        
        Ok(execution_result)
    }

    /// Execute arbitrage cycle: A -> B -> A
    async fn execute_arbitrage_cycle(&self, 
                                   wallet_keypair: &Keypair,
                                   token_a_mint: &str, 
                                   token_b_mint: &str, 
                                   amount: f64) -> Result<String> {
        
        info!("🔄 Ejecutando ciclo de arbitrage:");
        
        // Step 1: Get Jupiter swap transaction for A -> B
        info!("   1️⃣ {} -> {}", 
              self.get_token_symbol(token_a_mint), 
              self.get_token_symbol(token_b_mint));
        
        let swap1_tx = self.get_jupiter_swap_transaction(
            wallet_keypair, token_a_mint, token_b_mint, amount
        ).await?;
        
        let signature1 = self.send_and_confirm_transaction(&swap1_tx).await?;
        info!("      ✅ Transaction 1 confirmed: {}", signature1);
        
        // Brief delay to ensure balance update
        tokio::time::sleep(Duration::from_millis(1000)).await;
        
        // Step 2: Get intermediate balance
        let intermediate_balance = self.get_token_balance(wallet_keypair, token_b_mint).await?;
        info!("      Intermediate balance: {} {}", 
              intermediate_balance, self.get_token_symbol(token_b_mint));
        
        // Step 3: Get Jupiter swap transaction for B -> A
        info!("   2️⃣ {} -> {}", 
              self.get_token_symbol(token_b_mint), 
              self.get_token_symbol(token_a_mint));
        
        let swap2_tx = self.get_jupiter_swap_transaction(
            wallet_keypair, token_b_mint, token_a_mint, intermediate_balance
        ).await?;
        
        let signature2 = self.send_and_confirm_transaction(&swap2_tx).await?;
        info!("      ✅ Transaction 2 confirmed: {}", signature2);
        
        Ok(signature2) // Return final transaction signature
    }

    /// Get Jupiter swap transaction
    async fn get_jupiter_swap_transaction(&self, 
                                        wallet_keypair: &Keypair,
                                        input_mint: &str, 
                                        output_mint: &str, 
                                        amount: f64) -> Result<Transaction> {
        
        // Step 1: Get quote
        let quote = self.get_jupiter_quote(input_mint, output_mint, amount).await?;
        
        // Step 2: Get swap transaction
        let swap_url = format!("{}/swap", self.jupiter_url);
        let user_public_key = wallet_keypair.pubkey().to_string();
        
        let swap_request = serde_json::json!({
            "quoteResponse": quote,
            "userPublicKey": user_public_key,
            "wrapAndUnwrapSol": true,
            "prioritizationFeeLamports": self.config.priority_fee_lamports,
        });
        
        let client = reqwest::Client::new();
        let response = client.post(&swap_url)
            .json(&swap_request)
            .timeout(Duration::from_secs(self.config.timeout_seconds))
            .send()
            .await?;
        
        if !response.status().is_success() {
            return Err(anyhow!("Jupiter swap API error: {}", response.status()));
        }
        
        let swap_response: Value = response.json().await?;
        
        // Extract and deserialize transaction
        let tx_bytes = swap_response["swapTransaction"]
            .as_str()
            .ok_or_else(|| anyhow!("Missing swapTransaction in response"))?;
        
        let tx_data = general_purpose::STANDARD.decode(tx_bytes)
            .map_err(|_| anyhow!("Invalid transaction base64"))?;
        
        let mut transaction: Transaction = bincode::deserialize(&tx_data)
            .map_err(|_| anyhow!("Failed to deserialize transaction"))?;
        
        // Sign transaction
        transaction.sign(&[wallet_keypair], self.rpc_client.get_latest_blockhash()?);
        
        Ok(transaction)
    }

    /// Get Jupiter quote
    async fn get_jupiter_quote(&self, input_mint: &str, output_mint: &str, amount: f64) -> Result<Value> {
        let amount_lamports = (amount * self.get_token_decimals_multiplier(input_mint)) as u64;
        
        let url = format!(
            "{}/quote?inputMint={}&outputMint={}&amount={}&slippageBps={}",
            self.jupiter_url, input_mint, output_mint, amount_lamports, self.config.max_slippage_bps
        );
        
        let client = reqwest::Client::new();
        let response = client.get(&url)
            .timeout(Duration::from_secs(self.config.timeout_seconds))
            .send()
            .await?;
        
        if !response.status().is_success() {
            return Err(anyhow!("Jupiter quote API error: {}", response.status()));
        }
        
        let quote: Value = response.json().await?;
        Ok(quote)
    }

    /// Send and confirm transaction
    async fn send_and_confirm_transaction(&self, transaction: &Transaction) -> Result<String> {
        let signature = self.rpc_client.send_transaction(transaction)?;
        self.wait_for_confirmation(&signature.to_string()).await?;
        Ok(signature.to_string())
    }

    /// Wait for transaction confirmation
    async fn wait_for_confirmation(&self, signature: &str) -> Result<()> {
        let signature = signature.parse()?;
        let timeout = Duration::from_secs(self.config.timeout_seconds);
        let start = Instant::now();
        
        while start.elapsed() < timeout {
            match self.rpc_client.confirm_transaction(&signature) {
                Ok(confirmed) => {
                    if confirmed {
                        return Ok(());
                    }
                }
                Err(_) => {
                    // Continue waiting
                }
            }
            tokio::time::sleep(Duration::from_millis(500)).await;
        }
        
        Err(anyhow!("Transaction confirmation timeout"))
    }

    /// Get token balance for wallet with retry logic
    async fn get_token_balance_with_retry(&self, wallet_keypair: &Keypair, token_mint: &str, max_retries: u32) -> Result<f64> {
        let mut last_error = None;
        
        for attempt in 1..=max_retries {
            match self.get_token_balance(wallet_keypair, token_mint).await {
                Ok(balance) => return Ok(balance),
                Err(e) => {
                    warn!("⚠️ Balance check attempt {} failed: {}", attempt, e);
                    last_error = Some(e);
                    if attempt < max_retries {
                        tokio::time::sleep(Duration::from_millis(1000)).await;
                    }
                }
            }
        }
        
        Err(last_error.unwrap_or_else(|| anyhow!("Failed to get balance after {} retries", max_retries)))
    }

    /// Get token balance for wallet
    async fn get_token_balance(&self, wallet_keypair: &Keypair, token_mint: &str) -> Result<f64> {
        if token_mint == "So11111111111111111111111111111111111111112" {
            // SOL balance
            let balance = self.rpc_client.get_balance(&wallet_keypair.pubkey())?;
            Ok(balance as f64 / 1_000_000_000.0)
        } else {
            // SPL Token balance
            let mint_pubkey = Pubkey::from_str(token_mint)?;
            let ata = get_associated_token_address(&wallet_keypair.pubkey(), &mint_pubkey);
            
            match self.rpc_client.get_token_account_balance(&ata) {
                Ok(balance) => {
                    let amount: f64 = balance.ui_amount.unwrap_or(0.0);
                    Ok(amount)
                }
                Err(_) => Ok(0.0), // Account doesn't exist or has zero balance
            }
        }
    }

    /// Validate execution preconditions
    async fn validate_execution_preconditions(&self, 
                                            wallet_keypair: &Keypair, 
                                            token_mint: &str, 
                                            amount: f64) -> Result<()> {
        
        info!("🔍 Validando precondiciones de ejecución");
        
        // Check wallet balance
        let current_balance = self.get_token_balance(wallet_keypair, token_mint).await?;
        if current_balance < amount {
            return Err(anyhow!("Insufficient balance: {} < {} {}", 
                             current_balance, amount, self.get_token_symbol(token_mint)));
        }
        
        // Check RPC connection
        let _latest_blockhash = self.rpc_client.get_latest_blockhash()
            .map_err(|e| anyhow!("RPC connection failed: {}", e))?;
        
        // Check Jupiter API availability
        let health_url = format!("{}/quote?inputMint=So11111111111111111111111111111111111111112&outputMint=EPjFWdd5AufqSSqeM2qN1xzybapC8G4wEGGkZwyTDt1v&amount=1000000000", 
                               self.jupiter_url);
        
        let client = reqwest::Client::new();
        let response = client.get(&health_url)
            .timeout(Duration::from_secs(5))
            .send()
            .await?;
        
        if !response.status().is_success() {
            return Err(anyhow!("Jupiter API not available: {}", response.status()));
        }
        
        info!("✅ Todas las precondiciones validadas");
        Ok(())
    }

    /// Get token decimals multiplier
    fn get_token_decimals_multiplier(&self, mint: &str) -> f64 {
        match mint {
            "So11111111111111111111111111111111111111112" => 1_000_000_000.0, // SOL
            "EPjFWdd5AufqSSqeM2qN1xzybapC8G4wEGGkZwyTDt1v" => 1_000_000.0,     // USDC
            "4k3Dyjzvzp8eMZWUXbBCjEvwSkkk59S5iCNLY3QrkX6R" => 1_000_000.0,     // RAY
            "DezXAZ8z7PnrnRJjz3wXBoRgixCa6xjnB7YaB1pPB263" => 100_000.0,       // BONK
            _ => 1_000_000_000.0, // Default
        }
    }

    /// Get token symbol for display
    fn get_token_symbol(&self, mint: &str) -> &str {
        match mint {
            "So11111111111111111111111111111111111111112" => "SOL",
            "EPjFWdd5AufqSSqeM2qN1xzybapC8G4wEGGkZwyTDt1v" => "USDC",
            "4k3Dyjzvzp8eMZWUXbBCjEvwSkkk59S5iCNLY3QrkX6R" => "RAY",
            "DezXAZ8z7PnrnRJjz3wXBoRgixCa6xjnB7YaB1pPB263" => "BONK",
            _ => "UNKNOWN",
        }
    }

    /// Log execution result
    fn log_execution_result(&self, result: &ExecutionResult) {
        if result.success {
            info!("✅ EJECUCIÓN EXITOSA:");
            info!("   Signature: {:?}", result.transaction_signature);
            info!("   Balance inicial: {:.9}", result.initial_balance);
            info!("   Balance final: {:.9}", result.final_balance);
            info!("   Profit real: {:.9} SOL", result.actual_profit);
            info!("   Tiempo ejecución: {}ms", result.execution_time_ms);
            info!("   Fees pagados: ~{:.9} SOL", result.gas_fees_paid);
            
            if result.actual_profit > 0.0 {
                info!("   🎉 ARBITRAGE RENTABLE CONFIRMADO!");
                let roi_percentage = (result.actual_profit / result.initial_balance) * 100.0;
                info!("   📊 ROI: {:.4}%", roi_percentage);
            } else {
                warn!("   ⚠️ Resultado: pérdida o break-even");
            }
        } else {
            error!("❌ EJECUCIÓN FALLIDA:");
            error!("   Error: {:?}", result.error_message);
            error!("   Tiempo ejecución: {}ms", result.execution_time_ms);
            error!("   💡 Tip: Revisar condiciones de mercado y balance de wallet");
        }
    }

    /// Analyze execution performance
    fn analyze_execution_performance(&self, result: &ExecutionResult) {
        info!("📊 ANÁLISIS DE PERFORMANCE:");
        
        // Execution speed analysis
        let speed_category = if result.execution_time_ms < 5000 {
            "🚀 Muy rápida"
        } else if result.execution_time_ms < 15000 {
            "⚡ Rápida"
        } else if result.execution_time_ms < 30000 {
            "🐢 Lenta"
        } else {
            "❌ Muy lenta"
        };
        
        info!("   Velocidad: {} ({}ms)", speed_category, result.execution_time_ms);
        
        // Profit analysis
        let profit_lamports = (result.actual_profit * 1_000_000_000.0) as i64;
        let fee_ratio = profit_lamports as f64 / 15000.0; // 15k lamports = documented fee cost
        
        info!("   Profit en lamports: {}", profit_lamports);
        info!("   Fee ratio: {:.2}x", fee_ratio);
        
        if fee_ratio > 3.0 {
            info!("   🎯 Excelente ejecución: Profit > 3x fees");
        } else if fee_ratio > 1.0 {
            info!("   ✅ Ejecución rentable: Profit > fees");
        } else {
            warn!("   ⚠️ Ejecución marginal: Profit < fees");
        }
        
        // Market timing analysis
        if result.execution_time_ms < 10000 && result.actual_profit > 0.0 {
            info!("   🎖️ EJECUCIÓN ÓPTIMA: Rápida y rentable");
        }
    }
}

/// Create executor with mainnet configuration
pub fn create_mainnet_executor() -> RealExecutor {
    let rpc_url = "https://api.mainnet-beta.solana.com".to_string();
    let config = ExecutionConfig::default();
    RealExecutor::new(rpc_url, config)
}

/// Create executor with devnet configuration for testing
pub fn create_devnet_executor() -> RealExecutor {
    let rpc_url = "https://api.devnet.solana.com".to_string();
    let config = ExecutionConfig {
        max_slippage_bps: 300, // Higher slippage tolerance for devnet
        ..ExecutionConfig::default()
    };
    RealExecutor::new(rpc_url, config)
}

/// Execute safe arbitrage with full validation - public interface
pub async fn execute_safe_arbitrage(wallet_keypair: &Keypair, 
                                  token_a_mint: &str, 
                                  token_b_mint: &str, 
                                  amount: f64,
                                  use_mainnet: bool) -> Result<ExecutionResult> {
    let executor = if use_mainnet {
        create_mainnet_executor()
    } else {
        create_devnet_executor()
    };
    
    executor.execute_arbitrage(wallet_keypair, token_a_mint, token_b_mint, amount).await
}

/// Simulate arbitrage execution without spending real money - for testing
pub async fn simulate_arbitrage_execution(token_a_mint: &str, 
                                        token_b_mint: &str, 
                                        amount: f64) -> Result<ExecutionResult> {
    info!("🎭 SIMULANDO EJECUCIÓN DE ARBITRAGE");
    info!("   Par: {} <-> {}", get_token_symbol_static(token_a_mint), get_token_symbol_static(token_b_mint));
    info!("   Amount: {} {}", amount, get_token_symbol_static(token_a_mint));
    
    let execution_start = Instant::now();
    
    // Simulate with some realistic delay
    tokio::time::sleep(Duration::from_millis(2000)).await;
    
    // Create a simulated successful result
    let simulated_profit = amount * 0.001; // 0.1% simulated profit
    
    let result = ExecutionResult {
        success: true,
        transaction_signature: Some("SIMULATION_TX_12345...".to_string()),
        initial_balance: amount,
        final_balance: amount + simulated_profit,
        actual_profit: simulated_profit,
        execution_time_ms: execution_start.elapsed().as_millis(),
        gas_fees_paid: 0.000015,
        error_message: None,
    };
    
    info!("✅ SIMULACIÓN COMPLETADA:");
    info!("   Profit simulado: {:.9} SOL", simulated_profit);
    info!("   Tiempo simulado: {}ms", result.execution_time_ms);
    info!("   💡 Nota: Esta es una simulación - no se ejecutaron transacciones reales");
    
    Ok(result)
}

/// Get token symbol for static use
fn get_token_symbol_static(mint: &str) -> &str {
    match mint {
        "So11111111111111111111111111111111111111112" => "SOL",
        "EPjFWdd5AufqSSqeM2qN1xzybapC8G4wEGGkZwyTDt1v" => "USDC",
        "4k3Dyjzvzp8eMZWUXbBCjEvwSkkk59S5iCNLY3QrkX6R" => "RAY",
        "DezXAZ8z7PnrnRJjz3wXBoRgixCa6xjnB7YaB1pPB263" => "BONK",
        _ => "UNKNOWN",
    }
}
