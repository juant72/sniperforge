// ===== SABER INTEGRATION MODULE =====
// Integración con Saber para obtener datos reales de pools
// URL: https://registry.saber.so/data/pools-info.mainnet.json

use std::collections::HashMap;
use std::time::{Duration, Instant};
use anyhow::{Result, anyhow};
use serde::{Deserialize, Serialize};
use tracing::{info, warn, debug};

// ===== SABER API CONSTANTS =====
const SABER_POOLS_API: &str = "https://registry.saber.so/data/pools-info.mainnet.json";
const SABER_TIMEOUT_SECONDS: u64 = 15;
const SABER_CACHE_TTL_SECONDS: u64 = 300; // 5 minutes cache

// ===== SABER DATA STRUCTURES =====
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SaberPool {
    pub name: Option<String>,
    pub address: Option<String>,
    pub tvl: Option<f64>,
    pub volume24h: Option<f64>,
    pub tokens: Option<Vec<SaberToken>>,
    pub token_a: Option<SaberToken>,
    pub token_b: Option<SaberToken>,
    pub fee: Option<f64>,
    pub liquidity: Option<f64>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SaberToken {
    pub symbol: Option<String>,
    pub mint: Option<String>,
    pub decimals: Option<u8>,
    pub reserve: Option<f64>,
}

#[derive(Debug, Clone)]
pub struct SaberPoolCache {
    pools: Vec<SaberPool>,
    last_updated: Instant,
    ttl: Duration,
}

#[derive(Debug, Clone)]
pub struct SaberIntegration {
    client: reqwest::Client,
    cache: Option<SaberPoolCache>,
    target_tokens: Vec<String>,
}

impl SaberIntegration {
    pub fn new() -> Self {
        let client = reqwest::Client::builder()
            .timeout(Duration::from_secs(SABER_TIMEOUT_SECONDS))
            .build()
            .expect("Failed to create HTTP client");

        // Tokens objetivo para PROPOSAL-003
        let target_tokens = vec![
            "SOL".to_string(),
            "USDC".to_string(),
            "USDT".to_string(),
            "BONK".to_string(),
            "RAY".to_string(),
            "ORCA".to_string(),
            "PYTH".to_string(),
            "JTO".to_string(),
        ];

        Self {
            client,
            cache: None,
            target_tokens,
        }
    }

    /// Obtener pools de Saber con cache inteligente
    pub async fn get_pools(&mut self) -> Result<Vec<SaberPool>> {
        // Verificar si el cache es válido
        if let Some(ref cache) = self.cache {
            if cache.last_updated.elapsed() < cache.ttl {
                debug!("Using cached Saber pools data");
                return Ok(cache.pools.clone()); // Return clone to avoid borrowing issues
            }
        }

        // Fetch fresh data
        info!("🗡️ Fetching fresh Saber pools data...");
        let pools = self.fetch_pools_from_api().await?;
        
        // Update cache
        self.cache = Some(SaberPoolCache {
            pools: pools.clone(),
            last_updated: Instant::now(),
            ttl: Duration::from_secs(SABER_CACHE_TTL_SECONDS),
        });

        info!("✅ Saber pools data updated - {} pools cached", pools.len());
        Ok(pools)
    }

    /// Obtener pools relevantes para PROPOSAL-003 tokens
    pub async fn get_relevant_pools(&mut self) -> Result<Vec<SaberPool>> {
        let all_pools = self.get_pools().await?;
        let mut relevant_pools = Vec::new();

        for pool in &all_pools { // Use reference to avoid move
            if self.is_pool_relevant(pool) {
                relevant_pools.push(pool.clone());
            }
        }

        info!("🎯 Found {} relevant Saber pools for PROPOSAL-003", relevant_pools.len());
        Ok(relevant_pools)
    }

    /// Verificar si un pool es relevante para nuestros tokens objetivo
    fn is_pool_relevant(&self, pool: &SaberPool) -> bool {
        // Verificar por nombre del pool
        if let Some(name) = &pool.name {
            for token in &self.target_tokens {
                if name.to_uppercase().contains(token) {
                    return true;
                }
            }
        }

        // Verificar por tokens individuales
        if let Some(tokens) = &pool.tokens {
            for token in tokens {
                if let Some(symbol) = &token.symbol {
                    if self.target_tokens.contains(&symbol.to_uppercase()) {
                        return true;
                    }
                }
            }
        }

        // Verificar token_a y token_b
        if let Some(token_a) = &pool.token_a {
            if let Some(symbol) = &token_a.symbol {
                if self.target_tokens.contains(&symbol.to_uppercase()) {
                    return true;
                }
            }
        }

        if let Some(token_b) = &pool.token_b {
            if let Some(symbol) = &token_b.symbol {
                if self.target_tokens.contains(&symbol.to_uppercase()) {
                    return true;
                }
            }
        }

        false
    }

    /// Fetch pools from Saber API
    async fn fetch_pools_from_api(&self) -> Result<Vec<SaberPool>> {
        debug!("Fetching from Saber API: {}", SABER_POOLS_API);
        
        let response = self.client
            .get(SABER_POOLS_API)
            .send()
            .await
            .map_err(|e| anyhow!("Failed to fetch Saber pools: {}", e))?;

        if !response.status().is_success() {
            return Err(anyhow!("Saber API returned error: {}", response.status()));
        }

        let pools_data: serde_json::Value = response.json().await
            .map_err(|e| anyhow!("Failed to parse Saber response: {}", e))?;

        // Handle different response formats
        let pools_array = if let Some(direct_array) = pools_data.as_array() {
            direct_array.clone()
        } else if let Some(pools_object) = pools_data.as_object() {
            // Look for arrays within the object
            let mut found_array = Vec::new();
            for (key, value) in pools_object {
                if value.is_array() {
                    if let Some(arr) = value.as_array() {
                        if arr.len() > 5 { // Must have reasonable number of pools
                            debug!("Using '{}' array with {} elements", key, arr.len());
                            found_array = arr.clone();
                            break;
                        }
                    }
                }
            }
            found_array
        } else {
            return Err(anyhow!("Unexpected Saber API response format"));
        };

        // Parse pools
        let mut pools = Vec::new();
        for pool_value in pools_array {
            match serde_json::from_value::<SaberPool>(pool_value) {
                Ok(pool) => pools.push(pool),
                Err(e) => {
                    debug!("Failed to parse pool: {}", e);
                    // Continue with other pools
                }
            }
        }

        if pools.is_empty() {
            warn!("No valid pools found in Saber response");
        }

        Ok(pools)
    }

    /// Get pool statistics for monitoring
    pub async fn get_pool_stats(&mut self) -> Result<SaberPoolStats> {
        let pools = self.get_pools().await?;
        
        // Clone target_tokens to avoid borrowing conflicts
        let target_tokens = self.target_tokens.clone();
        
        let mut relevant_pools = Vec::new();
        for pool in &pools {
            if self.is_pool_relevant(pool) {
                relevant_pools.push(pool.clone());
            }
        }

        let total_tvl: f64 = relevant_pools.iter()
            .filter_map(|p| p.tvl)
            .sum();

        let total_volume: f64 = relevant_pools.iter()
            .filter_map(|p| p.volume24h)
            .sum();

        let mut token_distribution = HashMap::new();
        for token in &target_tokens {
            let count = relevant_pools.iter()
                .filter(|p| self.pool_contains_token_static(p, token))
                .count();
            token_distribution.insert(token.clone(), count);
        }

        Ok(SaberPoolStats {
            total_pools: pools.len(),
            relevant_pools: relevant_pools.len(),
            total_tvl,
            total_volume_24h: total_volume,
            token_distribution,
        })
    }

    fn pool_contains_token(&self, pool: &SaberPool, token: &str) -> bool {
        if let Some(name) = &pool.name {
            if name.to_uppercase().contains(token) {
                return true;
            }
        }

        if let Some(tokens) = &pool.tokens {
            for t in tokens {
                if let Some(symbol) = &t.symbol {
                    if symbol.to_uppercase() == token.to_uppercase() {
                        return true;
                    }
                }
            }
        }

        false
    }
    
    fn pool_contains_token_static(pool: &SaberPool, token: &str) -> bool {
        if let Some(name) = &pool.name {
            if name.to_uppercase().contains(token) {
                return true;
            }
        }

        if let Some(tokens) = &pool.tokens {
            for t in tokens {
                if let Some(symbol) = &t.symbol {
                    if symbol.to_uppercase() == token.to_uppercase() {
                        return true;
                    }
                }
            }
        }

        false
    }
}

#[derive(Debug, Clone)]
pub struct SaberPoolStats {
    pub total_pools: usize,
    pub relevant_pools: usize,
    pub total_tvl: f64,
    pub total_volume_24h: f64,
    pub token_distribution: HashMap<String, usize>,
}

impl SaberPoolStats {
    pub fn log_stats(&self) {
        info!("📊 Saber Pool Statistics:");
        info!("   Total pools: {}", self.total_pools);
        info!("   Relevant pools: {}", self.relevant_pools);
        info!("   Total TVL: ${:.2}", self.total_tvl);
        info!("   24h Volume: ${:.2}", self.total_volume_24h);
        info!("   Token distribution:");
        for (token, count) in &self.token_distribution {
            if *count > 0 {
                info!("     {}: {} pools", token, count);
            }
        }
    }
}
