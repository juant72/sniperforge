use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::{RwLock, mpsc};
use anyhow::Result;
use tracing::{info, warn, error, debug};
use serde::{Serialize, Deserialize};
use sysinfo::System;

use crate::config::Config;
use crate::types::{PlatformError, HealthStatus};

/// System metrics collected by the monitoring system
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SystemMetrics {
    pub cpu_usage_percent: f64,
    pub memory_usage_mb: u64,
    pub memory_total_mb: u64,
    pub disk_usage_percent: f64,
    pub network_bytes_sent: u64,
    pub network_bytes_received: u64,
    pub process_count: usize,
    pub uptime_seconds: u64,
    pub timestamp: chrono::DateTime<chrono::Utc>,
}

/// Application-specific metrics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ApplicationMetrics {
    pub active_bots: usize,
    pub total_transactions: u64,
    pub successful_transactions: u64,
    pub failed_transactions: u64,
    pub average_latency_ms: f64,
    pub rpc_calls_per_minute: u64,
    pub memory_usage_mb: u64,
    pub cpu_usage_percent: f64,
    pub timestamp: chrono::DateTime<chrono::Utc>,
}

/// Alert configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AlertConfig {
    pub name: String,
    pub metric_type: MetricType,
    pub threshold: f64,
    pub comparison: AlertComparison,
    pub cooldown_minutes: u64,
    pub severity: AlertSeverity,
    pub enabled: bool,
}

/// Types of metrics that can be monitored
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum MetricType {
    CpuUsage,
    MemoryUsage,
    DiskUsage,
    LatencyMs,
    ErrorRate,
    TransactionRate,
    Custom(String),
}

/// Alert comparison operators
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum AlertComparison {
    GreaterThan,
    LessThan,
    Equal,
}

/// Alert severity levels
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, PartialOrd, Ord)]
pub enum AlertSeverity {
    Info,
    Warning,
    Error,
    Critical,
}

impl std::fmt::Display for AlertSeverity {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            AlertSeverity::Info => write!(f, "INFO"),
            AlertSeverity::Warning => write!(f, "WARNING"),
            AlertSeverity::Error => write!(f, "ERROR"),
            AlertSeverity::Critical => write!(f, "CRITICAL"),
        }
    }
}

/// Alert event
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AlertEvent {
    pub id: uuid::Uuid,
    pub alert_config: AlertConfig,
    pub current_value: f64,
    pub message: String,
    pub severity: AlertSeverity,
    pub timestamp: chrono::DateTime<chrono::Utc>,
    pub acknowledged: bool,
}

/// Monitoring system for collecting and analyzing metrics
pub struct MonitoringSystem {
    config: Config,
    system_info: Arc<RwLock<System>>,
    system_metrics: Arc<RwLock<Vec<SystemMetrics>>>,
    app_metrics: Arc<RwLock<Vec<ApplicationMetrics>>>,
    alert_configs: Arc<RwLock<HashMap<String, AlertConfig>>>,
    active_alerts: Arc<RwLock<HashMap<uuid::Uuid, AlertEvent>>>,
    alert_history: Arc<RwLock<Vec<AlertEvent>>>,
    last_alert_times: Arc<RwLock<HashMap<String, chrono::DateTime<chrono::Utc>>>>,
    metrics_buffer_size: usize,
    shutdown_tx: mpsc::Sender<()>,
    is_running: Arc<RwLock<bool>>,
}

