use anyhow::Result;
use solana_client::rpc_client::RpcClient;
use solana_sdk::{
    commitment_config::CommitmentConfig, 
    pubkey::Pubkey,
    signature::Signature,
    transaction::Transaction,
    account::Account,
    slot_history::Slot,
};
use std::sync::Arc;
use std::time::{Duration, Instant};
use tokio::sync::{RwLock, Semaphore};
use tracing::{info, warn, error, debug};
use serde_json::Value;

use crate::config::Config;
use crate::types::{HealthStatus, Priority};

// Raydium Program IDs
pub const RAYDIUM_AMM_PROGRAM_ID: &str = "675kPX9MHTjS2zt1qfr1NYHuzeLXfQM9H24wFSUt1Mp8";
pub const RAYDIUM_LIQUIDITY_POOL_V4: &str = "5Q544fKrFoe6tsEbD7S8EmxGTJYAKtTVhAW5Q5pge4j1";

// Pool detection and monitoring
#[derive(Debug, Clone)]
pub struct PoolDetectionConfig {
    pub min_liquidity_usd: f64,
    pub max_pool_age_minutes: u64,
    pub check_interval_seconds: u64,
}

impl Default for PoolDetectionConfig {
    fn default() -> Self {
        Self {
            min_liquidity_usd: 5000.0,
            max_pool_age_minutes: 60,
            check_interval_seconds: 10,
        }
    }
}

pub struct RpcConnectionPool {
    primary_client: Arc<RpcClient>,
    backup_clients: Vec<Arc<RpcClient>>,
    connection_semaphore: Arc<Semaphore>,
    config: RpcPoolConfig,
    is_running: Arc<RwLock<bool>>,
    stats: Arc<RwLock<RpcStats>>,
}

#[derive(Debug, Clone)]
struct RpcPoolConfig {
    pool_size: usize,
    connection_timeout: Duration,
    request_timeout: Duration,
}

#[derive(Debug, Default, Clone)]
pub struct RpcStats {
    pub total_requests: u64,
    pub successful_requests: u64,
    pub failed_requests: u64,
    pub avg_response_time_ms: f64,
    pub active_connections: usize,
}

impl RpcConnectionPool {
    pub async fn new(config: &Config) -> Result<Self> {
        info!("🌐 Initializing RPC connection pool");
        
        // Initialize crypto provider for rustls to fix "no process-level CryptoProvider available"
        Self::init_crypto_provider();
        
        let pool_config = RpcPoolConfig {
            pool_size: config.shared_services.rpc_pool_size,
            connection_timeout: Duration::from_millis(config.network.connection_timeout_ms),
            request_timeout: Duration::from_millis(config.network.request_timeout_ms),
        };
          // Create primary RPC client
        let primary_client = Arc::new(RpcClient::new_with_commitment(
            config.network.primary_rpc().to_string(),
            CommitmentConfig::confirmed(),
        ));
        
        // Create backup RPC clients
        let mut backup_clients = Vec::new();
        for backup_url in config.network.backup_rpc() {
            let client = Arc::new(RpcClient::new_with_commitment(
                backup_url.clone(),
                CommitmentConfig::confirmed(),
            ));
            backup_clients.push(client);
        }
        
        // Create connection semaphore
        let connection_semaphore = Arc::new(Semaphore::new(pool_config.pool_size));
        
        Ok(Self {
            primary_client,
            backup_clients,
            connection_semaphore,
            config: pool_config,
            is_running: Arc::new(RwLock::new(false)),
            stats: Arc::new(RwLock::new(RpcStats::default())),
        })
    }
    
