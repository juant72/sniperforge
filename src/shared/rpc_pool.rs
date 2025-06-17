use anyhow::Result;
use solana_client::rpc_client::RpcClient;
use solana_sdk::commitment_config::CommitmentConfig;
use std::sync::Arc;
use std::time::Duration;
use tokio::sync::{RwLock, Semaphore};
use tracing::{info, warn, error};

use crate::config::Config;
use crate::types::{HealthStatus, Priority};

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

#[derive(Debug, Default)]
struct RpcStats {
    total_requests: u64,
    successful_requests: u64,
    failed_requests: u64,
    avg_response_time_ms: f64,
    active_connections: usize,
}

impl RpcConnectionPool {
    pub async fn new(config: &Config) -> Result<Self> {
        info!("🌐 Initializing RPC connection pool");
        
        let pool_config = RpcPoolConfig {
            pool_size: config.shared_services.rpc_pool_size,
            connection_timeout: Duration::from_millis(config.network.connection_timeout_ms),
            request_timeout: Duration::from_millis(config.network.request_timeout_ms),
        };
        
        // Create primary RPC client
        let primary_client = Arc::new(RpcClient::new_with_commitment(
            config.network.primary_rpc.clone(),
            CommitmentConfig::confirmed(),
        ));
        
        // Create backup RPC clients
        let mut backup_clients = Vec::new();
        for backup_url in &config.network.backup_rpc {
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
        if let Err(e) = self.test_connection(&self.primary_client).await {
            warn!("⚠️ Primary RPC connection test failed: {}", e);
            
            // Test backup connections
            let mut any_working = false;
            for (i, client) in self.backup_clients.iter().enumerate() {
                if self.test_connection(client).await.is_ok() {
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
        if self.test_connection(&self.primary_client).await.is_ok() {
            return Ok(RpcClientHandle {
                client: self.primary_client.clone(),
                _permit,
                stats: self.stats.clone(),
            });
        }
        
        // Fallback to backup clients
        for client in &self.backup_clients {
            if self.test_connection(client).await.is_ok() {
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
        let primary_healthy = self.test_connection(&self.primary_client).await.is_ok();
        
        // Test at least one backup
        let backup_healthy = if !primary_healthy {
            self.backup_clients.iter()
                .any(|client| futures::executor::block_on(self.test_connection(client)).is_ok())
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
    
    pub async fn get_stats(&self) -> std::collections::HashMap<String, String> {
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
    }
    
    async fn test_connection(&self, client: &RpcClient) -> Result<()> {
        // Simple health check - get latest blockhash
        let start_time = std::time::Instant::now();
        
        match client.get_latest_blockhash() {
            Ok(_) => {
                let response_time = start_time.elapsed().as_millis() as f64;
                
                // Update stats
                {
                    let mut stats = self.stats.write().await;
                    stats.successful_requests += 1;
                    stats.total_requests += 1;
                    
                    // Update rolling average
                    let total_responses = stats.successful_requests as f64;
                    stats.avg_response_time_ms = 
                        (stats.avg_response_time_ms * (total_responses - 1.0) + response_time) / total_responses;
                }
                
                Ok(())
            }
            Err(e) => {
                // Update stats
                {
                    let mut stats = self.stats.write().await;
                    stats.failed_requests += 1;
                    stats.total_requests += 1;
                }
                
                Err(anyhow::anyhow!("RPC connection test failed: {}", e))
            }
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
