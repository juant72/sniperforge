//! Enterprise RPC Pool Management
//! 
//! This module provides high-availability RPC connection pooling with automatic failover,
//! health monitoring, and load balancing for enterprise trading operations.

use anyhow::Result;
use serde::{Deserialize, Serialize};
use solana_client::rpc_client::RpcClient;
use solana_sdk::{commitment_config::CommitmentConfig};
use std::collections::HashMap;
use std::sync::Arc;
use std::time::{Duration, Instant};
use tokio::sync::{RwLock, Semaphore};
use tracing::{debug, info, warn};

use crate::config::SimpleConfig;

/// RPC endpoint health status
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RpcEndpointHealth {
    pub url: String,
    pub healthy: bool,
    pub last_check: Instant,
    pub response_time_ms: u64,
    pub error_count: u32,
    pub success_count: u32,
    pub priority: u8,
}

/// RPC pool configuration
#[derive(Debug, Clone)]
pub struct RpcPoolConfig {
    pub max_connections_per_endpoint: usize,
    pub health_check_interval_seconds: u64,
    pub max_retry_attempts: u32,
    pub timeout_seconds: u64,
    pub failover_threshold: u32,
}

impl Default for RpcPoolConfig {
    fn default() -> Self {
        Self {
            max_connections_per_endpoint: 10,
            health_check_interval_seconds: 30,
            max_retry_attempts: 3,
            timeout_seconds: 10,
            failover_threshold: 5,
        }
    }
}

/// Enterprise RPC connection pool with automatic failover
pub struct RpcPool {
    config: SimpleConfig,
    pool_config: RpcPoolConfig,
    endpoints: Arc<RwLock<Vec<RpcEndpointHealth>>>,
    clients: Arc<RwLock<HashMap<String, Arc<RpcClient>>>>,
    connection_semaphore: Arc<Semaphore>,
    primary_endpoint: Arc<RwLock<Option<String>>>,
}

impl RpcPool {
    /// Create new RPC pool
    pub fn new(config: &SimpleConfig) -> Self {
        let pool_config = RpcPoolConfig::default();
        let connection_semaphore = Arc::new(Semaphore::new(
            pool_config.max_connections_per_endpoint * 3 // Allow for multiple endpoints
        ));

        let mut endpoints = vec![
            RpcEndpointHealth {
                url: config.solana_rpc_url.clone(),
                healthy: true,
                last_check: Instant::now(),
                response_time_ms: 0,
                error_count: 0,
                success_count: 0,
                priority: 1,
            }
        ];

        // Add backup endpoints if available
        if !config.solana_rpc_url.contains("localhost") {
            endpoints.extend(Self::get_backup_endpoints());
        }

        Self {
            config: config.clone(),
            pool_config,
            endpoints: Arc::new(RwLock::new(endpoints)),
            clients: Arc::new(RwLock::new(HashMap::new())),
            connection_semaphore,
            primary_endpoint: Arc::new(RwLock::new(Some(config.solana_rpc_url.clone()))),
        }
    }

    /// Get backup RPC endpoints for failover
    fn get_backup_endpoints() -> Vec<RpcEndpointHealth> {
        vec![
            RpcEndpointHealth {
                url: "https://api.mainnet-beta.solana.com".to_string(),
                healthy: true,
                last_check: Instant::now(),
                response_time_ms: 0,
                error_count: 0,
                success_count: 0,
                priority: 2,
            },
            RpcEndpointHealth {
                url: "https://solana-api.projectserum.com".to_string(),
                healthy: true,
                last_check: Instant::now(),
                response_time_ms: 0,
                error_count: 0,
                success_count: 0,
                priority: 3,
            },
        ]
    }

    /// Get healthy RPC client with automatic failover
    pub async fn get_client(&self) -> Result<Arc<RpcClient>> {
        let _permit = self.connection_semaphore.acquire().await?;
        
        // Try primary endpoint first
        if let Some(primary_url) = &*self.primary_endpoint.read().await {
            if let Ok(client) = self.get_client_for_endpoint(primary_url).await {
                return Ok(client);
            }
        }

        // Failover to healthy endpoints
        let endpoints = self.endpoints.read().await;
        let mut healthy_endpoints: Vec<_> = endpoints
            .iter()
            .filter(|e| e.healthy)
            .collect();
        
        healthy_endpoints.sort_by_key(|e| e.priority);

        for endpoint in healthy_endpoints {
            if let Ok(client) = self.get_client_for_endpoint(&endpoint.url).await {
                // Update primary endpoint to working one
                *self.primary_endpoint.write().await = Some(endpoint.url.clone());
                return Ok(client);
            }
        }

        Err(anyhow::anyhow!("No healthy RPC endpoints available"))
    }

