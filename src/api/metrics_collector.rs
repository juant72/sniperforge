//! Metrics Collector Module
//! 
//! This module provides comprehensive metrics collection and aggregation for the SniperForge ecosystem.
//! It collects performance metrics, trading metrics, and system metrics for analysis and monitoring.

use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;
use tokio::time::{Duration, Instant};
use uuid::Uuid;

use crate::api::bot_interface::{BotType, TradeMetrics, PerformanceMetrics};

/// ✅ ENRIQUECIMIENTO: Comprehensive system metrics summary
#[derive(Debug, Clone)]
pub struct SystemMetricsSummary {
    pub total_bots: u64,
    pub active_bots: u64,
    pub uptime_seconds: u64,
    pub memory_usage_mb: f64,
    pub cpu_usage_percent: f64,
    pub total_trades: u64,
    pub total_volume_usd: f64,
    pub total_profit_usd: f64,
    pub last_updated: Instant,
}

impl Default for SystemMetricsSummary {
    fn default() -> Self {
        Self {
            total_bots: 0,
            active_bots: 0,
            uptime_seconds: 0,
            memory_usage_mb: 0.0,
            cpu_usage_percent: 0.0,
            total_trades: 0,
            total_volume_usd: 0.0,
            total_profit_usd: 0.0,
            last_updated: Instant::now(),
        }
    }
}

/// ✅ ENRIQUECIMIENTO: Enhanced bot metrics collection
#[derive(Debug, Clone)]
pub struct BotMetricsCollection {
    pub bot_id: Uuid,
    pub bot_type: BotType,
    pub creation_time: Instant,
    pub last_update: Instant,
    pub trade_metrics: TradeMetrics,
    pub performance_metrics: PerformanceMetrics,
    pub custom_metrics: HashMap<String, f64>,
}

/// ✅ ENRIQUECIMIENTO: Metric point for time series tracking
#[derive(Debug, Clone)]
pub struct MetricPoint {
    pub timestamp: Instant,
    pub value: f64,
}

/// Metrics collection errors
#[derive(Debug, thiserror::Error)]
pub enum MetricsError {
    #[error("Metrics not found for bot: {0}")]
    BotNotFound(Uuid),
    
    #[error("Invalid metric value: {0}")]
    InvalidValue(String),
    
    #[error("Aggregation failed: {0}")]
    AggregationFailed(String),
    
    #[error("Storage error: {0}")]
    StorageError(String),
}

/// Aggregated metric statistics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MetricStats {
    pub count: u64,
    pub sum: f64,
    pub min: f64,
    pub max: f64,
    pub average: f64,
    pub standard_deviation: f64,
    pub percentiles: HashMap<u8, f64>, // P50, P95, P99
}

/// Trading performance metrics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TradingMetrics {
    pub total_trades: u64,
    pub successful_trades: u64,
    pub failed_trades: u64,
    pub total_volume: f64,
    pub total_profit: f64,
    pub total_loss: f64,
    pub win_rate: f64,
    pub profit_factor: f64,
    pub sharpe_ratio: f64,
    pub max_drawdown: f64,
    pub average_trade_duration_ms: f64,
    pub best_trade: f64,
    pub worst_trade: f64,
}

impl Default for TradingMetrics {
    fn default() -> Self {
        Self {
            total_trades: 0,
            successful_trades: 0,
            failed_trades: 0,
            total_volume: 0.0,
            total_profit: 0.0,
            total_loss: 0.0,
            win_rate: 0.0,
            profit_factor: 0.0,
            sharpe_ratio: 0.0,
            max_drawdown: 0.0,
            average_trade_duration_ms: 0.0,
            best_trade: 0.0,
            worst_trade: 0.0,
        }
    }
}

/// System performance metrics
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct SystemMetrics {
    pub cpu_usage: f64,
    pub memory_usage: f64,
    pub disk_usage: f64,
    pub network_latency: f64,
    pub request_rate: f64,
    pub error_rate: f64,
}

/// Metrics aggregation configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MetricsConfig {
    pub collection_interval_seconds: u64,
    pub retention_hours: u32,
    pub max_points_per_metric: usize,
    pub aggregation_windows: Vec<Duration>,
    pub enable_percentiles: bool,
    pub enable_trading_metrics: bool,
    pub custom_metrics_enabled: bool,
}

impl Default for MetricsConfig {
    fn default() -> Self {
        Self {
            collection_interval_seconds: 10,
            retention_hours: 168, // 1 week
            max_points_per_metric: 10000,
            aggregation_windows: vec![
                Duration::from_secs(60),    // 1 minute
                Duration::from_secs(300),   // 5 minutes
                Duration::from_secs(3600),  // 1 hour
                Duration::from_secs(86400), // 1 day
            ],
            enable_percentiles: true,
            enable_trading_metrics: true,
            custom_metrics_enabled: true,
        }
    }
}