    pub async fn start(&self) -> Result<()> {
        info!("🚀 Starting RPC connection pool");
        
        *self.is_running.write().await = true;
          // Test primary connection
        if let Err(e) = self.test_connection(self.primary_client.clone()).await {
            warn!("⚠️ Primary RPC connection test failed: {}", e);
            
            // Test backup connections
            let mut any_working = false;
            for (i, client) in self.backup_clients.iter().enumerate() {
                if self.test_connection(client.clone()).await.is_ok() {
                    info!("✅ Backup RPC {} is working", i);
                    any_working = true;
                    break;
                }
            }
            
            if !any_working {
                return Err(anyhow::anyhow!("No working RPC connections available"));
            }
        } else {
            info!("✅ Primary RPC connection established");
        }
        
        info!("✅ RPC connection pool started");
        Ok(())
    }
    
    pub async fn stop(&self) -> Result<()> {
        info!("🛑 Stopping RPC connection pool");
        
        *self.is_running.write().await = false;
        
        // Wait for all active connections to complete
        let _permits = self.connection_semaphore.acquire_many(self.config.pool_size as u32).await?;
        
        info!("✅ RPC connection pool stopped");
        Ok(())
    }
    
    pub async fn get_client(&self, _priority: Priority) -> Result<RpcClientHandle> {
        let _permit = self.connection_semaphore.acquire().await?;
        
        // Update stats
        {
            let mut stats = self.stats.write().await;
            stats.active_connections += 1;
        }
          // Try primary client first
        if self.test_connection(self.primary_client.clone()).await.is_ok() {
            return Ok(RpcClientHandle {
                client: self.primary_client.clone(),
                _permit,
                stats: self.stats.clone(),
            });
        }
        
        // Fallback to backup clients
        for client in &self.backup_clients {
            if self.test_connection(client.clone()).await.is_ok() {
                return Ok(RpcClientHandle {
                    client: client.clone(),
                    _permit,
                    stats: self.stats.clone(),
                });
            }
        }
        
        Err(anyhow::anyhow!("No working RPC clients available"))
    }
    
    pub async fn health_check(&self) -> Result<HealthStatus> {
        let is_running = *self.is_running.read().await;
        
        if !is_running {
            return Ok(HealthStatus {
                is_healthy: false,
                component: "RpcConnectionPool".to_string(),
                message: Some("Pool not running".to_string()),
                checked_at: chrono::Utc::now(),
                metrics: std::collections::HashMap::new(),
            });
        }
          // Test primary connection
        let primary_healthy = self.test_connection(self.primary_client.clone()).await.is_ok();
        
        // Test at least one backup
        let backup_healthy = if !primary_healthy {
            self.backup_clients.iter()
                .any(|client| futures::executor::block_on(self.test_connection(client.clone())).is_ok())
        } else {
            true
        };
        
        let is_healthy = primary_healthy || backup_healthy;
        
        Ok(HealthStatus {
            is_healthy,
            component: "RpcConnectionPool".to_string(),
            message: if is_healthy {
                None
            } else {
                Some("No working RPC connections".to_string())
            },
            checked_at: chrono::Utc::now(),
            metrics: std::collections::HashMap::new(),
        })
    }
      pub async fn get_connection_count(&self) -> usize {
        self.stats.read().await.active_connections
    }
      /// Get RPC pool statistics
    pub async fn get_stats(&self) -> RpcStats {
        self.stats.read().await.clone()
    }

