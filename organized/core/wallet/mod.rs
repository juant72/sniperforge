// ================================================================================
// WALLET MANAGEMENT - Gestión de Carteras
// ================================================================================

use std::sync::Arc;
use std::path::Path;
use anyhow::{Result, Context};
use tracing::{info, debug, warn, error};
use serde::{Serialize, Deserialize};
use solana_client::rpc_client::RpcClient;
use solana_sdk::{
    signature::{Keypair, Signature, Signer},
    pubkey::Pubkey,
    transaction::Transaction,
    commitment_config::CommitmentConfig,
    system_instruction,
    native_token::LAMPORTS_PER_SOL,
};
use tokio::fs;

use crate::{CoreResult, CoreError};

/// Gestor de carteras
pub struct WalletManager {
    keypair: Arc<Keypair>,
    rpc_client: Arc<RpcClient>,
    config: WalletConfig,
    statistics: WalletStatistics,
}

/// Configuración de wallet
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WalletConfig {
    pub private_key_path: String,
    pub rpc_url: String,
    pub commitment: String,
    pub min_balance_sol: f64,
    pub max_balance_usage_percentage: f64,
    pub auto_refill_enabled: bool,
    pub auto_refill_threshold_sol: f64,
    pub auto_refill_amount_sol: f64,
}

/// Estadísticas de wallet
#[derive(Debug, Clone, Default)]
pub struct WalletStatistics {
    pub transactions_sent: u64,
    pub successful_transactions: u64,
    pub failed_transactions: u64,
    pub total_fees_paid_sol: f64,
    pub total_volume_sol: f64,
    pub avg_transaction_time_ms: f64,
}

/// Resultado de transacción
#[derive(Debug, Clone)]
pub struct TransactionResult {
    pub success: bool,
    pub transaction_id: Option<String>,
    pub confirmation_time_ms: u64,
    pub fee_paid_lamports: u64,
    pub error_message: Option<String>,
    pub block_height: Option<u64>,
}

/// Balance de wallet
#[derive(Debug, Clone)]
pub struct WalletBalance {
    pub sol_balance: f64,
    pub lamports_balance: u64,
    pub usable_balance_sol: f64,
    pub reserved_balance_sol: f64,
    pub last_updated: chrono::DateTime<chrono::Utc>,
}

impl WalletManager {
    /// Crear nuevo gestor de wallet
    pub async fn new(config: &WalletConfig) -> CoreResult<Self> {
        info!("💼 Inicializando WalletManager");
        
        // Cargar keypair
        let keypair = Self::load_keypair(&config.private_key_path).await?;
        let keypair = Arc::new(keypair);
        
        // Crear cliente RPC
        let commitment = match config.commitment.as_str() {
            "processed" => CommitmentConfig::processed(),
            "confirmed" => CommitmentConfig::confirmed(),
            "finalized" => CommitmentConfig::finalized(),
            _ => CommitmentConfig::confirmed(),
        };
        
        let rpc_client = Arc::new(RpcClient::new_with_commitment(
            config.rpc_url.clone(),
            commitment,
        ));
        
        // Verificar conexión y balance
        let balance = Self::get_balance_internal(&rpc_client, &keypair.pubkey()).await?;
        
        info!("✅ Wallet inicializada: {} (Balance: {:.6} SOL)", 
              keypair.pubkey(), balance);
        
        // Verificar balance mínimo
        if balance < config.min_balance_sol {
            warn!("⚠️  Balance bajo: {:.6} SOL (mínimo: {:.6} SOL)", 
                  balance, config.min_balance_sol);
        }
        
        Ok(Self {
            keypair,
            rpc_client,
            config: config.clone(),
            statistics: WalletStatistics::default(),
        })
    }
    
    /// Cargar keypair desde archivo
    async fn load_keypair(path: &str) -> CoreResult<Keypair> {
        debug!("🔑 Cargando keypair desde: {}", path);
        
        if !Path::new(path).exists() {
            return Err(CoreError::Wallet(format!("Keypair file not found: {}", path)));
        }
        
        let keypair_data = fs::read_to_string(path).await
            .context("Failed to read keypair file")?;
        
        // Try to parse as JSON array first (standard format)
        if let Ok(bytes) = serde_json::from_str::<Vec<u8>>(&keypair_data) {
            if bytes.len() == 64 {
                return Keypair::from_bytes(&bytes)
                    .map_err(|e| CoreError::Wallet(format!("Invalid keypair format: {}", e)));
            }
        }
        
        // Try to parse as base58 string
        if let Ok(decoded) = bs58::decode(&keypair_data.trim()).into_vec() {
            if decoded.len() == 64 {
                return Keypair::from_bytes(&decoded)
                    .map_err(|e| CoreError::Wallet(format!("Invalid keypair format: {}", e)));
            }
        }
        
        Err(CoreError::Wallet("Unsupported keypair format".to_string()))
    }
    