/// Metrics collector service
pub struct MetricsCollector {
    config: MetricsConfig,
    bot_metrics: Arc<RwLock<HashMap<Uuid, BotMetricsCollection>>>,
    system_metrics: Arc<RwLock<SystemMetrics>>,
    metric_series: Arc<RwLock<HashMap<String, Vec<MetricPoint>>>>,
    aggregated_metrics: Arc<RwLock<HashMap<String, HashMap<Duration, MetricStats>>>>,
    start_time: Instant,
}

impl MetricsCollector {
    /// Create a new metrics collector
    pub fn new(config: MetricsConfig) -> Self {
        Self {
            config,
            bot_metrics: Arc::new(RwLock::new(HashMap::new())),
            system_metrics: Arc::new(RwLock::new(SystemMetrics::default())),
            metric_series: Arc::new(RwLock::new(HashMap::new())),
            aggregated_metrics: Arc::new(RwLock::new(HashMap::new())),
            start_time: Instant::now(),
        }
    }

    /// Start the metrics collection service
    pub async fn start(&self) -> Result<(), MetricsError> {
        let _collection_interval = Duration::from_secs(self.config.collection_interval_seconds);
        
        // ✅ ENRIQUECIMIENTO: Placeholder for future periodic collection
        tracing::info!("🚀 MetricsCollector started successfully");
        
        Ok(())
    }
    
    /// ✅ ENRIQUECIMIENTO: Record bot creation event with enhanced metadata
    pub async fn record_bot_creation(&self, bot_id: Uuid, bot_type: &BotType) -> Result<(), MetricsError> {
        tracing::info!("📊 Recording bot creation event: {} of type {:?}", bot_id, bot_type);
        
        let mut metrics = self.bot_metrics.write().await;
        metrics.insert(bot_id, BotMetricsCollection {
            bot_id,
            bot_type: bot_type.clone(),
            creation_time: Instant::now(),
            last_update: Instant::now(),
            trade_metrics: TradeMetrics::default(),
            performance_metrics: PerformanceMetrics::default(),
            custom_metrics: HashMap::new(),
        });
        
        // Record system-level event
        self.record_metric_point("bots_created_total", 1.0).await?;
        self.record_metric_point(&format!("bots_created_{:?}", bot_type).to_lowercase(), 1.0).await?;
        
        Ok(())
    }
    
    /// ✅ ENRIQUECIMIENTO: Record bot start event with lifecycle tracking
    pub async fn record_bot_start(&self, bot_id: Uuid) -> Result<(), MetricsError> {
        tracing::info!("🚀 Recording bot start event: {}", bot_id);
        
        if let Some(bot_metrics) = self.bot_metrics.write().await.get_mut(&bot_id) {
            bot_metrics.last_update = Instant::now();
        }
        
        self.record_metric_point("bots_started_total", 1.0).await?;
        self.record_metric_point(&format!("bot_{}_starts", bot_id), 1.0).await?;
        
        Ok(())
    }
    
    /// ✅ ENRIQUECIMIENTO: Record bot stop event with lifecycle tracking
    pub async fn record_bot_stop(&self, bot_id: Uuid) -> Result<(), MetricsError> {
        tracing::info!("🛑 Recording bot stop event: {}", bot_id);
        
        if let Some(bot_metrics) = self.bot_metrics.write().await.get_mut(&bot_id) {
            bot_metrics.last_update = Instant::now();
        }
        
        self.record_metric_point("bots_stopped_total", 1.0).await?;
        self.record_metric_point(&format!("bot_{}_stops", bot_id), 1.0).await?;
        
        Ok(())
    }
    
    /// ✅ ENRIQUECIMIENTO: Get comprehensive system summary with enhanced metrics
    pub async fn get_system_summary(&self) -> Result<SystemMetricsSummary, MetricsError> {
        tracing::info!("📈 Generating comprehensive system metrics summary");
        
        let bot_metrics = self.bot_metrics.read().await;
        let system_metrics = self.system_metrics.read().await;
        
        let total_bots = bot_metrics.len();
        let uptime_seconds = self.start_time.elapsed().as_secs();
        
        // ✅ ENRIQUECIMIENTO: Utilizar aggregated_metrics para análisis avanzado
        self.update_aggregated_metrics().await;
        let aggregated = self.aggregated_metrics.read().await;
        
        // Calculate aggregated performance metrics
        let total_trades: u64 = bot_metrics.values()
            .map(|m| m.trade_metrics.total_trades)
            .sum();
            
        let total_volume: f64 = bot_metrics.values()
            .map(|m| m.trade_metrics.total_volume_usd)
            .sum();
            
        let total_profit: f64 = bot_metrics.values()
            .map(|m| m.trade_metrics.total_pnl_usd)
            .sum();

        // ✅ ENRIQUECIMIENTO: Añadir métricas avanzadas del aggregated_metrics
        let avg_profit_per_bot = if total_bots > 0 { total_profit / total_bots as f64 } else { 0.0 };
        tracing::debug!("📊 Calculated average profit per bot: ${:.2}", avg_profit_per_bot);
        
        // Log metrics aggregation summary
        tracing::debug!("🔢 Active aggregated metrics: {}", aggregated.len());
        
        Ok(SystemMetricsSummary {
            total_bots: total_bots as u64,
            active_bots: bot_metrics.len() as u64, // TODO: Filter by active status
            uptime_seconds,
            memory_usage_mb: system_metrics.memory_usage,
            cpu_usage_percent: system_metrics.cpu_usage,
            total_trades,
            total_volume_usd: total_volume,
            total_profit_usd: total_profit,
            last_updated: Instant::now(),
        })
    }
    
