// ===== TRANSACTION EXECUTOR MODULE =====
// Ejecutor de transacciones para mainnet con confirmación

use anyhow::Result;
use solana_sdk::signature::{Keypair, Signature};
use solana_sdk::transaction::Transaction;
use solana_client::rpc_config::RpcSendTransactionConfig;
use solana_transaction_status::UiTransactionEncoding;

use crate::types::*;

pub struct TransactionExecutor;

impl TransactionExecutor {
    /// SIGN AND SEND TRANSACTION TO MAINNET
    pub async fn sign_and_send_transaction(
        engine: &ProfessionalArbitrageEngine,
        mut transaction: Transaction, 
        wallet: &Keypair
    ) -> Result<Signature> {
        // Get latest blockhash
        let latest_blockhash = engine.client.get_latest_blockhash()?;
        transaction.message.recent_blockhash = latest_blockhash;
        
        // Sign transaction
        transaction.sign(&[wallet], latest_blockhash);
        
        // Send and confirm transaction
        let signature = engine.client
            .send_and_confirm_transaction_with_spinner_and_config(
                &transaction,
                solana_sdk::commitment_config::CommitmentConfig::confirmed(),
                RpcSendTransactionConfig {
                    skip_preflight: false,
                    preflight_commitment: Some(solana_sdk::commitment_config::CommitmentLevel::Processed),
                    encoding: Some(UiTransactionEncoding::Base64),
                    max_retries: Some(3),
                    min_context_slot: None,
                },
            )?;
        
        Ok(signature)
    }
}
