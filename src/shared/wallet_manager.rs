use anyhow::Result;
use solana_client::rpc_client::RpcClient;
use solana_sdk::{
    pubkey::Pubkey,
    signature::Signature,
    signer::{keypair::Keypair, Signer},
    transaction::Transaction,
};
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;
use tracing::{debug, error, info, warn};
// use serde::{Serialize, Deserialize}; // Temporarily commented for compilation

use crate::config::{Config, WalletEnvironmentConfig};
use crate::types::{HealthStatus, PlatformError};

/// Wallet configuration for different purposes
#[derive(Debug, Clone)]
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
#[derive(Debug, Clone)]
pub enum WalletType {
    Trading,   // Main trading wallet
    Fee,       // For transaction fees
    Emergency, // Emergency backup wallet
    Testing,   // For testing purposes
}

/// Risk management settings per wallet
#[derive(Debug, Clone)]
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

        info!(
            "✅ Wallet Manager initialized with {} wallets",
            manager.wallets.read().await.len()
        );

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
                "Wallet not available for transaction".to_string(),
            )
            .into());
        }

        let wallets = self.wallets.read().await;
        if let Some(wallet) = wallets.get(wallet_name) {
            // Check risk management
            if wallet.config.risk_management.require_confirmation {
                info!(
                    "🔐 Transaction requires confirmation: {} SOL - {}",
                    amount_sol, description
                );
                // In a real implementation, this would wait for user confirmation
            } // Sign the transaction
            let keypair = wallet.keypair.as_ref();
            transaction.try_sign(&[keypair], transaction.message.recent_blockhash)?;

            // Update daily volume
            {
                let mut daily_volumes = self.daily_volumes.write().await;
                let current_volume = daily_volumes.get(wallet_name).cloned().unwrap_or(0.0);
                daily_volumes.insert(wallet_name.to_string(), current_volume + amount_sol);
            }

            info!(
                "✅ Transaction signed with wallet: {} ({} SOL)",
                wallet_name, amount_sol
            );
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

        wallets.get(wallet_name).map(|wallet| WalletInfo {
            name: wallet.config.name.clone(),
            wallet_type: wallet.config.wallet_type.clone(),
            pubkey: wallet.pubkey,
            balance_sol: wallet.balance_sol,
            daily_volume: *daily_volumes.get(wallet_name).unwrap_or(&0.0),
            is_locked: wallet.is_locked,
            lock_reason: wallet.lock_reason.clone(),
            last_balance_check: wallet.last_balance_check,
        })
    }

    /// List all managed wallets
    pub async fn list_wallets(&self) -> Vec<WalletInfo> {
        let wallets = self.wallets.read().await;
        let daily_volumes = self.daily_volumes.read().await;

        wallets
            .values()
            .map(|wallet| WalletInfo {
                name: wallet.config.name.clone(),
                wallet_type: wallet.config.wallet_type.clone(),
                pubkey: wallet.pubkey,
                balance_sol: wallet.balance_sol,
                daily_volume: *daily_volumes.get(&wallet.config.name).unwrap_or(&0.0),
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

        let locked_wallets: Vec<_> = wallets
            .values()
            .filter(|w| w.is_locked)
            .map(|w| w.config.name.clone())
            .collect();
        if locked_wallets.len() == wallets.len() && !wallets.is_empty() {
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
            let bytes = bs58::decode(keypair_data).into_vec().map_err(|e| {
                PlatformError::WalletManagement(format!("Invalid keypair data: {}", e))
            })?;

            Keypair::from_bytes(&bytes).map_err(|e| {
                PlatformError::WalletManagement(format!("Failed to create keypair: {}", e)).into()
            })
        } else if let Some(keypair_path) = &config.keypair_path {
            // Load from file
            let keypair_bytes = std::fs::read(keypair_path).map_err(|e| {
                PlatformError::WalletManagement(format!("Failed to read keypair file: {}", e))
            })?;

            Keypair::from_bytes(&keypair_bytes).map_err(|e| {
                PlatformError::WalletManagement(format!("Failed to create keypair: {}", e)).into()
            })
        } else {
            // Generate new keypair
            warn!(
                "No keypair specified for wallet {}, generating new one",
                config.name
            );
            Ok(Keypair::new())
        }
    }
    /// Load wallets from configuration - Sprint 1.5 implementation
    async fn load_configured_wallets(&mut self) -> Result<()> {
        info!("🔐 Loading wallet configurations for Sprint 1.5");

        // Check if auto-generation is enabled
        if let Some(wallet_config) = &self.config.wallets {
            if wallet_config.auto_generate {
                info!("🎯 Auto-generating wallets for hybrid trading mode");

                // DevNet wallet (real keypair)
                if wallet_config.devnet.enabled {
                    let devnet_wallet = self.create_devnet_wallet(&wallet_config.devnet).await?;
                    self.add_wallet(devnet_wallet).await?;
                    info!("✅ DevNet wallet created for real trading");
                    // Auto-airdrop if enabled
                    if wallet_config.auto_airdrop_devnet {
                        match self
                            .airdrop_devnet_sol(
                                "devnet-trading",
                                wallet_config.devnet_airdrop_amount,
                            )
                            .await
                        {
                            Ok(_) => {
                                info!("✅ DevNet airdrop successful");
                            }
                            Err(e) => {
                                warn!("⚠️  DevNet airdrop failed (rate limit or network): {}", e);
                                warn!("💡 This is normal when testing frequently. Wallet will still work with existing balance.");
                            }
                        }
                    }
                }

                // MainNet wallet (real trading only)
                if wallet_config.mainnet.enabled {
                    // For real MainNet trading, require explicit keypair configuration
                    warn!("⚠️ MainNet real trading requires explicit wallet configuration");
                    warn!("⚠️ Add your MainNet keypair to config to enable real trading");
                }
            }
        } else {
            // Fallback to default wallet for backward compatibility
            warn!("⚠️ No wallet config found, creating default wallet");
            let default_config = WalletConfig {
                name: "default-trading".to_string(),
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
        }

        Ok(())
    }

    /// Create DevNet wallet with real keypair
    async fn create_devnet_wallet(
        &self,
        env_config: &WalletEnvironmentConfig,
    ) -> Result<WalletConfig> {
        info!("🧪 Creating DevNet wallet with real keypair");

        // Generate new keypair for devnet
        let keypair = Keypair::new();
        let keypair_bytes = keypair.to_bytes();
        let keypair_base58 = bs58::encode(keypair_bytes).into_string();

        info!("🔑 Generated DevNet keypair: {}", keypair.pubkey());

        Ok(WalletConfig {
            name: "devnet-trading".to_string(),
            wallet_type: WalletType::Trading,
            keypair_path: None,
            keypair_data: Some(keypair_base58),
            max_sol_balance: env_config.initial_balance_sol.unwrap_or(10.0),
            min_sol_balance: 0.1,
            risk_management: RiskManagement {
                max_transaction_amount: env_config.max_trade_amount_sol,
                daily_limit: env_config.max_trade_amount_sol * 10.0,
                require_confirmation: false,
                emergency_stop_threshold: 0.05,
            },
        })
    }

    /// Create MainNet wallet with real keypair (real trading)
    async fn create_mainnet_wallet(
        &self,
        _env_config: &WalletEnvironmentConfig,
    ) -> Result<WalletConfig> {
        error!("� MainNet real trading not yet implemented - requires security review");
        return Err(anyhow::anyhow!("MainNet real trading disabled for safety"));
    }

    /// Airdrop SOL on devnet for testing
    async fn airdrop_devnet_sol(&self, wallet_name: &str, amount: f64) -> Result<()> {
        info!(
            "💧 Requesting {} SOL airdrop for wallet: {}",
            amount, wallet_name
        );

        // Get the wallet
        let wallets = self.wallets.read().await;
        if let Some(wallet) = wallets.get(wallet_name) {
            if let Some(keypair_data) = &wallet.config.keypair_data {
                // Decode keypair
                let keypair_bytes = bs58::decode(keypair_data)
                    .into_vec()
                    .map_err(|e| anyhow::anyhow!("Failed to decode keypair: {}", e))?;
                let keypair = Keypair::from_bytes(&keypair_bytes)?;
                info!("💸 Requesting airdrop for pubkey: {}", keypair.pubkey());

                // Real airdrop request to devnet
                let rpc_client = solana_client::rpc_client::RpcClient::new(
                    "https://api.devnet.solana.com".to_string(),
                );
                let lamports = (amount * 1_000_000_000.0) as u64; // Convert SOL to lamports

                match rpc_client.request_airdrop(&keypair.pubkey(), lamports) {
                    Ok(signature) => {
                        info!("✅ Airdrop request successful! Signature: {}", signature);
                        info!(
                            "🎯 Requested {} SOL ({} lamports) for wallet {}",
                            amount, lamports, wallet_name
                        );

                        // Wait a moment for confirmation
                        tokio::time::sleep(tokio::time::Duration::from_secs(2)).await;

                        // Check balance
                        match rpc_client.get_balance(&keypair.pubkey()) {
                            Ok(balance) => {
                                let balance_sol = balance as f64 / 1_000_000_000.0;
                                info!(
                                    "💰 Current wallet balance: {} SOL ({} lamports)",
                                    balance_sol, balance
                                );
                            }
                            Err(e) => warn!("⚠️ Could not check balance: {}", e),
                        }
                    }
                    Err(e) => {
                        error!("❌ Airdrop request failed: {}", e);
                        warn!("💡 Note: DevNet faucet might be rate-limited or temporarily unavailable");
                        return Err(anyhow::anyhow!("Airdrop failed: {}", e));
                    }
                }
                // rpc_client.confirm_transaction(&signature)?;

                return Ok(());
            }
        }

        Err(anyhow::anyhow!(
            "Wallet not found or no keypair: {}",
            wallet_name
        ))
    }
    /// Start balance monitoring task
    async fn start_balance_monitoring(&self) {
        let _wallets = self.wallets.clone();

        tokio::spawn(async move {
            let mut interval = tokio::time::interval(tokio::time::Duration::from_secs(30));

            loop {
                interval.tick().await;

                // In a real implementation, this would check balances via RPC
                debug!("Balance monitoring tick");
            }
        });
    }
    /// Start daily volume reset task
    async fn start_daily_reset_task(&self) {
        let daily_volumes = self.daily_volumes.clone();

        tokio::spawn(async move {
            let mut interval =
                tokio::time::interval(tokio::time::Duration::from_secs(24 * 60 * 60)); // 24 hours

            loop {
                interval.tick().await;

                let mut volumes = daily_volumes.write().await;
                volumes.clear();
                info!("🔄 Daily volume limits reset");
            }
        });
    }

    /// Get wallet address as string (for Jupiter integration)
    pub async fn get_wallet_address(&self, wallet_name: &str) -> Result<String> {
        let wallets = self.wallets.read().await;
        if let Some(wallet) = wallets.get(wallet_name) {
            Ok(wallet.pubkey.to_string())
        } else {
            Err(
                PlatformError::WalletManagement(format!("Wallet '{}' not found", wallet_name))
                    .into(),
            )
        }
    }

    /// Get wallet keypair for signing (internal use)
    pub async fn get_wallet_keypair(&self, wallet_name: &str) -> Result<Arc<Keypair>> {
        let wallets = self.wallets.read().await;
        if let Some(wallet) = wallets.get(wallet_name) {
            Ok(wallet.keypair.clone())
        } else {
            Err(
                PlatformError::WalletManagement(format!("Wallet '{}' not found", wallet_name))
                    .into(),
            )
        }
    }

    /// Get main trading wallet address
    pub async fn get_main_wallet_address(&self) -> Result<String> {
        // Try to get trading wallet first
        if let Ok(address) = self.get_wallet_address("trading").await {
            return Ok(address);
        }

        // If no trading wallet, try to get the first available wallet
        let wallets = self.wallets.read().await;
        if let Some((_, wallet)) = wallets.iter().next() {
            return Ok(wallet.pubkey.to_string());
        }

        Err(PlatformError::WalletManagement("No wallets available".to_string()).into())
    }
}

/// Wallet information for external use
#[derive(Debug, Clone)]
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
