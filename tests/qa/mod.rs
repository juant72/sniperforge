pub mod integration;
pub mod unit;
pub mod stress;
pub mod performance;

use tracing::{info, error};

/// QA Test Result
#[derive(Debug, Clone)]
pub struct QATestResult {
    pub test_name: String,
    pub passed: bool,
    pub duration_ms: u64,
    pub error_message: Option<String>,
    pub details: Vec<String>,
}

/// QA Test Suite
pub struct QATestSuite {
    pub name: String,
    pub results: Vec<QATestResult>,
}

impl QATestSuite {
    pub fn new(name: String) -> Self {
        Self {
            name,
            results: Vec::new(),
        }
    }

    pub fn add_result(&mut self, result: QATestResult) {
        self.results.push(result);
    }

    pub fn get_summary(&self) -> QASummary {
        let total = self.results.len();
        let passed = self.results.iter().filter(|r| r.passed).count();
        let failed = total - passed;
        let total_duration = self.results.iter().map(|r| r.duration_ms).sum();

        QASummary {
            suite_name: self.name.clone(),
            total_tests: total,
            passed_tests: passed,
            failed_tests: failed,
            success_rate: if total > 0 { (passed as f64 / total as f64) * 100.0 } else { 0.0 },
            total_duration_ms: total_duration,
            failed_test_names: self.results.iter()
                .filter(|r| !r.passed)
                .map(|r| r.test_name.clone())
                .collect(),
        }
    }

    pub fn print_report(&self) {
        let summary = self.get_summary();

        info!("📋 QA Test Suite Report: {}", self.name);
        info!("{}", "=".repeat(50));
        info!("✅ Passed: {}/{}", summary.passed_tests, summary.total_tests);
        info!("❌ Failed: {}", summary.failed_tests);
        info!("📊 Success Rate: {:.1}%", summary.success_rate);
        info!("⏱️ Total Duration: {}ms", summary.total_duration_ms);

        if !summary.failed_test_names.is_empty() {
            info!("🔍 Failed Tests:");
            for test_name in &summary.failed_test_names {
                error!("  ❌ {}", test_name);
            }
        }

        info!("{}", "=".repeat(50));

        // Print detailed results
        for result in &self.results {
            let status = if result.passed { "✅" } else { "❌" };
            info!("{} {} ({}ms)", status, result.test_name, result.duration_ms);

            if !result.passed {
                if let Some(error) = &result.error_message {
                    error!("   Error: {}", error);
                }
            }

            for detail in &result.details {
                info!("   {}", detail);
            }
        }
    }
}

/// QA Summary
#[derive(Debug)]
pub struct QASummary {
    pub suite_name: String,
    pub total_tests: usize,
    pub passed_tests: usize,
    pub failed_tests: usize,
    pub success_rate: f64,
    pub total_duration_ms: u64,
    pub failed_test_names: Vec<String>,
}

/// QA Test Trait
#[async_trait::async_trait]
pub trait QATest {
    async fn run(&self) -> QATestResult;
    fn name(&self) -> String;
    fn description(&self) -> String;
}

/// QA Test Macros for easier test creation
macro_rules! qa_test {
    ($name:expr, $test_fn:expr) => {{
        let start_time = std::time::Instant::now();
        let test_name = $name.to_string();

        match $test_fn.await {
            Ok(details) => QATestResult {
                test_name,
                passed: true,
                duration_ms: start_time.elapsed().as_millis() as u64,
                error_message: None,
                details,
            },
            Err(e) => QATestResult {
                test_name,
                passed: false,
                duration_ms: start_time.elapsed().as_millis() as u64,
                error_message: Some(e.to_string()),
                details: vec![],
            },
        }
    }};
}

macro_rules! qa_assert {
    ($condition:expr, $message:expr) => {
        if !$condition {
            return Err(anyhow::anyhow!($message));
        }
    };
}

macro_rules! qa_assert_eq {
    ($left:expr, $right:expr, $message:expr) => {
        if $left != $right {
            return Err(anyhow::anyhow!("{}: expected {:?}, got {:?}", $message, $right, $left));
        }
    };
}

// Re-export macros for use in tests
pub(crate) use qa_test;
pub(crate) use qa_assert;
pub(crate) use qa_assert_eq;
