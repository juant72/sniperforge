// ===== ADVANCED PERFORMANCE OPTIMIZER MODULE =====
// CPU, memory, and I/O optimization for high-frequency trading

use anyhow::Result;
use serde::{Serialize, Deserialize};
use std::collections::HashMap;
use std::sync::Arc;
use std::time::{Duration, Instant};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PerformanceMetrics {
    pub cpu_usage_percent: f64,
    pub memory_usage_mb: f64,
    pub network_latency_ms: f64,
    pub requests_per_second: f64,
    pub cache_hit_rate: f64,
    pub average_response_time_ms: f64,
    pub error_rate_percent: f64,
    pub throughput_ops_per_sec: f64,
}

#[derive(Debug, Clone)]
pub struct CacheEntry<T> {
    data: T,
    created_at: Instant,
    access_count: u64,
    last_accessed: Instant,
}

#[derive(Debug)]
pub struct AdaptiveCache<T> {
    entries: HashMap<String, CacheEntry<T>>,
    max_size: usize,
    ttl: Duration,
    hit_count: u64,
    miss_count: u64,
}

impl<T: Clone> AdaptiveCache<T> {
    pub fn new(max_size: usize, ttl_ms: u64) -> Self {
        Self {
            entries: HashMap::new(),
            max_size,
            ttl: Duration::from_millis(ttl_ms),
            hit_count: 0,
            miss_count: 0,
        }
    }
    
    pub fn get(&mut self, key: &str) -> Option<T> {
        if let Some(entry) = self.entries.get_mut(key) {
            if entry.created_at.elapsed() < self.ttl {
                entry.access_count += 1;
                entry.last_accessed = Instant::now();
                self.hit_count += 1;
                return Some(entry.data.clone());
            } else {
                self.entries.remove(key);
            }
        }
        self.miss_count += 1;
        None
    }
    
    pub fn put(&mut self, key: String, value: T) {
        if self.entries.len() >= self.max_size {
            self.evict_least_used();
        }
        
        let entry = CacheEntry {
            data: value,
            created_at: Instant::now(),
            access_count: 1,
            last_accessed: Instant::now(),
        };
        
        self.entries.insert(key, entry);
    }
    
    fn evict_least_used(&mut self) {
        if let Some((key_to_remove, _)) = self.entries
            .iter()
            .min_by_key(|(_, entry)| (entry.access_count, entry.last_accessed))
            .map(|(k, v)| (k.clone(), v.clone()))
        {
            self.entries.remove(&key_to_remove);
        }
    }
    
    pub fn hit_rate(&self) -> f64 {
        let total = self.hit_count + self.miss_count;
        if total == 0 { 0.0 } else { self.hit_count as f64 / total as f64 }
    }
    
    pub fn clear(&mut self) {
        self.entries.clear();
        self.hit_count = 0;
        self.miss_count = 0;
    }
}

#[derive(Debug)]
pub struct RateLimiter {
    requests: Vec<Instant>,
    max_requests: usize,
    window: Duration,
}

impl RateLimiter {
    pub fn new(max_requests: usize, window_ms: u64) -> Self {
        Self {
            requests: Vec::new(),
            max_requests,
            window: Duration::from_millis(window_ms),
        }
    }
    
    pub fn try_acquire(&mut self) -> bool {
        let now = Instant::now();
        
        // Remove old requests outside the window
        self.requests.retain(|&time| now.duration_since(time) < self.window);
        
        if self.requests.len() < self.max_requests {
            self.requests.push(now);
            true
        } else {
            false
        }
    }
    
    pub fn requests_in_window(&self) -> usize {
        let now = Instant::now();
        self.requests.iter()
            .filter(|&&time| now.duration_since(time) < self.window)
            .count()
    }
}

#[derive(Debug)]
pub struct ConnectionPool {
    available: Vec<u32>, // Connection IDs
    in_use: Vec<u32>,
    max_connections: usize,
    next_id: u32,
}

