//! Health Monitoring Module
//! 
//! This module provides comprehensive health monitoring capabilities for the SniperForge ecosystem.
//! It monitors bot health, system resources, and provides alerting mechanisms.

use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;
use tokio::time::{Duration, Instant};
use uuid::Uuid;

use crate::api::bot_interface::{HealthStatus, BotMetrics};

/// Health monitoring errors
#[derive(Debug, thiserror::Error)]
pub enum HealthError {
    #[error("Health check failed: {0}")]
    CheckFailed(String),
    
    #[error("Bot not found: {0}")]
    BotNotFound(Uuid),
    
    #[error("System resource error: {0}")]
    SystemResourceError(String),
    
    #[error("Alert delivery failed: {0}")]
    AlertDeliveryFailed(String),
}

/// System health metrics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SystemHealthMetrics {
    pub timestamp: chrono::DateTime<chrono::Utc>,
    pub cpu_usage_percent: f64,
    pub memory_usage_percent: f64,
    pub disk_usage_percent: f64,
    pub network_latency_ms: f64,
    pub active_connections: u32,
    pub uptime_seconds: u64,
    pub version: String,
}

/// Bot health record
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BotHealthRecord {
    pub bot_id: Uuid,
    pub status: HealthStatus,
    pub last_heartbeat: chrono::DateTime<chrono::Utc>,
    pub metrics: BotMetrics,
    pub error_count: u32,
    pub warnings: Vec<String>,
    pub performance_score: f64,
}

/// Health check configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HealthCheckConfig {
    pub check_interval_seconds: u64,
    pub timeout_seconds: u64,
    pub failure_threshold: u32,
    pub warning_threshold: u32,
    pub recovery_threshold: u32,
    pub alert_enabled: bool,
    pub metrics_retention_hours: u32,
}

impl Default for HealthCheckConfig {
    fn default() -> Self {
        Self {
            check_interval_seconds: 30,
            timeout_seconds: 10,
            failure_threshold: 3,
            warning_threshold: 2,
            recovery_threshold: 2,
            alert_enabled: true,
            metrics_retention_hours: 24,
        }
    }
}

/// Alert severity levels
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum AlertSeverity {
    Critical,
    Warning,
    Info,
}

/// Health alert
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HealthAlert {
    pub id: Uuid,
    pub severity: AlertSeverity,
    pub title: String,
    pub message: String,
    pub bot_id: Option<Uuid>,
    pub timestamp: chrono::DateTime<chrono::Utc>,
    pub acknowledged: bool,
    pub resolved: bool,
}

/// Health monitoring service
pub struct HealthMonitor {
    config: HealthCheckConfig,
    bot_health: Arc<RwLock<HashMap<Uuid, BotHealthRecord>>>,
    system_metrics: Arc<RwLock<Vec<SystemHealthMetrics>>>,
    alerts: Arc<RwLock<Vec<HealthAlert>>>,
    check_history: Arc<RwLock<HashMap<Uuid, Vec<HealthCheckResult>>>>,
    start_time: Instant,
}

/// Health check result
#[derive(Debug, Clone)]
pub struct HealthCheckResult {
    pub timestamp: chrono::DateTime<chrono::Utc>,
    pub status: HealthStatus,
    pub response_time_ms: u64,
    pub error_message: Option<String>,
}

impl HealthMonitor {
    /// Create a new health monitor
    pub fn new(config: HealthCheckConfig) -> Self {
        Self {
            config,
            bot_health: Arc::new(RwLock::new(HashMap::new())),
            system_metrics: Arc::new(RwLock::new(Vec::new())),
            alerts: Arc::new(RwLock::new(Vec::new())),
            check_history: Arc::new(RwLock::new(HashMap::new())),
            start_time: Instant::now(),
        }
    }

    /// Start the health monitoring service
    pub async fn start(&self) -> Result<(), HealthError> {
        let check_interval = Duration::from_secs(self.config.check_interval_seconds);
        
        // Start periodic health checks
        let bot_health = Arc::clone(&self.bot_health);
        let system_metrics = Arc::clone(&self.system_metrics);
        let alerts = Arc::clone(&self.alerts);
        let config = self.config.clone();
        let start_time = self.start_time;
        
        tokio::spawn(async move {
            let mut interval = tokio::time::interval(check_interval);
            
            loop {
                interval.tick().await;
                
                // Perform system health check
                if let Ok(system_health) = Self::collect_system_metrics(start_time).await {
                    let mut metrics = system_metrics.write().await;
                    metrics.push(system_health.clone());
                    
                    // Keep only recent metrics
                    let retention_duration = chrono::Duration::hours(config.metrics_retention_hours as i64);
                    let cutoff_time = chrono::Utc::now() - retention_duration;
                    metrics.retain(|m| m.timestamp > cutoff_time);
                    
                    // Check for system alerts
                    Self::check_system_alerts(&system_health, &alerts, &config).await;
                }
                
                // Perform bot health checks
                Self::check_bot_health(&bot_health, &alerts, &config).await;
            }
        });
        
        Ok(())
    }

