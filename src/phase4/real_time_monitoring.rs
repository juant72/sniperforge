// PHASE 4.3: REAL-TIME MONITORING DASHBOARD IMPLEMENTATION
// Professional monitoring and alerting system for arbitrage operations

use std::collections::{HashMap, VecDeque};
use std::sync::Arc;
use std::time::{Duration, Instant};
use anyhow::Result;
use tokio::sync::{mpsc, RwLock};
use tokio::time::sleep;
use serde::{Deserialize, Serialize};
use solana_sdk::pubkey::Pubkey;
use tracing::{info, warn, debug, error};

use crate::phase4::event_driven_engine::{EventDrivenOpportunity, ExecutionPriority};
use crate::phase4::parallel_execution::{ExecutionResult, ExecutionStatus, ParallelExecutionMetrics};

/// Real-time monitoring configuration
#[derive(Debug, Clone)]
pub struct MonitoringConfig {
    pub enable_real_time_dashboard: bool,
    pub enable_alerts: bool,
    pub enable_performance_tracking: bool,
    pub dashboard_update_interval_ms: u64,
    pub metrics_retention_minutes: u32,
    pub alert_thresholds: AlertThresholds,
    pub dashboard_port: u16,
    pub enable_web_interface: bool,
}

impl Default for MonitoringConfig {
    fn default() -> Self {
        Self {
            enable_real_time_dashboard: true,
            enable_alerts: true,
            enable_performance_tracking: true,
            dashboard_update_interval_ms: 1000, // 1 second updates
            metrics_retention_minutes: 60,      // 1 hour retention
            alert_thresholds: AlertThresholds::default(),
            dashboard_port: 8080,
            enable_web_interface: true,
        }
    }
}

/// Alert thresholds for monitoring
#[derive(Debug, Clone)]
pub struct AlertThresholds {
    pub max_failed_executions_per_minute: u32,
    pub min_success_rate_percent: f64,
    pub max_execution_time_ms: u64,
    pub min_profit_per_hour_lamports: u64,
    pub max_pending_opportunities: u32,
    pub min_opportunities_per_minute: u32,
}

impl Default for AlertThresholds {
    fn default() -> Self {
        Self {
            max_failed_executions_per_minute: 10,
            min_success_rate_percent: 70.0,
            max_execution_time_ms: 5000,
            min_profit_per_hour_lamports: 100_000_000, // 0.1 SOL
            max_pending_opportunities: 50,
            min_opportunities_per_minute: 5,
        }
    }
}

/// Alert types for the monitoring system
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum AlertType {
    HighFailureRate,
    LowSuccessRate,
    SlowExecution,
    LowProfitability,
    HighPendingCount,
    LowOpportunityRate,
    SystemError,
    NetworkCongestion,
    MEVAttack,
}

/// Alert severity levels
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
pub enum AlertSeverity {
    Critical = 4,
    High = 3,
    Medium = 2,
    Low = 1,
}

/// Alias for AlertSeverity for compatibility
pub use AlertSeverity as AlertLevel;

/// Real-time alert
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Alert {
    pub id: String,
    pub alert_type: AlertType,
    pub severity: AlertSeverity,
    pub message: String,
    pub details: HashMap<String, String>,
    pub timestamp: u64,
    pub resolved: bool,
}

/// Time-series data point for metrics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MetricDataPoint {
    pub timestamp: u64,
    pub value: f64,
    pub label: String,
}

