use anyhow::Result;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::{Mutex, RwLock, Semaphore};
use tracing::{debug, error, info, warn};
use uuid::Uuid;

use crate::types::PlatformError;

/// Types of resources that can be managed
#[derive(Debug, Clone, Hash, Eq, PartialEq, Serialize, Deserialize)]
pub enum ResourceType {
    RpcConnection,
    ComputeUnit,
    MemoryPool,
    DiskSpace,
    NetworkBandwidth,
    Custom(String),
}

/// Resource allocation request
#[derive(Debug, Clone)]
pub struct ResourceRequest {
    pub id: Uuid,
    pub requester: String,
    pub resource_type: ResourceType,
    pub amount: u64,
    pub priority: ResourcePriority,
    pub timeout: Option<chrono::Duration>,
}

/// Resource allocation priority
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub enum ResourcePriority {
    Low = 1,
    Normal = 2,
    High = 3,
    Critical = 4,
}

/// Resource allocation result
#[derive(Debug, Clone)]
pub struct ResourceAllocation {
    pub request_id: Uuid,
    pub allocation_id: Uuid,
    pub resource_type: ResourceType,
    pub amount: u64,
    pub allocated_at: chrono::DateTime<chrono::Utc>,
    pub expires_at: Option<chrono::DateTime<chrono::Utc>>,
}

/// Resource pool configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ResourcePoolConfig {
    pub resource_type: ResourceType,
    pub max_capacity: u64,
    pub reserved_capacity: u64,
    pub allocation_timeout: chrono::Duration,
    pub cleanup_interval: chrono::Duration,
}

/// Resource pool for managing specific resource types
struct ResourcePool {
    config: ResourcePoolConfig,
    semaphore: Arc<Semaphore>,
    allocations: Arc<RwLock<HashMap<Uuid, ResourceAllocation>>>,
    usage_stats: Arc<Mutex<ResourceUsageStats>>,
}

/// Resource usage statistics
#[derive(Debug, Clone, Default, Serialize)]
pub struct ResourceUsageStats {
    pub total_requests: u64,
    pub successful_allocations: u64,
    pub failed_allocations: u64,
    pub current_usage: u64,
    pub peak_usage: u64,
    pub average_allocation_duration: f64,
}

/// Main resource coordinator
pub struct ResourceCoordinator {
    pools: Arc<RwLock<HashMap<ResourceType, ResourcePool>>>,
    pending_requests: Arc<RwLock<HashMap<Uuid, ResourceRequest>>>,
    allocation_history: Arc<RwLock<Vec<ResourceAllocation>>>,
    shutdown_tx: tokio::sync::mpsc::Sender<()>,
}

impl ResourceCoordinator {
    pub fn new() -> Self {
        let (shutdown_tx, _) = tokio::sync::mpsc::channel(1);

        Self {
            pools: Arc::new(RwLock::new(HashMap::new())),
            pending_requests: Arc::new(RwLock::new(HashMap::new())),
            allocation_history: Arc::new(RwLock::new(Vec::new())),
            shutdown_tx,
        }
    }

    /// Start the resource coordinator
    pub async fn start(&self) -> Result<()> {
        info!("Starting Resource Coordinator");

        // Initialize default resource pools
        self.initialize_default_pools().await?;

        // Start cleanup task
        self.start_cleanup_task().await;

        Ok(())
    }

    /// Stop the resource coordinator
    pub async fn stop(&self) -> Result<()> {
        info!("Stopping Resource Coordinator");

        // Release all allocations
        self.release_all_allocations().await?;

        // Send shutdown signal
        let _ = self.shutdown_tx.send(()).await;

        Ok(())
    }

    /// Add a resource pool
    pub async fn add_resource_pool(&self, config: ResourcePoolConfig) -> Result<()> {
        let resource_type = config.resource_type.clone();
        let available_permits = config.max_capacity.saturating_sub(config.reserved_capacity);

        let pool = ResourcePool {
            config: config.clone(),
            semaphore: Arc::new(Semaphore::new(available_permits as usize)),
            allocations: Arc::new(RwLock::new(HashMap::new())),
            usage_stats: Arc::new(Mutex::new(ResourceUsageStats::default())),
        };

        let mut pools = self.pools.write().await;
        pools.insert(resource_type.clone(), pool);

        info!(
            "Added resource pool: {:?} with capacity {}",
            resource_type, config.max_capacity
        );
        Ok(())
    }

