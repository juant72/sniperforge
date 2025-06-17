use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;
use anyhow::Result;
use tracing::{info, warn, error, debug};
use solana_sdk::{
    signer::{Signer, keypair::Keypair},
    pubkey::Pubkey,
    signature::Signature,
    transaction::Transaction,
};
use serde::{Serialize, Deserialize};

use crate::config::Config;
use crate::types::{PlatformError, HealthStatus};

/// Wallet configuration for different purposes
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WalletConfig {
    pub name: String,
    pub wallet_type: WalletType,
    pub keypair_path: Option<String>,
    pub keypair_data: Option<String>, // Base58 encoded
    pub max_sol_balance: f64,
    pub min_sol_balance: f64,
    pub risk_management: RiskManagement,
}

/// Types of wallets in the system
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum WalletType {
    Trading,      // Main trading wallet
    Fee,          // For transaction fees
    Emergency,    // Emergency backup wallet
    Testing,      // For testing purposes
}

/// Risk management settings per wallet
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RiskManagement {
    pub max_transaction_amount: f64,
    pub daily_limit: f64,
    pub require_confirmation: bool,
    pub emergency_stop_threshold: f64,
}

/// Managed wallet instance
#[derive(Debug, Clone)]
pub struct ManagedWallet {
    pub config: WalletConfig,
    pub keypair: Arc<Keypair>,
    pub pubkey: Pubkey,
    pub balance_sol: f64,
    pub last_balance_check: chrono::DateTime<chrono::Utc>,
    pub daily_volume: f64,
    pub is_locked: bool,
    pub lock_reason: Option<String>,
}

/// Transaction signing request
#[derive(Debug, Clone)]
pub struct SigningRequest {
    pub wallet_name: String,
    pub transaction: Transaction,
    pub amount_sol: f64,
    pub description: String,
    pub urgency: TransactionUrgency,
}

/// Transaction urgency levels
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub enum TransactionUrgency {
    Low,
    Normal,
    High,
    Critical,
}

/// Wallet manager for handling multiple wallets securely
pub struct WalletManager {
    wallets: Arc<RwLock<HashMap<String, ManagedWallet>>>,
    config: Config,
    daily_volumes: Arc<RwLock<HashMap<String, f64>>>,
    emergency_stop: Arc<RwLock<bool>>,
}

impl WalletManager {
    pub async fn new(config: &Config) -> Result<Self> {
        info!("🔐 Initializing Wallet Manager");
        
        let mut manager = Self {
            wallets: Arc::new(RwLock::new(HashMap::new())),
            config: config.clone(),
            daily_volumes: Arc::new(RwLock::new(HashMap::new())),
            emergency_stop: Arc::new(RwLock::new(false)),
        };

        // Load configured wallets
        manager.load_configured_wallets().await?;
        
        info!("✅ Wallet Manager initialized with {} wallets", 
              manager.wallets.read().await.len());
        
        Ok(manager)
    }

    /// Start the wallet manager
    pub async fn start(&self) -> Result<()> {
        info!("🚀 Starting Wallet Manager");
        
        // Start balance monitoring task
        self.start_balance_monitoring().await;
        
        // Start daily reset task
        self.start_daily_reset_task().await;
        
        Ok(())
    }

    /// Stop the wallet manager
    pub async fn stop(&self) -> Result<()> {
        info!("🛑 Stopping Wallet Manager");
        
        // Lock all wallets
        let mut wallets = self.wallets.write().await;
        for wallet in wallets.values_mut() {
            wallet.is_locked = true;
            wallet.lock_reason = Some("System shutdown".to_string());
        }
        
        Ok(())
    }

    /// Add a new wallet to management
    pub async fn add_wallet(&self, config: WalletConfig) -> Result<()> {
        let keypair = self.load_keypair(&config).await?;
        let pubkey = keypair.pubkey();
        
        let wallet = ManagedWallet {
            config: config.clone(),
            keypair: Arc::new(keypair),
            pubkey,
            balance_sol: 0.0,
            last_balance_check: chrono::Utc::now(),
            daily_volume: 0.0,
            is_locked: false,
            lock_reason: None,
        };

        let mut wallets = self.wallets.write().await;
        wallets.insert(config.name.clone(), wallet);
        
        info!("💳 Added wallet: {} ({})", config.name, pubkey);
        Ok(())
    }

    /// Get wallet public key
    pub async fn get_wallet_pubkey(&self, wallet_name: &str) -> Option<Pubkey> {
        let wallets = self.wallets.read().await;
        wallets.get(wallet_name).map(|w| w.pubkey)
    }

    /// Get wallet balance
    pub async fn get_wallet_balance(&self, wallet_name: &str) -> Option<f64> {
        let wallets = self.wallets.read().await;
        wallets.get(wallet_name).map(|w| w.balance_sol)
    }

