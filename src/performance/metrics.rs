use std::time::{Duration, Instant, SystemTime, UNIX_EPOCH};
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MetricSample {
    pub timestamp: u64,
    pub value: f64,
    pub metadata: HashMap<String, String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MetricSeries {
    pub name: String,
    pub unit: String,
    pub samples: Vec<MetricSample>,
    pub target_value: Option<f64>,
    pub alert_threshold: Option<f64>,
}

#[derive(Debug)]
pub struct MetricsCollector {
    series: Arc<RwLock<HashMap<String, MetricSeries>>>,
    collection_interval: Duration,
    is_collecting: Arc<std::sync::atomic::AtomicBool>,
}

impl MetricsCollector {
    pub fn new(collection_interval: Duration) -> Self {
        Self {
            series: Arc::new(RwLock::new(HashMap::new())),
            collection_interval,
            is_collecting: Arc::new(std::sync::atomic::AtomicBool::new(false)),
        }
    }

    pub async fn start_collection(&self) {
        if self.is_collecting.swap(true, std::sync::atomic::Ordering::Relaxed) {
            println!("⚠️ Metrics collection already running");
            return;
        }

        println!("📊 Starting metrics collection (interval: {:.1}s)", 
            self.collection_interval.as_secs_f64());

        // Initialize core metrics
        self.init_core_metrics().await;

        // Start collection loop
        let series = Arc::clone(&self.series);
        let interval = self.collection_interval;
        let is_collecting = Arc::clone(&self.is_collecting);

        tokio::spawn(async move {
            let mut interval_timer = tokio::time::interval(interval);
            
            while is_collecting.load(std::sync::atomic::Ordering::Relaxed) {
                interval_timer.tick().await;
                
                // Collect system metrics
                Self::collect_system_metrics(&series).await;
            }
        });
    }

    pub fn stop_collection(&self) {
        self.is_collecting.store(false, std::sync::atomic::Ordering::Relaxed);
        println!("⏹️ Metrics collection stopped");
    }

    async fn init_core_metrics(&self) {
        let mut series = self.series.write().await;
        
        // Performance metrics
        series.insert("detection_latency".to_string(), MetricSeries {
            name: "Detection Latency".to_string(),
            unit: "ms".to_string(),
            samples: Vec::new(),
            target_value: Some(100.0), // Sprint 2 target: <100ms
            alert_threshold: Some(150.0),
        });

        series.insert("execution_latency".to_string(), MetricSeries {
            name: "Execution Latency".to_string(),
            unit: "ms".to_string(),
            samples: Vec::new(),
            target_value: Some(50.0), // Sprint 2 target: <50ms
            alert_threshold: Some(75.0),
        });

        series.insert("pool_scan_rate".to_string(), MetricSeries {
            name: "Pool Scan Rate".to_string(),
            unit: "pools/sec".to_string(),
            samples: Vec::new(),
            target_value: Some(100.0), // Sprint 2 target: >100/sec
            alert_threshold: Some(50.0),
        });

        series.insert("memory_usage".to_string(), MetricSeries {
            name: "Memory Usage".to_string(),
            unit: "MB".to_string(),
            samples: Vec::new(),
            target_value: Some(35.0), // Sprint 2 target: <35MB
            alert_threshold: Some(50.0),
        });

        series.insert("cpu_usage".to_string(), MetricSeries {
            name: "CPU Usage".to_string(),
            unit: "%".to_string(),
            samples: Vec::new(),
            target_value: Some(85.0), // Sprint 2 target: >85% efficiency
            alert_threshold: Some(95.0),
        });

        series.insert("rpc_latency".to_string(), MetricSeries {
            name: "RPC Latency".to_string(),
            unit: "ms".to_string(),
            samples: Vec::new(),
            target_value: Some(30.0), // Sprint 2 target: <30ms
            alert_threshold: Some(50.0),
        });

        series.insert("jupiter_latency".to_string(), MetricSeries {
            name: "Jupiter API Latency".to_string(),
            unit: "ms".to_string(),
            samples: Vec::new(),
            target_value: Some(30.0), // Current: ~29ms (very good)
            alert_threshold: Some(50.0),
        });

        series.insert("websocket_latency".to_string(), MetricSeries {
            name: "WebSocket Latency".to_string(),
            unit: "ms".to_string(),
            samples: Vec::new(),
            target_value: Some(20.0),
            alert_threshold: Some(40.0),
        });

        println!("✅ Initialized {} core metrics", series.len());
    }

    async fn collect_system_metrics(series: &Arc<RwLock<HashMap<String, MetricSeries>>>) {
        let timestamp = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_secs();

        let mut series_guard = series.write().await;

        // Collect memory usage
        if let Some(memory_series) = series_guard.get_mut("memory_usage") {
            let memory_value = Self::get_memory_usage().await;
            memory_series.samples.push(MetricSample {
                timestamp,
                value: memory_value,
                metadata: HashMap::new(),
            });
        }

        // Collect CPU usage
        if let Some(cpu_series) = series_guard.get_mut("cpu_usage") {
            let cpu_value = Self::get_cpu_usage().await;
            cpu_series.samples.push(MetricSample {
                timestamp,
                value: cpu_value,
                metadata: HashMap::new(),
            });
        }

        // Keep only recent samples (last 1000)
        for (_, series) in series_guard.iter_mut() {
            if series.samples.len() > 1000 {
                series.samples.drain(0..series.samples.len() - 1000);
            }
        }
    }

    pub async fn record_metric(&self, metric_name: &str, value: f64, metadata: Option<HashMap<String, String>>) {
        let timestamp = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_secs();

        let mut series = self.series.write().await;
        if let Some(metric_series) = series.get_mut(metric_name) {
            metric_series.samples.push(MetricSample {
                timestamp,
                value,
                metadata: metadata.unwrap_or_default(),
            });

            // Check alert threshold
            if let Some(threshold) = metric_series.alert_threshold {
                if (metric_name.contains("latency") || metric_name.contains("usage")) && value > threshold {
                    println!("🚨 ALERT: {} exceeded threshold! Value: {:.2}{}, Threshold: {:.2}{}", 
                        metric_series.name, value, metric_series.unit, threshold, metric_series.unit);
                } else if metric_name.contains("rate") && value < threshold {
                    println!("🚨 ALERT: {} below threshold! Value: {:.2}{}, Threshold: {:.2}{}", 
                        metric_series.name, value, metric_series.unit, threshold, metric_series.unit);
                }
            }
        }
    }

    pub async fn get_metric_summary(&self, metric_name: &str) -> Option<MetricSummary> {
        let series = self.series.read().await;
        if let Some(metric_series) = series.get(metric_name) {
            if metric_series.samples.is_empty() {
                return None;
            }

            let values: Vec<f64> = metric_series.samples.iter().map(|s| s.value).collect();
            let count = values.len();
            let sum: f64 = values.iter().sum();
            let avg = sum / count as f64;
            let min = values.iter().cloned().fold(f64::INFINITY, f64::min);
            let max = values.iter().cloned().fold(f64::NEG_INFINITY, f64::max);

            // Calculate percentiles
            let mut sorted_values = values;
            sorted_values.sort_by(|a, b| a.partial_cmp(b).unwrap());
            let p50 = sorted_values[count / 2];
            let p95 = sorted_values[(count as f64 * 0.95) as usize];
            let p99 = sorted_values[(count as f64 * 0.99) as usize];

            Some(MetricSummary {
                name: metric_series.name.clone(),
                unit: metric_series.unit.clone(),
                count,
                avg,
                min,
                max,
                p50,
                p95,
                p99,
                target_value: metric_series.target_value,
                meets_target: Self::meets_target(&metric_series.name, avg, metric_series.target_value),
            })
        } else {
            None
        }
    }

    pub async fn generate_metrics_report(&self) -> String {
        let mut report = String::new();
        
        report.push_str("📊 SPRINT 2 METRICS DASHBOARD\n");
        report.push_str("==============================\n\n");

        let metric_names = vec![
            "detection_latency",
            "execution_latency", 
            "pool_scan_rate",
            "memory_usage",
            "cpu_usage",
            "rpc_latency",
            "jupiter_latency",
            "websocket_latency",
        ];

        for metric_name in metric_names {
            if let Some(summary) = self.get_metric_summary(metric_name).await {
                report.push_str(&format!("🔸 {}\n", summary.name));
                report.push_str(&format!("   Current: {:.2}{}\n", summary.avg, summary.unit));
                
                if let Some(target) = summary.target_value {
                    report.push_str(&format!("   Target: {:.2}{} | Status: {}\n", 
                        target, summary.unit, 
                        if summary.meets_target { "✅" } else { "❌" }));
                }
                
                report.push_str(&format!("   Range: {:.2} - {:.2}{}\n", 
                    summary.min, summary.max, summary.unit));
                report.push_str(&format!("   P50: {:.2} | P95: {:.2} | P99: {:.2}{}\n", 
                    summary.p50, summary.p95, summary.p99, summary.unit));
                report.push_str(&format!("   Samples: {}\n\n", summary.count));
            }
        }

        // Overall performance score
        let score = self.calculate_overall_score().await;
        report.push_str(&format!("🎯 Overall Performance Score: {:.1}/100\n", score));
        
        if score >= 80.0 {
            report.push_str("✅ Excellent performance - ready for production!\n");
        } else if score >= 60.0 {
            report.push_str("⚠️ Good performance - minor optimizations needed\n");
        } else {
            report.push_str("❌ Performance needs improvement - optimization required\n");
        }

        report
    }

    async fn calculate_overall_score(&self) -> f64 {
        let mut total_score = 0.0;
        let mut scored_metrics = 0;

        let metric_weights = HashMap::from([
            ("detection_latency", 25.0),
            ("execution_latency", 25.0),
            ("pool_scan_rate", 20.0),
            ("memory_usage", 15.0),
            ("cpu_usage", 10.0),
            ("rpc_latency", 5.0),
        ]);

        for (metric_name, weight) in metric_weights {
            if let Some(summary) = self.get_metric_summary(metric_name).await {
                let metric_score = if summary.meets_target { 100.0 } else {
                    // Calculate score based on how close to target
                    if let Some(target) = summary.target_value {
                        let ratio = if metric_name.contains("rate") {
                            summary.avg / target
                        } else {
                            target / summary.avg
                        };
                        (ratio * 100.0).min(100.0).max(0.0)
                    } else {
                        50.0 // No target available
                    }
                };
                
                total_score += metric_score * (weight / 100.0);
                scored_metrics += 1;
            }
        }

        if scored_metrics > 0 {
            total_score
        } else {
            0.0
        }
    }

    fn meets_target(metric_name: &str, value: f64, target: Option<f64>) -> bool {
        if let Some(target_val) = target {
            if metric_name.contains("latency") || metric_name.contains("usage") {
                value <= target_val
            } else if metric_name.contains("rate") {
                value >= target_val
            } else {
                false
            }
        } else {
            false
        }
    }

    // Mock system monitoring methods
    async fn get_memory_usage() -> f64 {
        45.0 + (rand::random::<f64>() * 10.0)
    }

    async fn get_cpu_usage() -> f64 {
        75.0 + (rand::random::<f64>() * 15.0)
    }
}

#[derive(Debug, Clone)]
pub struct MetricSummary {
    pub name: String,
    pub unit: String,
    pub count: usize,
    pub avg: f64,
    pub min: f64,
    pub max: f64,
    pub p50: f64,
    pub p95: f64,
    pub p99: f64,
    pub target_value: Option<f64>,
    pub meets_target: bool,
}
