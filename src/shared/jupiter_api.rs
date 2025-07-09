//! Jupiter API Integration - Main Module
//! 
//! High-level Jupiter API integration with business logic, wallet integration,
//! and transaction execution. This is the main interface for trading operations.

use anyhow::{Result, anyhow};
use tracing::{info, warn, error, debug};
use solana_client::rpc_client::RpcClient;
use solana_sdk::{
    pubkey::Pubkey, 
    signature::{Signature, Keypair, Signer}, 
    transaction::{Transaction, VersionedTransaction},
    commitment_config::CommitmentConfig,
    message::VersionedMessage,
    hash::Hash,
};
use base64::{Engine as _, engine::general_purpose};
use chrono::Utc;

use super::jupiter_client::JupiterClient;
use super::jupiter_config::JupiterConfig;
use super::jupiter_types::*;
use super::network_config::NetworkConfig;

/// Main Jupiter API wrapper with business logic
#[derive(Debug)]
pub struct Jupiter {
    pub client: JupiterClient,
    config: JupiterConfig,
    network_config: NetworkConfig,
}

impl Jupiter {
    /// Create new Jupiter instance
    pub async fn new(config: &JupiterConfig) -> Result<Self> {
        let client = JupiterClient::new(config).await?;
        
        // Determine network config based on RPC endpoint
        let network_config = if config.rpc_endpoint.contains("devnet") {
            NetworkConfig::devnet()
        } else if config.rpc_endpoint.contains("mainnet") {
            NetworkConfig::mainnet()
        } else if config.rpc_endpoint.contains("testnet") {
            NetworkConfig::testnet()
        } else {
            warn!("Unknown network from RPC endpoint: {}, defaulting to DevNet", config.rpc_endpoint);
            NetworkConfig::devnet()
        };
        
        // Validate that Jupiter is available for this network
        if !network_config.has_jupiter() {
            return Err(anyhow!("Jupiter is not available for network: {}", network_config.network));
        }
        
        info!("🌐 Jupiter initialized for {} network", network_config.network);
        info!("🔧 Jupiter Program ID: {:?}", network_config.program_ids.jupiter_program);
        
        Ok(Self { 
            client,
            config: config.clone(),
            network_config,
        })
    }

    /// Create new Jupiter instance with explicit network config
    pub async fn new_with_network(config: &JupiterConfig, network_config: NetworkConfig) -> Result<Self> {
        let client = JupiterClient::new(config).await?;
        
        // Validate that Jupiter is available for this network
        if !network_config.has_jupiter() {
            return Err(anyhow!("Jupiter is not available for network: {}", network_config.network));
        }
        
        // Validate network config
        network_config.validate()
            .map_err(|e| anyhow!("Invalid network configuration: {}", e))?;
        
        info!("🌐 Jupiter initialized for {} network (explicit config)", network_config.network);
        info!("🔧 Jupiter Program ID: {:?}", network_config.program_ids.jupiter_program);
        
        Ok(Self { 
            client,
            config: config.clone(),
            network_config,
        })
    }

    /// Get token price using the client
    pub async fn get_token_price(&self, mint: &str) -> Result<TokenPrice> {
        match self.client.get_price(mint).await? {
            Some(price) => Ok(TokenPrice {
                price,
                volume_24h: None,
                market_cap: None,
            }),
            None => Err(anyhow!("No price data found for token: {}", mint))
        }
    }

    /// Get quote for a swap with automatic unit conversion
    pub async fn get_quote(
        &self,
        input_mint: &str,
        output_mint: &str,
        amount: f64,
        slippage_bps: u16,
    ) -> Result<QuoteResponse> {
        let quote_request = QuoteRequest {
            inputMint: input_mint.to_string(),
            outputMint: output_mint.to_string(),
            amount: (amount * 1_000_000_000.0) as u64, // Convert to lamports
            slippageBps: slippage_bps,
        };

        let quote = self.client.get_quote(quote_request).await?;

        Ok(quote)
    }