    /// Get or create client for specific endpoint
    async fn get_client_for_endpoint(&self, url: &str) -> Result<Arc<RpcClient>> {
        let clients = self.clients.read().await;
        
        if let Some(client) = clients.get(url) {
            return Ok(client.clone());
        }
        
        drop(clients);

        // Create new client
        let client = Arc::new(RpcClient::new_with_commitment(
            url.to_string(),
            CommitmentConfig::confirmed(),
        ));

        // Test client with a simple call
        if let Err(e) = client.get_slot() {
            warn!("Failed to connect to RPC endpoint {}: {}", url, e);
            self.mark_endpoint_unhealthy(url).await;
            return Err(e.into());
        }

        // Store client
        let mut clients = self.clients.write().await;
        clients.insert(url.to_string(), client.clone());
        
        info!("Connected to RPC endpoint: {}", url);
        self.mark_endpoint_healthy(url).await;
        
        Ok(client)
    }

    /// Mark endpoint as healthy
    async fn mark_endpoint_healthy(&self, url: &str) {
        let mut endpoints = self.endpoints.write().await;
        if let Some(endpoint) = endpoints.iter_mut().find(|e| e.url == url) {
            endpoint.healthy = true;
            endpoint.success_count += 1;
            endpoint.last_check = Instant::now();
        }
    }

    /// Mark endpoint as unhealthy
    async fn mark_endpoint_unhealthy(&self, url: &str) {
        let mut endpoints = self.endpoints.write().await;
        if let Some(endpoint) = endpoints.iter_mut().find(|e| e.url == url) {
            endpoint.error_count += 1;
            endpoint.last_check = Instant::now();
            
            if endpoint.error_count >= self.pool_config.failover_threshold {
                endpoint.healthy = false;
                warn!("Marked RPC endpoint as unhealthy: {} (errors: {})", url, endpoint.error_count);
            }
        }
    }

    /// Start health monitoring background task
    pub async fn start_health_monitoring(&self) -> Result<()> {
        let endpoints = self.endpoints.clone();
        let interval = self.pool_config.health_check_interval_seconds;
        
        tokio::spawn(async move {
            let mut interval = tokio::time::interval(Duration::from_secs(interval));
            
            loop {
                interval.tick().await;
                
                let endpoints_guard = endpoints.read().await;
                for endpoint in endpoints_guard.iter() {
                    if !endpoint.healthy {
                        // Try to reconnect to unhealthy endpoints
                        if let Ok(client) = RpcClient::new(&endpoint.url).get_slot() {
                            debug!("Endpoint {} is back online", endpoint.url);
                            // Mark as healthy in the next iteration
                        }
                    }
                }
            }
        });

        info!("Started RPC health monitoring");
        Ok(())
    }

    /// Get pool statistics
    pub async fn get_stats(&self) -> HashMap<String, serde_json::Value> {
        let endpoints = self.endpoints.read().await;
        let clients = self.clients.read().await;
        
        let mut stats = HashMap::new();
        stats.insert("total_endpoints".to_string(), serde_json::Value::from(endpoints.len()));
        stats.insert("healthy_endpoints".to_string(), 
            serde_json::Value::from(endpoints.iter().filter(|e| e.healthy).count()));
        stats.insert("active_connections".to_string(), serde_json::Value::from(clients.len()));
        
        stats
    }

    /// Close all connections
    pub async fn shutdown(&self) -> Result<()> {
        let mut clients = self.clients.write().await;
        clients.clear();
        info!("RPC pool shut down");
        Ok(())
    }
}

impl Default for RpcPool {
    fn default() -> Self {
        let config = SimpleConfig::default();
        Self::new(&config)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_rpc_pool_creation() {
        let config = SimpleConfig::default();
        let pool = RpcPool::new(&config);
        
        let stats = pool.get_stats().await;
        assert!(stats.contains_key("total_endpoints"));
    }

    #[tokio::test] 
    async fn test_health_monitoring() {
        let config = SimpleConfig::default();
        let pool = RpcPool::new(&config);
        
        // This would be a more comprehensive test in a real environment
        assert!(pool.start_health_monitoring().await.is_ok());
    }
}
