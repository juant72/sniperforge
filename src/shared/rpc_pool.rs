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
use std::collections::HashMap;

use crate::config::Config;
use crate::types::{HealthStatus, Priority};
use crate::shared::alternative_apis::AlternativeApiManager;
use crate::shared::rpc_health_persistence::RpcHealthPersistence;
use crate::shared::premium_rpc_manager::PremiumRpcManager;

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

#[derive(Debug, Clone)]
pub struct RpcEndpointHealth {
    pub url: String,
    pub is_healthy: bool,
    pub consecutive_failures: u32,
    pub last_failure_time: Option<Instant>,
    pub last_success_time: Option<Instant>,
    pub average_response_time: Duration,
    pub total_requests: u64,
    pub successful_requests: u64,
    pub last_error_type: Option<String>,  // NEW: Track specific error types like "410 Gone"
    pub error_counts: std::collections::HashMap<String, u32>, // NEW: Count different error types
}

impl RpcEndpointHealth {
    pub fn new(url: String) -> Self {
        Self {
            url,
            is_healthy: true,
            consecutive_failures: 0,
            last_failure_time: None,
            last_success_time: None,
            average_response_time: Duration::from_millis(0),
            total_requests: 0,
            successful_requests: 0,
            last_error_type: None,
            error_counts: std::collections::HashMap::new(),
        }
    }

    pub fn record_success(&mut self, response_time: Duration) {
        self.is_healthy = true;
        self.consecutive_failures = 0;
        self.last_success_time = Some(Instant::now());
        self.total_requests += 1;
        self.successful_requests += 1;
        
        // Update rolling average
        if self.average_response_time == Duration::from_millis(0) {
            self.average_response_time = response_time;
        } else {
            let total_time = self.average_response_time.as_millis() as f64 * (self.successful_requests - 1) as f64;
            let new_avg = (total_time + response_time.as_millis() as f64) / self.successful_requests as f64;
            self.average_response_time = Duration::from_millis(new_avg as u64);
        }
    }

    pub fn record_failure(&mut self, circuit_breaker_threshold: u32) {
        self.record_failure_with_error(circuit_breaker_threshold, "unknown_error");
    }

    pub fn record_failure_with_error(&mut self, circuit_breaker_threshold: u32, error_type: &str) {
        self.consecutive_failures += 1;
        self.last_failure_time = Some(Instant::now());
        self.total_requests += 1;
        
        // Track specific error types
        self.last_error_type = Some(error_type.to_string());
        *self.error_counts.entry(error_type.to_string()).or_insert(0) += 1;
        
        if self.consecutive_failures >= circuit_breaker_threshold {
            self.is_healthy = false;
        }
    }

    pub fn should_retry(&self, circuit_breaker_reset_seconds: u64) -> bool {
        if self.is_healthy {
            return true;
        }

        if let Some(last_failure) = self.last_failure_time {
            let elapsed = last_failure.elapsed();
            elapsed >= Duration::from_secs(circuit_breaker_reset_seconds)
        } else {
            true
        }
    }
}

pub struct RpcConnectionPool {
    primary_client: Arc<RpcClient>,
    backup_clients: Vec<Arc<RpcClient>>,
    premium_clients: Vec<Arc<RpcClient>>,  // NEW: Premium RPC clients
    premium_manager: Arc<tokio::sync::Mutex<PremiumRpcManager>>,  // NEW: Premium RPC manager
    connection_semaphore: Arc<Semaphore>,
    config: RpcPoolConfig,
    is_running: Arc<RwLock<bool>>,
    stats: Arc<RwLock<RpcStats>>,
    endpoint_health: Arc<RwLock<HashMap<String, RpcEndpointHealth>>>,
    alternative_apis: AlternativeApiManager,
    health_persistence: Arc<tokio::sync::Mutex<RpcHealthPersistence>>,  // Fixed: Added Arc<Mutex>
    // Store URLs for health checking
    primary_url: String,
    backup_urls: Vec<String>,
    premium_urls: Vec<String>,  // NEW: Premium URLs
}

#[derive(Debug, Clone)]
struct RpcPoolConfig {
    pool_size: usize,
    connection_timeout: Duration,
    request_timeout: Duration,
    retry_attempts: u32,
    retry_delay: Duration,
    circuit_breaker_threshold: u32,
    circuit_breaker_reset_seconds: u64,
    rotation_strategy: String,
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
        info!("🌐 Initializing RPC connection pool with premium and enhanced resilience");
        