    /// Get RPC pool statistics as HashMap for monitoring
    pub async fn get_stats_map(&self) -> std::collections::HashMap<String, String> {
        let stats = self.stats.read().await;
        let mut metrics = std::collections::HashMap::new();
        
        metrics.insert("total_requests".to_string(), stats.total_requests.to_string());
        metrics.insert("successful_requests".to_string(), stats.successful_requests.to_string());
        metrics.insert("failed_requests".to_string(), stats.failed_requests.to_string());
        metrics.insert("success_rate".to_string(), {
            if stats.total_requests > 0 {
                format!("{:.2}%", (stats.successful_requests as f64 / stats.total_requests as f64) * 100.0)
            } else {
                "0.00%".to_string()
            }
        });
        metrics.insert("avg_response_time_ms".to_string(), format!("{:.2}", stats.avg_response_time_ms));
        metrics.insert("active_connections".to_string(), stats.active_connections.to_string());
        
        metrics
    }    /// Test connection to RPC endpoint
    async fn test_connection(&self, client: Arc<RpcClient>) -> Result<()> {
        let start = Instant::now();
        
        // Try to get slot number - simple connectivity test
        match tokio::task::spawn_blocking(move || client.get_slot()).await {
            Ok(Ok(slot)) => {
                let response_time = start.elapsed().as_millis();
                debug!("✅ RPC connection test successful - Current slot: {}, Response time: {}ms", slot, response_time);
                
                // Update stats
                let mut stats = self.stats.write().await;
                stats.successful_requests += 1;
                stats.avg_response_time_ms = (stats.avg_response_time_ms + response_time as f64) / 2.0;
                
                Ok(())
            }
            Ok(Err(e)) => {
                let mut stats = self.stats.write().await;
                stats.failed_requests += 1;
                Err(anyhow::anyhow!("RPC connection test failed: {}", e))
            }
            Err(e) => {
                let mut stats = self.stats.write().await;
                stats.failed_requests += 1;
                Err(anyhow::anyhow!("Task join error: {}", e))
            }
        }
    }/// Get current slot from blockchain
    pub async fn get_current_slot(&self) -> Result<Slot> {
        self.execute_with_failover(|client| async move {
            let client_clone = client.clone();
            match tokio::task::spawn_blocking(move || client_clone.get_slot()).await {
                Ok(Ok(slot)) => Ok(slot),
                Ok(Err(e)) => Err(anyhow::anyhow!("RPC error: {}", e)),
                Err(e) => Err(anyhow::anyhow!("Task join error: {}", e)),
            }
        }).await
    }

    /// Get account info for a given pubkey
    pub async fn get_account_info(&self, pubkey: &Pubkey) -> Result<Option<Account>> {
        let pubkey = *pubkey;
        match self.execute_with_failover(|client| async move {
            let client_clone = client.clone();
            match tokio::task::spawn_blocking(move || client_clone.get_account(&pubkey)).await {
                Ok(Ok(account)) => Ok(account),
                Ok(Err(e)) => Err(anyhow::anyhow!("RPC error: {}", e)),
                Err(e) => Err(anyhow::anyhow!("Task join error: {}", e)),
            }
        }).await {
            Ok(account) => Ok(Some(account)),
            Err(_) => Ok(None),
        }
    }

    /// Get balance of an account
    pub async fn get_balance(&self, pubkey: &Pubkey) -> Result<u64> {
        let pubkey = *pubkey;
        self.execute_with_failover(|client| async move {
            let client_clone = client.clone();
            match tokio::task::spawn_blocking(move || client_clone.get_balance(&pubkey)).await {
                Ok(Ok(balance)) => Ok(balance),
                Ok(Err(e)) => Err(anyhow::anyhow!("RPC error: {}", e)),
                Err(e) => Err(anyhow::anyhow!("Task join error: {}", e)),
            }
        }).await
    }    /// Send and confirm transaction
    pub async fn send_and_confirm_transaction(&self, transaction: &Transaction) -> Result<Signature> {
        let transaction_clone = transaction.clone();
        self.execute_with_failover(move |client| {
            let transaction = transaction_clone.clone();
            async move {
                let client_clone = client.clone();
                match tokio::task::spawn_blocking(move || {
                    client_clone.send_and_confirm_transaction(&transaction)
                }).await {
                    Ok(Ok(signature)) => Ok(signature),
                    Ok(Err(e)) => Err(anyhow::anyhow!("RPC error: {}", e)),
                    Err(e) => Err(anyhow::anyhow!("Task join error: {}", e)),
                }
            }
        }).await
    }

    /// Get recent blockhash
    pub async fn get_latest_blockhash(&self) -> Result<solana_sdk::hash::Hash> {
        self.execute_with_failover(|client| async move {
            let client_clone = client.clone();
            match tokio::task::spawn_blocking(move || client_clone.get_latest_blockhash()).await {
                Ok(Ok(hash)) => Ok(hash),
                Ok(Err(e)) => Err(anyhow::anyhow!("RPC error: {}", e)),
                Err(e) => Err(anyhow::anyhow!("Task join error: {}", e)),
            }
        }).await
    }

