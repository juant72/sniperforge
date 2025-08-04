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

use crate::api::bot_interface::{BotMetrics, BotType};

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

/// Time series metric point
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MetricPoint {
    pub timestamp: chrono::DateTime<chrono::Utc>,
    pub value: f64,
    pub labels: HashMap<String, String>,
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

/// System performance metrics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SystemMetrics {
    pub cpu_usage: MetricStats,
    pub memory_usage: MetricStats,
    pub disk_usage: MetricStats,
    pub network_latency: MetricStats,
    pub request_rate: MetricStats,
    pub error_rate: MetricStats,
}

/// Bot-specific metrics collection
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BotMetricsCollection {
    pub bot_id: Uuid,
    pub bot_type: BotType,
    pub performance: BotMetrics,
    pub trading: Option<TradingMetrics>,
    pub custom_metrics: HashMap<String, Vec<MetricPoint>>,
    pub last_updated: chrono::DateTime<chrono::Utc>,
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
        let collection_interval = Duration::from_secs(self.config.collection_interval_seconds);
        
        // Start periodic metrics collection
        let _bot_metrics = Arc::clone(&self.bot_metrics);
        let system_metrics = Arc::clone(&self.system_metrics);
        let metric_series = Arc::clone(&self.metric_series);
        let aggregated_metrics = Arc::clone(&self.aggregated_metrics);
        let config = self.config.clone();
        
        tokio::spawn(async move {
            let mut interval = tokio::time::interval(collection_interval);
            
            loop {
                interval.tick().await;
                
                // Collect system metrics
                if let Ok(sys_metrics) = Self::collect_system_metrics().await {
                    *system_metrics.write().await = sys_metrics;
                }
                
                // Update aggregated metrics
                Self::update_aggregations(&metric_series, &aggregated_metrics, &config).await;
                
                // Clean up old metrics
                Self::cleanup_old_metrics(&metric_series, &config).await;
            }
        });
        
        Ok(())
    }

    /// Register a bot for metrics collection
    pub async fn register_bot(&self, bot_id: Uuid, bot_type: BotType) -> Result<(), MetricsError> {
        let collection = BotMetricsCollection {
            bot_id,
            bot_type,
            performance: BotMetrics::default(),
            trading: if self.config.enable_trading_metrics {
                Some(TradingMetrics::default())
            } else {
                None
            },
            custom_metrics: HashMap::new(),
            last_updated: chrono::Utc::now(),
        };
        
        self.bot_metrics.write().await.insert(bot_id, collection);
        
        Ok(())
    }

    /// Unregister a bot from metrics collection
    pub async fn unregister_bot(&self, bot_id: Uuid) -> Result<(), MetricsError> {
        self.bot_metrics.write().await.remove(&bot_id);
        
        // Remove bot-specific metric series
        let mut series = self.metric_series.write().await;
        series.retain(|key, _| !key.starts_with(&format!("bot.{}", bot_id)));
        
        Ok(())
    }

    /// Update bot performance metrics
    pub async fn update_bot_metrics(
        &self,
        bot_id: Uuid,
        metrics: BotMetrics,
    ) -> Result<(), MetricsError> {
        let mut bot_metrics = self.bot_metrics.write().await;
        
        if let Some(collection) = bot_metrics.get_mut(&bot_id) {
            collection.performance = metrics.clone();
            collection.last_updated = chrono::Utc::now();
            
            // Add metrics to time series
            self.add_metric_point(
                format!("bot.{}.latency", bot_id),
                metrics.performance.avg_response_time_ms,
                HashMap::new(),
            ).await;
            
            self.add_metric_point(
                format!("bot.{}.throughput", bot_id),
                metrics.performance.throughput_per_second,
                HashMap::new(),
            ).await;
            
            self.add_metric_point(
                format!("bot.{}.uptime", bot_id),
                (metrics.operational.uptime_seconds as f64 / (24.0 * 3600.0)) * 100.0, // Convert to percentage
                HashMap::new(),
            ).await;
            
            self.add_metric_point(
                format!("bot.{}.errors", bot_id),
                metrics.operational.error_count as f64,
                HashMap::new(),
            ).await;
        } else {
            return Err(MetricsError::BotNotFound(bot_id));
        }
        
        Ok(())
    }

    /// Update trading metrics for a bot
    pub async fn update_trading_metrics(
        &self,
        bot_id: Uuid,
        trading_metrics: TradingMetrics,
    ) -> Result<(), MetricsError> {
        if !self.config.enable_trading_metrics {
            return Ok(());
        }
        
        let mut bot_metrics = self.bot_metrics.write().await;
        
        if let Some(collection) = bot_metrics.get_mut(&bot_id) {
            collection.trading = Some(trading_metrics.clone());
            collection.last_updated = chrono::Utc::now();
            
            // Add trading metrics to time series
            self.add_metric_point(
                format!("bot.{}.total_trades", bot_id),
                trading_metrics.total_trades as f64,
                HashMap::new(),
            ).await;
            
            self.add_metric_point(
                format!("bot.{}.win_rate", bot_id),
                trading_metrics.win_rate,
                HashMap::new(),
            ).await;
            
            self.add_metric_point(
                format!("bot.{}.profit", bot_id),
                trading_metrics.total_profit,
                HashMap::new(),
            ).await;
            
            self.add_metric_point(
                format!("bot.{}.profit_factor", bot_id),
                trading_metrics.profit_factor,
                HashMap::new(),
            ).await;
        } else {
            return Err(MetricsError::BotNotFound(bot_id));
        }
        
        Ok(())
    }

    /// Add a custom metric point
    pub async fn add_custom_metric(
        &self,
        bot_id: Uuid,
        metric_name: String,
        value: f64,
        labels: HashMap<String, String>,
    ) -> Result<(), MetricsError> {
        if !self.config.custom_metrics_enabled {
            return Ok(());
        }
        
        let mut bot_metrics = self.bot_metrics.write().await;
        
        if let Some(collection) = bot_metrics.get_mut(&bot_id) {
            let metric_point = MetricPoint {
                timestamp: chrono::Utc::now(),
                value,
                labels: labels.clone(),
            };
            
            collection.custom_metrics
                .entry(metric_name.clone())
                .or_insert_with(Vec::new)
                .push(metric_point);
            
            collection.last_updated = chrono::Utc::now();
            
            // Add to global time series
            self.add_metric_point(
                format!("bot.{}.custom.{}", bot_id, metric_name),
                value,
                labels,
            ).await;
        } else {
            return Err(MetricsError::BotNotFound(bot_id));
        }
        
        Ok(())
    }

    /// Get bot metrics collection
    pub async fn get_bot_metrics(&self, bot_id: Uuid) -> Option<BotMetricsCollection> {
        self.bot_metrics.read().await.get(&bot_id).cloned()
    }

    /// Get all bot metrics
    pub async fn get_all_bot_metrics(&self) -> HashMap<Uuid, BotMetricsCollection> {
        self.bot_metrics.read().await.clone()
    }

    /// Get system metrics
    pub async fn get_system_metrics(&self) -> SystemMetrics {
        self.system_metrics.read().await.clone()
    }

    /// Get metric time series
    pub async fn get_metric_series(
        &self,
        metric_name: &str,
        from: Option<chrono::DateTime<chrono::Utc>>,
        to: Option<chrono::DateTime<chrono::Utc>>,
    ) -> Vec<MetricPoint> {
        let series = self.metric_series.read().await;
        
        if let Some(points) = series.get(metric_name) {
            points.iter()
                .filter(|p| {
                    let after_from = from.map_or(true, |f| p.timestamp >= f);
                    let before_to = to.map_or(true, |t| p.timestamp <= t);
                    after_from && before_to
                })
                .cloned()
                .collect()
        } else {
            Vec::new()
        }
    }

    /// Get aggregated metrics
    pub async fn get_aggregated_metrics(
        &self,
        metric_name: &str,
        window: Duration,
    ) -> Option<MetricStats> {
        let aggregated = self.aggregated_metrics.read().await;
        aggregated.get(metric_name)?.get(&window).cloned()
    }

    /// Get metrics summary for all bots
    pub async fn get_metrics_summary(&self) -> MetricsSummary {
        let bot_metrics = self.bot_metrics.read().await;
        let system_metrics = self.system_metrics.read().await;
        
        let total_bots = bot_metrics.len();
        let active_bots = bot_metrics.values()
            .filter(|m| chrono::Utc::now() - m.last_updated < chrono::Duration::minutes(5))
            .count();
        
        let total_trades = bot_metrics.values()
            .filter_map(|m| m.trading.as_ref())
            .map(|t| t.total_trades)
            .sum();
        
        let total_profit = bot_metrics.values()
            .filter_map(|m| m.trading.as_ref())
            .map(|t| t.total_profit)
            .sum();
        
        let average_latency = bot_metrics.values()
            .map(|m| m.performance.performance.avg_response_time_ms)
            .sum::<f64>() / total_bots.max(1) as f64;
        
        let total_errors = bot_metrics.values()
            .map(|m| m.performance.operational.error_count)
            .sum();
        
        MetricsSummary {
            timestamp: chrono::Utc::now(),
            system_metrics: system_metrics.clone(),
            total_bots,
            active_bots,
            total_trades,
            total_profit,
            average_latency,
            total_errors,
            uptime_seconds: self.start_time.elapsed().as_secs(),
        }
    }

    /// Add a metric point to time series
    async fn add_metric_point(
        &self,
        metric_name: String,
        value: f64,
        labels: HashMap<String, String>,
    ) {
        let mut series = self.metric_series.write().await;
        
        let metric_point = MetricPoint {
            timestamp: chrono::Utc::now(),
            value,
            labels,
        };
        
        let points = series.entry(metric_name).or_insert_with(Vec::new);
        points.push(metric_point);
        
        // Limit the number of points per metric
        if points.len() > self.config.max_points_per_metric {
            points.remove(0);
        }
    }

    /// Collect system metrics
    async fn collect_system_metrics() -> Result<SystemMetrics, MetricsError> {
        // TODO: Implement actual system metrics collection
        // For now, return mock data
        Ok(SystemMetrics::default())
    }

    /// Update metric aggregations
    async fn update_aggregations(
        metric_series: &Arc<RwLock<HashMap<String, Vec<MetricPoint>>>>,
        aggregated_metrics: &Arc<RwLock<HashMap<String, HashMap<Duration, MetricStats>>>>,
        config: &MetricsConfig,
    ) {
        let series = metric_series.read().await;
        let mut aggregated = aggregated_metrics.write().await;
        
        for (metric_name, points) in series.iter() {
            let metric_aggregations = aggregated.entry(metric_name.clone()).or_insert_with(HashMap::new);
            
            for &window in &config.aggregation_windows {
                let cutoff_time = chrono::Utc::now() - chrono::Duration::from_std(window).unwrap();
                let window_points: Vec<f64> = points.iter()
                    .filter(|p| p.timestamp >= cutoff_time)
                    .map(|p| p.value)
                    .collect();
                
                if !window_points.is_empty() {
                    let stats = Self::calculate_stats(&window_points, config.enable_percentiles);
                    metric_aggregations.insert(window, stats);
                }
            }
        }
    }

    /// Calculate statistics for a set of values
    fn calculate_stats(values: &[f64], enable_percentiles: bool) -> MetricStats {
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
        
        // Calculate percentiles if enabled
        let mut percentiles = HashMap::new();
        if enable_percentiles && count > 0 {
            let mut sorted_values = values.to_vec();
            sorted_values.sort_by(|a, b| a.partial_cmp(b).unwrap());
            
            percentiles.insert(50, Self::percentile(&sorted_values, 50.0));
            percentiles.insert(95, Self::percentile(&sorted_values, 95.0));
            percentiles.insert(99, Self::percentile(&sorted_values, 99.0));
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

    /// Calculate percentile value
    fn percentile(sorted_values: &[f64], percentile: f64) -> f64 {
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

    /// Clean up old metrics
    async fn cleanup_old_metrics(
        metric_series: &Arc<RwLock<HashMap<String, Vec<MetricPoint>>>>,
        config: &MetricsConfig,
    ) {
        let mut series = metric_series.write().await;
        let retention_duration = chrono::Duration::hours(config.retention_hours as i64);
        let cutoff_time = chrono::Utc::now() - retention_duration;
        
        for points in series.values_mut() {
            points.retain(|p| p.timestamp > cutoff_time);
        }
    }
}

/// Metrics summary structure
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MetricsSummary {
    pub timestamp: chrono::DateTime<chrono::Utc>,
    pub system_metrics: SystemMetrics,
    pub total_bots: usize,
    pub active_bots: usize,
    pub total_trades: u64,
    pub total_profit: f64,
    pub average_latency: f64,
    pub total_errors: u32,
    pub uptime_seconds: u64,
}

impl Default for SystemMetrics {
    fn default() -> Self {
        Self {
            cpu_usage: MetricStats::default(),
            memory_usage: MetricStats::default(),
            disk_usage: MetricStats::default(),
            network_latency: MetricStats::default(),
            request_rate: MetricStats::default(),
            error_rate: MetricStats::default(),
        }
    }
}

impl Default for MetricStats {
    fn default() -> Self {
        Self {
            count: 0,
            sum: 0.0,
            min: 0.0,
            max: 0.0,
            average: 0.0,
            standard_deviation: 0.0,
            percentiles: HashMap::new(),
        }
    }
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

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_metrics_collector_creation() {
        let config = MetricsConfig::default();
        let collector = MetricsCollector::new(config);
        
        let bot_id = Uuid::new_v4();
        collector.register_bot(bot_id, BotType::EnhancedArbitrage).await.unwrap();
        
        let metrics = collector.get_bot_metrics(bot_id).await;
        assert!(metrics.is_some());
        assert_eq!(metrics.unwrap().bot_id, bot_id);
    }

    #[tokio::test]
    async fn test_metrics_update() {
        let config = MetricsConfig::default();
        let collector = MetricsCollector::new(config);
        
        let bot_id = Uuid::new_v4();
        collector.register_bot(bot_id, BotType::EnhancedArbitrage).await.unwrap();
        
        let bot_metrics = BotMetrics {
            operational: crate::api::bot_interface::OperationalMetrics {
                uptime_seconds: 3600,
                error_count: 5,
                restart_count: 0,
                last_restart: None,
                memory_usage_mb: 256,
                cpu_usage_percent: 25.0,
            },
            trading: crate::api::bot_interface::TradingMetrics {
                total_trades: 100,
                successful_trades: 95,
                failed_trades: 5,
                total_volume: 10000.0,
                total_profit: 250.0,
                average_profit_per_trade: 2.5,
                max_drawdown: 5.0,
                sharpe_ratio: 1.5,
                win_rate: 0.95,
            },
            performance: crate::api::bot_interface::PerformanceMetrics {
                cpu_usage_percent: 25.0,
                memory_usage_mb: 256,
                network_io: crate::api::bot_interface::NetworkIOMetrics::default(),
                api_calls: crate::api::bot_interface::ApiCallMetrics::default(),
                avg_response_time_ms: 50.0,
                throughput_per_second: 100.0,
            },
            custom: serde_json::Value::Null,
            timestamp: chrono::Utc::now(),
        };
        
        collector.update_bot_metrics(bot_id, bot_metrics).await.unwrap();
        
        let collection = collector.get_bot_metrics(bot_id).await.unwrap();
        assert_eq!(collection.performance.performance.avg_response_time_ms, 50.0);
    }

    #[test]
    fn test_stats_calculation() {
        let values = vec![1.0, 2.0, 3.0, 4.0, 5.0];
        let stats = MetricsCollector::calculate_stats(&values, true);
        
        assert_eq!(stats.count, 5);
        assert_eq!(stats.average, 3.0);
        assert_eq!(stats.min, 1.0);
        assert_eq!(stats.max, 5.0);
        assert!(stats.percentiles.contains_key(&50));
    }
}