    /// Request resource allocation
    pub async fn request_resource(
        &self,
        requester: String,
        resource_type: ResourceType,
        amount: u64,
        priority: ResourcePriority,
        timeout: Option<chrono::Duration>,
    ) -> Result<Uuid> {
        let request_id = Uuid::new_v4();
        let request = ResourceRequest {
            id: request_id,
            requester,
            resource_type: resource_type.clone(),
            amount,
            priority,
            timeout,
        };

        // Add to pending requests
        {
            let mut pending = self.pending_requests.write().await;
            pending.insert(request_id, request.clone());
        }

        debug!(
            "Resource request queued: {:?} for {} units of {:?}",
            request_id, amount, resource_type
        );

        // Try to allocate immediately
        match self.try_allocate_resource(&request).await {
            Ok(allocation) => {
                // Remove from pending
                let mut pending = self.pending_requests.write().await;
                pending.remove(&request_id);

                info!(
                    "Resource allocated immediately: {:?}",
                    allocation.allocation_id
                );
            }
            Err(e) => {
                debug!("Immediate allocation failed: {}. Request queued.", e);
            }
        }

        Ok(request_id)
    }

    /// Try to allocate a resource
    async fn try_allocate_resource(&self, request: &ResourceRequest) -> Result<ResourceAllocation> {
        let pools = self.pools.read().await;

        if let Some(pool) = pools.get(&request.resource_type) {
            // Update stats
            {
                let mut stats = pool.usage_stats.lock().await;
                stats.total_requests += 1;
            } // Try to acquire permits
            let permits_needed = request.amount as u32;

            match pool.semaphore.try_acquire_many(permits_needed) {
                Ok(permit) => {
                    let allocation_id = Uuid::new_v4();
                    let allocation = ResourceAllocation {
                        request_id: request.id,
                        allocation_id,
                        resource_type: request.resource_type.clone(),
                        amount: request.amount,
                        allocated_at: chrono::Utc::now(),
                        expires_at: request.timeout.map(|d| chrono::Utc::now() + d),
                    };

                    // Store allocation
                    {
                        let mut allocations = pool.allocations.write().await;
                        allocations.insert(allocation_id, allocation.clone());
                    }

                    // Update stats
                    {
                        let mut stats = pool.usage_stats.lock().await;
                        stats.successful_allocations += 1;
                        stats.current_usage += request.amount;
                        if stats.current_usage > stats.peak_usage {
                            stats.peak_usage = stats.current_usage;
                        }
                    }

                    // Add to history
                    {
                        let mut history = self.allocation_history.write().await;
                        history.push(allocation.clone());

                        // Keep history limited
                        if history.len() > 10000 {
                            history.drain(0..1000);
                        }
                    }

                    // Forget the permit to prevent auto-release
                    permit.forget();

                    info!(
                        "Resource allocated: {:?} ({} units of {:?})",
                        allocation_id, request.amount, request.resource_type
                    );

                    Ok(allocation)
                }
                Err(_) => {
                    // Update stats
                    let mut stats = pool.usage_stats.lock().await;
                    stats.failed_allocations += 1;

                    Err(PlatformError::ResourceManagement(format!(
                        "Insufficient resources: {:?}",
                        request.resource_type
                    ))
                    .into())
                }
            }
        } else {
            Err(PlatformError::ResourceManagement(format!(
                "Resource pool not found: {:?}",
                request.resource_type
            ))
            .into())
        }
    }

    /// Release a specific resource allocation
    pub async fn release_resource(&self, allocation_id: Uuid) -> Result<()> {
        let pools = self.pools.read().await;

        for pool in pools.values() {
            let mut allocations = pool.allocations.write().await;

            if let Some(allocation) = allocations.remove(&allocation_id) {
                // Release semaphore permits
                pool.semaphore.add_permits(allocation.amount as usize);

                // Update stats
                {
                    let mut stats = pool.usage_stats.lock().await;
                    stats.current_usage = stats.current_usage.saturating_sub(allocation.amount);
                }

                info!(
                    "Resource released: {:?} ({} units of {:?})",
                    allocation_id, allocation.amount, allocation.resource_type
                );

                return Ok(());
            }
        }

        Err(PlatformError::ResourceManagement("Allocation not found".to_string()).into())
    }

    /// Get resource usage statistics
    pub async fn get_resource_stats(
        &self,
        resource_type: &ResourceType,
    ) -> Option<ResourceUsageStats> {
        let pools = self.pools.read().await;

        if let Some(pool) = pools.get(resource_type) {
            let stats = pool.usage_stats.lock().await;
            Some(stats.clone())
        } else {
            None
        }
    }

    /// Get all resource statistics
    pub async fn get_all_resource_stats(&self) -> HashMap<ResourceType, ResourceUsageStats> {
        let pools = self.pools.read().await;
        let mut all_stats = HashMap::new();

        for (resource_type, pool) in pools.iter() {
            let stats = pool.usage_stats.lock().await;
            all_stats.insert(resource_type.clone(), stats.clone());
        }

        all_stats
    }

    /// Get current allocations
    pub async fn get_active_allocations(&self) -> HashMap<ResourceType, Vec<ResourceAllocation>> {
        let pools = self.pools.read().await;
        let mut all_allocations = HashMap::new();

        for (resource_type, pool) in pools.iter() {
            let allocations = pool.allocations.read().await;
            all_allocations.insert(
                resource_type.clone(),
                allocations.values().cloned().collect(),
            );
        }

        all_allocations
    }

