pub mod arbitrage_bot_performance_tests;

use crate::qa::QATestSuite;
use anyhow::Result;

pub async fn run_all_performance_tests() -> Result<Vec<QATestSuite>> {
    let mut suites = Vec::new();

    // ArbitrageBot Performance Tests
    let performance_tests =
        arbitrage_bot_performance_tests::ArbitrageBotPerformanceTests::new().await?;
    let performance_suite = performance_tests.run_all_tests().await;
    suites.push(performance_suite);

    Ok(suites)
}
