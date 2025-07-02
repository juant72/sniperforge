use anyhow::{Context, Result};
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TransactionAnalysis {
    pub wallet_address: String,
    pub total_transactions: u32,
    pub recent_transactions: Vec<SolanaTransaction>,
    pub transaction_summary: TransactionSummary,
    pub last_updated: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SolanaTransaction {
    pub signature: String,
    pub block_time: Option<i64>,
    pub slot: u64,
    pub success: bool,
    pub fee: u64,
    pub transaction_type: TransactionType,
    pub sol_amount: Option<f64>,
    pub token_transfers: Vec<TokenTransfer>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum TransactionType {
    Transfer,
    TokenTransfer,
    Swap,
    StakeAccount,
    SystemProgram,
    TokenProgram,
    Unknown,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TokenTransfer {
    pub mint: String,
    pub amount: f64,
    pub from_address: Option<String>,
    pub to_address: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TransactionSummary {
    pub total_fees_paid: f64,
    pub total_sol_transferred: f64,
    pub successful_transactions: u32,
    pub failed_transactions: u32,
    pub unique_tokens_interacted: u32,
    pub swaps_count: u32,
    pub transfers_count: u32,
    pub most_active_day: Option<String>,
}

pub struct TransactionAnalyzer {
    network: String,
}

impl TransactionAnalyzer {
    pub fn new(network: &str) -> Self {
        Self {
            network: network.to_string(),
        }
    }

    /// Get real transaction history for a wallet
    pub async fn analyze_wallet_transactions(
        &self,
        wallet_address: &str,
        limit: usize,
    ) -> Result<TransactionAnalysis> {
        println!(
            "🔍 Analyzing REAL transaction history for wallet: {}",
            wallet_address
        );

        let rpc_url = match self.network.as_str() {
            "devnet" => "https://api.devnet.solana.com",
            "mainnet" => "https://api.mainnet-beta.solana.com",
            _ => return Err(anyhow::anyhow!("Invalid network")),
        };

        let wallet_addr = wallet_address.to_string();
        let url = rpc_url.to_string();

        let result = tokio::task::spawn_blocking(move || -> Result<Vec<SolanaTransaction>> {
            // Get recent transaction signatures
            let json_body = format!(
                r#"{{"jsonrpc":"2.0","id":1,"method":"getSignaturesForAddress","params":["{}",{{"limit":{}}}]}}"#,
                wallet_addr, limit
            );

            let response = ureq::post(&url)
                .header("Content-Type", "application/json")
                .send(&json_body)?;

            let response_text = response.into_string()?;
            let json: serde_json::Value = serde_json::from_str(&response_text)?;

            let mut transactions = Vec::new();

            if let Some(result) = json.get("result") {
                if let Some(signatures) = result.as_array() {
                    println!("✅ Found {} transaction signatures", signatures.len());

                    // For each signature, get detailed transaction info
                    for sig_info in signatures.iter().take(10) { // Limit to 10 for performance
                        if let Some(signature) = sig_info.get("signature").and_then(|s| s.as_str()) {
                            if let Some(block_time) = sig_info.get("blockTime") {
                                let success = sig_info.get("err").is_none();

                                // Get transaction details
                                let tx_json_body = format!(
                                    r#"{{"jsonrpc":"2.0","id":1,"method":"getTransaction","params":["{}",{{"encoding":"jsonParsed","commitment":"confirmed"}}]}}"#,
                                    signature
                                );

                                if let Ok(tx_response) = ureq::post(&url)
                                    .header("Content-Type", "application/json")
                                    .send(&tx_json_body)
                                {
                                    if let Ok(tx_text) = tx_response.into_string() {
                                        if let Ok(tx_json) = serde_json::from_str::<serde_json::Value>(&tx_text) {
                                            let transaction = parse_transaction_details(signature, block_time, success, &tx_json);
                                            transactions.push(transaction);
                                        }
                                    }
                                }
                            }
                        }
                    }
                }
            }

            Ok(transactions)
        }).await;

        let transactions = match result {
            Ok(Ok(txs)) => {
                println!("✅ Analyzed {} real transactions", txs.len());
                txs
            }
            Ok(Err(e)) => {
                println!("❌ Failed to get transactions: {}", e);
                Vec::new()
            }
            Err(e) => {
                println!("❌ Transaction analysis task failed: {}", e);
                Vec::new()
            }
        };

        // Calculate summary statistics
        let summary = self.calculate_transaction_summary(&transactions);

        Ok(TransactionAnalysis {
            wallet_address: wallet_address.to_string(),
            total_transactions: transactions.len() as u32,
            recent_transactions: transactions,
            transaction_summary: summary,
            last_updated: Utc::now(),
        })
    }

    fn calculate_transaction_summary(
        &self,
        transactions: &[SolanaTransaction],
    ) -> TransactionSummary {
        let mut total_fees_paid = 0.0;
        let mut total_sol_transferred = 0.0;
        let mut successful_transactions = 0;
        let mut failed_transactions = 0;
        let mut unique_tokens = std::collections::HashSet::new();
        let mut swaps_count = 0;
        let mut transfers_count = 0;

        for tx in transactions {
            // Calculate fees (convert from lamports to SOL)
            total_fees_paid += tx.fee as f64 / 1_000_000_000.0;

            if tx.success {
                successful_transactions += 1;
            } else {
                failed_transactions += 1;
            }

            // Count SOL transfers
            if let Some(sol_amount) = tx.sol_amount {
                total_sol_transferred += sol_amount;
            }

            // Count token interactions
            for token_transfer in &tx.token_transfers {
                unique_tokens.insert(token_transfer.mint.clone());
            }

            // Count transaction types
            match tx.transaction_type {
                TransactionType::Swap => swaps_count += 1,
                TransactionType::Transfer | TransactionType::TokenTransfer => transfers_count += 1,
                _ => {}
            }
        }

        TransactionSummary {
            total_fees_paid,
            total_sol_transferred,
            successful_transactions,
            failed_transactions,
            unique_tokens_interacted: unique_tokens.len() as u32,
            swaps_count,
            transfers_count,
            most_active_day: None, // Could be calculated from timestamps
        }
    }
}

fn parse_transaction_details(
    signature: &str,
    block_time: &serde_json::Value,
    success: bool,
    tx_json: &serde_json::Value,
) -> SolanaTransaction {
    let mut fee = 0u64;
    let mut transaction_type = TransactionType::Unknown;
    let mut sol_amount = None;
    let mut token_transfers = Vec::new();

    if let Some(result) = tx_json.get("result") {
        if let Some(meta) = result.get("meta") {
            // Get fee
            if let Some(fee_val) = meta.get("fee") {
                fee = fee_val.as_u64().unwrap_or(0);
            }

            // Analyze pre and post balances to determine SOL transfers
            if let (Some(pre_balances), Some(post_balances)) = (
                meta.get("preBalances").and_then(|b| b.as_array()),
                meta.get("postBalances").and_then(|b| b.as_array()),
            ) {
                for (i, (pre, post)) in pre_balances.iter().zip(post_balances.iter()).enumerate() {
                    if let (Some(pre_val), Some(post_val)) = (pre.as_u64(), post.as_u64()) {
                        if pre_val != post_val {
                            let diff = (post_val as i64 - pre_val as i64) as f64 / 1_000_000_000.0;
                            if diff.abs() > 0.001 {
                                // Significant SOL change
                                sol_amount = Some(diff.abs());
                                transaction_type = TransactionType::Transfer;
                            }
                        }
                    }
                }
            }

            // Check for token program interactions
            if let Some(inner_instructions) = meta.get("innerInstructions") {
                if inner_instructions
                    .as_array()
                    .map_or(false, |arr| !arr.is_empty())
                {
                    transaction_type = TransactionType::TokenProgram;
                }
            }
        }

        // Analyze transaction instructions
        if let Some(transaction) = result.get("transaction") {
            if let Some(message) = transaction.get("message") {
                if let Some(instructions) = message.get("instructions") {
                    if let Some(instr_array) = instructions.as_array() {
                        for instruction in instr_array {
                            if let Some(program_id) =
                                instruction.get("programId").and_then(|p| p.as_str())
                            {
                                match program_id {
                                    "TokenkegQfeZyiNwAJbNbGKPFXCWuBvf9Ss623VQ5DA" => {
                                        transaction_type = TransactionType::TokenTransfer;
                                    }
                                    "11111111111111111111111111111111" => {
                                        // System program - might be SOL transfer
                                        if transaction_type == TransactionType::Unknown {
                                            transaction_type = TransactionType::SystemProgram;
                                        }
                                    }
                                    _ => {}
                                }
                            }
                        }
                    }
                }
            }
        }
    }

    SolanaTransaction {
        signature: signature.to_string(),
        block_time: block_time.as_i64(),
        slot: 0, // Could be extracted from the response
        success,
        fee,
        transaction_type,
        sol_amount,
        token_transfers,
    }
}