    /// Initialize default resource pools
    async fn initialize_default_pools(&self) -> Result<()> {
        // RPC Connection pool
        let rpc_config = ResourcePoolConfig {
            resource_type: ResourceType::RpcConnection,
            max_capacity: 100,
            reserved_capacity: 10,
            allocation_timeout: chrono::Duration::seconds(30),
            cleanup_interval: chrono::Duration::minutes(5),
        };
        self.add_resource_pool(rpc_config).await?;

        // Compute units pool
        let compute_config = ResourcePoolConfig {
            resource_type: ResourceType::ComputeUnit,
            max_capacity: 1000000, // 1M compute units
            reserved_capacity: 100000,
            allocation_timeout: chrono::Duration::seconds(10),
            cleanup_interval: chrono::Duration::minutes(1),
        };
        self.add_resource_pool(compute_config).await?;

        // Memory pool
        let memory_config = ResourcePoolConfig {
            resource_type: ResourceType::MemoryPool,
            max_capacity: 1024 * 1024 * 1024,     // 1GB in bytes
            reserved_capacity: 100 * 1024 * 1024, // 100MB reserved
            allocation_timeout: chrono::Duration::seconds(5),
            cleanup_interval: chrono::Duration::minutes(2),
        };
        self.add_resource_pool(memory_config).await?;

        info!("Initialized default resource pools");
        Ok(())
    }

    /// Start cleanup task for expired allocations
    async fn start_cleanup_task(&self) {
        let pools = self.pools.clone();

        tokio::spawn(async move {
            let mut interval = tokio::time::interval(tokio::time::Duration::from_secs(60));

            loop {
                interval.tick().await;

                let pools_guard = pools.read().await;
                for pool in pools_guard.values() {
                    let mut allocations = pool.allocations.write().await;
                    let now = chrono::Utc::now();
                    let mut expired_allocations = Vec::new();

                    // Find expired allocations
                    for (id, allocation) in allocations.iter() {
                        if let Some(expires_at) = allocation.expires_at {
                            if now > expires_at {
                                expired_allocations.push(*id);
                            }
                        }
                    }

                    // Remove expired allocations
                    for allocation_id in expired_allocations {
                        if let Some(allocation) = allocations.remove(&allocation_id) {
                            pool.semaphore.add_permits(allocation.amount as usize);

                            // Update stats
                            let mut stats = pool.usage_stats.lock().await;
                            stats.current_usage =
                                stats.current_usage.saturating_sub(allocation.amount);

                            debug!("Cleaned up expired allocation: {:?}", allocation_id);
                        }
                    }
                }
            }
        });
    }

    /// Release all allocations (used during shutdown)
    async fn release_all_allocations(&self) -> Result<()> {
        let pools = self.pools.read().await;

        for pool in pools.values() {
            let mut allocations = pool.allocations.write().await;

            for allocation in allocations.values() {
                pool.semaphore.add_permits(allocation.amount as usize);
            }

            allocations.clear();

            // Reset stats
            let mut stats = pool.usage_stats.lock().await;
            stats.current_usage = 0;
        }

        info!("Released all resource allocations");
        Ok(())
    }

    /// Health check for resource coordinator
    pub async fn health_check(&self) -> Result<crate::types::HealthStatus> {
        use std::collections::HashMap;

        let resource_pools = self.pools.read().await;
        let mut total_available = 0;
        let mut total_capacity = 0;

        for pool in resource_pools.values() {
            total_available += pool.semaphore.available_permits();
            total_capacity += pool.config.max_capacity;
        }
        let usage_percent = if total_capacity > 0 {
            let used = total_capacity as isize - total_available as isize;
            (used as f64 / total_capacity as f64) * 100.0
        } else {
            0.0
        };

        let is_healthy = usage_percent < 90.0; // Consider unhealthy if >90% used

        let mut metrics = HashMap::new();
        metrics.insert(
            "usage_percent".to_string(),
            serde_json::Value::Number(serde_json::Number::from_f64(usage_percent).unwrap()),
        );
        metrics.insert(
            "total_capacity".to_string(),
            serde_json::Value::Number(serde_json::Number::from(total_capacity)),
        );
        metrics.insert(
            "total_available".to_string(),
            serde_json::Value::Number(serde_json::Number::from(total_available)),
        );

        Ok(crate::types::HealthStatus {
            is_healthy,
            component: "resource_coordinator".to_string(),
            message: if is_healthy {
                None
            } else {
                Some(format!("Resource usage high: {:.1}%", usage_percent))
            },
            checked_at: chrono::Utc::now(),
            metrics,
        })
    }
}

impl Default for ResourceCoordinator {
    fn default() -> Self {
        Self::new()
    }
}
