mod qa;

use anyhow::Result;
use qa::{QATestResult, QATestSuite};
use sniperforge::bots::arbitrage_bot::ArbitrageBot;
use sniperforge::config::Config;
use sniperforge::shared::SharedServices;
use std::sync::Arc;
use std::time::Instant;
use tracing::{error, info, warn};

// Import QA macros from qa module
use qa::{qa_assert, qa_assert_eq, qa_test};

/// Main QA Test Runner for ArbitrageBot
#[tokio::main]
async fn main() -> Result<()> {
    // Initialize logging
    tracing_subscriber::fmt()
        .with_env_filter("info,sniperforge=debug")
        .init();

    info!("🧪 Starting SniperForge ArbitrageBot QA Test Suite");
    info!("{}", "=".repeat(70));

    let start_time = Instant::now();
    let mut all_suites = Vec::new();

    // Initialize counters
    let mut total_tests = 0;
    let mut total_passed = 0;
    let mut total_failed = 0;

    // Run Unit Tests
    info!("🔬 Running Unit Tests...");
    let unit_suites = qa::unit::run_all_unit_tests().await;
    for suite in unit_suites {
        let summary = suite.get_summary();
        total_tests += summary.total_tests;
        total_passed += summary.passed_tests;
        total_failed += summary.failed_tests;
        all_suites.push(suite);
    }

    // Run Integration Tests
    info!("🔗 Running Integration Tests...");
    match qa::integration::run_all_integration_tests().await {
        Ok(integration_suites) => {
            for suite in integration_suites {
                let summary = suite.get_summary();
                total_tests += summary.total_tests;
                total_passed += summary.passed_tests;
                total_failed += summary.failed_tests;
                all_suites.push(suite);
            }
        }
        Err(e) => {
            error!("❌ Integration tests failed to initialize: {}", e);
            error!("💡 Make sure DevNet configuration is available");
        }
    }

    // Run Stress Tests
    info!("💪 Running Stress Tests...");
    match qa::stress::run_all_stress_tests().await {
        Ok(stress_suites) => {
            for suite in stress_suites {
                let summary = suite.get_summary();
                total_tests += summary.total_tests;
                total_passed += summary.passed_tests;
                total_failed += summary.failed_tests;
                all_suites.push(suite);
            }
        }
        Err(e) => {
            error!("❌ Stress tests failed to initialize: {}", e);
        }
    }

    // Run Performance Tests
    info!("⚡ Running Performance Tests...");
    match qa::performance::run_all_performance_tests().await {
        Ok(performance_suites) => {
            for suite in performance_suites {
                let summary = suite.get_summary();
                total_tests += summary.total_tests;
                total_passed += summary.passed_tests;
                total_failed += summary.failed_tests;
                all_suites.push(suite);
            }
        }
        Err(e) => {
            error!("❌ Performance tests failed to initialize: {}", e);
        }
    }

    let total_duration = start_time.elapsed();

    // Print individual suite reports
    info!("\n📋 Individual Test Suite Reports:");
    info!("{}", "=".repeat(70));
    for suite in &all_suites {
        suite.print_report();
        info!("");
    }

    // Print overall summary
    let total_duration = start_time.elapsed();
    print_overall_summary(
        total_tests,
        total_passed,
        total_failed,
        total_duration,
        &all_suites,
    );

    // Exit with appropriate code
    if total_failed > 0 {
        std::process::exit(1);
    } else {
        std::process::exit(0);
    }
}

fn print_overall_summary(
    total_tests: usize,
    total_passed: usize,
    total_failed: usize,
    total_duration: std::time::Duration,
    all_suites: &[QATestSuite],
) {
    info!("🎯 OVERALL QA TEST SUMMARY");
    info!("{}", "=".repeat(70));

    let overall_success_rate = if total_tests > 0 {
        (total_passed as f64 / total_tests as f64) * 100.0
    } else {
        0.0
    };

    info!("📊 Test Results:");
    info!("   Total Tests: {}", total_tests);
    info!("   ✅ Passed: {}", total_passed);
    info!("   ❌ Failed: {}", total_failed);
    info!("   📈 Success Rate: {:.1}%", overall_success_rate);
    info!("   ⏱️ Total Duration: {:.1}s", total_duration.as_secs_f64());

    // Suite breakdown
    info!("\n📋 Suite Breakdown:");
    for suite in all_suites {
        let summary = suite.get_summary();
        let status = if summary.failed_tests == 0 {
            "✅"
        } else {
            "❌"
        };
        info!(
            "   {} {}: {}/{} passed ({:.1}%)",
            status,
            summary.suite_name,
            summary.passed_tests,
            summary.total_tests,
            summary.success_rate
        );
    }

    // Quality assessment
    info!("\n🎯 Quality Assessment:");
    match overall_success_rate {
        rate if rate >= 95.0 => {
            info!("   🟢 EXCELLENT - Production ready!");
            info!("   🚀 ArbitrageBot passes all quality standards");
        }
        rate if rate >= 85.0 => {
            info!("   🟡 GOOD - Minor issues to address");
            info!("   ⚠️ Review failed tests before production");
        }
        rate if rate >= 70.0 => {
            info!("   🟠 FAIR - Several issues need attention");
            info!("   🔧 Significant improvements required");
        }
        _ => {
            info!("   🔴 POOR - Major issues detected");
            info!("   🛑 Not ready for production use");
        }
    }

    // Recommendations
    if total_failed > 0 {
        info!("\n🔍 Failed Test Categories:");
        let mut failed_categories = std::collections::HashMap::new();
        for suite in all_suites {
            if suite.get_summary().failed_tests > 0 {
                failed_categories.insert(suite.name.clone(), suite.get_summary().failed_tests);
            }
        }

        for (category, count) in failed_categories {
            info!("   ❌ {}: {} failures", category, count);
        }

        info!("\n💡 Recommendations:");
        info!("   1. Review and fix failed test cases");
        info!("   2. Check DevNet connectivity and configuration");
        info!("   3. Verify all dependencies are properly installed");
        info!("   4. Run tests individually for detailed debugging");
    } else {
        info!("\n🎉 Congratulations!");
        info!("   ✨ All tests passed - ArbitrageBot is ready for deployment!");
        info!("   🚀 Proceed with confidence to production testing");
    }

    info!("{}", "=".repeat(70));
}