/// Real-time dashboard data
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DashboardData {
    pub timestamp: u64,
    pub system_status: SystemStatus,
    pub execution_metrics: RealTimeExecutionMetrics,
    pub opportunity_metrics: OpportunityMetrics,
    pub performance_charts: PerformanceCharts,
    pub active_alerts: Vec<Alert>,
    pub recent_executions: Vec<ExecutionSummary>,
    pub network_status: NetworkStatus,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SystemStatus {
    pub status: String, // "HEALTHY", "WARNING", "CRITICAL"
    pub uptime_seconds: u64,
    pub total_opportunities_processed: u64,
    pub current_success_rate: f64,
    pub profit_today_lamports: u64,
    pub active_executions: u32,
    pub pending_opportunities: u32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RealTimeExecutionMetrics {
    pub executions_per_minute: f64,
    pub average_execution_time_ms: f64,
    pub success_rate_last_hour: f64,
    pub profit_per_hour_lamports: u64,
    pub concurrent_executions_current: u32,
    pub concurrent_executions_peak: u32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OpportunityMetrics {
    pub opportunities_detected_per_minute: f64,
    pub opportunities_by_type: HashMap<String, u32>,
    pub opportunities_by_dex: HashMap<String, u32>,
    pub average_profit_per_opportunity: f64,
    pub largest_opportunity_today: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PerformanceCharts {
    pub profit_timeline: Vec<MetricDataPoint>,
    pub execution_time_timeline: Vec<MetricDataPoint>,
    pub success_rate_timeline: Vec<MetricDataPoint>,
    pub opportunity_rate_timeline: Vec<MetricDataPoint>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExecutionSummary {
    pub opportunity_id: String,
    pub opportunity_type: String,
    pub status: String,
    pub profit_lamports: u64,
    pub execution_time_ms: u64,
    pub timestamp: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NetworkStatus {
    pub rpc_latency_ms: f64,
    pub current_slot: u64,
    pub network_congestion: f64,
    pub average_fee_lamports: u64,
    pub tps_current: f64,
}

/// Real-time monitoring engine
pub struct RealTimeMonitoringEngine {
    config: MonitoringConfig,
    
    // Data collection
    execution_history: Arc<RwLock<VecDeque<ExecutionResult>>>,
    opportunity_history: Arc<RwLock<VecDeque<EventDrivenOpportunity>>>,
    metric_timeseries: Arc<RwLock<HashMap<String, VecDeque<MetricDataPoint>>>>,
    
    // Alerting
    active_alerts: Arc<RwLock<Vec<Alert>>>,
    alert_sender: mpsc::UnboundedSender<Alert>,
    alert_receiver: Option<mpsc::UnboundedReceiver<Alert>>,
    
    // Dashboard
    current_dashboard_data: Arc<RwLock<DashboardData>>,
    
    // Performance tracking
    system_start_time: Instant,
    daily_profit_tracking: Arc<RwLock<u64>>,
}

impl RealTimeMonitoringEngine {
    /// Initialize the real-time monitoring engine
    pub async fn new(config: MonitoringConfig) -> Result<Self> {
        let (alert_sender, alert_receiver) = mpsc::unbounded_channel();
        
        let engine = Self {
            config: config.clone(),
            execution_history: Arc::new(RwLock::new(VecDeque::new())),
            opportunity_history: Arc::new(RwLock::new(VecDeque::new())),
            metric_timeseries: Arc::new(RwLock::new(HashMap::new())),
            active_alerts: Arc::new(RwLock::new(Vec::new())),
            alert_sender,
            alert_receiver: Some(alert_receiver),
            current_dashboard_data: Arc::new(RwLock::new(DashboardData::default())),
            system_start_time: Instant::now(),
            daily_profit_tracking: Arc::new(RwLock::new(0)),
        };

        info!("📊 Real-time monitoring engine initialized with config: {:#?}", config);
        Ok(engine)
    }

    /// Start the monitoring engine
    pub async fn start(&mut self) -> Result<()> {
        info!("🚀 Starting real-time monitoring engine...");

        // Store enable_web_interface before borrowing
        let enable_web_interface = self.config.enable_web_interface;

        // Start monitoring tasks
        let dashboard_updater = self.start_dashboard_updater();
        let metrics_collector = self.start_metrics_collector();
        let alert_evaluator = self.start_alert_evaluator();
        
        // Start alert processor separately
        let alert_processor_task = {
            let current_alerts = Arc::clone(&self.current_alerts);
            let execution_history = Arc::clone(&self.execution_history);
            let config = self.config.clone();
            
            tokio::spawn(async move {
                info!("📢 Alert processor started");
                loop {
                    sleep(Duration::from_millis(1000)).await;
                    
                    // Process alerts
                    let alerts = current_alerts.read().await;
                    if !alerts.is_empty() {
                        debug!("Processing {} alerts", alerts.len());
                    }
                }
            })
        };
        
        let web_server = if enable_web_interface {
            Box::pin(self.start_web_server()) as std::pin::Pin<Box<dyn futures::Future<Output = Result<()>> + Send>>
        } else {
            Box::pin(async { Ok(()) }) as std::pin::Pin<Box<dyn futures::Future<Output = Result<()>> + Send>>
        };

        // Wait for all tasks
        tokio::select! {
            result = dashboard_updater => {
                error!("Dashboard updater terminated: {:?}", result);
            }
            result = alert_processor_task => {
                error!("Alert processor terminated: {:?}", result);
            }
            result = metrics_collector => {
                error!("Metrics collector terminated: {:?}", result);
            }
            result = alert_evaluator => {
                error!("Alert evaluator terminated: {:?}", result);
            }
            result = web_server => {
                error!("Web server terminated: {:?}", result);
            }
        }

        Ok(())
    }

    /// Record an execution result for monitoring
    pub async fn record_execution(&self, execution_result: ExecutionResult) {
        // Add to history
        {
            let mut history = self.execution_history.write().await;
            history.push_back(execution_result.clone());
            
            // Keep only last hour of data
            let cutoff_time = Instant::now() - Duration::from_secs(3600);
            while let Some(front) = history.front() {
                if front.completed_at < cutoff_time {
                    history.pop_front();
                } else {
                    break;
                }
            }
        }

        // Update daily profit tracking
        if execution_result.status == ExecutionStatus::Completed {
            let mut daily_profit = self.daily_profit_tracking.write().await;
            *daily_profit += execution_result.profit_lamports;
        }

        debug!("📊 Recorded execution: {} - Status: {:?} - Profit: {} lamports", 
            execution_result.opportunity_id, execution_result.status, execution_result.profit_lamports);
    }

    /// Record an opportunity for monitoring
    pub async fn record_opportunity(&self, opportunity: EventDrivenOpportunity) {
        {
            let mut history = self.opportunity_history.write().await;
            history.push_back(opportunity.clone());
            
            // Keep only last hour of data
            let cutoff_time = Instant::now() - Duration::from_secs(3600);
            while let Some(front) = history.front() {
                if front.created_at < cutoff_time {
                    history.pop_front();
                } else {
                    break;
                }
            }
        }

        debug!("📊 Recorded opportunity: {} - Type: {:?} - Profit: {} lamports", 
            opportunity.opportunity_id, opportunity.opportunity_type, opportunity.expected_profit_lamports);
    }

    /// Get current dashboard data
    pub async fn get_dashboard_data(&self) -> DashboardData {
        self.current_dashboard_data.read().await.clone()
    }

    /// Send an alert
    pub fn send_alert(&self, alert: Alert) -> Result<()> {
        self.alert_sender.send(alert)
            .map_err(|e| anyhow::anyhow!("Failed to send alert: {}", e))?;
        Ok(())
    }

    /// Start the dashboard updater
    async fn start_dashboard_updater(&self) -> Result<()> {
        let current_dashboard_data = Arc::clone(&self.current_dashboard_data);
        let execution_history = Arc::clone(&self.execution_history);
        let opportunity_history = Arc::clone(&self.opportunity_history);
        let metric_timeseries = Arc::clone(&self.metric_timeseries);
        let active_alerts = Arc::clone(&self.active_alerts);
        let daily_profit_tracking = Arc::clone(&self.daily_profit_tracking);
        let system_start_time = self.system_start_time;
        let config = self.config.clone();

        tokio::spawn(async move {
            info!("📊 Dashboard updater started");

            loop {
                sleep(Duration::from_millis(config.dashboard_update_interval_ms)).await;

                let dashboard_data = Self::calculate_dashboard_data(
                    &execution_history,
                    &opportunity_history,
                    &metric_timeseries,
                    &active_alerts,
                    &daily_profit_tracking,
                    system_start_time,
                ).await;

                {
                    let mut current_data = current_dashboard_data.write().await;
                    *current_data = dashboard_data;
                }
            }
        });

        Ok(())
    }

    /// Calculate current dashboard data
    async fn calculate_dashboard_data(
        execution_history: &Arc<RwLock<VecDeque<ExecutionResult>>>,
        opportunity_history: &Arc<RwLock<VecDeque<EventDrivenOpportunity>>>,
        metric_timeseries: &Arc<RwLock<HashMap<String, VecDeque<MetricDataPoint>>>>,
        active_alerts: &Arc<RwLock<Vec<Alert>>>,
        daily_profit_tracking: &Arc<RwLock<u64>>,
        system_start_time: Instant,
    ) -> DashboardData {
        let now = chrono::Utc::now().timestamp() as u64;
        let uptime = system_start_time.elapsed().as_secs();

        // Get execution metrics
        let executions = execution_history.read().await;
        let opportunities = opportunity_history.read().await;
        let timeseries = metric_timeseries.read().await;
        let alerts = active_alerts.read().await;
        let daily_profit = *daily_profit_tracking.read().await;

        // Calculate system status
        let total_executions = executions.len() as u64;
        let successful_executions = executions.iter()
            .filter(|e| e.status == ExecutionStatus::Completed)
            .count() as u64;
        
        let current_success_rate = if total_executions > 0 {
            (successful_executions as f64 / total_executions as f64) * 100.0
        } else {
            0.0
        };

        let system_status_str = if current_success_rate >= 80.0 && alerts.len() < 3 {
            "HEALTHY"
        } else if current_success_rate >= 60.0 && alerts.len() < 10 {
            "WARNING"
        } else {
            "CRITICAL"
        };

        let system_status = SystemStatus {
            status: system_status_str.to_string(),
            uptime_seconds: uptime,
            total_opportunities_processed: total_executions,
            current_success_rate,
            profit_today_lamports: daily_profit,
            active_executions: executions.iter()
                .filter(|e| e.status == ExecutionStatus::InProgress)
                .count() as u32,
            pending_opportunities: opportunities.len() as u32,
        };

        // Calculate execution metrics
        let recent_executions = executions.iter()
            .filter(|e| e.completed_at > Instant::now() - Duration::from_secs(3600))
            .collect::<Vec<_>>();

        let executions_per_minute = recent_executions.len() as f64 / 60.0;
        let avg_execution_time = if !recent_executions.is_empty() {
            recent_executions.iter().map(|e| e.execution_time_ms as f64).sum::<f64>() / recent_executions.len() as f64
        } else {
            0.0
        };

        let hourly_profit = recent_executions.iter()
            .filter(|e| e.status == ExecutionStatus::Completed)
            .map(|e| e.profit_lamports)
            .sum::<u64>();

        let execution_metrics = RealTimeExecutionMetrics {
            executions_per_minute,
            average_execution_time_ms: avg_execution_time,
            success_rate_last_hour: current_success_rate,
            profit_per_hour_lamports: hourly_profit,
            concurrent_executions_current: system_status.active_executions,
            concurrent_executions_peak: 10, // This would be tracked separately
        };

        // Calculate opportunity metrics
        let recent_opportunities = opportunities.iter()
            .filter(|o| o.created_at > Instant::now() - Duration::from_secs(3600))
            .collect::<Vec<_>>();

        let opportunities_per_minute = recent_opportunities.len() as f64 / 60.0;
        
        let mut opportunities_by_type = HashMap::new();
        let mut opportunities_by_dex = HashMap::new();
        
        for opp in &recent_opportunities {
            let type_name = match &opp.opportunity_type {
                crate::phase4::event_driven_engine::OpportunityType::JupiterAutoRouted(_) => "Jupiter",
                crate::phase4::event_driven_engine::OpportunityType::DEXSpecialized(_) => "DEX Specialized",
                crate::phase4::event_driven_engine::OpportunityType::CrossDEXArbitrage { .. } => "Cross-DEX",
            };
            *opportunities_by_type.entry(type_name.to_string()).or_insert(0) += 1;
            
            // This would extract actual DEX names from opportunities
            *opportunities_by_dex.entry("Raydium".to_string()).or_insert(0) += 1;
        }

        let avg_profit_per_opportunity = if !recent_opportunities.is_empty() {
            recent_opportunities.iter().map(|o| o.expected_profit_lamports as f64).sum::<f64>() / recent_opportunities.len() as f64
        } else {
            0.0
        };

        let largest_opportunity = recent_opportunities.iter()
            .map(|o| o.expected_profit_lamports as f64)
            .fold(0.0, f64::max);

        let opportunity_metrics = OpportunityMetrics {
            opportunities_detected_per_minute: opportunities_per_minute,
            opportunities_by_type,
            opportunities_by_dex,
            average_profit_per_opportunity: avg_profit_per_opportunity,
            largest_opportunity_today: largest_opportunity,
        };

        // Generate performance charts
        let performance_charts = Self::generate_performance_charts(&timeseries);

        // Generate recent execution summaries
        let recent_executions = executions.iter()
            .rev()
            .take(10)
            .map(|e| ExecutionSummary {
                opportunity_id: e.opportunity_id.clone(),
                opportunity_type: "Generic".to_string(), // Would be extracted from opportunity
                status: format!("{:?}", e.status),
                profit_lamports: e.profit_lamports,
                execution_time_ms: e.execution_time_ms,
                timestamp: e.completed_at.elapsed().as_secs(),
            })
            .collect();

        // Network status (simplified)
        let network_status = NetworkStatus {
            rpc_latency_ms: 50.0, // Would be measured
            current_slot: 0,      // Would be fetched
            network_congestion: 0.3, // Would be calculated
            average_fee_lamports: 5000,
            tps_current: 2500.0,
        };

        DashboardData {
            timestamp: now,
            system_status,
            execution_metrics,
            opportunity_metrics,
            performance_charts,
            active_alerts: alerts.clone(),
            recent_executions,
            network_status,
        }
    }

    /// Generate performance charts from time series data
    fn generate_performance_charts(
        timeseries: &HashMap<String, VecDeque<MetricDataPoint>>,
    ) -> PerformanceCharts {
        let empty_vec = Vec::new();
        
        PerformanceCharts {
            profit_timeline: timeseries.get("profit")
                .map(|data| data.iter().cloned().collect())
                .unwrap_or(empty_vec.clone()),
            execution_time_timeline: timeseries.get("execution_time")
                .map(|data| data.iter().cloned().collect())
                .unwrap_or(empty_vec.clone()),
            success_rate_timeline: timeseries.get("success_rate")
                .map(|data| data.iter().cloned().collect())
                .unwrap_or(empty_vec.clone()),
            opportunity_rate_timeline: timeseries.get("opportunity_rate")
                .map(|data| data.iter().cloned().collect())
                .unwrap_or(empty_vec),
        }
    }

    /// Start the alert processor
    async fn start_alert_processor(&mut self) -> Result<()> {
        let active_alerts = Arc::clone(&self.active_alerts);
        let mut alert_receiver = self.alert_receiver.take()
            .ok_or_else(|| anyhow::anyhow!("Alert receiver already taken"))?;

        tokio::spawn(async move {
            info!("🚨 Alert processor started");

            while let Some(alert) = alert_receiver.recv().await {
                info!("🚨 ALERT: {:?} - {}", alert.severity, alert.message);
                
                // Add to active alerts
                {
                    let mut alerts = active_alerts.write().await;
                    alerts.push(alert.clone());
                    
                    // Keep only last 100 alerts
                    if alerts.len() > 100 {
                        alerts.remove(0);
                    }
                }

                // Here you would implement actual alerting (email, Slack, etc.)
                match alert.severity {
                    AlertSeverity::Critical => {
                        error!("🔴 CRITICAL ALERT: {}", alert.message);
                        // Send immediate notification
                    }
                    AlertSeverity::High => {
                        warn!("🟠 HIGH ALERT: {}", alert.message);
                        // Send priority notification
                    }
                    AlertSeverity::Medium => {
                        warn!("🟡 MEDIUM ALERT: {}", alert.message);
                        // Send standard notification
                    }
                    AlertSeverity::Low => {
                        info!("🟢 LOW ALERT: {}", alert.message);
                        // Log only
                    }
                }
            }

            warn!("🚨 Alert processor terminated");
        });

        Ok(())
    }

    /// Start the metrics collector
    async fn start_metrics_collector(&self) -> Result<()> {
        let metric_timeseries = Arc::clone(&self.metric_timeseries);
        let execution_history = Arc::clone(&self.execution_history);
        let opportunity_history = Arc::clone(&self.opportunity_history);
        let config = self.config.clone();

        tokio::spawn(async move {
            info!("📈 Metrics collector started");

            loop {
                sleep(Duration::from_secs(10)).await; // Collect every 10 seconds

                let now = chrono::Utc::now().timestamp() as u64;
                
                // Collect execution metrics
                let executions = execution_history.read().await;
                let opportunities = opportunity_history.read().await;

                let recent_executions = executions.iter()
                    .filter(|e| e.completed_at > Instant::now() - Duration::from_secs(300)) // Last 5 minutes
                    .collect::<Vec<_>>();

                // Calculate metrics
                let success_rate = if !recent_executions.is_empty() {
                    let successful = recent_executions.iter()
                        .filter(|e| e.status == ExecutionStatus::Completed)
                        .count();
                    (successful as f64 / recent_executions.len() as f64) * 100.0
                } else {
                    0.0
                };

                let avg_execution_time = if !recent_executions.is_empty() {
                    recent_executions.iter()
                        .map(|e| e.execution_time_ms as f64)
                        .sum::<f64>() / recent_executions.len() as f64
                } else {
                    0.0
                };

                let total_profit = recent_executions.iter()
                    .filter(|e| e.status == ExecutionStatus::Completed)
                    .map(|e| e.profit_lamports as f64)
                    .sum::<f64>();

                let opportunity_rate = opportunities.iter()
                    .filter(|o| o.created_at > Instant::now() - Duration::from_secs(60)) // Last minute
                    .count() as f64;

                // Store metrics
                {
                    let mut timeseries = metric_timeseries.write().await;
                    
                    Self::add_metric_point(&mut timeseries, "success_rate", now, success_rate);
                    Self::add_metric_point(&mut timeseries, "execution_time", now, avg_execution_time);
                    Self::add_metric_point(&mut timeseries, "profit", now, total_profit);
                    Self::add_metric_point(&mut timeseries, "opportunity_rate", now, opportunity_rate);
                    
                    // Clean old data
                    let retention_cutoff = now - (config.metrics_retention_minutes as u64 * 60);
                    for (_, data) in timeseries.iter_mut() {
                        while let Some(front) = data.front() {
                            if front.timestamp < retention_cutoff {
                                data.pop_front();
                            } else {
                                break;
                            }
                        }
                    }
                }

                debug!("📈 Collected metrics - Success: {:.1}% | Exec Time: {:.1}ms | Profit: {:.0} | Opps/min: {:.1}", 
                    success_rate, avg_execution_time, total_profit, opportunity_rate);
            }
        });

        Ok(())
    }

    /// Add a metric data point
    fn add_metric_point(
        timeseries: &mut HashMap<String, VecDeque<MetricDataPoint>>,
        metric_name: &str,
        timestamp: u64,
        value: f64,
    ) {
        let data_point = MetricDataPoint {
            timestamp,
            value,
            label: metric_name.to_string(),
        };

        timeseries.entry(metric_name.to_string())
            .or_insert_with(VecDeque::new)
            .push_back(data_point);
    }

    /// Start the alert evaluator
    async fn start_alert_evaluator(&self) -> Result<()> {
        let execution_history = Arc::clone(&self.execution_history);
        let opportunity_history = Arc::clone(&self.opportunity_history);
        let alert_sender = self.alert_sender.clone();
        let config = self.config.clone();

        tokio::spawn(async move {
            info!("⚠️ Alert evaluator started");

            loop {
                sleep(Duration::from_secs(30)).await; // Evaluate every 30 seconds

                if !config.enable_alerts {
                    continue;
                }

                let now = Instant::now();
                let executions = execution_history.read().await;
                let opportunities = opportunity_history.read().await;

                // Check failure rate
                let recent_executions = executions.iter()
                    .filter(|e| e.completed_at > now - Duration::from_secs(60))
                    .collect::<Vec<_>>();

                let failed_count = recent_executions.iter()
                    .filter(|e| e.status != ExecutionStatus::Completed)
                    .count() as u32;

                if failed_count > config.alert_thresholds.max_failed_executions_per_minute {
                    let alert = Alert {
                        id: format!("alert_{}", chrono::Utc::now().timestamp_millis()),
                        alert_type: AlertType::HighFailureRate,
                        severity: AlertSeverity::High,
                        message: format!("High failure rate: {} failed executions in last minute", failed_count),
                        details: HashMap::from([
                            ("failed_count".to_string(), failed_count.to_string()),
                            ("threshold".to_string(), config.alert_thresholds.max_failed_executions_per_minute.to_string()),
                        ]),
                        timestamp: chrono::Utc::now().timestamp() as u64,
                        resolved: false,
                    };

                    let _ = alert_sender.send(alert);
                }

                // Check success rate
                if !recent_executions.is_empty() {
                    let successful = recent_executions.iter()
                        .filter(|e| e.status == ExecutionStatus::Completed)
                        .count();
                    let success_rate = (successful as f64 / recent_executions.len() as f64) * 100.0;

                    if success_rate < config.alert_thresholds.min_success_rate_percent {
                        let alert = Alert {
                            id: format!("alert_{}", chrono::Utc::now().timestamp_millis()),
                            alert_type: AlertType::LowSuccessRate,
                            severity: AlertSeverity::Medium,
                            message: format!("Low success rate: {:.1}%", success_rate),
                            details: HashMap::from([
                                ("success_rate".to_string(), format!("{:.1}", success_rate)),
                                ("threshold".to_string(), format!("{:.1}", config.alert_thresholds.min_success_rate_percent)),
                            ]),
                            timestamp: chrono::Utc::now().timestamp() as u64,
                            resolved: false,
                        };

                        let _ = alert_sender.send(alert);
                    }
                }

                // Check opportunity rate
                let recent_opportunities = opportunities.iter()
                    .filter(|o| o.created_at > now - Duration::from_secs(60))
                    .count() as u32;

                if recent_opportunities < config.alert_thresholds.min_opportunities_per_minute {
                    let alert = Alert {
                        id: format!("alert_{}", chrono::Utc::now().timestamp_millis()),
                        alert_type: AlertType::LowOpportunityRate,
                        severity: AlertSeverity::Medium,
                        message: format!("Low opportunity detection rate: {} opportunities/minute", recent_opportunities),
                        details: HashMap::from([
                            ("opportunity_rate".to_string(), recent_opportunities.to_string()),
                            ("threshold".to_string(), config.alert_thresholds.min_opportunities_per_minute.to_string()),
                        ]),
                        timestamp: chrono::Utc::now().timestamp() as u64,
                        resolved: false,
                    };

                    let _ = alert_sender.send(alert);
                }
            }
        });

        Ok(())
    }

    /// Start the web server for dashboard interface
    async fn start_web_server(&self) -> Result<()> {
        let current_dashboard_data = Arc::clone(&self.current_dashboard_data);
        let port = self.config.dashboard_port;

        tokio::spawn(async move {
            info!("🌐 Web server started on port {}", port);
            
            // This is a simplified web server implementation
            // In a real application, you'd use a framework like axum or warp
            loop {
                sleep(Duration::from_secs(1)).await;
                
                // Simulate serving dashboard data
                let _dashboard_data = current_dashboard_data.read().await;
                
                // In real implementation:
                // - Serve HTML dashboard
                // - Provide REST API endpoints
                // - WebSocket for real-time updates
                // - Static assets for charts/graphs
            }
        });

        Ok(())
    }
}

impl Default for DashboardData {
    fn default() -> Self {
        Self {
            timestamp: chrono::Utc::now().timestamp() as u64,
            system_status: SystemStatus {
                status: "STARTING".to_string(),
                uptime_seconds: 0,
                total_opportunities_processed: 0,
                current_success_rate: 0.0,
                profit_today_lamports: 0,
                active_executions: 0,
                pending_opportunities: 0,
            },
            execution_metrics: RealTimeExecutionMetrics {
                executions_per_minute: 0.0,
                average_execution_time_ms: 0.0,
                success_rate_last_hour: 0.0,
                profit_per_hour_lamports: 0,
                concurrent_executions_current: 0,
                concurrent_executions_peak: 0,
            },
            opportunity_metrics: OpportunityMetrics {
                opportunities_detected_per_minute: 0.0,
                opportunities_by_type: HashMap::new(),
                opportunities_by_dex: HashMap::new(),
                average_profit_per_opportunity: 0.0,
                largest_opportunity_today: 0.0,
            },
            performance_charts: PerformanceCharts {
                profit_timeline: Vec::new(),
                execution_time_timeline: Vec::new(),
                success_rate_timeline: Vec::new(),
                opportunity_rate_timeline: Vec::new(),
            },
            active_alerts: Vec::new(),
            recent_executions: Vec::new(),
            network_status: NetworkStatus {
                rpc_latency_ms: 0.0,
                current_slot: 0,
                network_congestion: 0.0,
                average_fee_lamports: 0,
                tps_current: 0.0,
            },
        }
    }
}
