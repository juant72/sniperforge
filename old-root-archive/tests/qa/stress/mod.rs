pub mod arbitrage_bot_stress_tests;

use crate::qa::QATestSuite;
use anyhow::Result;

pub async fn run_all_stress_tests() -> Result<Vec<QATestSuite>> {
    let mut suites = Vec::new();

    // ArbitrageBot Stress Tests
    let stress_tests = arbitrage_bot_stress_tests::ArbitrageBotStressTests::new().await?;
    let stress_suite = stress_tests.run_all_tests().await;
    suites.push(stress_suite);

    Ok(suites)
}