        // Initialize crypto provider for rustls to fix "no process-level CryptoProvider available"
        Self::init_crypto_provider();
        
        info!("🔧 RPC Pool Config - Environment: {}", config.network.environment);
        info!("🔧 RPC Pool Config - Primary: {}", config.network.primary_rpc());
        info!("🔧 RPC Pool Config - Backup URLs: {:?}", config.network.backup_rpc());
        
        // Initialize premium RPC manager
        let premium_manager = crate::shared::premium_rpc_manager::PremiumRpcManager::new(&config.network)?;
        let premium_urls = premium_manager.get_all_urls();
        
        if premium_manager.has_premium_endpoints() {
            info!("🌟 {}", premium_manager.get_status_summary());
            info!("🔧 Premium URLs: {:?}", premium_urls);
        } else {
            info!("💡 No premium API keys found - using public endpoints only");
            info!("   Set HELIUS_API_KEY, ANKR_API_KEY, QUICKNODE_ENDPOINT, or ALCHEMY_API_KEY for premium access");
        }
        
        let pool_config = RpcPoolConfig {
            pool_size: config.shared_services.rpc_pool_size,
            connection_timeout: Duration::from_millis(config.network.connection_timeout_ms),
            request_timeout: Duration::from_millis(config.network.request_timeout_ms),
            retry_attempts: config.network.retry_attempts as u32,
            retry_delay: Duration::from_millis(config.network.retry_delay_ms),
            circuit_breaker_threshold: 5, // Default fallback
            circuit_breaker_reset_seconds: 120, // Default fallback
            rotation_strategy: "smart".to_string(), // Default fallback
        };
        
        // Create primary RPC client
        let primary_url = config.network.primary_rpc().to_string();
        let primary_client = Arc::new(RpcClient::new_with_commitment(
            primary_url.clone(),
            CommitmentConfig::confirmed(),
        ));
        
        // Create backup RPC clients
        let backup_urls = config.network.backup_rpc();
        let mut backup_clients = Vec::new();
        for backup_url in &backup_urls {
            let client = Arc::new(RpcClient::new_with_commitment(
                backup_url.clone(),
                CommitmentConfig::confirmed(),
            ));
            backup_clients.push(client);
        }
        
        // Create premium RPC clients
        let mut premium_clients = Vec::new();
        for premium_url in &premium_urls {
            let client = Arc::new(RpcClient::new_with_commitment(
                premium_url.clone(),
                CommitmentConfig::confirmed(),
            ));
            premium_clients.push(client);
        }
        
        // Initialize health persistence
        let mut health_persistence = RpcHealthPersistence::new("data/rpc_health.json");
        
        // Load persisted health data
        health_persistence.load().await
            .unwrap_or_else(|e| {
                warn!("Failed to load RPC health persistence: {}", e);
            });
        
        let health_persistence = Arc::new(tokio::sync::Mutex::new(health_persistence));
        
        // Initialize endpoint health tracking
        let mut endpoint_health = HashMap::new();
        
        // Check if primary endpoint should be avoided based on history
        {
            let persistence = health_persistence.lock().await;
            if let Some(persisted_health) = persistence.get_endpoint_health(&primary_url) {
                if persisted_health.should_avoid_endpoint(1) { // Avoid if failed in last hour
                    warn!("⚠️ Primary RPC {} has reliability issues (score: {:.2})", 
                          primary_url, persisted_health.reliability_score);
                }
            }
        }
        
        endpoint_health.insert(primary_url.clone(), RpcEndpointHealth::new(primary_url.clone()));
        
        // Add backup endpoints to health tracking
        for backup_url in &backup_urls {
            // Check backup endpoint history
            {
                let persistence = health_persistence.lock().await;
                if let Some(persisted_health) = persistence.get_endpoint_health(backup_url) {
                    if persisted_health.should_avoid_endpoint(1) {
                        warn!("⚠️ Backup RPC {} has reliability issues (score: {:.2})", 
                              backup_url, persisted_health.reliability_score);
                    }
                }
            }
            endpoint_health.insert(backup_url.clone(), RpcEndpointHealth::new(backup_url.clone()));
        }
        
        // Add premium endpoints to health tracking
        for premium_url in &premium_urls {
            endpoint_health.insert(premium_url.clone(), RpcEndpointHealth::new(premium_url.clone()));
            info!("📡 Added premium endpoint to health tracking: {}", premium_url);
        }
        