    /// Build swap transaction (returns transaction data, does NOT execute)
    /// Use execute_swap_with_wallet() for complete swap execution with signing
    pub async fn build_swap_transaction(&self, quote: &QuoteResponse, wallet_address: &str) -> Result<SwapResult> {
        info!("🔄 Building swap transaction...");
        
        // Create swap request with optimization
        let swap_request = SwapRequest {
            quoteResponse: quote.clone(),
            userPublicKey: wallet_address.to_string(),
            dynamicComputeUnitLimit: Some(true),
            dynamicSlippage: Some(true),
            prioritizationFeeLamports: Some(PrioritizationFee {
                priorityLevelWithMaxLamports: PriorityLevelConfig {
                    maxLamports: 1000000, // 0.001 SOL max priority fee for devnet
                    priorityLevel: "medium".to_string(), // Conservative for testing
                }
            }),
            asLegacyTransaction: Some(true), // Force legacy transaction format for DevNet
        };

        let _swap_response = self.client.build_swap_transaction(swap_request).await?;

        info!("✅ Swap transaction built successfully");
        info!("🔒 Use execute_swap_with_wallet() for complete transaction execution");

        Ok(SwapResult {
            success: true, // Transaction was built successfully
            transaction_signature: Some(format!("BUILT_{}", chrono::Utc::now().timestamp())),
            output_amount: quote.out_amount(),
            actual_slippage: quote.price_impact_pct(),
            fee_amount: 0.001, // Estimated fee from Jupiter
        })
    }

    /// Execute swap with wallet integration (main public interface)
    pub async fn execute_swap_with_wallet(
        &self,
        quote: &QuoteResponse,
        wallet_address: &str,
        wallet_keypair: Option<&Keypair>,
    ) -> Result<SwapExecutionResult> {
        self.execute_swap_with_wallet_internal(quote, wallet_address, wallet_keypair).await
    }