impl ConnectionPool {
    pub fn new(max_connections: usize) -> Self {
        Self {
            available: Vec::new(),
            in_use: Vec::new(),
            max_connections,
            next_id: 0,
        }
    }
    
    pub fn acquire(&mut self) -> Option<u32> {
        if let Some(conn_id) = self.available.pop() {
            self.in_use.push(conn_id);
            Some(conn_id)
        } else if self.in_use.len() < self.max_connections {
            let conn_id = self.next_id;
            self.next_id += 1;
            self.in_use.push(conn_id);
            Some(conn_id)
        } else {
            None
        }
    }
    
    pub fn release(&mut self, conn_id: u32) {
        if let Some(pos) = self.in_use.iter().position(|&id| id == conn_id) {
            self.in_use.remove(pos);
            self.available.push(conn_id);
        }
    }
    
    pub fn stats(&self) -> (usize, usize, usize) {
        (self.available.len(), self.in_use.len(), self.max_connections)
    }
}

#[derive(Debug)]
pub struct AdvancedPerformanceOptimizer {
    price_cache: AdaptiveCache<f64>,
    route_cache: AdaptiveCache<Vec<String>>,
    opportunity_cache: AdaptiveCache<serde_json::Value>,
    rate_limiter: RateLimiter,
    connection_pool: ConnectionPool,
    metrics: PerformanceMetrics,
    start_time: Instant,
    request_count: u64,
    total_response_time: Duration,
    error_count: u64,
    optimization_level: String,
}

impl AdvancedPerformanceOptimizer {
    pub fn new(optimization_level: &str) -> Self {
        let (cache_size, ttl_ms, max_rps, max_connections) = match optimization_level {
            "fast" => (1000, 2000, 100, 10),
            "balanced" => (5000, 5000, 50, 20),
            "thorough" => (10000, 10000, 20, 50),
            _ => (5000, 5000, 50, 20), // Default to balanced
        };
        
        Self {
            price_cache: AdaptiveCache::new(cache_size, ttl_ms),
            route_cache: AdaptiveCache::new(cache_size / 2, ttl_ms * 2),
            opportunity_cache: AdaptiveCache::new(cache_size / 4, ttl_ms / 2),
            rate_limiter: RateLimiter::new(max_rps, 60000), // Per minute
            connection_pool: ConnectionPool::new(max_connections),
            metrics: PerformanceMetrics {
                cpu_usage_percent: 0.0,
                memory_usage_mb: 0.0,
                network_latency_ms: 0.0,
                requests_per_second: 0.0,
                cache_hit_rate: 0.0,
                average_response_time_ms: 0.0,
                error_rate_percent: 0.0,
                throughput_ops_per_sec: 0.0,
            },
            start_time: Instant::now(),
            request_count: 0,
            total_response_time: Duration::from_millis(0),
            error_count: 0,
            optimization_level: optimization_level.to_string(),
        }
    }
    
    // Cache Management
    pub fn cache_price(&mut self, token_pair: &str, price: f64) {
        self.price_cache.put(token_pair.to_string(), price);
    }
    
    pub fn get_cached_price(&mut self, token_pair: &str) -> Option<f64> {
        self.price_cache.get(token_pair)
    }
    
    pub fn cache_route(&mut self, route_key: &str, route: Vec<String>) {
        self.route_cache.put(route_key.to_string(), route);
    }
    
    pub fn get_cached_route(&mut self, route_key: &str) -> Option<Vec<String>> {
        self.route_cache.get(route_key)
    }
    
    pub fn cache_opportunity(&mut self, opp_key: &str, opportunity: serde_json::Value) {
        self.opportunity_cache.put(opp_key.to_string(), opportunity);
    }
    
    pub fn get_cached_opportunity(&mut self, opp_key: &str) -> Option<serde_json::Value> {
        self.opportunity_cache.get(opp_key)
    }
    
    // Rate Limiting
    pub fn can_make_request(&mut self) -> bool {
        self.rate_limiter.try_acquire()
    }
    
    pub fn requests_remaining(&self) -> usize {
        self.rate_limiter.max_requests - self.rate_limiter.requests_in_window()
    }
    
