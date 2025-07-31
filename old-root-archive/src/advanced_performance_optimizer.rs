// ================================================================================
// ADVANCED PERFORMANCE OPTIMIZER - ACCIÓN 7
// ================================================================================
// Optimizaciones avanzadas para maximizar performance del sistema principal
// Basado en arbitrage_phase45_clean funcionando perfectamente
// ================================================================================

use std::sync::Arc;
use std::collections::HashMap;
use tokio::time::{Duration, Instant};
use tracing::{info, warn, error, debug};
use anyhow::Result;
use serde::{Deserialize, Serialize};

// use crate::arbitrage_bot_phase45_integrated::UnifiedOpportunity; // ❌ ELIMINADO
use crate::real_price_feeds::RealArbitrageOpportunity;

/// Configuración del optimizador de performance
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PerformanceConfig {
    pub max_concurrent_discoveries: usize,
    pub cache_ttl_seconds: u64,
    pub parallel_api_calls: bool,
    pub aggressive_caching: bool,
    pub memory_optimization: bool,
    pub latency_target_ms: u64,
}

impl Default for PerformanceConfig {
    fn default() -> Self {
        Self {
            max_concurrent_discoveries: 5,
            cache_ttl_seconds: 2,
            parallel_api_calls: true,
            aggressive_caching: true,
            memory_optimization: true,
            latency_target_ms: 100,
        }
    }
}

/// Métricas de performance en tiempo real
#[derive(Debug, Clone)]
pub struct PerformanceMetrics {
    pub discovery_time_ms: u64,
    pub api_response_times: HashMap<String, u64>,
    pub opportunities_per_second: f64,
    pub cache_hit_rate: f64,
    pub memory_usage_mb: f64,
    pub total_cycles: u64,
    pub successful_discoveries: u64,
}

impl Default for PerformanceMetrics {
    fn default() -> Self {
        Self {
            discovery_time_ms: 0,
            api_response_times: HashMap::new(),
            opportunities_per_second: 0.0,
            cache_hit_rate: 0.0,
            memory_usage_mb: 0.0,
            total_cycles: 0,
            successful_discoveries: 0,
        }
    }
}

/// Cache inteligente para oportunidades
#[derive(Debug, Clone)]
struct OpportunityCache {
    data: HashMap<String, (RealArbitrageOpportunity, Instant)>,
    hits: u64,
    misses: u64,
    ttl: Duration,
}

impl OpportunityCache {
    fn new(ttl_seconds: u64) -> Self {
        Self {
            data: HashMap::new(),
            hits: 0,
            misses: 0,
            ttl: Duration::from_secs(ttl_seconds),
        }
    }

    fn get(&mut self, key: &str) -> Option<RealArbitrageOpportunity> {
        if let Some((opportunity, timestamp)) = self.data.get(key) {
            if timestamp.elapsed() < self.ttl {
                self.hits += 1;
                debug!("🎯 Cache HIT para {}", key);
                return Some(opportunity.clone());
            } else {
                self.data.remove(key);
            }
        }
        self.misses += 1;
        debug!("❌ Cache MISS para {}", key);
        None
    }

    fn set(&mut self, key: String, opportunity: RealArbitrageOpportunity) {
        self.data.insert(key, (opportunity, Instant::now()));
        debug!("💾 Cache SET para oportunidad");
    }

    fn hit_rate(&self) -> f64 {
        let total = self.hits + self.misses;
        if total == 0 { 0.0 } else { self.hits as f64 / total as f64 }
    }

    fn cleanup_expired(&mut self) {
        let now = Instant::now();
        self.data.retain(|_, (_, timestamp)| now.duration_since(*timestamp) < self.ttl);
    }
}

/// Optimizador principal de performance
pub struct AdvancedPerformanceOptimizer {
    config: PerformanceConfig,
    metrics: PerformanceMetrics,
    cache: OpportunityCache,
    api_timings: HashMap<String, Vec<u64>>,
}

impl AdvancedPerformanceOptimizer {
    /// Crear nuevo optimizador con configuración
    pub fn new(config: PerformanceConfig) -> Self {
        info!("🚀 Inicializando Advanced Performance Optimizer");
        info!("   • Max concurrent discoveries: {}", config.max_concurrent_discoveries);
        info!("   • Cache TTL: {}s", config.cache_ttl_seconds);
        info!("   • Parallel API calls: {}", config.parallel_api_calls);
        info!("   • Target latency: {}ms", config.latency_target_ms);

        Self {
            cache: OpportunityCache::new(config.cache_ttl_seconds),
            metrics: PerformanceMetrics::default(),
            api_timings: HashMap::new(),
            config,
        }
    }

    /// Crear configuración para máximo performance
    pub fn max_performance_config() -> PerformanceConfig {
        PerformanceConfig {
            max_concurrent_discoveries: 8,
            cache_ttl_seconds: 1, // Cache más agresivo
            parallel_api_calls: true,
            aggressive_caching: true,
            memory_optimization: true,
            latency_target_ms: 50, // Target más agresivo
        }
    }