    /// Internal implementation of swap execution with full safety checks
    async fn execute_swap_with_wallet_internal(
        &self,
        quote: &QuoteResponse,
        wallet_address: &str,
        wallet_keypair: Option<&Keypair>,
    ) -> Result<SwapExecutionResult> {
        info!("🔄 Executing swap with wallet integration...");
        
        // CRITICAL SAFETY CHECKS - PREVENT WALLET DRAINING
        let swap_amount_sol = quote.in_amount();
        
        // Safety Check 1: Maximum swap amount protection
        let max_allowed_swap = if self.network_config.network == "MainNet" { 0.1 } else { 1.0 }; // 0.1 SOL max on MainNet, 1.0 SOL on DevNet
        if swap_amount_sol > max_allowed_swap {
            error!("🚨 SAFETY ABORT: Swap amount ({} SOL) exceeds maximum allowed ({} SOL) for {}", 
                   swap_amount_sol, max_allowed_swap, self.network_config.network);
            return Ok(SwapExecutionResult {
                success: false,
                transaction_signature: format!("SAFETY_ABORT_MAX_AMOUNT_{}", chrono::Utc::now().timestamp()),
                output_amount: 0.0,
                actual_slippage: 0.0,
                fee_amount: 0.0,
                block_height: 0,
                logs: vec![
                    format!("🚨 SAFETY ABORT: Swap amount ({} SOL) exceeds maximum allowed ({} SOL)", swap_amount_sol, max_allowed_swap),
                    format!("Maximum swap limit for {} is {} SOL", self.network_config.network, max_allowed_swap),
                    "Transaction blocked to prevent potential wallet draining".to_string(),
                ],
            });
        }
        
        // Safety Check 2: Verify wallet has sufficient balance (with safety margin)
        if let Some(keypair) = wallet_keypair {
            let rpc_client = RpcClient::new_with_commitment(
                self.network_config.rpc_endpoint.clone(),
                CommitmentConfig::confirmed(),
            );
            
            match rpc_client.get_balance(&keypair.pubkey()) {
                Ok(balance_lamports) => {
                    let balance_sol = balance_lamports as f64 / 1_000_000_000.0;
                    let required_balance = swap_amount_sol + 0.01; // Add 0.01 SOL for fees
                    
                    if balance_sol < required_balance {
                        warn!("⚠️ Insufficient balance: {} SOL required, {} SOL available", required_balance, balance_sol);
                        return Ok(SwapExecutionResult {
                            success: false,
                            transaction_signature: format!("INSUFFICIENT_BALANCE_{}", chrono::Utc::now().timestamp()),
                            output_amount: 0.0,
                            actual_slippage: 0.0,
                            fee_amount: 0.0,
                            block_height: 0,
                            logs: vec![
                                format!("Insufficient balance: {} SOL required, {} SOL available", required_balance, balance_sol),
                                "Transaction cancelled to prevent failed execution".to_string(),
                            ],
                        });
                    }
                    
                    // Additional safety for MainNet
                    if self.network_config.network == "MainNet" && swap_amount_sol > (balance_sol * 0.5) {
                        error!("🚨 MAINNET SAFETY: Attempting to swap >50% of wallet balance");
                        return Ok(SwapExecutionResult {
                            success: false,
                            transaction_signature: format!("MAINNET_SAFETY_ABORT_{}", chrono::Utc::now().timestamp()),
                            output_amount: 0.0,
                            actual_slippage: 0.0,
                            fee_amount: 0.0,
                            block_height: 0,
                            logs: vec![
                                "🚨 MAINNET SAFETY: Cannot swap >50% of wallet balance".to_string(),
                                format!("Swap amount: {} SOL, Wallet balance: {} SOL", swap_amount_sol, balance_sol),
                            ],
                        });
                    }
                    
                    info!("✅ Balance check passed: {} SOL available, {} SOL required", balance_sol, required_balance);
                }
                Err(e) => warn!("⚠️ Could not verify wallet balance: {}", e),
            }
        }
        
        // Step 1: Build the swap transaction
        let swap_request = SwapRequest {
            quoteResponse: quote.clone(),
            userPublicKey: wallet_address.to_string(),
            dynamicComputeUnitLimit: Some(true),
            dynamicSlippage: Some(true),
            prioritizationFeeLamports: Some(PrioritizationFee {
                priorityLevelWithMaxLamports: PriorityLevelConfig {
                    maxLamports: 1000000, // 0.001 SOL max priority fee
                    priorityLevel: "medium".to_string(),
                }
            }),
            asLegacyTransaction: Some(true), // Force legacy for DevNet compatibility
        };

        let swap_response = self.client.build_swap_transaction(swap_request).await?;
        
        // Step 2: Decode and sign the transaction
        if let Some(keypair) = wallet_keypair {
            // REAL IMPLEMENTATION: Execute the swap transaction on blockchain
            return self.execute_signed_transaction(&swap_response, keypair, quote).await;
        } else {
            // Return transaction data without execution (for demo/testing)
            warn!("⚠️ No keypair provided - returning transaction data only");
            return Ok(SwapExecutionResult {
                success: false,
                transaction_signature: format!("NO_KEYPAIR_PROVIDED_{}", chrono::Utc::now().timestamp()),
                output_amount: quote.out_amount(),
                actual_slippage: quote.price_impact_pct(),
                fee_amount: 0.001,
                block_height: 0,
                logs: vec![
                    "Transaction built but not executed (no keypair provided)".to_string(),
                    format!("Expected output: {} tokens", quote.out_amount()),
                ],
            });
        }
    }