    // Connection Management
    pub fn acquire_connection(&mut self) -> Option<u32> {
        self.connection_pool.acquire()
    }
    
    pub fn release_connection(&mut self, conn_id: u32) {
        self.connection_pool.release(conn_id);
    }
    
    // Performance Tracking
    pub fn record_request(&mut self, response_time: Duration, success: bool) {
        self.request_count += 1;
        self.total_response_time += response_time;
        
        if !success {
            self.error_count += 1;
        }
        
        self.update_metrics();
    }
    
    fn update_metrics(&mut self) {
        let elapsed = self.start_time.elapsed();
        let elapsed_secs = elapsed.as_secs_f64();
        
        if elapsed_secs > 0.0 {
            self.metrics.requests_per_second = self.request_count as f64 / elapsed_secs;
            self.metrics.throughput_ops_per_sec = self.request_count as f64 / elapsed_secs;
        }
        
        if self.request_count > 0 {
            self.metrics.average_response_time_ms = 
                self.total_response_time.as_millis() as f64 / self.request_count as f64;
            self.metrics.error_rate_percent = 
                (self.error_count as f64 / self.request_count as f64) * 100.0;
        }
        
        // Update cache hit rates
        self.metrics.cache_hit_rate = (
            self.price_cache.hit_rate() + 
            self.route_cache.hit_rate() + 
            self.opportunity_cache.hit_rate()
        ) / 3.0;
        
        // Simulate system metrics (in production, would use actual system calls)
        self.simulate_system_metrics();
    }
    
    fn simulate_system_metrics(&mut self) {
        // Simulate CPU usage based on request rate
        let target_cpu = match self.optimization_level.as_str() {
            "fast" => 30.0 + (self.metrics.requests_per_second * 0.5),
            "balanced" => 50.0 + (self.metrics.requests_per_second * 0.3),
            "thorough" => 70.0 + (self.metrics.requests_per_second * 0.2),
            _ => 50.0,
        };
        self.metrics.cpu_usage_percent = target_cpu.min(100.0);
        
        // Simulate memory usage
        let base_memory = match self.optimization_level.as_str() {
            "fast" => 100.0,
            "balanced" => 200.0,
            "thorough" => 400.0,
            _ => 200.0,
        };
        let cache_memory = (self.price_cache.entries.len() * 8) as f64 / 1024.0; // KB to MB
        self.metrics.memory_usage_mb = base_memory + cache_memory;
        
        // Simulate network latency
        self.metrics.network_latency_ms = 10.0 + rand::random::<f64>() * 20.0;
    }
    
    // Optimization Strategies
    pub fn optimize_for_latency(&mut self) {
        // Reduce cache TTL for fresher data
        self.price_cache.ttl = Duration::from_millis(1000);
        self.opportunity_cache.ttl = Duration::from_millis(500);
        
        // Increase rate limits for faster throughput
        self.rate_limiter = RateLimiter::new(200, 60000);
        
        tracing::info!("🚀 Optimized for low latency");
    }
    
    pub fn optimize_for_accuracy(&mut self) {
        // Increase cache size for better hit rates
        self.price_cache = AdaptiveCache::new(20000, 10000);
        self.route_cache = AdaptiveCache::new(10000, 20000);
        
        // Reduce rate limits for more careful processing
        self.rate_limiter = RateLimiter::new(30, 60000);
        
        tracing::info!("🎯 Optimized for accuracy");
    }
    
    pub fn optimize_for_memory(&mut self) {
        // Reduce cache sizes
        self.price_cache = AdaptiveCache::new(2000, 3000);
        self.route_cache = AdaptiveCache::new(1000, 5000);
        self.opportunity_cache = AdaptiveCache::new(500, 2000);
        
        tracing::info!("💾 Optimized for memory usage");
    }
    
