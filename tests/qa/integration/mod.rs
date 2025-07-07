pub mod arbitrage_bot_tests;

use anyhow::Result;
use crate::qa::QATestSuite;

pub async fn run_all_integration_tests() -> Result<Vec<QATestSuite>> {
    let mut suites = Vec::new();

    // ArbitrageBot Integration Tests
    let arbitrage_tests = arbitrage_bot_tests::ArbitrageBotIntegrationTests::new().await?;
    let arbitrage_suite = arbitrage_tests.run_all_tests().await;
    suites.push(arbitrage_suite);

    Ok(suites)
}