    /// Get program accounts (useful for pool detection)
    pub async fn get_program_accounts(&self, program_id: &Pubkey) -> Result<Vec<(Pubkey, Account)>> {
        let program_id = *program_id;
        self.execute_with_failover(|client| async move {
            let client_clone = client.clone();
            match tokio::task::spawn_blocking(move || {
                client_clone.get_program_accounts(&program_id)
            }).await {
                Ok(Ok(accounts)) => Ok(accounts),
                Ok(Err(e)) => Err(anyhow::anyhow!("RPC error: {}", e)),
                Err(e) => Err(anyhow::anyhow!("Task join error: {}", e)),
            }
        }).await
    }

    /// Get Raydium pools - specialized function for pool detection
    pub async fn get_raydium_pools(&self) -> Result<Vec<(Pubkey, Account)>> {
        let raydium_program_id = RAYDIUM_AMM_PROGRAM_ID.parse::<Pubkey>()?;
        self.get_program_accounts(&raydium_program_id).await
    }    /// Monitor for new Raydium pools
    pub async fn monitor_new_raydium_pools(&self, _config: PoolDetectionConfig) -> Result<Vec<Pubkey>> {
        debug!("🔍 Monitoring Raydium for new pools...");
        
        let raydium_pools = self.get_raydium_pools().await?;
        let mut new_pools = Vec::new();
        
        for (pubkey, account) in raydium_pools {
            // Basic validation - in a real implementation, you'd parse the account data
            // to extract pool information like liquidity, token pairs, creation time, etc.
            if account.lamports > 0 && !account.data.is_empty() {
                debug!("📊 Found potential pool: {}", pubkey);
                new_pools.push(pubkey);
                
                // Limit to prevent spam - remove this in production
                if new_pools.len() >= 5 {
                    break;
                }
            }
        }
        
        info!("🎯 Found {} potential Raydium pools", new_pools.len());
        Ok(new_pools)
    }    /// Validate if a pool meets our trading criteria
    pub async fn validate_pool_criteria(&self, pool_pubkey: &Pubkey, _config: &PoolDetectionConfig) -> Result<bool> {        // Get pool account data
        match self.get_account_info(pool_pubkey).await? {
            Some(account) => {
                // In a real implementation, you would:
                // 1. Parse the pool account data to extract liquidity info
                // 2. Check token pair information  
                // 3. Validate liquidity meets minimum requirements
                // 4. Check pool age
                // 5. Verify it's not a scam/rug pool                // For now, basic validation
                let has_adequate_data = account.data.len() > 100; // Minimum data size
                let has_adequate_balance = account.lamports > 1_000_000; // > 0.001 SOL
                
                let is_valid = has_adequate_data && has_adequate_balance;
                
                if is_valid {
                    info!("✅ Pool {} meets criteria", pool_pubkey);
                } else {
                    debug!("❌ Pool {} does not meet criteria", pool_pubkey);
                }
                
                Ok(is_valid)
            }
            None => {
                debug!("❌ Pool {} account not found", pool_pubkey);
                Ok(false)
            }
        }
    }    /// Get real-time pool data for trading decisions
    pub async fn get_pool_market_data(&self, pool_pubkey: &Pubkey) -> Result<PoolMarketData> {
        match self.get_account_info(pool_pubkey).await? {
            Some(_account) => {
                // In a real implementation, parse the account data to extract:
                // - Current token reserves
                // - Price information
                // - Trading volume
                // - Fees collected
                
                // For now, return simulated data
                Ok(PoolMarketData {
                    pool_address: *pool_pubkey,
                    token_a_reserve: 100_000_000, // Simulated
                    token_b_reserve: 50_000_000,  // Simulated  
                    total_liquidity_usd: 10_000.0,
                    price_token_a_in_b: 0.5,
                    volume_24h_usd: 50_000.0,
                    last_updated: chrono::Utc::now(),
                })
            }
            None => Err(anyhow::anyhow!("Pool not found: {}", pool_pubkey))
        }
    }

