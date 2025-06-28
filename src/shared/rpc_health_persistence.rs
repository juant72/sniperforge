//! RPC Health Persistence - Persist endpoint health across restarts
//! 
//! This module saves/loads RPC endpoint health data to avoid retrying
//! known problematic endpoints immediately after restart.

use anyhow::Result;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::Path;
use std::time::{Duration, SystemTime, UNIX_EPOCH};
use tokio::fs;
use tracing::{info, warn, error};

use super::rpc_pool::RpcEndpointHealth;

#[derive(Debug, Serialize, Deserialize)]
pub struct PersistedEndpointHealth {
    pub url: String,
    pub is_healthy: bool,
    pub consecutive_failures: u32,
    pub last_failure_timestamp: Option<u64>, // Unix timestamp
    pub last_success_timestamp: Option<u64>, // Unix timestamp
    pub average_response_time_ms: u64,
    pub total_requests: u64,
    pub successful_requests: u64,
    pub failure_reasons: Vec<String>, // Track specific error types like "410 Gone"
}

#[derive(Debug, Serialize, Deserialize)]
pub struct RpcHealthCache {
    pub version: String,
    pub last_updated: u64,
    pub endpoints: HashMap<String, PersistedEndpointHealth>,
}

impl RpcHealthCache {
    pub fn new() -> Self {
        Self {
            version: "1.0".to_string(),
            last_updated: SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .unwrap_or_default()
                .as_secs(),
            endpoints: HashMap::new(),
        }
    }
}

pub struct RpcHealthPersistence {
    cache_file: String,
    max_age_hours: u64,
}

impl RpcHealthPersistence {
    pub fn new(cache_file: String, max_age_hours: u64) -> Self {
        Self {
            cache_file,
            max_age_hours,
        }
    }

    /// Load persisted health data
    pub async fn load(&self) -> Result<HashMap<String, RpcEndpointHealth>> {
        if !Path::new(&self.cache_file).exists() {
            info!("📋 No RPC health cache found, starting fresh");
            return Ok(HashMap::new());
        }

        let content = fs::read_to_string(&self.cache_file).await?;
        let cache: RpcHealthCache = serde_json::from_str(&content)?;

        // Check if cache is too old
        let now = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap_or_default()
            .as_secs();
        
        let cache_age_hours = (now - cache.last_updated) / 3600;
        if cache_age_hours > self.max_age_hours {
            warn!("📋 RPC health cache is {} hours old, ignoring", cache_age_hours);
            return Ok(HashMap::new());
        }

        info!("📋 Loaded RPC health cache with {} endpoints", cache.endpoints.len());

        // Convert persisted data to runtime format
        let mut runtime_health = HashMap::new();
        for (url, persisted) in cache.endpoints {
            // Only restore if not too many recent failures
            if persisted.consecutive_failures < 10 {
                let health = RpcEndpointHealth {
                    url: url.clone(),
                    is_healthy: persisted.is_healthy,
                    consecutive_failures: persisted.consecutive_failures,
                    last_failure_time: persisted.last_failure_timestamp
                        .map(|ts| {
                            let duration_since_epoch = Duration::from_secs(ts);
                            std::time::Instant::now() - Duration::from_secs(now - ts)
                        }),
                    last_success_time: persisted.last_success_timestamp
                        .map(|ts| {
                            let duration_since_epoch = Duration::from_secs(ts);
                            std::time::Instant::now() - Duration::from_secs(now - ts)
                        }),
                    average_response_time: Duration::from_millis(persisted.average_response_time_ms),
                    total_requests: persisted.total_requests,
                    successful_requests: persisted.successful_requests,
                };

                // Log problematic endpoints
                if !persisted.is_healthy {
                    let reasons = persisted.failure_reasons.join(", ");
                    warn!("📋 Restored unhealthy endpoint: {} (failures: {}, reasons: {})", 
                          url, persisted.consecutive_failures, reasons);
                }

                runtime_health.insert(url, health);
            } else {
                warn!("📋 Skipping endpoint {} due to too many failures ({})", 
                      url, persisted.consecutive_failures);
            }
        }

        Ok(runtime_health)
    }

    /// Save current health data
    pub async fn save(&self, health_data: &HashMap<String, RpcEndpointHealth>) -> Result<()> {
        let now = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap_or_default()
            .as_secs();

        let mut cache = RpcHealthCache::new();
        cache.last_updated = now;

        for (url, health) in health_data {
            // Determine failure reasons (basic classification)
            let mut failure_reasons = Vec::new();
            if !health.is_healthy {
                if health.consecutive_failures >= 5 {
                    failure_reasons.push("circuit_breaker_triggered".to_string());
                }
                if health.average_response_time > Duration::from_secs(30) {
                    failure_reasons.push("slow_response".to_string());
                }
                // Note: Specific error types like "410 Gone" would need to be
                // tracked in RpcEndpointHealth for more detailed reasons
            }

            let persisted = PersistedEndpointHealth {
                url: url.clone(),
                is_healthy: health.is_healthy,
                consecutive_failures: health.consecutive_failures,
                last_failure_timestamp: health.last_failure_time
                    .map(|_| now), // Approximate - we don't have exact timestamp
                last_success_timestamp: health.last_success_time
                    .map(|_| now), // Approximate - we don't have exact timestamp
                average_response_time_ms: health.average_response_time.as_millis() as u64,
                total_requests: health.total_requests,
                successful_requests: health.successful_requests,
                failure_reasons,
            };

            cache.endpoints.insert(url.clone(), persisted);
        }

        let content = serde_json::to_string_pretty(&cache)?;
        fs::write(&self.cache_file, content).await?;

        info!("💾 Saved RPC health cache with {} endpoints", cache.endpoints.len());
        Ok(())
    }

    /// Get statistics about persisted data
    pub async fn get_stats(&self) -> Result<String> {
        if !Path::new(&self.cache_file).exists() {
            return Ok("No cache file exists".to_string());
        }

        let content = fs::read_to_string(&self.cache_file).await?;
        let cache: RpcHealthCache = serde_json::from_str(&content)?;

        let healthy_count = cache.endpoints.values()
            .filter(|e| e.is_healthy)
            .count();
        
        let unhealthy_count = cache.endpoints.len() - healthy_count;

        let avg_failures = if !cache.endpoints.is_empty() {
            cache.endpoints.values()
                .map(|e| e.consecutive_failures as f64)
                .sum::<f64>() / cache.endpoints.len() as f64
        } else {
            0.0
        };

        Ok(format!(
            "RPC Health Cache Stats:\n\
             - Total endpoints: {}\n\
             - Healthy: {}\n\
             - Unhealthy: {}\n\
             - Avg failures: {:.1}\n\
             - Last updated: {} hours ago",
            cache.endpoints.len(),
            healthy_count,
            unhealthy_count,
            avg_failures,
            (SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .unwrap_or_default()
                .as_secs() - cache.last_updated) / 3600
        ))
    }
}

impl Default for RpcHealthPersistence {
    fn default() -> Self {
        Self::new(
            "cache/rpc_health.json".to_string(),
            24, // 24 hours max age
        )
    }
}