impl MonitoringSystem {
    pub fn new(config: &Config) -> Result<Self> {
        info!("📊 Initializing Monitoring System");
        
        let mut system = System::new_all();
        system.refresh_all();
        
        let (shutdown_tx, _) = mpsc::channel(1);
        
        let monitoring = Self {
            config: config.clone(),
            system_info: Arc::new(RwLock::new(system)),
            system_metrics: Arc::new(RwLock::new(Vec::new())),
            app_metrics: Arc::new(RwLock::new(Vec::new())),
            alert_configs: Arc::new(RwLock::new(HashMap::new())),
            active_alerts: Arc::new(RwLock::new(HashMap::new())),
            alert_history: Arc::new(RwLock::new(Vec::new())),
            last_alert_times: Arc::new(RwLock::new(HashMap::new())),
            metrics_buffer_size: 1000,
            shutdown_tx,
            is_running: Arc::new(RwLock::new(false)),
        };        info!("✅ Monitoring System initialized");
        Ok(monitoring)
    }

    /// Start the monitoring system
    pub async fn start(&self) -> Result<()> {
        info!("🚀 Starting Monitoring System");
        
        *self.is_running.write().await = true;
        
        // Start metrics collection
        self.start_metrics_collection().await;
        
        // Start alert evaluation
        self.start_alert_evaluation().await;
        
        // Start cleanup tasks
        self.start_cleanup_tasks().await;
        
        info!("✅ Monitoring System started successfully");
        Ok(())
    }

    /// Stop the monitoring system
    pub async fn stop(&self) -> Result<()> {
        info!("🛑 Stopping Monitoring System");
        
        *self.is_running.write().await = false;
        
        // Send shutdown signal
        let _ = self.shutdown_tx.send(()).await;
        
        Ok(())
    }

    /// Collect current system metrics
    pub async fn collect_system_metrics(&self) -> Result<SystemMetrics> {
        let mut system = self.system_info.write().await;
        system.refresh_all();
        
        let metrics = SystemMetrics {
            cpu_usage_percent: system.global_cpu_info().cpu_usage() as f64,
            memory_usage_mb: (system.used_memory() / 1024 / 1024),
            memory_total_mb: (system.total_memory() / 1024 / 1024),
            disk_usage_percent: self.calculate_disk_usage(&system).await,
            network_bytes_sent: 0, // Placeholder - would need network interface monitoring
            network_bytes_received: 0, // Placeholder
            process_count: system.processes().len(),
            uptime_seconds: System::uptime(),
            timestamp: chrono::Utc::now(),
        };
          // Store in buffer
        {
            let mut system_metrics = self.system_metrics.write().await;
            system_metrics.push(metrics.clone());
            
            // Keep buffer size limited
            if system_metrics.len() > self.metrics_buffer_size {
                let drain_count = system_metrics.len() - self.metrics_buffer_size;
                system_metrics.drain(0..drain_count);
            }
        }
          debug!("📊 Collected system metrics: CPU {:.1}%, Memory {}MB", 
               metrics.cpu_usage_percent, metrics.memory_usage_mb);
        
        Ok(metrics)
    }

    /// Record application metrics
    pub async fn record_app_metrics(&self, metrics: ApplicationMetrics) -> Result<()> {
        let mut app_metrics = self.app_metrics.write().await;
        app_metrics.push(metrics.clone());
        
        // Keep buffer size limited
        if app_metrics.len() > self.metrics_buffer_size {
            let drain_count = app_metrics.len() - self.metrics_buffer_size;
            app_metrics.drain(0..drain_count);
        }
        
        debug!("📊 Recorded app metrics: {} bots, {:.1}ms latency", 
               metrics.active_bots, metrics.average_latency_ms);
        
        Ok(())
    }

    /// Add alert configuration
    pub async fn add_alert(&self, alert_config: AlertConfig) -> Result<()> {
        let mut alert_configs = self.alert_configs.write().await;
        alert_configs.insert(alert_config.name.clone(), alert_config.clone());
        
        info!("🚨 Added alert: {} ({:?} {} {})", 
              alert_config.name, alert_config.metric_type, 
              match alert_config.comparison {
                  AlertComparison::GreaterThan => ">",
                  AlertComparison::LessThan => "<",
                  AlertComparison::Equal => "==",
              },
              alert_config.threshold);
        
        Ok(())
    }