    // Adaptive Optimization
    pub fn auto_tune(&mut self) {
        let metrics = &self.metrics;
        
        // Auto-tune based on current performance
        if metrics.cpu_usage_percent > 90.0 {
            self.optimize_for_memory();
            tracing::warn!("🔧 Auto-tuned for high CPU usage");
        } else if metrics.average_response_time_ms > 1000.0 {
            self.optimize_for_latency();
            tracing::warn!("🔧 Auto-tuned for high latency");
        } else if metrics.error_rate_percent > 5.0 {
            self.optimize_for_accuracy();
            tracing::warn!("🔧 Auto-tuned for high error rate");
        }
        
        // Clear caches if hit rate is too low
        if metrics.cache_hit_rate < 0.3 {
            self.clear_all_caches();
            tracing::info!("🧹 Cleared caches due to low hit rate");
        }
    }
    
    // Cleanup Operations
    pub fn clear_all_caches(&mut self) {
        self.price_cache.clear();
        self.route_cache.clear();
        self.opportunity_cache.clear();
    }
    
    pub fn cleanup_expired(&mut self) {
        // Force cache cleanup by getting non-existent keys
        self.price_cache.get("__cleanup__");
        self.route_cache.get("__cleanup__");
        self.opportunity_cache.get("__cleanup__");
    }
    
    // Monitoring and Reporting
    pub fn get_metrics(&self) -> &PerformanceMetrics {
        &self.metrics
    }
    
    pub fn get_detailed_stats(&self) -> HashMap<String, serde_json::Value> {
        let mut stats = HashMap::new();
        
        stats.insert("optimization_level".to_string(), self.optimization_level.clone().into());
        stats.insert("uptime_seconds".to_string(), self.start_time.elapsed().as_secs().into());
        stats.insert("total_requests".to_string(), self.request_count.into());
        stats.insert("total_errors".to_string(), self.error_count.into());
        
        // Cache statistics
        stats.insert("price_cache_hit_rate".to_string(), self.price_cache.hit_rate().into());
        stats.insert("route_cache_hit_rate".to_string(), self.route_cache.hit_rate().into());
        stats.insert("opportunity_cache_hit_rate".to_string(), self.opportunity_cache.hit_rate().into());
        stats.insert("price_cache_size".to_string(), self.price_cache.entries.len().into());
        stats.insert("route_cache_size".to_string(), self.route_cache.entries.len().into());
        stats.insert("opportunity_cache_size".to_string(), self.opportunity_cache.entries.len().into());
        
        // Connection pool stats
        let (available, in_use, max_conn) = self.connection_pool.stats();
        stats.insert("connections_available".to_string(), available.into());
        stats.insert("connections_in_use".to_string(), in_use.into());
        stats.insert("connections_max".to_string(), max_conn.into());
        
        // Rate limiter stats
        stats.insert("requests_in_window".to_string(), self.rate_limiter.requests_in_window().into());
        stats.insert("max_requests_per_window".to_string(), self.rate_limiter.max_requests.into());
        
        // Performance metrics
        stats.insert("cpu_usage_percent".to_string(), self.metrics.cpu_usage_percent.into());
        stats.insert("memory_usage_mb".to_string(), self.metrics.memory_usage_mb.into());
        stats.insert("requests_per_second".to_string(), self.metrics.requests_per_second.into());
        stats.insert("average_response_time_ms".to_string(), self.metrics.average_response_time_ms.into());
        stats.insert("error_rate_percent".to_string(), self.metrics.error_rate_percent.into());
        
        stats
    }
    
    pub fn health_check(&self) -> Result<()> {
        if self.metrics.cpu_usage_percent > 95.0 {
            return Err(anyhow::anyhow!("CPU usage too high: {:.1}%", self.metrics.cpu_usage_percent));
        }
        
        if self.metrics.memory_usage_mb > 1000.0 {
            return Err(anyhow::anyhow!("Memory usage too high: {:.1} MB", self.metrics.memory_usage_mb));
        }
        
        if self.metrics.error_rate_percent > 10.0 {
            return Err(anyhow::anyhow!("Error rate too high: {:.1}%", self.metrics.error_rate_percent));
        }
        
        Ok(())
    }
}