    /// Check if wallet is available for transactions
    pub async fn is_wallet_available(&self, wallet_name: &str, amount_sol: f64) -> Result<bool> {
        // Check emergency stop
        if *self.emergency_stop.read().await {
            return Ok(false);
        }

        let wallets = self.wallets.read().await;
        if let Some(wallet) = wallets.get(wallet_name) {
            // Check if wallet is locked
            if wallet.is_locked {
                return Ok(false);
            }

            // Check balance
            if wallet.balance_sol < amount_sol {
                return Ok(false);
            }

            // Check daily limits
            let daily_volumes = self.daily_volumes.read().await;
            let current_volume = daily_volumes.get(wallet_name).unwrap_or(&0.0);
            
            if current_volume + amount_sol > wallet.config.risk_management.daily_limit {
                return Ok(false);
            }

            // Check transaction limits
            if amount_sol > wallet.config.risk_management.max_transaction_amount {
                return Ok(false);
            }

            Ok(true)
        } else {
            Err(PlatformError::WalletManagement("Wallet not found".to_string()).into())
        }
    }

    /// Sign a transaction with specified wallet
    pub async fn sign_transaction(
        &self,
        wallet_name: &str,
        mut transaction: Transaction,
        amount_sol: f64,
        description: String,
    ) -> Result<Transaction> {
        // Validate wallet availability
        if !self.is_wallet_available(wallet_name, amount_sol).await? {
            return Err(PlatformError::WalletManagement(
                "Wallet not available for transaction".to_string()
            ).into());
        }

        let wallets = self.wallets.read().await;
        if let Some(wallet) = wallets.get(wallet_name) {
            // Check risk management
            if wallet.config.risk_management.require_confirmation {
                info!("🔐 Transaction requires confirmation: {} SOL - {}", amount_sol, description);
                // In a real implementation, this would wait for user confirmation
            }            // Sign the transaction
            let keypair = wallet.keypair.as_ref();
            transaction.try_sign(&[keypair], transaction.message.recent_blockhash)?;

            // Update daily volume
            {
                let mut daily_volumes = self.daily_volumes.write().await;
                let current_volume = daily_volumes.get(wallet_name).cloned().unwrap_or(0.0);
                daily_volumes.insert(wallet_name.to_string(), current_volume + amount_sol);
            }

            info!("✅ Transaction signed with wallet: {} ({} SOL)", wallet_name, amount_sol);
            Ok(transaction)
        } else {
            Err(PlatformError::WalletManagement("Wallet not found".to_string()).into())
        }
    }

    /// Lock a wallet (prevent transactions)
    pub async fn lock_wallet(&self, wallet_name: &str, reason: String) -> Result<()> {
        let mut wallets = self.wallets.write().await;
        if let Some(wallet) = wallets.get_mut(wallet_name) {
            wallet.is_locked = true;
            wallet.lock_reason = Some(reason.clone());
            warn!("🔒 Wallet locked: {} - {}", wallet_name, reason);
            Ok(())
        } else {
            Err(PlatformError::WalletManagement("Wallet not found".to_string()).into())
        }
    }

    /// Unlock a wallet
    pub async fn unlock_wallet(&self, wallet_name: &str) -> Result<()> {
        let mut wallets = self.wallets.write().await;
        if let Some(wallet) = wallets.get_mut(wallet_name) {
            wallet.is_locked = false;
            wallet.lock_reason = None;
            info!("🔓 Wallet unlocked: {}", wallet_name);
            Ok(())
        } else {
            Err(PlatformError::WalletManagement("Wallet not found".to_string()).into())
        }
    }

    /// Trigger emergency stop (lock all wallets)
    pub async fn emergency_stop(&self, reason: String) -> Result<()> {
        error!("🚨 EMERGENCY STOP TRIGGERED: {}", reason);
        
        *self.emergency_stop.write().await = true;
        
        let mut wallets = self.wallets.write().await;
        for (name, wallet) in wallets.iter_mut() {
            wallet.is_locked = true;
            wallet.lock_reason = Some(format!("Emergency stop: {}", reason));
            error!("🔒 Emergency lock applied to wallet: {}", name);
        }
        
        Ok(())
    }

    /// Get wallet information
    pub async fn get_wallet_info(&self, wallet_name: &str) -> Option<WalletInfo> {
        let wallets = self.wallets.read().await;
        let daily_volumes = self.daily_volumes.read().await;
        
        if let Some(wallet) = wallets.get(wallet_name) {
            Some(WalletInfo {
                name: wallet.config.name.clone(),
                wallet_type: wallet.config.wallet_type.clone(),
                pubkey: wallet.pubkey,
                balance_sol: wallet.balance_sol,
                daily_volume: daily_volumes.get(wallet_name).unwrap_or(&0.0).clone(),
                is_locked: wallet.is_locked,
                lock_reason: wallet.lock_reason.clone(),
                last_balance_check: wallet.last_balance_check,
            })
        } else {
            None
        }
    }

