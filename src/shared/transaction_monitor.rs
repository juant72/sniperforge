//! Transaction Monitoring System
//! 
//! Monitors Solana transactions to confirm completion and handle failures
//! Critical for trading safety - ensures swaps are actually executed

use anyhow::{Result, anyhow};
use serde::{Deserialize, Serialize};
use solana_client::rpc_client::RpcClient;
use solana_sdk::signature::Signature;
use std::collections::HashMap;
use std::str::FromStr;
use std::time::{Duration, Instant};
use tokio::time::timeout;
use tracing::{info, warn, error, debug};

/// Transaction status tracking
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum TransactionStatus {
    Pending,
    Confirmed,
    Finalized,
    Failed(String),
    Timeout,
}

/// Transaction monitoring result
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TransactionResult {
    pub signature: String,
    pub status: TransactionStatus,
    pub confirmation_time: Option<Duration>,
    pub slot: Option<u64>,
    pub block_hash: Option<String>,
    pub fee_paid: Option<u64>,
    pub compute_units_consumed: Option<u64>,
    pub error_message: Option<String>,
    pub logs: Vec<String>,
}

/// Configuration for transaction monitoring
#[derive(Debug, Clone)]
pub struct TransactionMonitorConfig {
    pub max_confirmation_time: Duration,
    pub polling_interval: Duration,
    pub commitment_level: String, // "confirmed" or "finalized"
    pub retry_attempts: u32,
}

impl Default for TransactionMonitorConfig {
    fn default() -> Self {
        Self {
            max_confirmation_time: Duration::from_secs(60), // 1 minute max
            polling_interval: Duration::from_millis(500),   // Check every 500ms
            commitment_level: "confirmed".to_string(),
            retry_attempts: 3,
        }
    }
}

/// Real-time transaction monitor for Solana trades
pub struct TransactionMonitor {
    rpc_client: RpcClient,
    config: TransactionMonitorConfig,
    pending_transactions: HashMap<String, Instant>,
}

impl TransactionMonitor {
    /// Create new transaction monitor
    pub fn new(rpc_endpoint: &str, config: TransactionMonitorConfig) -> Self {
        let rpc_client = RpcClient::new(rpc_endpoint.to_string());
        
        Self {
            rpc_client,
            config,
            pending_transactions: HashMap::new(),
        }
    }

    /// Monitor a transaction until completion or timeout
    pub async fn monitor_transaction(&mut self, signature: &str) -> Result<TransactionResult> {
        info!("🔍 Monitoring transaction: {}", signature);
        
        let sig = Signature::from_str(signature)
            .map_err(|e| anyhow!("Invalid signature format: {}", e))?;
        
        let start_time = Instant::now();
        self.pending_transactions.insert(signature.to_string(), start_time);
        
        // Monitor with timeout
        let result = timeout(
            self.config.max_confirmation_time,
            self.poll_transaction_status(&sig)
        ).await;
        
        // Remove from pending
        self.pending_transactions.remove(signature);
        
        match result {
            Ok(transaction_result) => {
                let confirmation_time = start_time.elapsed();
                info!("✅ Transaction {} completed in {:?}", 
                      signature, confirmation_time);
                
                let mut final_result = transaction_result?;
                final_result.confirmation_time = Some(confirmation_time);
                Ok(final_result)
            }
            Err(_) => {
                warn!("⏰ Transaction {} timed out after {:?}", 
                      signature, self.config.max_confirmation_time);
                
                Ok(TransactionResult {
                    signature: signature.to_string(),
                    status: TransactionStatus::Timeout,
                    confirmation_time: Some(start_time.elapsed()),
                    slot: None,
                    block_hash: None,
                    fee_paid: None,
                    compute_units_consumed: None,
                    error_message: Some("Transaction confirmation timeout".to_string()),
                    logs: vec![],
                })
            }
        }
    }

    /// Poll transaction status until confirmed or failed
    async fn poll_transaction_status(&self, signature: &Signature) -> Result<TransactionResult> {
        let mut attempt = 0;
        
        loop {
            attempt += 1;
            
            match self.check_transaction_status(signature).await {
                Ok(result) => {
                    match result.status {
                        TransactionStatus::Confirmed | 
                        TransactionStatus::Finalized => {
                            debug!("✅ Transaction confirmed on attempt {}", attempt);
                            return Ok(result);
                        }
                        TransactionStatus::Failed(_) => {
                            warn!("❌ Transaction failed on attempt {}", attempt);
                            return Ok(result);
                        }
                        TransactionStatus::Pending => {
                            debug!("⏳ Transaction still pending (attempt {})", attempt);
                            // Continue polling
                        }
                        TransactionStatus::Timeout => {
                            return Ok(result);
                        }
                    }
                }
                Err(e) => {
                    if attempt >= self.config.retry_attempts {
                        error!("❌ Failed to check transaction after {} attempts: {}", 
                               attempt, e);
                        return Err(e);
                    }
                    warn!("⚠️ Attempt {} failed, retrying: {}", attempt, e);
                }
            }
            
            // Wait before next poll
            tokio::time::sleep(self.config.polling_interval).await;
        }
    }