    /// Execute RPC call with automatic failover to backup clients
    async fn execute_with_failover<F, Fut, T>(&self, operation: F) -> Result<T>
    where
        F: Fn(Arc<RpcClient>) -> Fut + Send + Sync,
        Fut: std::future::Future<Output = Result<T>> + Send,
        T: Send,
    {
        let _permit = self.connection_semaphore.acquire().await?;
        let start = Instant::now();
        
        // Update stats
        {
            let mut stats = self.stats.write().await;
            stats.total_requests += 1;
            stats.active_connections += 1;
        }

        // Try primary client first
        match operation(self.primary_client.clone()).await {
            Ok(result) => {
                self.update_success_stats(start).await;
                return Ok(result);
            }
            Err(e) => {
                warn!("Primary RPC client failed: {}", e);
            }
        }

        // Try backup clients
        for (i, client) in self.backup_clients.iter().enumerate() {
            match operation(client.clone()).await {
                Ok(result) => {
                    info!("✅ Backup RPC client {} succeeded", i);
                    self.update_success_stats(start).await;
                    return Ok(result);
                }
                Err(e) => {
                    warn!("Backup RPC client {} failed: {}", i, e);
                }
            }
        }

        self.update_failure_stats().await;
        Err(anyhow::anyhow!("All RPC clients failed"))
    }

    async fn update_success_stats(&self, start: Instant) {
        let mut stats = self.stats.write().await;
        stats.successful_requests += 1;
        stats.active_connections -= 1;
        let response_time = start.elapsed().as_millis() as f64;
        stats.avg_response_time_ms = if stats.avg_response_time_ms == 0.0 {
            response_time
        } else {
            (stats.avg_response_time_ms + response_time) / 2.0
        };
    }

    async fn update_failure_stats(&self) {
        let mut stats = self.stats.write().await;
        stats.failed_requests += 1;
        stats.active_connections -= 1;
    }    fn init_crypto_provider() {
        use std::sync::Once;
        static INIT: Once = Once::new();
        
        INIT.call_once(|| {
            // Initialize rustls default crypto provider to fix:
            // "no process-level CryptoProvider available"
            
            debug!("🔐 Setting up crypto provider for TLS connections...");
            
            // For rustls 0.23+, we need to explicitly install a crypto provider
            // This MUST be done once at program startup before any TLS operations
            
            // Try to install the ring crypto provider
            let result = rustls::crypto::ring::default_provider().install_default();
            
            match result {
                Ok(()) => {
                    debug!("✅ Ring crypto provider installed successfully");
                }
                Err(_) => {
                    // Provider was already installed, which is fine
                    debug!("ℹ️  Crypto provider was already installed");
                }
            }
            
            debug!("✅ Crypto setup completed");
        });
    }
}
pub struct RpcClientHandle<'a> {
    client: Arc<RpcClient>,
    _permit: tokio::sync::SemaphorePermit<'a>,
    stats: Arc<RwLock<RpcStats>>,
}

impl<'a> RpcClientHandle<'a> {
    pub fn client(&self) -> &RpcClient {
        &self.client
    }
}

impl<'a> Drop for RpcClientHandle<'a> {
    fn drop(&mut self) {
        // Update stats when handle is dropped
        let stats = self.stats.clone();
        tokio::spawn(async move {
            let mut stats = stats.write().await;
            stats.active_connections = stats.active_connections.saturating_sub(1);
        });
    }
}

#[derive(Debug, Clone)]
pub struct PoolMarketData {
    pub pool_address: Pubkey,
    pub token_a_reserve: u64,
    pub token_b_reserve: u64,
    pub total_liquidity_usd: f64,
    pub price_token_a_in_b: f64,
    pub volume_24h_usd: f64,
    pub last_updated: chrono::DateTime<chrono::Utc>,
}