    /// Remove alert configuration
    pub async fn remove_alert(&self, alert_name: &str) -> Result<()> {
        let mut alert_configs = self.alert_configs.write().await;
        
        if alert_configs.remove(alert_name).is_some() {
            info!("🚨 Removed alert: {}", alert_name);
            Ok(())
        } else {
            Err(PlatformError::Monitoring("Alert not found".to_string()).into())
        }
    }

    /// Acknowledge an alert
    pub async fn acknowledge_alert(&self, alert_id: uuid::Uuid) -> Result<()> {
        let mut active_alerts = self.active_alerts.write().await;
        
        if let Some(alert) = active_alerts.get_mut(&alert_id) {
            alert.acknowledged = true;
            info!("✅ Alert acknowledged: {}", alert_id);
            Ok(())
        } else {
            Err(PlatformError::Monitoring("Alert not found".to_string()).into())
        }
    }

    /// Get current system metrics
    pub async fn get_latest_system_metrics(&self) -> Option<SystemMetrics> {
        let system_metrics = self.system_metrics.read().await;
        system_metrics.last().cloned()
    }

    /// Get current application metrics
    pub async fn get_latest_app_metrics(&self) -> Option<ApplicationMetrics> {
        let app_metrics = self.app_metrics.read().await;
        app_metrics.last().cloned()
    }

    /// Get active alerts
    pub async fn get_active_alerts(&self) -> Vec<AlertEvent> {
        let active_alerts = self.active_alerts.read().await;
        active_alerts.values().cloned().collect()
    }

    /// Get alert history
    pub async fn get_alert_history(&self, limit: Option<usize>) -> Vec<AlertEvent> {
        let alert_history = self.alert_history.read().await;
        let limit = limit.unwrap_or(100).min(alert_history.len());
        
        alert_history.iter()
            .rev()
            .take(limit)
            .cloned()
            .collect()
    }

    /// Get monitoring statistics
    pub async fn get_stats(&self) -> MonitoringStats {
        let alert_configs = self.alert_configs.read().await;
        let active_alerts = self.active_alerts.read().await;
        let system_metrics = self.system_metrics.read().await;
        let app_metrics = self.app_metrics.read().await;
        
        MonitoringStats {
            alert_configs: alert_configs.len(),
            active_alerts: active_alerts.len(),
            system_metrics_collected: system_metrics.len(),
            app_metrics_collected: app_metrics.len(),
            is_running: *self.is_running.read().await,
        }
    }    /// Health check
    pub async fn health_check(&self) -> Result<HealthStatus> {
        let is_running = *self.is_running.read().await;
        
        if !is_running {
            return Ok(HealthStatus {
                is_healthy: false,
                component: "monitoring_system".to_string(),
                message: Some("Monitoring system not running".to_string()),
                checked_at: chrono::Utc::now(),
                metrics: HashMap::new(),
            });
        }

        let active_alerts = self.active_alerts.read().await;
        let critical_alerts: Vec<_> = active_alerts.values()
            .filter(|alert| alert.severity == AlertSeverity::Critical && !alert.acknowledged)
            .collect();

        if !critical_alerts.is_empty() {
            Ok(HealthStatus {
                is_healthy: false,
                component: "monitoring_system".to_string(),
                message: Some(format!("{} critical alerts active", critical_alerts.len())),
                checked_at: chrono::Utc::now(),
                metrics: HashMap::new(),
            })
        } else {
            let warning_alerts: Vec<_> = active_alerts.values()
                .filter(|alert| alert.severity == AlertSeverity::Warning && !alert.acknowledged)
                .collect();

            Ok(HealthStatus {
                is_healthy: true,
                component: "monitoring_system".to_string(),
                message: if warning_alerts.is_empty() {
                    None
                } else {
                    Some(format!("{} warning alerts active", warning_alerts.len()))
                },
                checked_at: chrono::Utc::now(),
                metrics: HashMap::new(),
            })
        }
    }