    /// Register a bot for health monitoring
    pub async fn register_bot(&self, bot_id: Uuid) -> Result<(), HealthError> {
        let health_record = BotHealthRecord {
            bot_id,
            status: crate::api::bot_interface::HealthLevel::Unknown,
            last_heartbeat: chrono::Utc::now(),
            metrics: BotMetrics::default(),
            error_count: 0,
            warnings: Vec::new(),
            performance_score: 0.0,
        };
        
        self.bot_health.write().await.insert(bot_id, health_record);
        self.check_history.write().await.insert(bot_id, Vec::new());
        
        Ok(())
    }

    /// Unregister a bot from health monitoring
    pub async fn unregister_bot(&self, bot_id: Uuid) -> Result<(), HealthError> {
        self.bot_health.write().await.remove(&bot_id);
        self.check_history.write().await.remove(&bot_id);
        
        Ok(())
    }

    /// Update bot health status
    pub async fn update_bot_health(
        &self,
        bot_id: Uuid,
        status: HealthStatus,
        metrics: BotMetrics,
    ) -> Result<(), HealthError> {
        let mut bot_health = self.bot_health.write().await;
        
        if let Some(record) = bot_health.get_mut(&bot_id) {
            record.status = status.clone();
            record.last_heartbeat = chrono::Utc::now();
            record.metrics = metrics.clone();
            record.performance_score = self.calculate_performance_score(&metrics);
            
            // Update check history
            let check_result = HealthCheckResult {
                timestamp: chrono::Utc::now(),
                status,
                response_time_ms: 0, // TODO: Measure actual response time
                error_message: None,
            };
            
            if let Ok(mut history) = self.check_history.try_write() {
                if let Some(bot_history) = history.get_mut(&bot_id) {
                    bot_history.push(check_result);
                    
                    // Keep only recent history
                    let retention_duration = chrono::Duration::hours(self.config.metrics_retention_hours as i64);
                    let cutoff_time = chrono::Utc::now() - retention_duration;
                    bot_history.retain(|r| r.timestamp > cutoff_time);
                }
            }
        } else {
            return Err(HealthError::BotNotFound(bot_id));
        }
        
        Ok(())
    }

    /// Get bot health status
    pub async fn get_bot_health(&self, bot_id: Uuid) -> Option<BotHealthRecord> {
        self.bot_health.read().await.get(&bot_id).cloned()
    }

    /// Get all bot health records
    pub async fn get_all_bot_health(&self) -> HashMap<Uuid, BotHealthRecord> {
        self.bot_health.read().await.clone()
    }

    /// Get system health metrics
    pub async fn get_system_health(&self) -> Vec<SystemHealthMetrics> {
        self.system_metrics.read().await.clone()
    }

    /// Get latest system health
    pub async fn get_latest_system_health(&self) -> Option<SystemHealthMetrics> {
        self.system_metrics.read().await.last().cloned()
    }

    /// Get health alerts
    pub async fn get_alerts(&self, severity: Option<AlertSeverity>) -> Vec<HealthAlert> {
        let alerts = self.alerts.read().await;
        
        match severity {
            Some(sev) => alerts.iter().filter(|a| a.severity == sev).cloned().collect(),
            None => alerts.clone(),
        }
    }

    /// Acknowledge alert
    pub async fn acknowledge_alert(&self, alert_id: Uuid) -> Result<(), HealthError> {
        let mut alerts = self.alerts.write().await;
        
        if let Some(alert) = alerts.iter_mut().find(|a| a.id == alert_id) {
            alert.acknowledged = true;
        }
        
        Ok(())
    }

    /// Resolve alert
    pub async fn resolve_alert(&self, alert_id: Uuid) -> Result<(), HealthError> {
        let mut alerts = self.alerts.write().await;
        
        if let Some(alert) = alerts.iter_mut().find(|a| a.id == alert_id) {
            alert.resolved = true;
        }
        
        Ok(())
    }

    /// Get bot health check history
    pub async fn get_bot_health_history(&self, bot_id: Uuid) -> Vec<HealthCheckResult> {
        self.check_history.read().await.get(&bot_id).cloned().unwrap_or_default()
    }

