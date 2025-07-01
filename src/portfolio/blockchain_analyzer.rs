use anyhow::{Result, Context};
use serde::{Deserialize, Serialize};
use solana_client::rpc_client::RpcClient;
use solana_sdk::{pubkey::Pubkey, signature::Signature};
use std::str::FromStr;
use std::collections::HashMap;
use tokio::time::{Duration, timeout};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TransactionHistory {
    pub wallet_address: String,
    pub transactions: Vec<TransactionRecord>,
    pub total_count: usize,
    pub last_updated: chrono::DateTime<chrono::Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TransactionRecord {
    pub signature: String,
    pub block_time: Option<i64>,
    pub slot: u64,
    pub transaction_type: TransactionType,
    pub sol_change: f64,
    pub token_changes: Vec<TokenChange>,
    pub fee: f64,
    pub status: TransactionStatus,
    pub programs: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum TransactionType {
    Transfer,
    TokenSwap,
    LiquidityProvision,
    Staking,
    NFTTrade,
    ProgramInteraction,
    Unknown,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TokenChange {
    pub mint: String,
    pub symbol: String,
    pub amount_change: f64,
    pub decimals: u8,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum TransactionStatus {
    Success,
    Failed,
    Unknown,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PortfolioPerformance {
    pub wallet_address: String,
    pub time_period: String,
    pub initial_value_usd: f64,
    pub current_value_usd: f64,
    pub total_return: f64,
    pub total_return_percentage: f64,
    pub realized_pnl: f64,
    pub unrealized_pnl: f64,
    pub total_fees_paid: f64,
    pub transaction_count: usize,
    pub win_rate: f64,
    pub sharpe_ratio: Option<f64>,
    pub max_drawdown: Option<f64>,
    pub calculated_at: chrono::DateTime<chrono::Utc>,
}

#[derive(Debug, Clone)]
pub struct BlockchainAnalyzer {
    rpc_client: RpcClient,
    network: String,
}

impl BlockchainAnalyzer {
    pub fn new(network: &str) -> Result<Self> {
        let rpc_url = match network {
            "devnet" => "https://api.devnet.solana.com",
            "mainnet" => "https://api.mainnet-beta.solana.com",
            _ => return Err(anyhow::anyhow!("Invalid network: {}", network)),
        };

        let rpc_client = RpcClient::new(rpc_url.to_string());

        Ok(Self {
            rpc_client,
            network: network.to_string(),
        })
    }

    pub async fn get_transaction_history(&self, wallet_address: &str, limit: usize) -> Result<TransactionHistory> {
        println!("📊 Analyzing transaction history for: {}", wallet_address);

        let pubkey = Pubkey::from_str(wallet_address)
            .context("Invalid wallet address format")?;

        // Get recent transaction signatures with timeout
        let signatures = timeout(Duration::from_secs(20), async {
            self.rpc_client.get_signatures_for_address_with_config(
                &pubkey,
                solana_client::rpc_config::RpcGetSignaturesForAddressConfig {
                    limit: Some(limit),
                    ..Default::default()
                },
            )
        }).await
            .context("Timeout getting transaction signatures")?
            .context("Failed to get transaction signatures")?;

        let mut transactions = Vec::new();
        let mut processed = 0;
        let max_process = std::cmp::min(signatures.len(), 50); // Limit processing to avoid timeouts

        for sig_info in signatures.iter().take(max_process) {
            if processed >= max_process {
                break;
            }

            match self.analyze_transaction(&sig_info.signature, wallet_address).await {
                Ok(tx_record) => {
                    transactions.push(tx_record);
                    processed += 1;
                },
                Err(e) => {
                    eprintln!("⚠️ Failed to analyze transaction {}: {}", sig_info.signature, e);
                }
            }

            // Small delay to avoid rate limiting
            tokio::time::sleep(Duration::from_millis(100)).await;
        }

        Ok(TransactionHistory {
            wallet_address: wallet_address.to_string(),
            transactions,
            total_count: signatures.len(),
            last_updated: chrono::Utc::now(),
        })
    }

    async fn analyze_transaction(&self, signature: &str, wallet_address: &str) -> Result<TransactionRecord> {
        let sig = Signature::from_str(signature)
            .context("Invalid transaction signature")?;

        let transaction = timeout(Duration::from_secs(10), async {
            self.rpc_client.get_transaction_with_config(
                &sig,
                solana_client::rpc_config::RpcTransactionConfig {
                    encoding: Some(solana_transaction_status::UiTransactionEncoding::Json),
                    commitment: Some(solana_client::rpc_config::RpcGetConfirmedTransactionConfig::default().commitment),
                    max_supported_transaction_version: Some(0),
                },
            )
        }).await
            .context("Timeout getting transaction details")?
            .context("Failed to get transaction details")?;

        let block_time = transaction.block_time;
        let slot = transaction.slot;

        // Extract transaction metadata
        let meta = transaction.transaction.meta.unwrap_or_default();
        let fee = meta.fee as f64 / 1_000_000_000.0; // Convert lamports to SOL

        // Determine transaction status
        let status = if meta.err.is_some() {
            TransactionStatus::Failed
        } else {
            TransactionStatus::Success
        };

        // Analyze balance changes
        let wallet_pubkey = Pubkey::from_str(wallet_address)?;
        let mut sol_change = 0.0;

        if let (Some(pre_balances), Some(post_balances)) = (meta.pre_balances, meta.post_balances) {
            // Find wallet's account index
            if let Some(account_keys) = transaction.transaction.transaction.message.account_keys() {
                for (index, account_key) in account_keys.iter().enumerate() {
                    if *account_key == wallet_pubkey {
                        if let (Some(pre), Some(post)) = (pre_balances.get(index), post_balances.get(index)) {
                            sol_change = (*post as f64 - *pre as f64) / 1_000_000_000.0;
                        }
                        break;
                    }
                }
            }
        }

        // Analyze token changes (simplified)
        let token_changes = self.analyze_token_changes(&meta, wallet_address).await
            .unwrap_or_default();

        // Determine transaction type based on program interactions
        let programs: Vec<String> = if let Some(account_keys) = transaction.transaction.transaction.message.account_keys() {
            account_keys.iter().map(|key| key.to_string()).collect()
        } else {
            Vec::new()
        };

        let transaction_type = self.classify_transaction(&programs, &token_changes);

        Ok(TransactionRecord {
            signature: signature.to_string(),
            block_time,
            slot,
            transaction_type,
            sol_change,
            token_changes,
            fee,
            status,
            programs,
        })
    }

    async fn analyze_token_changes(&self, _meta: &solana_transaction_status::UiTransactionStatusMeta, _wallet_address: &str) -> Result<Vec<TokenChange>> {
        // This is a simplified implementation
        // In a full implementation, we would parse the transaction logs and account changes
        // to determine exact token balance changes
        Ok(Vec::new())
    }

    fn classify_transaction(&self, programs: &[String], token_changes: &[TokenChange]) -> TransactionType {
        // Simplified classification based on program interactions
        for program in programs {
            if program.contains("Jupiter") || program.contains("Raydium") || program.contains("Orca") {
                return TransactionType::TokenSwap;
            }
            if program.contains("Stake") {
                return TransactionType::Staking;
            }
            if program.contains("Metaplex") || program.contains("NFT") {
                return TransactionType::NFTTrade;
            }
        }

        if !token_changes.is_empty() {
            TransactionType::TokenSwap
        } else {
            TransactionType::Transfer
        }
    }

    pub async fn calculate_portfolio_performance(&self, wallet_address: &str, time_period_days: u32) -> Result<PortfolioPerformance> {
        println!("📈 Calculating portfolio performance for: {}", wallet_address);

        // This is a simplified implementation
        // In a full implementation, we would:
        // 1. Get historical transactions
        // 2. Calculate portfolio value at different time points
        // 3. Compute performance metrics

        let history = self.get_transaction_history(wallet_address, 100).await?;

        let total_fees = history.transactions.iter()
            .map(|tx| tx.fee)
            .sum::<f64>();

        let successful_txs = history.transactions.iter()
            .filter(|tx| matches!(tx.status, TransactionStatus::Success))
            .count();

        let win_rate = if history.transactions.len() > 0 {
            successful_txs as f64 / history.transactions.len() as f64
        } else {
            0.0
        };

        Ok(PortfolioPerformance {
            wallet_address: wallet_address.to_string(),
            time_period: format!("{} days", time_period_days),
            initial_value_usd: 0.0, // Would need historical price data
            current_value_usd: 0.0, // Would need current portfolio value
            total_return: 0.0,
            total_return_percentage: 0.0,
            realized_pnl: 0.0,
            unrealized_pnl: 0.0,
            total_fees_paid: total_fees,
            transaction_count: history.transactions.len(),
            win_rate,
            sharpe_ratio: None,
            max_drawdown: None,
            calculated_at: chrono::Utc::now(),
        })
    }

    pub fn get_network(&self) -> &str {
        &self.network
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_blockchain_analyzer_creation() {
        let analyzer = BlockchainAnalyzer::new("devnet").unwrap();
        assert_eq!(analyzer.get_network(), "devnet");
    }

    #[tokio::test]
    async fn test_invalid_network() {
        let result = BlockchainAnalyzer::new("invalid");
        assert!(result.is_err());
    }
}