    /// Setup default alert configurations
    async fn setup_default_alerts(&self) -> Result<()> {
        // High CPU usage alert
        let cpu_alert = AlertConfig {
            name: "high_cpu_usage".to_string(),
            metric_type: MetricType::CpuUsage,
            threshold: 80.0,
            comparison: AlertComparison::GreaterThan,
            cooldown_minutes: 5,
            severity: AlertSeverity::Warning,
            enabled: true,
        };
        self.add_alert(cpu_alert).await?;

        // High memory usage alert
        let memory_alert = AlertConfig {
            name: "high_memory_usage".to_string(),
            metric_type: MetricType::MemoryUsage,
            threshold: 90.0,
            comparison: AlertComparison::GreaterThan,
            cooldown_minutes: 5,
            severity: AlertSeverity::Error,
            enabled: true,
        };
        self.add_alert(memory_alert).await?;

        // High latency alert
        let latency_alert = AlertConfig {
            name: "high_latency".to_string(),
            metric_type: MetricType::LatencyMs,
            threshold: 1000.0,
            comparison: AlertComparison::GreaterThan,
            cooldown_minutes: 2,
            severity: AlertSeverity::Warning,
            enabled: true,
        };
        self.add_alert(latency_alert).await?;

        info!("✅ Setup default alerts");
        Ok(())
    }

    /// Start metrics collection task
    async fn start_metrics_collection(&self) {
        let monitoring_system = self.system_info.clone();
        let system_metrics = self.system_metrics.clone();
        let is_running = self.is_running.clone();
        let metrics_buffer_size = self.metrics_buffer_size;
        
        tokio::spawn(async move {
            let mut interval = tokio::time::interval(std::time::Duration::from_secs(10));
            
            loop {
                interval.tick().await;
                
                if !*is_running.read().await {
                    break;
                }
                
                // Collect system metrics
                let mut system = monitoring_system.write().await;
                system.refresh_all();
                  let metrics = SystemMetrics {
                    cpu_usage_percent: system.global_cpu_info().cpu_usage() as f64,
                    memory_usage_mb: (system.used_memory() / 1024 / 1024),
                    memory_total_mb: (system.total_memory() / 1024 / 1024),
                    disk_usage_percent: 0.0, // Placeholder
                    network_bytes_sent: 0,
                    network_bytes_received: 0,
                    process_count: system.processes().len(),
                    uptime_seconds: System::uptime(),
                    timestamp: chrono::Utc::now(),
                };
                
                // Store metrics
                {
                    let mut metrics_vec = system_metrics.write().await;
                    metrics_vec.push(metrics);
                      if metrics_vec.len() > metrics_buffer_size {
                        let drain_count = metrics_vec.len() - metrics_buffer_size;
                        metrics_vec.drain(0..drain_count);
                    }
                }
                
                debug!("📊 Collected system metrics");
            }
        });
    }