    /// Generate health report
    pub async fn generate_health_report(&self) -> HealthReport {
        let bot_health = self.bot_health.read().await.clone();
        let system_metrics = self.get_latest_system_health().await;
        let alerts = self.get_alerts(None).await;
        
        let total_bots = bot_health.len();
        let healthy_bots = bot_health.values().filter(|h| h.status == crate::api::bot_interface::HealthLevel::Healthy).count();
        let unhealthy_bots = bot_health.values().filter(|h| h.status == crate::api::bot_interface::HealthLevel::Unhealthy).count();
        let unknown_bots = bot_health.values().filter(|h| h.status == crate::api::bot_interface::HealthLevel::Unknown).count();
        
        let critical_alerts = alerts.iter().filter(|a| a.severity == AlertSeverity::Critical && !a.resolved).count();
        let warning_alerts = alerts.iter().filter(|a| a.severity == AlertSeverity::Warning && !a.resolved).count();
        
        HealthReport {
            timestamp: chrono::Utc::now(),
            system_health: system_metrics,
            bot_summary: BotHealthSummary {
                total_bots,
                healthy_bots,
                unhealthy_bots,
                unknown_bots,
                average_performance_score: bot_health.values()
                    .map(|h| h.performance_score)
                    .sum::<f64>() / total_bots.max(1) as f64,
            },
            alert_summary: AlertSummary {
                critical_alerts,
                warning_alerts,
                total_alerts: alerts.len(),
                unresolved_alerts: alerts.iter().filter(|a| !a.resolved).count(),
            },
            uptime_seconds: self.start_time.elapsed().as_secs(),
        }
    }

    /// Collect system metrics
    async fn collect_system_metrics(start_time: Instant) -> Result<SystemHealthMetrics, HealthError> {
        // TODO: Implement actual system metrics collection
        // For now, return mock data
        Ok(SystemHealthMetrics {
            timestamp: chrono::Utc::now(),
            cpu_usage_percent: 25.0,
            memory_usage_percent: 45.0,
            disk_usage_percent: 60.0,
            network_latency_ms: 2.5,
            active_connections: 150,
            uptime_seconds: start_time.elapsed().as_secs(),
            version: env!("CARGO_PKG_VERSION").to_string(),
        })
    }

    /// Check for system alerts
    async fn check_system_alerts(
        metrics: &SystemHealthMetrics,
        alerts: &Arc<RwLock<Vec<HealthAlert>>>,
        config: &HealthCheckConfig,
    ) {
        let mut new_alerts = Vec::new();
        
        // Check CPU usage
        if metrics.cpu_usage_percent > 90.0 {
            new_alerts.push(HealthAlert {
                id: Uuid::new_v4(),
                severity: AlertSeverity::Critical,
                title: "High CPU Usage".to_string(),
                message: format!("CPU usage is {}%", metrics.cpu_usage_percent),
                bot_id: None,
                timestamp: chrono::Utc::now(),
                acknowledged: false,
                resolved: false,
            });
        } else if metrics.cpu_usage_percent > 80.0 {
            new_alerts.push(HealthAlert {
                id: Uuid::new_v4(),
                severity: AlertSeverity::Warning,
                title: "Elevated CPU Usage".to_string(),
                message: format!("CPU usage is {}%", metrics.cpu_usage_percent),
                bot_id: None,
                timestamp: chrono::Utc::now(),
                acknowledged: false,
                resolved: false,
            });
        }
        
        // Check memory usage
        if metrics.memory_usage_percent > 95.0 {
            new_alerts.push(HealthAlert {
                id: Uuid::new_v4(),
                severity: AlertSeverity::Critical,
                title: "High Memory Usage".to_string(),
                message: format!("Memory usage is {}%", metrics.memory_usage_percent),
                bot_id: None,
                timestamp: chrono::Utc::now(),
                acknowledged: false,
                resolved: false,
            });
        } else if metrics.memory_usage_percent > 85.0 {
            new_alerts.push(HealthAlert {
                id: Uuid::new_v4(),
                severity: AlertSeverity::Warning,
                title: "Elevated Memory Usage".to_string(),
                message: format!("Memory usage is {}%", metrics.memory_usage_percent),
                bot_id: None,
                timestamp: chrono::Utc::now(),
                acknowledged: false,
                resolved: false,
            });
        }
        
        // Add new alerts
        if !new_alerts.is_empty() && config.alert_enabled {
            let mut alerts_guard = alerts.write().await;
            alerts_guard.extend(new_alerts);
        }
    }