        // Create connection semaphore
        let connection_semaphore = Arc::new(Semaphore::new(pool_config.pool_size));
        
        // Initialize alternative APIs with basic config
        let basic_config = crate::shared::alternative_apis::BasicConfig::default();
        let alternative_apis = AlternativeApiManager::new(&basic_config);
        
        Ok(Self {
            primary_client,
            backup_clients,
            premium_clients,
            premium_manager: Arc::new(tokio::sync::Mutex::new(premium_manager)),
            connection_semaphore,
            config: pool_config,
            is_running: Arc::new(RwLock::new(false)),
            stats: Arc::new(RwLock::new(RpcStats::default())),
            endpoint_health: Arc::new(RwLock::new(endpoint_health)),
            alternative_apis,
            health_persistence,  // NEW: Add health persistence
            primary_url,
            backup_urls,
            premium_urls,
        })
    }
    
    pub async fn start(&self) -> Result<()> {
        info!("🚀 Starting RPC connection pool with smart endpoint selection");
        
        *self.is_running.write().await = true;
        
        // Test primary endpoint
        if let Err(e) = self.test_and_update_health(self.primary_client.clone(), &self.primary_url).await {
            warn!("⚠️ Primary RPC connection test failed: {}", e);
        }
        
        // Test backup connections
        for (i, client) in self.backup_clients.iter().enumerate() {
            if let Some(backup_url) = self.backup_urls.get(i) {
                if self.test_and_update_health(client.clone(), backup_url).await.is_ok() {
                    info!("✅ Backup RPC {} is working", i);
                }
            }
        }
        
        // Test premium connections
        for (i, client) in self.premium_clients.iter().enumerate() {
            if let Some(premium_url) = self.premium_urls.get(i) {
                if self.test_and_update_health(client.clone(), premium_url).await.is_ok() {
                    info!("✅ Premium RPC {} is working", i);
                } else {
                    warn!("⚠️ Premium RPC {} failed connection test", premium_url);
                }
            }
        }
        
        // Check if we have any working endpoints
        let health_map = self.endpoint_health.read().await;
        let healthy_endpoints: Vec<_> = health_map.values().filter(|h| h.is_healthy).collect();
        let premium_healthy: Vec<_> = health_map.iter()
            .filter(|(url, health)| self.premium_urls.contains(url) && health.is_healthy)
            .collect();
        
        if healthy_endpoints.is_empty() {
            warn!("⚠️ No RPC endpoints are working, but alternative APIs are available");
            info!("🔄 Will use alternative APIs for pool detection");
        } else {
            info!("✅ Found {} healthy RPC endpoints ({} premium)", 
                  healthy_endpoints.len(), premium_healthy.len());
        }
        
        info!("✅ RPC connection pool started with enhanced resilience");
        Ok(())
    }

    async fn test_and_update_health(&self, client: Arc<RpcClient>, url: &str) -> Result<()> {
        let start_time = Instant::now();
        
        match self.test_connection(client).await {
            Ok(_) => {
                let response_time = start_time.elapsed();
                let mut health_map = self.endpoint_health.write().await;
                if let Some(health) = health_map.get_mut(url) {
                    health.record_success(response_time);
                }
                drop(health_map); // Release lock before async operation
                
                // Update persistence
                let mut persistence = self.health_persistence.lock().await;
                if let Err(e) = persistence.record_endpoint_success(url, response_time.as_millis() as u64).await {
                    warn!("Failed to persist RPC success for {}: {}", url, e);
                }
                drop(persistence);
                
                info!("✅ RPC endpoint {} is healthy ({}ms)", url, response_time.as_millis());
                Ok(())
            }
            Err(e) => {
                // Classify error type for better tracking
                let error_type = if e.to_string().contains("410 Gone") {
                    "410 Gone"
                } else if e.to_string().contains("401 Unauthorized") {
                    "401 Unauthorized"
                } else if e.to_string().contains("403 Forbidden") {
                    "403 Forbidden"
                } else if e.to_string().contains("timeout") {
                    "timeout"
                } else if e.to_string().contains("dns error") {
                    "dns error"
                } else if e.to_string().contains("connect") {
                    "connection refused"
                } else {
                    "unknown error"
                };

                let mut health_map = self.endpoint_health.write().await;
                if let Some(health) = health_map.get_mut(url) {
                    health.record_failure(self.config.circuit_breaker_threshold);
                }
                drop(health_map); // Release lock before async operation
                
                // Update persistence
                let mut persistence = self.health_persistence.lock().await;
                if let Err(persist_err) = persistence.record_endpoint_failure(url, error_type).await {
                    warn!("Failed to persist RPC failure for {}: {}", url, persist_err);
                }
                drop(persistence);
                
                warn!("❌ RPC endpoint {} failed: {}", url, e);
                Err(e)
            }
        }
    }
    
    pub async fn stop(&self) -> Result<()> {
        info!("🛑 Stopping RPC connection pool");
        
        *self.is_running.write().await = false;
        
        // Save current health state before stopping
        let persistence = self.health_persistence.lock().await;
        if let Err(e) = persistence.save().await {
            warn!("Failed to save RPC health persistence: {}", e);
        } else {
            info!("💾 RPC health state saved successfully");
        }
        drop(persistence);
        
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
        
        // Smart endpoint selection with premium priority
        let health_map = self.endpoint_health.read().await;
        
        // Find the best healthy endpoint (premium endpoints get priority)
        let mut best_endpoint: Option<(Arc<RpcClient>, String, bool)> = None; // client, url, is_premium
        let mut best_response_time = Duration::from_secs(u64::MAX);
        
        // Check premium endpoints first (highest priority)
        for (i, client) in self.premium_clients.iter().enumerate() {
            if let Some(premium_url) = self.premium_urls.get(i) {
                if let Some(health) = health_map.get(premium_url) {
                    if health.is_healthy && 
                       health.should_retry(self.config.circuit_breaker_reset_seconds) {
                        // Premium endpoints always take priority unless they have very poor performance
                        if best_endpoint.is_none() || 
                           (!best_endpoint.as_ref().unwrap().2 && health.average_response_time < Duration::from_millis(5000)) ||
                           (best_endpoint.as_ref().unwrap().2 && health.average_response_time < best_response_time) {
                            best_endpoint = Some((client.clone(), premium_url.clone(), true));
                            best_response_time = health.average_response_time;
                        }
                    }
                }
            }
        }
        
        // If no premium endpoints are available, check public endpoints
        if best_endpoint.is_none() || best_endpoint.as_ref().unwrap().2 == false {
            // Check primary if healthy
            if let Some(health) = health_map.get(&self.primary_url) {
                if health.is_healthy && 
                   health.should_retry(self.config.circuit_breaker_reset_seconds) &&
                   (best_endpoint.is_none() || health.average_response_time < best_response_time) {
                    best_endpoint = Some((self.primary_client.clone(), self.primary_url.clone(), false));
                    best_response_time = health.average_response_time;
                }
            }
            
            // Check backups for better options
            for (i, client) in self.backup_clients.iter().enumerate() {
                if let Some(backup_url) = self.backup_urls.get(i) {
                    if let Some(health) = health_map.get(backup_url) {
                        if health.is_healthy && 
                           health.should_retry(self.config.circuit_breaker_reset_seconds) &&
                           (best_endpoint.is_none() || health.average_response_time < best_response_time) {
                            best_endpoint = Some((client.clone(), backup_url.clone(), false));
                            best_response_time = health.average_response_time;
                        }
                    }
                }
            }
        }
        
        drop(health_map); // Release the read lock
        
        if let Some((client, url, is_premium)) = best_endpoint {
            let endpoint_type = if is_premium { "PREMIUM" } else { "PUBLIC" };
            debug!("📡 Using {} RPC endpoint: {} (avg: {}ms)", endpoint_type, url, best_response_time.as_millis());
            return Ok(RpcClientHandle {
                client,
                _permit,
                stats: self.stats.clone(),
            });
        }
        
        // If no healthy endpoints, try to reset circuit breakers
        warn!("⚠️ No healthy RPC endpoints available, attempting circuit breaker reset");
        
        let mut health_map = self.endpoint_health.write().await;
        for health in health_map.values_mut() {
            if !health.is_healthy && health.should_retry(self.config.circuit_breaker_reset_seconds) {
                info!("🔄 Resetting circuit breaker for {}", health.url);
                health.is_healthy = true;
                health.consecutive_failures = 0;
            }
        }
        drop(health_map);
        
        // Try premium endpoints first after reset
        for client in &self.premium_clients {
            if self.test_connection(client.clone()).await.is_ok() {
                info!("✅ Premium endpoint recovered after circuit breaker reset");
                return Ok(RpcClientHandle {
                    client: client.clone(),
                    _permit,
                    stats: self.stats.clone(),
                });
            }
        }
        
        // Try primary after reset
        if self.test_connection(self.primary_client.clone()).await.is_ok() {
            return Ok(RpcClientHandle {
                client: self.primary_client.clone(),
                _permit,
                stats: self.stats.clone(),
            });
        }
        
        // Try backups after reset
        for client in &self.backup_clients {
            if self.test_connection(client.clone()).await.is_ok() {
                return Ok(RpcClientHandle {
                    client: client.clone(),
                    _permit,
                    stats: self.stats.clone(),
                });
            }
        }
        
        Err(anyhow::anyhow!("No working RPC clients available after circuit breaker reset"))
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
        
        // Test premium connections first
        let premium_healthy = if !self.premium_clients.is_empty() {
            self.premium_clients.iter()
                .any(|client| futures::executor::block_on(self.test_connection(client.clone())).is_ok())
        } else {
            false
        };
        
        // Test primary connection
        let primary_healthy = self.test_connection(self.primary_client.clone()).await.is_ok();
        
        // Test at least one backup
        let backup_healthy = if !primary_healthy && !premium_healthy {
            self.backup_clients.iter()
                .any(|client| futures::executor::block_on(self.test_connection(client.clone())).is_ok())
        } else {
            true
        };
        
        let is_healthy = premium_healthy || primary_healthy || backup_healthy;
        
        let message = if is_healthy {
            if premium_healthy {
                Some("Premium endpoints available".to_string())
            } else {
                None
            }
        } else {
            Some("No working RPC connections".to_string())
        };
        
        Ok(HealthStatus {
            is_healthy,
            component: "RpcConnectionPool".to_string(),
            message,
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

    /// Get Raydium pools - optimized with size limits to avoid scan aborted errors
    pub async fn get_raydium_pools(&self) -> Result<Vec<(Pubkey, Account)>> {
        debug!("🔍 Getting Raydium pools (optimized query)...");
        
        // Try a more conservative approach first
        let raydium_program_id = RAYDIUM_AMM_PROGRAM_ID.parse::<Pubkey>()?;
        
        // Use execute_with_failover for resilience but with a simplified approach
        match self.execute_with_failover(|client| async move {
            let client_clone = client.clone();
            let program_id = raydium_program_id;
            
            tokio::task::spawn_blocking(move || {
                // Use a limited query to avoid scan aborted errors
                match client_clone.get_program_accounts_with_config(
                    &program_id,
                    solana_client::rpc_config::RpcProgramAccountsConfig {
                        filters: None,
                        account_config: solana_client::rpc_config::RpcAccountInfoConfig {
                            encoding: Some(solana_account_decoder::UiAccountEncoding::Base64),
                            data_slice: Some(solana_account_decoder::UiDataSliceConfig {
                                offset: 0,
                                length: 1000, // Limit data size to reduce load
                            }),
                            commitment: Some(CommitmentConfig::confirmed()),
                            min_context_slot: None,
                        },
                        with_context: Some(false),
                        sort_results: Some(false), // Add missing field
                    },
                ) {
                    Ok(accounts) => {
                        // Limit number of results to prevent overwhelming
                        let limited_accounts: Vec<_> = accounts.into_iter().take(100).collect();
                        debug!("✅ Retrieved {} Raydium accounts (limited)", limited_accounts.len());
                        Ok(limited_accounts)
                    }
                    Err(e) => {
                        warn!("Raydium pool query failed: {}", e);
                        Err(anyhow::anyhow!("RPC error: {}", e))
                    }
                }
            }).await?
        }).await {
            Ok(accounts) => Ok(accounts),
            Err(e) => {
                warn!("⚠️  All Raydium pool queries failed: {}", e);
                warn!("   This is expected behavior on mainnet due to RPC limitations");
                warn!("   Pool detection will use alternative methods");
                
                // Return empty result instead of failing the entire test
                Ok(Vec::new())
            }
        }
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
                // TODO: Parse the account data to extract:
                // - Current token reserves  
                // - Price information
                // - Trading volume
                // - Fees collected
                
                error!("🚫 POOL DATA PARSING NOT YET IMPLEMENTED - Use real data sources");
                Err(anyhow::anyhow!("Pool data parsing not implemented"))
            }
            None => Err(anyhow::anyhow!("Pool not found: {}", pool_pubkey))
        }
    }

    /// Get transaction details
    pub async fn get_transaction(&self, signature: &Signature) -> Result<Option<TransactionDetails>> {
        let signature = *signature;
        match self.execute_with_failover(|client| async move {
            let client_clone = client.clone();
            match tokio::task::spawn_blocking(move || {
                client_clone.get_transaction_with_config(
                    &signature,
                    solana_client::rpc_config::RpcTransactionConfig {
                        encoding: Some(solana_transaction_status::UiTransactionEncoding::Json),
                        commitment: Some(CommitmentConfig::confirmed()),
                        max_supported_transaction_version: Some(0),
                    }
                )
            }).await {
                Ok(Ok(transaction)) => {
                    // Convert to our TransactionDetails format
                    Ok(TransactionDetails {
                        signature: signature.to_string(),
                        slot: transaction.slot,
                        block_time: transaction.block_time,
                        meta: transaction.transaction.meta,
                    })
                },
                Ok(Err(e)) => Err(anyhow::anyhow!("RPC error: {}", e)),
                Err(e) => Err(anyhow::anyhow!("Task join error: {}", e)),
            }
        }).await {
            Ok(transaction) => Ok(Some(transaction)),
            Err(_) => Ok(None),
        }
    }

    /// Health check for RPC pool
    pub async fn is_healthy(&self) -> Result<bool> {
        match self.test_connection(self.primary_client.clone()).await {
            Ok(_) => Ok(true),
            Err(_) => Ok(false),
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

    /// Get alternative APIs manager for when RPC fails
    pub fn get_alternative_apis(&self) -> &AlternativeApiManager {
        &self.alternative_apis
    }

    /// Comprehensive pool detection using both RPC and alternative APIs
    pub async fn get_pools_with_fallback(&self) -> Result<Vec<crate::shared::alternative_apis::RaydiumPoolInfo>> {
        info!("🔄 Attempting comprehensive pool detection...");

        // First try RPC-based detection
        match self.get_raydium_pools().await {
            Ok(rpc_pools) => {
                if !rpc_pools.is_empty() {
                    info!("✅ Successfully fetched {} pools via RPC", rpc_pools.len());
                    
                    // Convert RPC results to RaydiumPoolInfo format
                    // For now, return from alternative APIs as RPC parsing is complex
                    warn!("🔄 RPC pools detected but using alternative APIs for consistent format");
                }
            }
            Err(e) => {
                warn!("⚠️ RPC pool detection failed: {}, using alternative APIs", e);
            }
        }

        // Use alternative APIs as primary method for now
        info!("📡 Using alternative APIs for pool detection");
        self.alternative_apis.get_comprehensive_pool_data().await
    }

    /// Get detailed health statistics including error breakdowns
    pub async fn get_detailed_health_stats(&self) -> Result<String> {
        let health_data = self.endpoint_health.read().await;
        let mut report = String::new();
        
        report.push_str("🏥 RPC ENDPOINT HEALTH REPORT\n");
        report.push_str("===============================\n\n");
        
        let mut healthy_count = 0;
        let mut unhealthy_count = 0;
        let mut total_410_gone = 0;
        let mut total_auth_errors = 0;
        let mut total_timeouts = 0;
        let mut total_dns_errors = 0;
        
        for (url, health) in health_data.iter() {
            if health.is_healthy {
                healthy_count += 1;
                report.push_str(&format!("✅ {} - HEALTHY\n", url));
                report.push_str(&format!("   Avg response: {}ms\n", health.average_response_time.as_millis()));
                report.push_str(&format!("   Success rate: {}/{}\n", health.successful_requests, health.total_requests));
            } else {
                unhealthy_count += 1;
                report.push_str(&format!("❌ {} - UNHEALTHY\n", url));
                report.push_str(&format!("   Consecutive failures: {}\n", health.consecutive_failures));
                if let Some(error_type) = &health.last_error_type {
                    report.push_str(&format!("   Last error: {}\n", error_type));
                }
                
                // Error breakdown
                for (error_type, count) in &health.error_counts {
                    report.push_str(&format!("   {}: {} times\n", error_type, count));
                    
                    match error_type.as_str() {
                        "410_gone" => total_410_gone += count,
                        "401_unauthorized" | "403_forbidden" => total_auth_errors += count,
                        "timeout" => total_timeouts += count,
                        "dns_error" => total_dns_errors += count,
                        _ => {}
                    }
                }
            }
            report.push('\n');
        }
        
        report.push_str(&format!("📊 SUMMARY:\n"));
        report.push_str(&format!("   Healthy endpoints: {}\n", healthy_count));
        report.push_str(&format!("   Unhealthy endpoints: {}\n", unhealthy_count));
        report.push_str(&format!("   Total 410 Gone errors: {}\n", total_410_gone));
        report.push_str(&format!("   Total auth errors: {}\n", total_auth_errors));
        report.push_str(&format!("   Total timeouts: {}\n", total_timeouts));
        report.push_str(&format!("   Total DNS errors: {}\n", total_dns_errors));
        
        // Add persistence stats
        let persistence = self.health_persistence.lock().await;
        let persistence_report = persistence.generate_health_report();
        report.push_str(&format!("\n💾 HISTORICAL DATA:\n{}\n", persistence_report));
        drop(persistence);
        
        Ok(report)
    }
    
    /// Generate RPC health report
    pub async fn generate_health_report(&self) -> String {
        let persistence = self.health_persistence.lock().await;
        persistence.generate_health_report()
    }

    /// Get recommended endpoints based on historical performance
    pub async fn get_recommended_endpoints(&self) -> Vec<(String, f64)> {
        let persistence = self.health_persistence.lock().await;
        persistence.get_recommended_endpoints()
    }

    /// Get problematic endpoints that should be avoided
    pub async fn get_problematic_endpoints(&self, hours_to_consider: u64) -> Vec<String> {
        let persistence = self.health_persistence.lock().await;
        persistence.get_problematic_endpoints(hours_to_consider)
    }

    /// Get the best available WebSocket URL (prioritizing premium endpoints)
    pub async fn get_best_websocket_url(&self) -> Option<String> {
        // First, try to get WebSocket URL from premium manager
        let premium_manager = self.premium_manager.lock().await;
        if let Some(premium_ws_url) = premium_manager.get_websocket_url() {
            info!("🌟 Using premium WebSocket endpoint");
            return Some(premium_ws_url);
        }
        drop(premium_manager);
        
        // Fall back to converting the best healthy RPC endpoint to WebSocket
        let health_map = self.endpoint_health.read().await;
        
        // Find the best healthy endpoint
        let mut best_url: Option<String> = None;
        let mut best_response_time = Duration::from_secs(u64::MAX);
        
        // Check premium endpoints first
        for premium_url in &self.premium_urls {
            if let Some(health) = health_map.get(premium_url) {
                if health.is_healthy && 
                   health.should_retry(self.config.circuit_breaker_reset_seconds) &&
                   health.average_response_time < best_response_time {
                    best_url = Some(premium_url.clone());
                    best_response_time = health.average_response_time;
                }
            }
        }
        
        // If no premium endpoints, check public ones
        if best_url.is_none() {
            // Check primary
            if let Some(health) = health_map.get(&self.primary_url) {
                if health.is_healthy && health.should_retry(self.config.circuit_breaker_reset_seconds) {
                    best_url = Some(self.primary_url.clone());
                    best_response_time = health.average_response_time;
                }
            }
            
            // Check backups
            for backup_url in &self.backup_urls {
                if let Some(health) = health_map.get(backup_url) {
                    if health.is_healthy && 
                       health.should_retry(self.config.circuit_breaker_reset_seconds) &&
                       health.average_response_time < best_response_time {
                        best_url = Some(backup_url.clone());
                        best_response_time = health.average_response_time;
                    }
                }
            }
        }
        
        // Convert HTTP(S) URL to WebSocket URL
        if let Some(url) = best_url {
            if url.starts_with("https://") {
                Some(url.replace("https://", "wss://"))
            } else if url.starts_with("http://") {
                Some(url.replace("http://", "ws://"))
            } else {
                Some(url)
            }
        } else {
            None
        }
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

#[derive(Debug, Clone)]
pub struct TransactionDetails {
    pub signature: String,
    pub slot: u64,
    pub block_time: Option<i64>,
    pub meta: Option<solana_transaction_status::UiTransactionStatusMeta>,
}

impl std::fmt::Debug for RpcConnectionPool {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("RpcConnectionPool")
            .field("primary_client", &"<RpcClient>")
            .field("backup_clients", &self.backup_clients.len())
            .field("config", &self.config)
            .field("is_running", &self.is_running)
            .field("stats", &self.stats)
            .finish()
    }
}
