use std::sync::Arc;
use std::sync::atomic::{AtomicU64, Ordering};
use std::time::{Duration, Instant};
use std::collections::HashMap;
use tokio::sync::RwLock;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProfileEvent {
    pub timestamp: u64,
    pub event_type: String,
    pub duration_ms: f64,
    pub memory_delta_mb: f64,
    pub cpu_usage: f64,
    pub metadata: HashMap<String, String>,
}

#[derive(Debug)]
pub struct SystemProfiler {
    events: Arc<RwLock<Vec<ProfileEvent>>>,
    start_time: Instant,
    memory_baseline: Arc<AtomicU64>,
    event_counter: Arc<AtomicU64>,
}

impl SystemProfiler {
    pub fn new() -> Self {
        Self {
            events: Arc::new(RwLock::new(Vec::new())),
            start_time: Instant::now(),
            memory_baseline: Arc::new(AtomicU64::new(0)),
            event_counter: Arc::new(AtomicU64::new(0)),
        }
    }

    pub async fn start_profiling(&self) {
        println!("🔍 Starting System Performance Profiling");
        
        // Set memory baseline
        let baseline_memory = self.get_current_memory_usage().await;
        self.memory_baseline.store(baseline_memory as u64, Ordering::Relaxed);
        
        println!("📊 Memory Baseline: {:.2} MB", baseline_memory);
    }

    pub async fn record_event(&self, event_type: &str, duration: Duration, metadata: HashMap<String, String>) {
        let current_memory = self.get_current_memory_usage().await;
        let baseline_memory = self.memory_baseline.load(Ordering::Relaxed) as f64;
        let memory_delta = current_memory - baseline_memory;
        
        let cpu_usage = self.get_current_cpu_usage().await;
        
        let event = ProfileEvent {
            timestamp: self.start_time.elapsed().as_millis() as u64,
            event_type: event_type.to_string(),
            duration_ms: duration.as_secs_f64() * 1000.0,
            memory_delta_mb: memory_delta,
            cpu_usage,
            metadata,
        };
        
        let mut events = self.events.write().await;
        events.push(event);
        
        let count = self.event_counter.fetch_add(1, Ordering::Relaxed);
        
        if count % 10 == 0 {
            println!("📈 Profile Event #{}: {} - {:.2}ms (Δ{:.2}MB, CPU:{:.1}%)", 
                count + 1, event_type, duration.as_secs_f64() * 1000.0, memory_delta, cpu_usage);
        }
    }

    pub async fn profile_function<F, T>(&self, name: &str, func: F) -> T 
    where 
        F: std::future::Future<Output = T>,
    {
        let start = Instant::now();
        let result = func.await;
        let duration = start.elapsed();
        
        let mut metadata = HashMap::new();
        metadata.insert("function".to_string(), name.to_string());
        
        self.record_event(&format!("function_{}", name), duration, metadata).await;
        
        result
    }

