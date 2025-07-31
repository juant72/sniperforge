pub mod arbitrage_bot_unit_tests;

use crate::qa::QATestSuite;

pub async fn run_all_unit_tests() -> Vec<QATestSuite> {
    let mut suites = Vec::new();

    // ArbitrageBot Unit Tests
    let arbitrage_suite = arbitrage_bot_unit_tests::ArbitrageBotUnitTests::run_all_tests().await;
    suites.push(arbitrage_suite);

    suites
}