    /// Check current status of a transaction
    async fn check_transaction_status(&self, signature: &Signature) -> Result<TransactionResult> {
        debug!("🔍 Checking status for signature: {}", signature);
        
        // Get transaction details
        let signature_status = self.rpc_client
            .get_signature_status(signature)
            .map_err(|e| anyhow!("Failed to get signature status: {}", e))?;
        
        match signature_status {
            Some(status_result) => {
                match status_result {
                    Ok(()) => {
                        // Transaction succeeded, get full details
                        self.get_transaction_details(signature).await
                    }
                    Err(transaction_error) => {
                        // Transaction failed
                        warn!("❌ Transaction failed: {:?}", transaction_error);
                        
                        Ok(TransactionResult {
                            signature: signature.to_string(),
                            status: TransactionStatus::Failed(format!("{:?}", transaction_error)),
                            confirmation_time: None,
                            slot: None,
                            block_hash: None,
                            fee_paid: None,
                            compute_units_consumed: None,
                            error_message: Some(format!("{:?}", transaction_error)),
                            logs: vec![],
                        })
                    }
                }
            }
            None => {
                // Transaction not found or still pending
                debug!("⏳ Transaction not yet confirmed");
                
                Ok(TransactionResult {
                    signature: signature.to_string(),
                    status: TransactionStatus::Pending,
                    confirmation_time: None,
                    slot: None,
                    block_hash: None,
                    fee_paid: None,
                    compute_units_consumed: None,
                    error_message: None,
                    logs: vec![],
                })
            }
        }
    }

    /// Get detailed transaction information
    async fn get_transaction_details(&self, signature: &Signature) -> Result<TransactionResult> {
        use solana_client::rpc_config::RpcTransactionConfig;
        use solana_transaction_status::UiTransactionEncoding;
        
        let config = RpcTransactionConfig {
            encoding: Some(UiTransactionEncoding::Json),
            commitment: Some(
                solana_sdk::commitment_config::CommitmentConfig {
                    commitment: match self.config.commitment_level.as_str() {
                        "finalized" => solana_sdk::commitment_config::CommitmentLevel::Finalized,
                        _ => solana_sdk::commitment_config::CommitmentLevel::Confirmed,
                    }
                }
            ),
            max_supported_transaction_version: Some(0),
        };
        
        match self.rpc_client.get_transaction_with_config(signature, config) {
            Ok(transaction) => {
                debug!("📊 Got transaction details for {}", signature);
                
                let slot = transaction.slot;
                let block_hash = None; // Block hash not directly available in this structure
                
                // Extract fee information
                let fee_paid = transaction.transaction.meta
                    .as_ref()
                    .map(|meta| meta.fee);
                
                // Extract compute units (handle OptionSerializer)
                let compute_units_consumed = transaction.transaction.meta
                    .as_ref()
                    .and_then(|meta| {
                        // Handle OptionSerializer by extracting the inner value
                        match &meta.compute_units_consumed {
                            solana_transaction_status::option_serializer::OptionSerializer::Some(units) => Some(*units),
                            _ => None,
                        }
                    });
                
                // Extract logs (handle OptionSerializer)
                let logs = transaction.transaction.meta
                    .as_ref()
                    .map(|meta| {
                        match &meta.log_messages {
                            solana_transaction_status::option_serializer::OptionSerializer::Some(messages) => messages.clone(),
                            _ => vec![],
                        }
                    })
                    .unwrap_or_default();
                
                // Determine final status
                let status = if let Some(meta) = &transaction.transaction.meta {
                    if meta.err.is_some() {
                        TransactionStatus::Failed("Transaction execution failed".to_string())
                    } else {
                        match self.config.commitment_level.as_str() {
                            "finalized" => TransactionStatus::Finalized,
                            _ => TransactionStatus::Confirmed,
                        }
                    }
                } else {
                    TransactionStatus::Confirmed
                };
                
                Ok(TransactionResult {
                    signature: signature.to_string(),
                    status,
                    confirmation_time: None, // Will be set by caller
                    slot: Some(slot),
                    block_hash,
                    fee_paid,
                    compute_units_consumed,
                    error_message: None,
                    logs,
                })
            }
            Err(e) => {
                warn!("⚠️ Could not get transaction details: {}", e);
                
                // Return basic confirmed status if we can't get details
                Ok(TransactionResult {
                    signature: signature.to_string(),
                    status: TransactionStatus::Confirmed,
                    confirmation_time: None,
                    slot: None,
                    block_hash: None,
                    fee_paid: None,
                    compute_units_consumed: None,
                    error_message: Some(format!("Could not retrieve details: {}", e)),
                    logs: vec![],
                })
            }
        }
    }