    pub async fn generate_profile_report(&self) -> String {
        let events = self.events.read().await;
        let mut report = String::new();
        
        report.push_str("🔍 SYSTEM PERFORMANCE PROFILE REPORT\n");
        report.push_str("=====================================\n\n");
        
        // Summary statistics
        let total_events = events.len();
        let total_duration: f64 = events.iter().map(|e| e.duration_ms).sum();
        let avg_duration = if total_events > 0 { total_duration / total_events as f64 } else { 0.0 };
        
        report.push_str(&format!("📊 Summary:\n"));
        report.push_str(&format!("   Total Events: {}\n", total_events));
        report.push_str(&format!("   Total Runtime: {:.2}ms\n", total_duration));
        report.push_str(&format!("   Average Event Duration: {:.2}ms\n", avg_duration));
        
        // Memory analysis
        let max_memory = events.iter().map(|e| e.memory_delta_mb).fold(0.0, f64::max);
        let min_memory = events.iter().map(|e| e.memory_delta_mb).fold(0.0, f64::min);
        let avg_memory: f64 = events.iter().map(|e| e.memory_delta_mb).sum::<f64>() / total_events as f64;
        
        report.push_str(&format!("\n💾 Memory Analysis:\n"));
        report.push_str(&format!("   Peak Memory Delta: +{:.2} MB\n", max_memory));
        report.push_str(&format!("   Min Memory Delta: {:.2} MB\n", min_memory));
        report.push_str(&format!("   Average Memory Delta: {:.2} MB\n", avg_memory));
        
        // CPU analysis
        let max_cpu = events.iter().map(|e| e.cpu_usage).fold(0.0, f64::max);
        let avg_cpu: f64 = events.iter().map(|e| e.cpu_usage).sum::<f64>() / total_events as f64;
        
        report.push_str(&format!("\n🖥️ CPU Analysis:\n"));
        report.push_str(&format!("   Peak CPU Usage: {:.1}%\n", max_cpu));
        report.push_str(&format!("   Average CPU Usage: {:.1}%\n", avg_cpu));
        
        // Event type breakdown
        let mut event_types: HashMap<String, Vec<&ProfileEvent>> = HashMap::new();
        for event in events.iter() {
            event_types.entry(event.event_type.clone()).or_insert_with(Vec::new).push(event);
        }
        
        report.push_str(&format!("\n📋 Event Type Breakdown:\n"));
        for (event_type, type_events) in event_types {
            let count = type_events.len();
            let total_time: f64 = type_events.iter().map(|e| e.duration_ms).sum();
            let avg_time = total_time / count as f64;
            
            report.push_str(&format!("   {}: {} events, {:.2}ms avg, {:.2}ms total\n", 
                event_type, count, avg_time, total_time));
        }
        
        // Slowest events
        let mut sorted_events = events.clone();
        sorted_events.sort_by(|a, b| b.duration_ms.partial_cmp(&a.duration_ms).unwrap());
        
        report.push_str(&format!("\n🐌 Slowest Events (Top 5):\n"));
        for (i, event) in sorted_events.iter().take(5).enumerate() {
            report.push_str(&format!("   {}. {} - {:.2}ms (t+{}ms)\n", 
                i + 1, event.event_type, event.duration_ms, event.timestamp));
        }
        
        // Performance hotspots
        report.push_str(&format!("\n🔥 Performance Hotspots:\n"));
        if avg_duration > 100.0 {
            report.push_str(&format!("   ⚠️ High average event duration: {:.2}ms\n", avg_duration));
        }
        if max_memory > 10.0 {
            report.push_str(&format!("   ⚠️ High memory usage spike: +{:.2}MB\n", max_memory));
        }
        if max_cpu > 90.0 {
            report.push_str(&format!("   ⚠️ High CPU usage: {:.1}%\n", max_cpu));
        }
        
        // Recommendations
        report.push_str(&format!("\n💡 Optimization Recommendations:\n"));
        if avg_duration > 50.0 {
            report.push_str("   • Consider async optimization for long-running operations\n");
        }
        if max_memory > 20.0 {
            report.push_str("   • Implement memory pooling for large allocations\n");
        }
        if avg_cpu > 80.0 {
            report.push_str("   • Consider parallel processing for CPU-intensive tasks\n");
        }
        
        report
    }

    pub async fn save_profile_data(&self, filename: &str) -> anyhow::Result<()> {
        let events = self.events.read().await;
        let json_data = serde_json::to_string_pretty(&*events)?;
        
        tokio::fs::write(filename, json_data).await?;
        println!("💾 Profile data saved to: {}", filename);
        
        Ok(())
    }

    // System monitoring methods (mock implementations)
    async fn get_current_memory_usage(&self) -> f64 {
        // Mock implementation - replace with actual system memory usage
        // On Windows, could use Windows API to get process memory
        // For now, simulate memory usage
        45.0 + (rand::random::<f64>() * 5.0) // Base 45MB + random variation
    }

    async fn get_current_cpu_usage(&self) -> f64 {
        // Mock implementation - replace with actual CPU usage measurement
        // Could use system APIs or libraries like `sysinfo`
        75.0 + (rand::random::<f64>() * 10.0) // Base 75% + random variation
    }
}

// Helper macros for easy profiling
#[macro_export]
macro_rules! profile_block {
    ($profiler:expr, $name:expr, $block:block) => {{
        let start = std::time::Instant::now();
        let result = $block;
        let duration = start.elapsed();
        
        let mut metadata = std::collections::HashMap::new();
        metadata.insert("block".to_string(), $name.to_string());
        
        $profiler.record_event(&format!("block_{}", $name), duration, metadata).await;
        result
    }};
}

pub use profile_block;