    /// Start alert evaluation task
    async fn start_alert_evaluation(&self) {
        let alert_configs = self.alert_configs.clone();
        let active_alerts = self.active_alerts.clone();
        let alert_history = self.alert_history.clone();
        let last_alert_times = self.last_alert_times.clone();
        let system_metrics = self.system_metrics.clone();
        let app_metrics = self.app_metrics.clone();
        let is_running = self.is_running.clone();
        
        tokio::spawn(async move {
            let mut interval = tokio::time::interval(std::time::Duration::from_secs(30));
            
            loop {
                interval.tick().await;
                
                if !*is_running.read().await {
                    break;
                }
                
                let configs = alert_configs.read().await;
                let latest_system = {
                    let system_metrics_vec = system_metrics.read().await;
                    system_metrics_vec.last().cloned()
                };
                let latest_app = {
                    let app_metrics_vec = app_metrics.read().await;
                    app_metrics_vec.last().cloned()
                };
                
                for alert_config in configs.values() {
                    if !alert_config.enabled {
                        continue;
                    }
                    
                    // Check cooldown
                    let last_alert_times_guard = last_alert_times.read().await;
                    if let Some(last_time) = last_alert_times_guard.get(&alert_config.name) {
                        let cooldown = chrono::Duration::minutes(alert_config.cooldown_minutes as i64);
                        if chrono::Utc::now().signed_duration_since(*last_time) < cooldown {
                            continue;
                        }
                    }
                    drop(last_alert_times_guard);
                    
                    // Evaluate alert condition
                    let current_value = match &alert_config.metric_type {
                        MetricType::CpuUsage => latest_system.as_ref().map(|m| m.cpu_usage_percent),
                        MetricType::MemoryUsage => latest_system.as_ref().map(|m| {
                            (m.memory_usage_mb as f64 / m.memory_total_mb as f64) * 100.0
                        }),
                        MetricType::LatencyMs => latest_app.as_ref().map(|m| m.average_latency_ms),
                        _ => None,
                    };
                    
                    if let Some(value) = current_value {
                        let triggered = match alert_config.comparison {
                            AlertComparison::GreaterThan => value > alert_config.threshold,
                            AlertComparison::LessThan => value < alert_config.threshold,
                            AlertComparison::Equal => (value - alert_config.threshold).abs() < 0.001,
                        };
                        
                        if triggered {
                            // Create alert event
                            let alert_event = AlertEvent {
                                id: uuid::Uuid::new_v4(),
                                alert_config: alert_config.clone(),
                                current_value: value,
                                message: format!("Alert triggered: {} = {:.2}", alert_config.name, value),
                                severity: alert_config.severity.clone(),
                                timestamp: chrono::Utc::now(),
                                acknowledged: false,
                            };
                            
                            // Add to active alerts
                            {
                                let mut active = active_alerts.write().await;
                                active.insert(alert_event.id, alert_event.clone());
                            }
                            
                            // Add to history
                            {
                                let mut history = alert_history.write().await;
                                history.push(alert_event.clone());
                                
                                // Keep history limited
                                if history.len() > 10000 {
                                    history.drain(0..1000);
                                }
                            }
                            
                            // Update last alert time
                            {
                                let mut last_times = last_alert_times.write().await;
                                last_times.insert(alert_config.name.clone(), chrono::Utc::now());
                            }
                            
                            warn!("🚨 Alert triggered: {} ({})", alert_event.message, alert_event.severity);
                        }
                    }
                }
                
                debug!("🔍 Alert evaluation completed");
            }
        });
    }

    /// Start cleanup tasks
    async fn start_cleanup_tasks(&self) {
        let active_alerts = self.active_alerts.clone();
        let is_running = self.is_running.clone();
        
        tokio::spawn(async move {
            let mut interval = tokio::time::interval(std::time::Duration::from_secs(300)); // 5 minutes
            
            loop {
                interval.tick().await;
                
                if !*is_running.read().await {
                    break;
                }
                
                // Clean up acknowledged alerts older than 1 hour
                {
                    let mut active = active_alerts.write().await;
                    let now = chrono::Utc::now();
                    let cleanup_threshold = chrono::Duration::hours(1);
                    
                    active.retain(|_, alert| {
                        if alert.acknowledged {
                            now.signed_duration_since(alert.timestamp) <= cleanup_threshold
                        } else {
                            true
                        }
                    });
                }
                
                debug!("🧹 Alert cleanup completed");
            }
        });
    }

    /// Calculate disk usage percentage
    async fn calculate_disk_usage(&self, _system: &System) -> f64 {
        // Placeholder - would need to implement disk usage calculation
        // This is complex and system-specific
        0.0
    }
}

/// Monitoring system statistics
#[derive(Debug, Serialize)]
pub struct MonitoringStats {
    pub alert_configs: usize,
    pub active_alerts: usize,
    pub system_metrics_collected: usize,
    pub app_metrics_collected: usize,
    pub is_running: bool,
}
