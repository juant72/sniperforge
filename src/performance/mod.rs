// Performance benchmarking and monitoring module
pub mod benchmarks;
pub mod profiler;
pub mod metrics;

use std::time::{Duration, Instant};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PerformanceMetrics {
    pub detection_latency: Duration,
    pub execution_latency: Duration,
    pub total_end_to_end: Duration,
    pub pool_scan_rate: f64, // pools per second
    pub memory_usage_mb: f64,
    pub cpu_usage_percent: f64,
    pub rpc_call_latency: Duration,
    pub jupiter_api_latency: Duration,
    pub websocket_latency: Duration,
}

impl PerformanceMetrics {
    pub fn new() -> Self {
        Self {
            detection_latency: Duration::from_millis(0),
            execution_latency: Duration::from_millis(0),
            total_end_to_end: Duration::from_millis(0),
            pool_scan_rate: 0.0,
            memory_usage_mb: 0.0,
            cpu_usage_percent: 0.0,
            rpc_call_latency: Duration::from_millis(0),
            jupiter_api_latency: Duration::from_millis(0),
            websocket_latency: Duration::from_millis(0),
        }
    }

    pub fn meets_sprint2_targets(&self) -> bool {
        self.detection_latency < Duration::from_millis(100) &&
        self.execution_latency < Duration::from_millis(50) &&
        self.pool_scan_rate > 100.0 &&
        self.memory_usage_mb < 35.0 &&
        self.cpu_usage_percent > 85.0
    }

    pub fn performance_score(&self) -> f64 {
        let mut score = 100.0;
        
        // Detection latency penalty (target: <100ms)
        if self.detection_latency > Duration::from_millis(100) {
            score -= (self.detection_latency.as_millis() as f64 / 10.0).min(30.0);
        }
        
        // Execution latency penalty (target: <50ms)
        if self.execution_latency > Duration::from_millis(50) {
            score -= (self.execution_latency.as_millis() as f64 / 5.0).min(25.0);
        }
        
        // Pool scan rate penalty (target: >100/sec)
        if self.pool_scan_rate < 100.0 {
            score -= (100.0 - self.pool_scan_rate).min(20.0);
        }
        
        // Memory usage penalty (target: <35MB)
        if self.memory_usage_mb > 35.0 {
            score -= (self.memory_usage_mb - 35.0).min(15.0);
        }
        
        // CPU efficiency penalty (target: >85%)
        if self.cpu_usage_percent < 85.0 {
            score -= (85.0 - self.cpu_usage_percent).min(10.0);
        }
        
        score.max(0.0)
    }
}

#[derive(Debug)]
pub struct PerformanceTimer {
    start: Instant,
    label: String,
}

impl PerformanceTimer {
    pub fn new(label: &str) -> Self {
        Self {
            start: Instant::now(),
            label: label.to_string(),
        }
    }

    pub fn elapsed(&self) -> Duration {
        self.start.elapsed()
    }

    pub fn stop_and_log(&self) -> Duration {
        let elapsed = self.elapsed();
        println!("⏱️ {}: {:.2}ms", self.label, elapsed.as_secs_f64() * 1000.0);
        elapsed
    }
}

pub fn start_timer(label: &str) -> PerformanceTimer {
    PerformanceTimer::new(label)
}