    /// Obtener balance SOL
    pub async fn get_sol_balance(&self) -> CoreResult<f64> {
        let balance = Self::get_balance_internal(&self.rpc_client, &self.keypair.pubkey()).await?;
        Ok(balance)
    }
    
    /// Obtener balance interno
    async fn get_balance_internal(rpc_client: &RpcClient, pubkey: &Pubkey) -> CoreResult<f64> {
        let balance_lamports = rpc_client.get_balance(pubkey)
            .map_err(|e| CoreError::Rpc(format!("Failed to get balance: {}", e)))?;
        
        Ok(balance_lamports as f64 / LAMPORTS_PER_SOL as f64)
    }
    
    /// Obtener balance detallado
    pub async fn get_wallet_balance(&self) -> CoreResult<WalletBalance> {
        let lamports_balance = self.rpc_client.get_balance(&self.keypair.pubkey())
            .map_err(|e| CoreError::Rpc(format!("Failed to get balance: {}", e)))?;
        
        let sol_balance = lamports_balance as f64 / LAMPORTS_PER_SOL as f64;
        
        // Calcular balance usable (respetando límites de configuración)
        let max_usable = sol_balance * self.config.max_balance_usage_percentage;
        let reserved = self.config.min_balance_sol;
        let usable_balance = (max_usable - reserved).max(0.0);
        
        Ok(WalletBalance {
            sol_balance,
            lamports_balance,
            usable_balance_sol: usable_balance,
            reserved_balance_sol: reserved,
            last_updated: chrono::Utc::now(),
        })
    }
    
    /// Enviar transacción
    pub async fn submit_transaction(&mut self, transaction: &Transaction) -> CoreResult<TransactionResult> {
        let start_time = std::time::Instant::now();
        self.statistics.transactions_sent += 1;
        
        debug!("📤 Enviando transacción...");
        
        // Verificar balance antes de enviar
        let balance = self.get_sol_balance().await?;
        if balance < self.config.min_balance_sol {
            return Err(CoreError::Wallet(format!(
                "Insufficient balance: {:.6} SOL (minimum: {:.6} SOL)", 
                balance, self.config.min_balance_sol
            )));
        }
        
        // Enviar transacción
        match self.rpc_client.send_and_confirm_transaction(transaction) {
            Ok(signature) => {
                let confirmation_time = start_time.elapsed().as_millis() as u64;
                
                // Obtener información de la transacción
                let tx_info = self.rpc_client.get_transaction(&signature, 
                    solana_client::rpc_config::RpcTransactionConfig::default());
                
                let (fee_paid, block_height) = if let Ok(tx) = tx_info {
                    (tx.transaction.meta.as_ref().map(|m| m.fee).unwrap_or(0),
                     tx.slot.map(|s| s as u64))
                } else {
                    (5000, None) // Default fee estimate
                };
                
                // Actualizar estadísticas
                self.statistics.successful_transactions += 1;
                self.statistics.total_fees_paid_sol += fee_paid as f64 / LAMPORTS_PER_SOL as f64;
                self.update_avg_transaction_time(confirmation_time);
                
                info!("✅ Transacción confirmada: {} ({}ms)", 
                      signature, confirmation_time);
                
                Ok(TransactionResult {
                    success: true,
                    transaction_id: Some(signature.to_string()),
                    confirmation_time_ms: confirmation_time,
                    fee_paid_lamports: fee_paid,
                    error_message: None,
                    block_height,
                })
            },
            Err(e) => {
                let confirmation_time = start_time.elapsed().as_millis() as u64;
                self.statistics.failed_transactions += 1;
                
                error!("❌ Error en transacción: {}", e);
                
                Ok(TransactionResult {
                    success: false,
                    transaction_id: None,
                    confirmation_time_ms: confirmation_time,
                    fee_paid_lamports: 0,
                    error_message: Some(e.to_string()),
                    block_height: None,
                })
            }
        }
    }
    
    /// Actualizar tiempo promedio de transacción
    fn update_avg_transaction_time(&mut self, new_time_ms: u64) {
        let total_successful = self.statistics.successful_transactions;
        if total_successful == 1 {
            self.statistics.avg_transaction_time_ms = new_time_ms as f64;
        } else {
            self.statistics.avg_transaction_time_ms = 
                (self.statistics.avg_transaction_time_ms * (total_successful - 1) as f64 + new_time_ms as f64) 
                / total_successful as f64;
        }
    }
    