    /// Monitor multiple transactions concurrently
    pub async fn monitor_multiple_transactions(
        &mut self, 
        signatures: Vec<String>
    ) -> Result<Vec<TransactionResult>> {
        info!("🔍 Monitoring {} transactions concurrently", signatures.len());
        
        let mut handles = Vec::new();
        
        for signature in signatures {
            let monitor_config = self.config.clone();
            let rpc_endpoint = self.rpc_client.url();
            
            let handle = tokio::spawn(async move {
                let mut monitor = TransactionMonitor::new(&rpc_endpoint, monitor_config);
                monitor.monitor_transaction(&signature).await
            });
            
            handles.push(handle);
        }
        
        let mut results = Vec::new();
        for handle in handles {
            match handle.await {
                Ok(Ok(result)) => results.push(result),
                Ok(Err(e)) => {
                    error!("❌ Transaction monitoring failed: {}", e);
                    // Add failed result
                    results.push(TransactionResult {
                        signature: "unknown".to_string(),
                        status: TransactionStatus::Failed(e.to_string()),
                        confirmation_time: None,
                        slot: None,
                        block_hash: None,
                        fee_paid: None,
                        compute_units_consumed: None,
                        error_message: Some(e.to_string()),
                        logs: vec![],
                    });
                }
                Err(e) => {
                    error!("❌ Task panicked: {}", e);
                }
            }
        }
        
        Ok(results)
    }

    /// Get status of all pending transactions
    pub fn get_pending_transactions(&self) -> Vec<(String, Duration)> {
        let now = Instant::now();
        self.pending_transactions
            .iter()
            .map(|(sig, start_time)| (sig.clone(), now.duration_since(*start_time)))
            .collect()
    }

    /// Clear old pending transactions (cleanup)
    pub fn cleanup_pending_transactions(&mut self) {
        let now = Instant::now();
        let max_age = self.config.max_confirmation_time * 2; // Give extra time
        
        self.pending_transactions.retain(|_, start_time| {
            now.duration_since(*start_time) < max_age
        });
    }

    /// Check if transaction monitoring is healthy
    pub fn health_check(&self) -> Result<()> {
        // Simple health check - try to get latest blockhash
        match self.rpc_client.get_latest_blockhash() {
            Ok(_) => {
                debug!("✅ Transaction monitor RPC connection healthy");
                Ok(())
            }
            Err(e) => {
                error!("❌ Transaction monitor RPC connection failed: {}", e);
                Err(anyhow!("RPC connection unhealthy: {}", e))
            }
        }
    }
}

/// Convenience function to monitor a single transaction with default config
pub async fn monitor_transaction_simple(
    rpc_endpoint: &str,
    signature: &str
) -> Result<TransactionResult> {
    let config = TransactionMonitorConfig::default();
    let mut monitor = TransactionMonitor::new(rpc_endpoint, config);
    monitor.monitor_transaction(signature).await
}

/// Test function for transaction monitoring
pub async fn test_transaction_monitoring() -> Result<()> {
    println!("🧪 Testing Transaction Monitoring System");
    
    // Create test monitor with short timeouts for testing
    let config = TransactionMonitorConfig {
        max_confirmation_time: Duration::from_secs(10),
        polling_interval: Duration::from_millis(200),
        commitment_level: "confirmed".to_string(),
        retry_attempts: 2,
    };
    
    let monitor = TransactionMonitor::new(
        "https://api.devnet.solana.com",
        config
    );
    
    // Test health check
    match monitor.health_check() {
        Ok(_) => println!("✅ Health check passed"),
        Err(e) => {
            println!("❌ Health check failed: {}", e);
            return Err(e);
        }
    }
    
    println!("✅ Transaction monitoring system test completed");
    Ok(())
}