    /// Optimizar discovery de oportunidades con paralelización
    pub async fn optimize_opportunity_discovery<F, Fut>(
        &mut self,
        discovery_fn: F,
    ) -> Result<Vec<RealArbitrageOpportunity>>
    where
        F: Fn() -> Fut + Send + Sync + Clone + 'static,
        Fut: std::future::Future<Output = Result<Vec<RealArbitrageOpportunity>>> + Send,
    {
        let start_time = Instant::now();
        self.metrics.total_cycles += 1;

        info!("🔍 Optimizando discovery de oportunidades (ciclo #{})", self.metrics.total_cycles);

        // Cleanup de cache expirado
        self.cache.cleanup_expired();

        // Ejecutar discovery con paralelización si está habilitada
        let opportunities = if self.config.parallel_api_calls {
            self.parallel_discovery(discovery_fn).await?
        } else {
            discovery_fn().await?
        };

        // Actualizar métricas
        let discovery_time = start_time.elapsed();
        self.metrics.discovery_time_ms = discovery_time.as_millis() as u64;
        
        if !opportunities.is_empty() {
            self.metrics.successful_discoveries += 1;
            self.metrics.opportunities_per_second = opportunities.len() as f64 / discovery_time.as_secs_f64();
        }

        self.metrics.cache_hit_rate = self.cache.hit_rate();

        // Log de performance
        if self.metrics.discovery_time_ms > self.config.latency_target_ms {
            warn!("⚠️ Discovery time {}ms exceeds target {}ms", 
                  self.metrics.discovery_time_ms, self.config.latency_target_ms);
        } else {
            debug!("✅ Discovery time {}ms within target", self.metrics.discovery_time_ms);
        }

        info!("📊 Performance metrics:");
        info!("   • Discovery time: {}ms", self.metrics.discovery_time_ms);
        info!("   • Opportunities found: {}", opportunities.len());
        info!("   • Cache hit rate: {:.1}%", self.metrics.cache_hit_rate * 100.0);
        info!("   • Ops/sec: {:.2}", self.metrics.opportunities_per_second);

        Ok(opportunities)
    }

    /// Discovery paralelo con concurrencia controlada
    async fn parallel_discovery<F, Fut>(
        &mut self,
        discovery_fn: F,
    ) -> Result<Vec<RealArbitrageOpportunity>>
    where
        F: Fn() -> Fut + Send + Sync + Clone + 'static,
        Fut: std::future::Future<Output = Result<Vec<RealArbitrageOpportunity>>> + Send,
    {
        debug!("🔄 Ejecutando discovery paralelo");

        // Ejecutar múltiples discoveries en paralelo
        let mut handles = Vec::new();
        
        for i in 0..self.config.max_concurrent_discoveries {
            let discovery_fn_clone = discovery_fn.clone();
            let handle = tokio::spawn(async move {
                debug!("🧵 Discovery thread #{} iniciado", i);
                discovery_fn_clone().await
            });
            handles.push(handle);
        }

        // Recopilar resultados
        let mut all_opportunities = Vec::new();
        let mut successful_calls = 0;

        for (i, handle) in handles.into_iter().enumerate() {
            match handle.await {
                Ok(Ok(opportunities)) => {
                    debug!("✅ Discovery thread #{} completado: {} oportunidades", i, opportunities.len());
                    all_opportunities.extend(opportunities);
                    successful_calls += 1;
                }
                Ok(Err(e)) => {
                    warn!("⚠️ Discovery thread #{} falló: {}", i, e);
                }
                Err(e) => {
                    error!("❌ Discovery thread #{} panic: {}", i, e);
                }
            }
        }

        info!("🔄 Parallel discovery completado: {}/{} threads exitosos", 
              successful_calls, self.config.max_concurrent_discoveries);

        // Deduplicar oportunidades
        self.deduplicate_opportunities(&mut all_opportunities);

        Ok(all_opportunities)
    }

    /// Deduplicar oportunidades similares
    fn deduplicate_opportunities(&self, opportunities: &mut Vec<RealArbitrageOpportunity>) {
        let original_count = opportunities.len();
        
        // Ordenar por profit descendente y deduplicar por token/route similar
        opportunities.sort_by(|a, b| b.estimated_profit_sol.partial_cmp(&a.estimated_profit_sol).unwrap());
        
        let mut seen_routes = std::collections::HashSet::new();
        opportunities.retain(|op| {
            let route_key = format!("{}_{}", 
                op.token_mint, 
                op.token_symbol
            );
            
            if seen_routes.contains(&route_key) {
                false
            } else {
                seen_routes.insert(route_key);
                true
            }
        });

        if opportunities.len() != original_count {
            debug!("🔄 Deduplicación: {} → {} oportunidades", original_count, opportunities.len());
        }
    }