    /// Crear transacción de transferencia SOL
    pub async fn create_sol_transfer(&self, to: &Pubkey, amount_sol: f64) -> CoreResult<Transaction> {
        let amount_lamports = (amount_sol * LAMPORTS_PER_SOL as f64) as u64;
        
        // Verificar balance suficiente
        let balance = self.get_wallet_balance().await?;
        if balance.usable_balance_sol < amount_sol {
            return Err(CoreError::Wallet(format!(
                "Insufficient usable balance: {:.6} SOL (required: {:.6} SOL)", 
                balance.usable_balance_sol, amount_sol
            )));
        }
        
        // Obtener recent blockhash
        let recent_blockhash = self.rpc_client.get_latest_blockhash()
            .map_err(|e| CoreError::Rpc(format!("Failed to get recent blockhash: {}", e)))?;
        
        // Crear instrucción de transferencia
        let instruction = system_instruction::transfer(
            &self.keypair.pubkey(),
            to,
            amount_lamports,
        );
        
        // Crear transacción
        let transaction = Transaction::new_signed_with_payer(
            &[instruction],
            Some(&self.keypair.pubkey()),
            &[&*self.keypair],
            recent_blockhash,
        );
        
        Ok(transaction)
    }
    
    /// Verificar si el auto-refill está disponible y es necesario
    pub async fn check_auto_refill(&self) -> CoreResult<bool> {
        if !self.config.auto_refill_enabled {
            return Ok(false);
        }
        
        let balance = self.get_sol_balance().await?;
        Ok(balance < self.config.auto_refill_threshold_sol)
    }
    
    /// Ejecutar auto-refill (simulado - en producción conectaría con exchange)
    pub async fn execute_auto_refill(&mut self) -> CoreResult<TransactionResult> {
        warn!("🔄 Auto-refill solicitado pero no implementado en esta versión");
        
        // En producción, esto conectaría con un exchange o funding source
        // Por ahora, solo registramos el evento
        
        Ok(TransactionResult {
            success: false,
            transaction_id: None,
            confirmation_time_ms: 0,
            fee_paid_lamports: 0,
            error_message: Some("Auto-refill not implemented".to_string()),
            block_height: None,
        })
    }
    
    /// Obtener pubkey del wallet
    pub fn get_pubkey(&self) -> Pubkey {
        self.keypair.pubkey()
    }
    
    /// Obtener estadísticas
    pub fn get_statistics(&self) -> &WalletStatistics {
        &self.statistics
    }
    
    /// Generar nuevo keypair y guardarlo
    pub async fn generate_new_keypair(path: &str) -> CoreResult<Pubkey> {
        let keypair = Keypair::new();
        let pubkey = keypair.pubkey();
        
        // Convertir a formato JSON (array de bytes)
        let keypair_bytes = keypair.to_bytes();
        let keypair_json = serde_json::to_string_pretty(&keypair_bytes.to_vec())
            .map_err(|e| CoreError::Wallet(format!("Failed to serialize keypair: {}", e)))?;
        
        // Guardar a archivo
        fs::write(path, keypair_json).await
            .context("Failed to write keypair file")?;
        
        info!("🔑 Nuevo keypair generado: {} (guardado en: {})", pubkey, path);
        
        Ok(pubkey)
    }
    
    /// Validar configuración de wallet
    pub fn validate_config(config: &WalletConfig) -> CoreResult<()> {
        if config.min_balance_sol <= 0.0 {
            return Err(CoreError::Configuration("min_balance_sol must be > 0".to_string()));
        }
        
        if config.max_balance_usage_percentage <= 0.0 || config.max_balance_usage_percentage > 1.0 {
            return Err(CoreError::Configuration(
                "max_balance_usage_percentage must be between 0 and 1".to_string()
            ));
        }
        
        if config.auto_refill_enabled {
            if config.auto_refill_threshold_sol <= 0.0 {
                return Err(CoreError::Configuration(
                    "auto_refill_threshold_sol must be > 0 when auto_refill is enabled".to_string()
                ));
            }
            
            if config.auto_refill_amount_sol <= 0.0 {
                return Err(CoreError::Configuration(
                    "auto_refill_amount_sol must be > 0 when auto_refill is enabled".to_string()
                ));
            }
        }
        
        Ok(())
    }
}

impl Default for WalletConfig {
    fn default() -> Self {
        Self {
            private_key_path: "wallet.json".to_string(),
            rpc_url: "https://api.mainnet-beta.solana.com".to_string(),
            commitment: "confirmed".to_string(),
            min_balance_sol: 0.05,
            max_balance_usage_percentage: 0.8,
            auto_refill_enabled: false,
            auto_refill_threshold_sol: 0.1,
            auto_refill_amount_sol: 1.0,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_wallet_config_validation() {
        let mut config = WalletConfig::default();
        assert!(WalletManager::validate_config(&config).is_ok());
        
        config.min_balance_sol = -1.0;
        assert!(WalletManager::validate_config(&config).is_err());
        
        config.min_balance_sol = 0.05;
        config.max_balance_usage_percentage = 1.5;
        assert!(WalletManager::validate_config(&config).is_err());
    }
    
    #[test]
    fn test_wallet_balance_calculation() {
        // Test balance calculations
        let lamports = 1_000_000_000u64; // 1 SOL
        let sol = lamports as f64 / LAMPORTS_PER_SOL as f64;
        assert_eq!(sol, 1.0);
    }
}