    /// Execute the signed transaction on the blockchain
    async fn execute_signed_transaction(
        &self,
        swap_response: &SwapResponse,
        keypair: &Keypair,
        quote: &QuoteResponse,
    ) -> Result<SwapExecutionResult> {
        info!("🔐 Signing and executing transaction on blockchain...");
        
        // Decode the transaction from base64
        let transaction_data = general_purpose::STANDARD
            .decode(&swap_response.swapTransaction)
            .map_err(|e| anyhow!("Failed to decode transaction: {}", e))?;
        
        // For safety, log transaction details
        info!("📋 Transaction details:");
        info!("   Swap amount: {} SOL", quote.in_amount());
        info!("   Expected output: {} tokens", quote.out_amount());
        info!("   Price impact: {}%", quote.price_impact_pct());
        info!("   Network: {}", self.network_config.network);
        info!("   Jupiter Program ID: {:?}", self.network_config.program_ids.jupiter_program);
        
        // Additional safety check for MainNet
        if self.network_config.network == "MainNet" {
            error!("🚨 MAINNET EXECUTION DISABLED FOR SAFETY");
            warn!("   This is a safety measure to prevent accidental MainNet trades");
            warn!("   Network: {}", self.network_config.network);
            warn!("   Amount: {} SOL", quote.in_amount());
            
            return Ok(SwapExecutionResult {
                success: false,
                transaction_signature: format!("MAINNET_DISABLED_{}", chrono::Utc::now().timestamp()),
                output_amount: 0.0,
                actual_slippage: 0.0,
                fee_amount: 0.0,
                block_height: 0,
                logs: vec![
                    "🚨 MainNet execution disabled for safety".to_string(),
                    "Enable MainNet trading only after thorough testing".to_string(),
                ],
            });
        }
        
        // Execute on DevNet/TestNet
        let rpc_client = RpcClient::new_with_commitment(
            self.network_config.rpc_endpoint.clone(),
            CommitmentConfig::confirmed(),
        );
        
        // Deserialize and sign the transaction
        let versioned_transaction: VersionedTransaction = bincode::deserialize(&transaction_data)
            .map_err(|e| anyhow!("Failed to deserialize versioned transaction: {}", e))?;
        
        let message = &versioned_transaction.message;
        let mut transaction = match message {
            VersionedMessage::Legacy(legacy_message) => {
                Transaction::new_unsigned(legacy_message.clone())
            }
            VersionedMessage::V0(_) => {
                return Err(anyhow!("V0 transactions not supported yet"));
            }
        };
        
        // CRITICAL: Validate Program IDs in the transaction
        info!("� Validating transaction Program IDs for {} network...", self.network_config.network);
        let jupiter_program_id = self.network_config.program_ids.jupiter_program.unwrap();
        let system_program_id = self.network_config.program_ids.system_program;
        let token_program_id = self.network_config.program_ids.token_program;
        
        // Check that the transaction uses the correct Program IDs
        let mut jupiter_found = false;
        let mut invalid_programs = Vec::new();
        
        for instruction in &transaction.message.instructions {
            let program_id = transaction.message.account_keys[instruction.program_id_index as usize];
            
            if program_id == jupiter_program_id {
                jupiter_found = true;
                info!("✅ Found correct Jupiter Program ID: {}", program_id);
            } else if program_id == system_program_id {
                info!("✅ Found correct System Program ID: {}", program_id);
            } else if program_id == token_program_id {
                info!("✅ Found correct Token Program ID: {}", program_id);
            } else {
                // Check if it's a known program for this network
                let is_known = program_id == self.network_config.program_ids.associated_token_program ||
                              program_id == self.network_config.program_ids.compute_budget_program ||
                              Some(program_id) == self.network_config.program_ids.orca_whirlpool_program ||
                              Some(program_id) == self.network_config.program_ids.spl_token_swap_program;
                
                if is_known {
                    info!("✅ Found known Program ID: {}", program_id);
                } else {
                    warn!("⚠️ Unknown Program ID: {}", program_id);
                    invalid_programs.push(program_id);
                }
            }
        }
        
        if !jupiter_found {
            error!("❌ Jupiter Program ID not found in transaction for {}", self.network_config.network);
            return Ok(SwapExecutionResult {
                success: false,
                transaction_signature: format!("INVALID_JUPITER_PROGRAM_{}", chrono::Utc::now().timestamp()),
                output_amount: 0.0,
                actual_slippage: 0.0,
                fee_amount: 0.0,
                block_height: 0,
                logs: vec![
                    format!("Jupiter Program ID {} not found in transaction", jupiter_program_id),
                    format!("This transaction may be for a different network than {}", self.network_config.network),
                ],
            });
        }
        
        if !invalid_programs.is_empty() {
            warn!("⚠️ Found {} unknown Program IDs - transaction may fail", invalid_programs.len());
            for program_id in &invalid_programs {
                warn!("   Unknown Program: {}", program_id);
            }
        }
        
        info!("✅ Program ID validation completed for {}", self.network_config.network);
        
        info!("📝 Transaction validated for {} network", self.network_config.network);
        
        // Get recent blockhash for the transaction
        match rpc_client.get_latest_blockhash() {
            Ok(blockhash) => {
                transaction.message.recent_blockhash = blockhash;
                info!("✅ Updated transaction with recent blockhash: {}", blockhash);
            }
            Err(e) => {
                error!("❌ Failed to get recent blockhash: {}", e);
                return Ok(SwapExecutionResult {
                    success: false,
                    transaction_signature: format!("BLOCKHASH_ERROR_{}", chrono::Utc::now().timestamp()),
                    output_amount: 0.0,
                    actual_slippage: 0.0,
                    fee_amount: 0.0,
                    block_height: 0,
                    logs: vec![format!("Failed to get recent blockhash: {}", e)],
                });
            }
        }
        
        info!("🚀 SPRINT 1: Sending legacy transaction to {} blockchain...", self.network_config.network);
        
        // Sign the transaction
        match transaction.try_sign(&[keypair], transaction.message.recent_blockhash) {
            Ok(_) => {
                info!("✅ Transaction signed successfully");
            }
            Err(e) => {
                error!("❌ Failed to sign transaction on {}: {} (data len: {})", 
                       self.network_config.network, e, transaction_data.len());
                return Ok(SwapExecutionResult {
                    success: false,
                    transaction_signature: format!("SIGNING_ERROR_{}", chrono::Utc::now().timestamp()),
                    output_amount: 0.0,
                    actual_slippage: 0.0,
                    fee_amount: 0.0,
                    block_height: 0,
                    logs: vec![format!("Transaction signing failed: {}", e)],
                });
            }
        }
        
        // Send the transaction
        info!("📡 Sending legacy transaction to {} blockchain...", self.network_config.network);
        
        match rpc_client.send_and_confirm_transaction(&transaction) {
            Ok(signature) => {
                info!("✅ SPRINT 1: Real swap executed successfully on {}!", self.network_config.network);
                info!("🎯 Transaction signature: {}", signature);
                info!("💰 Expected output: {} tokens", quote.out_amount());
                info!("📊 Price impact: {}%", quote.price_impact_pct());
                
                // Get the slot for additional verification
                let slot = match rpc_client.get_slot() {
                    Ok(slot) => slot,
                    Err(_) => 0, // Fallback if slot fetch fails
                };
                
                Ok(SwapExecutionResult {
                    success: true,
                    transaction_signature: signature.to_string(),
                    output_amount: quote.out_amount(),
                    actual_slippage: quote.price_impact_pct(),
                    fee_amount: 0.001, // Estimated from priority fee
                    block_height: slot,
                    logs: vec![
                        format!("Real swap executed on {}", self.network_config.network),
                        format!("Signature: {}", signature),
                        format!("✅ REAL SWAP COMPLETED ON {} (Legacy)", self.network_config.network.to_uppercase()),
                    ],
                })
            }
            Err(e) => {
                error!("❌ Transaction execution failed: {}", e);
                Ok(SwapExecutionResult {
                    success: false,
                    transaction_signature: format!("EXECUTION_FAILED_{}", chrono::Utc::now().timestamp()),
                    output_amount: 0.0,
                    actual_slippage: 0.0,
                    fee_amount: 0.0,
                    block_height: 0,
                    logs: vec![format!("Transaction execution failed: {}", e)],
                })
            }
        }
    }

    /// Check if Jupiter is properly configured
    pub fn is_configured(&self) -> bool {
        self.client.is_configured()
    }
}

// Backward compatibility - delegate to client
impl Jupiter {
    /// Execute swap (legacy method for backward compatibility)
    /// Prefer execute_swap_with_wallet() for full functionality
    pub async fn execute_swap(&self, quote: &QuoteResponse, wallet_address: &str) -> Result<SwapResult> {
        self.build_swap_transaction(quote, wallet_address).await
    }
}