    /// Check bot health
    async fn check_bot_health(
        bot_health: &Arc<RwLock<HashMap<Uuid, BotHealthRecord>>>,
        alerts: &Arc<RwLock<Vec<HealthAlert>>>,
        config: &HealthCheckConfig,
    ) {
        let mut new_alerts = Vec::new();
        let now = chrono::Utc::now();
        let stale_threshold = chrono::Duration::seconds(config.timeout_seconds as i64 * 2);
        
        let bot_health_guard = bot_health.read().await;
        
        for (bot_id, record) in bot_health_guard.iter() {
            // Check for stale heartbeats
            if now - record.last_heartbeat > stale_threshold {
                new_alerts.push(HealthAlert {
                    id: Uuid::new_v4(),
                    severity: AlertSeverity::Critical,
                    title: "Bot Unresponsive".to_string(),
                    message: format!("Bot {} has not sent a heartbeat in {} seconds", 
                                   bot_id, (now - record.last_heartbeat).num_seconds()),
                    bot_id: Some(*bot_id),
                    timestamp: now,
                    acknowledged: false,
                    resolved: false,
                });
            }
            
            // Check error count
            if record.error_count > config.failure_threshold {
                new_alerts.push(HealthAlert {
                    id: Uuid::new_v4(),
                    severity: AlertSeverity::Critical,
                    title: "High Error Rate".to_string(),
                    message: format!("Bot {} has {} errors", bot_id, record.error_count),
                    bot_id: Some(*bot_id),
                    timestamp: now,
                    acknowledged: false,
                    resolved: false,
                });
            }
            
            // Check performance score
            if record.performance_score < 0.5 {
                new_alerts.push(HealthAlert {
                    id: Uuid::new_v4(),
                    severity: AlertSeverity::Warning,
                    title: "Low Performance Score".to_string(),
                    message: format!("Bot {} performance score is {:.2}", bot_id, record.performance_score),
                    bot_id: Some(*bot_id),
                    timestamp: now,
                    acknowledged: false,
                    resolved: false,
                });
            }
        }
        
        // Add new alerts
        if !new_alerts.is_empty() && config.alert_enabled {
            let mut alerts_guard = alerts.write().await;
            alerts_guard.extend(new_alerts);
        }
    }

    /// Calculate performance score based on metrics
    fn calculate_performance_score(&self, metrics: &BotMetrics) -> f64 {
        // Simple scoring algorithm - can be enhanced
        let latency_score = if metrics.performance.avg_response_time_ms < 100.0 { 1.0 } else { 100.0 / metrics.performance.avg_response_time_ms };
        let error_score = if metrics.operational.error_count == 0 { 1.0 } else { 1.0 / (1.0 + metrics.operational.error_count as f64) };
        let uptime_score = metrics.operational.uptime_seconds as f64 / (24.0 * 3600.0); // Normalize by 24 hours
        
        (latency_score + error_score + uptime_score) / 3.0
    }
}

/// Health report structure
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HealthReport {
    pub timestamp: chrono::DateTime<chrono::Utc>,
    pub system_health: Option<SystemHealthMetrics>,
    pub bot_summary: BotHealthSummary,
    pub alert_summary: AlertSummary,
    pub uptime_seconds: u64,
}

/// Bot health summary
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BotHealthSummary {
    pub total_bots: usize,
    pub healthy_bots: usize,
    pub unhealthy_bots: usize,
    pub unknown_bots: usize,
    pub average_performance_score: f64,
}

/// Alert summary
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AlertSummary {
    pub critical_alerts: usize,
    pub warning_alerts: usize,
    pub total_alerts: usize,
    pub unresolved_alerts: usize,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_health_monitor_creation() {
        let config = HealthCheckConfig::default();
        let monitor = HealthMonitor::new(config);
        
        let bot_id = Uuid::new_v4();
        monitor.register_bot(bot_id).await.unwrap();
        
        let health = monitor.get_bot_health(bot_id).await;
        assert!(health.is_some());
        assert_eq!(health.unwrap().bot_id, bot_id);
    }

    #[tokio::test]
    async fn test_bot_health_update() {
        let config = HealthCheckConfig::default();
        let monitor = HealthMonitor::new(config);
        
        let bot_id = Uuid::new_v4();
        monitor.register_bot(bot_id).await.unwrap();
        
        let metrics = BotMetrics::default();
        monitor.update_bot_health(bot_id, HealthStatus::Healthy, metrics).await.unwrap();
        
        let health = monitor.get_bot_health(bot_id).await.unwrap();
        assert_eq!(health.status, HealthStatus::Healthy);
    }

    #[tokio::test]
    async fn test_health_report_generation() {
        let config = HealthCheckConfig::default();
        let monitor = HealthMonitor::new(config);
        
        let bot_id = Uuid::new_v4();
        monitor.register_bot(bot_id).await.unwrap();
        
        let report = monitor.generate_health_report().await;
        assert_eq!(report.bot_summary.total_bots, 1);
        assert_eq!(report.bot_summary.unknown_bots, 1);
    }
}