    /// List all managed wallets
    pub async fn list_wallets(&self) -> Vec<WalletInfo> {
        let wallets = self.wallets.read().await;
        let daily_volumes = self.daily_volumes.read().await;
        
        wallets.values()
            .map(|wallet| WalletInfo {
                name: wallet.config.name.clone(),
                wallet_type: wallet.config.wallet_type.clone(),
                pubkey: wallet.pubkey,
                balance_sol: wallet.balance_sol,
                daily_volume: daily_volumes.get(&wallet.config.name).unwrap_or(&0.0).clone(),
                is_locked: wallet.is_locked,
                lock_reason: wallet.lock_reason.clone(),
                last_balance_check: wallet.last_balance_check,
            })
            .collect()
    }

    /// Health check
    pub async fn health_check(&self) -> Result<HealthStatus> {
        let wallets = self.wallets.read().await;
        let emergency_stop = *self.emergency_stop.read().await;
          if emergency_stop {
            return Ok(HealthStatus {
                is_healthy: false,
                component: "wallet_manager".to_string(),
                message: Some("Emergency stop activated".to_string()),
                checked_at: chrono::Utc::now(),
                metrics: HashMap::new(),
            });
        }

        let locked_wallets: Vec<_> = wallets.values()
            .filter(|w| w.is_locked)
            .map(|w| w.config.name.clone())
            .collect();        if locked_wallets.len() == wallets.len() && !wallets.is_empty() {
            Ok(HealthStatus {
                is_healthy: false,
                component: "wallet_manager".to_string(),
                message: Some("All wallets are locked".to_string()),
                checked_at: chrono::Utc::now(),
                metrics: HashMap::new(),
            })
        } else if !locked_wallets.is_empty() {
            Ok(HealthStatus {
                is_healthy: true,
                component: "wallet_manager".to_string(),
                message: Some(format!("Some wallets locked: {:?}", locked_wallets)),
                checked_at: chrono::Utc::now(),
                metrics: HashMap::new(),
            })
        } else {
            Ok(HealthStatus {
                is_healthy: true,
                component: "wallet_manager".to_string(),
                message: None,
                checked_at: chrono::Utc::now(),
                metrics: HashMap::new(),
            })
        }
    }

    /// Load keypair from configuration
    async fn load_keypair(&self, config: &WalletConfig) -> Result<Keypair> {
        if let Some(keypair_data) = &config.keypair_data {
            // Load from base58 string
            let bytes = bs58::decode(keypair_data)
                .into_vec()
                .map_err(|e| PlatformError::WalletManagement(format!("Invalid keypair data: {}", e)))?;
            
            Keypair::from_bytes(&bytes)
                .map_err(|e| PlatformError::WalletManagement(format!("Failed to create keypair: {}", e)).into())
        } else if let Some(keypair_path) = &config.keypair_path {
            // Load from file
            let keypair_bytes = std::fs::read(keypair_path)
                .map_err(|e| PlatformError::WalletManagement(format!("Failed to read keypair file: {}", e)))?;
            
            Keypair::from_bytes(&keypair_bytes)
                .map_err(|e| PlatformError::WalletManagement(format!("Failed to create keypair: {}", e)).into())
        } else {
            // Generate new keypair
            warn!("No keypair specified for wallet {}, generating new one", config.name);
            Ok(Keypair::new())
        }
    }

    /// Load wallets from configuration
    async fn load_configured_wallets(&mut self) -> Result<()> {
        // This would load wallet configurations from the config file
        // For now, we'll create a default trading wallet
        
        let default_config = WalletConfig {
            name: "main-trading".to_string(),
            wallet_type: WalletType::Trading,
            keypair_path: None,
            keypair_data: None,
            max_sol_balance: 100.0,
            min_sol_balance: 1.0,
            risk_management: RiskManagement {
                max_transaction_amount: 10.0,
                daily_limit: 50.0,
                require_confirmation: false,
                emergency_stop_threshold: 0.1,
            },
        };

        self.add_wallet(default_config).await?;
        Ok(())
    }    /// Start balance monitoring task
    async fn start_balance_monitoring(&self) {
        let _wallets = self.wallets.clone();
        
        tokio::spawn(async move {
            let mut interval = tokio::time::interval(tokio::time::Duration::from_secs(30));
            
            loop {
                interval.tick().await;
                
                // In a real implementation, this would check balances via RPC
                debug!("Balance monitoring tick (placeholder)");
            }
        });
    }    /// Start daily volume reset task
    async fn start_daily_reset_task(&self) {
        let daily_volumes = self.daily_volumes.clone();
        
        tokio::spawn(async move {
            let mut interval = tokio::time::interval(tokio::time::Duration::from_secs(24 * 60 * 60)); // 24 hours
            
            loop {
                interval.tick().await;
                
                let mut volumes = daily_volumes.write().await;                volumes.clear();
                info!("🔄 Daily volume limits reset");
            }
        });
    }
}

/// Wallet information for external use
#[derive(Debug, Clone, Serialize)]
pub struct WalletInfo {
    pub name: String,
    pub wallet_type: WalletType,
    pub pubkey: Pubkey,
    pub balance_sol: f64,
    pub daily_volume: f64,
    pub is_locked: bool,
    pub lock_reason: Option<String>,
    pub last_balance_check: chrono::DateTime<chrono::Utc>,
}