    /// ✅ ENRIQUECIMIENTO: Actualizar métricas agregadas para análisis avanzado
    async fn update_aggregated_metrics(&self) {
        let metric_series = self.metric_series.read().await;
        let mut aggregated = self.aggregated_metrics.write().await;
        
        for (metric_name, points) in metric_series.iter() {
            if points.is_empty() {
                continue;
            }
            
            let metric_aggregations = aggregated.entry(metric_name.clone()).or_insert_with(HashMap::new);
            
            // ✅ ENRIQUECIMIENTO: Procesar ventanas de agregación definidas en config
            for &window in &self.config.aggregation_windows {
                let cutoff_time = Instant::now() - window;
                let window_points: Vec<f64> = points.iter()
                    .filter(|p| p.timestamp >= cutoff_time)
                    .map(|p| p.value)
                    .collect();
                
                if !window_points.is_empty() {
                    let stats = self.calculate_metric_stats(&window_points);
                    metric_aggregations.insert(window, stats);
                    
                    tracing::trace!("📈 Updated aggregation for '{}' (window: {:?}): {} points", 
                                   metric_name, window, window_points.len());
                }
            }
        }
    }
    
    /// ✅ ENRIQUECIMIENTO: Calcular estadísticas métricas avanzadas
    fn calculate_metric_stats(&self, values: &[f64]) -> MetricStats {
        let count = values.len() as u64;
        let sum = values.iter().sum::<f64>();
        let min = values.iter().fold(f64::INFINITY, |a, &b| a.min(b));
        let max = values.iter().fold(f64::NEG_INFINITY, |a, &b| a.max(b));
        let average = sum / count as f64;
        
        // Calculate standard deviation
        let variance = values.iter()
            .map(|v| (v - average).powi(2))
            .sum::<f64>() / count as f64;
        let standard_deviation = variance.sqrt();
        
        // ✅ ENRIQUECIMIENTO: Calcular percentiles si está habilitado
        let mut percentiles = HashMap::new();
        if self.config.enable_percentiles && count > 0 {
            let mut sorted_values = values.to_vec();
            sorted_values.sort_by(|a, b| a.partial_cmp(b).unwrap());
            
            percentiles.insert(50, self.calculate_percentile(&sorted_values, 50.0));
            percentiles.insert(95, self.calculate_percentile(&sorted_values, 95.0));
            percentiles.insert(99, self.calculate_percentile(&sorted_values, 99.0));
            
            tracing::trace!("📊 Calculated percentiles: P50={:.2}, P95={:.2}, P99={:.2}", 
                           percentiles[&50], percentiles[&95], percentiles[&99]);
        }
        
        MetricStats {
            count,
            sum,
            min,
            max,
            average,
            standard_deviation,
            percentiles,
        }
    }
    
    /// ✅ ENRIQUECIMIENTO: Calcular percentil específico
    fn calculate_percentile(&self, sorted_values: &[f64], percentile: f64) -> f64 {
        let index = (percentile / 100.0) * (sorted_values.len() - 1) as f64;
        let lower_index = index.floor() as usize;
        let upper_index = index.ceil() as usize;
        
        if lower_index == upper_index {
            sorted_values[lower_index]
        } else {
            let fraction = index - lower_index as f64;
            sorted_values[lower_index] * (1.0 - fraction) + sorted_values[upper_index] * fraction
        }
    }
    
    /// ✅ ENRIQUECIMIENTO: Record individual metric points for time series
    async fn record_metric_point(&self, metric_name: &str, value: f64) -> Result<(), MetricsError> {
        let point = MetricPoint {
            timestamp: Instant::now(),
            value,
        };
        
        let mut series = self.metric_series.write().await;
        series.entry(metric_name.to_string())
            .or_insert_with(Vec::new)
            .push(point);
        
        // Maintain series length limits
        if let Some(series_data) = series.get_mut(metric_name) {
            if series_data.len() > self.config.max_points_per_metric {
                series_data.remove(0);
            }
        }
        
        Ok(())
    }
}