    /// Registrar timing de API call
    pub fn record_api_timing(&mut self, api_name: String, duration_ms: u64) {
        self.api_timings.entry(api_name.clone()).or_insert_with(Vec::new).push(duration_ms);
        self.metrics.api_response_times.insert(api_name.clone(), duration_ms);
        
        // Mantener solo los últimos 100 timings por API
        if let Some(timings) = self.api_timings.get_mut(&api_name) {
            if timings.len() > 100 {
                timings.drain(0..timings.len()-100);
            }
        }
    }

    /// Obtener métricas actuales
    pub fn get_metrics(&self) -> &PerformanceMetrics {
        &self.metrics
    }

    /// Obtener promedio de timing para API
    pub fn get_api_average_timing(&self, api_name: &str) -> Option<f64> {
        self.api_timings.get(api_name).map(|timings| {
            if timings.is_empty() {
                0.0
            } else {
                timings.iter().sum::<u64>() as f64 / timings.len() as f64
            }
        })
    }

    /// Generar reporte de performance
    pub fn generate_performance_report(&self) -> String {
        let mut report = String::new();
        
        report.push_str("📊 REPORTE DE PERFORMANCE\n");
        report.push_str("========================\n");
        report.push_str(&format!("• Total cycles: {}\n", self.metrics.total_cycles));
        report.push_str(&format!("• Successful discoveries: {}\n", self.metrics.successful_discoveries));
        report.push_str(&format!("• Success rate: {:.1}%\n", 
            if self.metrics.total_cycles > 0 {
                (self.metrics.successful_discoveries as f64 / self.metrics.total_cycles as f64) * 100.0
            } else { 0.0 }
        ));
        report.push_str(&format!("• Avg discovery time: {}ms\n", self.metrics.discovery_time_ms));
        report.push_str(&format!("• Cache hit rate: {:.1}%\n", self.metrics.cache_hit_rate * 100.0));
        report.push_str(&format!("• Opportunities/sec: {:.2}\n", self.metrics.opportunities_per_second));
        
        report.push_str("\n📡 API TIMINGS:\n");
        for (api, timing) in &self.metrics.api_response_times {
            if let Some(avg) = self.get_api_average_timing(api) {
                report.push_str(&format!("• {}: {}ms (avg: {:.1}ms)\n", api, timing, avg));
            }
        }
        
        report
    }

    /// Optimizar configuración basada en métricas
    pub fn auto_optimize_config(&mut self) {
        info!("🔧 Auto-optimizando configuración basada en métricas");
        
        // Si latency promedio es alta, reducir concurrencia
        if self.metrics.discovery_time_ms > self.config.latency_target_ms * 2 {
            if self.config.max_concurrent_discoveries > 2 {
                self.config.max_concurrent_discoveries -= 1;
                info!("⬇️ Reduciendo concurrencia a {}", self.config.max_concurrent_discoveries);
            }
        }
        
        // Si cache hit rate es bajo, aumentar TTL
        if self.metrics.cache_hit_rate < 0.3 && self.config.cache_ttl_seconds < 5 {
            self.config.cache_ttl_seconds += 1;
            info!("⬆️ Aumentando cache TTL a {}s", self.config.cache_ttl_seconds);
            self.cache.ttl = Duration::from_secs(self.config.cache_ttl_seconds);
        }
        
        // Si performance es excelente, aumentar agresividad
        if self.metrics.discovery_time_ms < self.config.latency_target_ms / 2 
            && self.metrics.cache_hit_rate > 0.7 {
            if self.config.max_concurrent_discoveries < 10 {
                self.config.max_concurrent_discoveries += 1;
                info!("⬆️ Aumentando concurrencia a {}", self.config.max_concurrent_discoveries);
            }
        }
    }
}

/// Helper para medición de timing
pub struct TimingHelper {
    start: Instant,
    name: String,
}

impl TimingHelper {
    pub fn new(name: String) -> Self {
        Self {
            start: Instant::now(),
            name,
        }
    }

    pub fn finish(self) -> (String, u64) {
        let duration_ms = self.start.elapsed().as_millis() as u64;
        debug!("⏱️ {}: {}ms", self.name, duration_ms);
        (self.name, duration_ms)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_performance_optimizer() {
        let config = PerformanceConfig::default();
        let mut optimizer = AdvancedPerformanceOptimizer::new(config);
        
        // Test discovery function mock
        let discovery_fn = || async { 
            Ok(vec![]) // Mock empty opportunities
        };
        
        let result = optimizer.optimize_opportunity_discovery(discovery_fn).await;
        assert!(result.is_ok());
    }

    #[test]
    fn test_cache_functionality() {
        let mut cache = OpportunityCache::new(2);
        assert_eq!(cache.hit_rate(), 0.0);
        
        // Test miss
        assert!(cache.get("test").is_none());
        assert_eq!(cache.hit_rate(), 0.0);
    }
}
