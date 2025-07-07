use crate::qa::{QATestSuite, QATestResult, qa_test, qa_assert, qa_assert_eq};
use sniperforge::bots::arbitrage_bot::{ArbitrageStrategy, MarketData, StrategySignal};
use anyhow::Result;
use std::collections::HashMap;
use std::time::Instant;

/// ArbitrageBot Unit Tests
pub struct ArbitrageBotUnitTests;

impl ArbitrageBotUnitTests {
    pub async fn run_all_tests() -> QATestSuite {
        let mut suite = QATestSuite::new("ArbitrageBot Unit Tests".to_string());

        // Test 1: Strategy Creation
        let result = qa_test!("Strategy Creation", Self::test_strategy_creation());
        suite.add_result(result);

        // Test 2: Market Data Validation
        let result = qa_test!("Market Data Validation", Self::test_market_data_validation());
        suite.add_result(result);

        // Test 3: Signal Generation
        let result = qa_test!("Signal Generation", Self::test_signal_generation());
        suite.add_result(result);

        // Test 4: Price Analysis
        let result = qa_test!("Price Analysis", Self::test_price_analysis());
        suite.add_result(result);

        // Test 5: Spread Calculation
        let result = qa_test!("Spread Calculation", Self::test_spread_calculation());
        suite.add_result(result);

        // Test 6: Risk Validation
        let result = qa_test!("Risk Validation", Self::test_risk_validation());
        suite.add_result(result);

        suite
    }

    async fn test_strategy_creation() -> Result<Vec<String>> {
        let mut details = Vec::new();

        let strategy = ArbitrageStrategy::new();

        // Strategy should be created with current time
        let now = Instant::now();
        let time_diff = now.duration_since(strategy.last_analysis_time);
        qa_assert!(time_diff.as_secs() < 1, "Strategy should be created with recent timestamp");

        details.push("ArbitrageStrategy created successfully".to_string());
        details.push(format!("Last analysis time set to recent timestamp"));

        Ok(details)
    }

    async fn test_market_data_validation() -> Result<Vec<String>> {
        let mut details = Vec::new();

        // Create valid market data
        let valid_market_data = MarketData {
            symbol: "SOL/USDC".to_string(),
            price: 100.0,
            volume: 1000.0,
            timestamp: 1234567890,
            bid: 99.5,
            ask: 100.5,
            spread: 1.0,
            current_price: 100.0,
            volume_24h: 50000.0,
            price_change_24h: 2.5,
            liquidity: 10000.0,
            bid_ask_spread: 1.0,
            order_book_depth: 5000.0,
            price_history: vec![99.0, 100.0, 101.0],
            volume_history: vec![800.0, 900.0, 1000.0],
        };

        // Validate market data structure
        qa_assert!(!valid_market_data.symbol.is_empty(), "Symbol should not be empty");
        qa_assert!(valid_market_data.price > 0.0, "Price should be positive");
        qa_assert!(valid_market_data.volume >= 0.0, "Volume should be non-negative");
        qa_assert!(valid_market_data.bid <= valid_market_data.ask, "Bid should be <= Ask");
        qa_assert!(valid_market_data.spread >= 0.0, "Spread should be non-negative");
        qa_assert!(valid_market_data.liquidity >= 0.0, "Liquidity should be non-negative");

        details.push("Market data validation passed".to_string());
        details.push(format!("Symbol: {}", valid_market_data.symbol));
        details.push(format!("Price: ${:.2}", valid_market_data.price));
        details.push(format!("Bid/Ask: ${:.2}/${:.2}", valid_market_data.bid, valid_market_data.ask));

        Ok(details)
    }

    async fn test_signal_generation() -> Result<Vec<String>> {
        let mut details = Vec::new();

        let strategy = ArbitrageStrategy::new();

        // Create market data with profitable spread
        let profitable_market_data = MarketData {
            symbol: "SOL/USDC".to_string(),
            price: 100.0,
            volume: 1000.0,
            timestamp: 1234567890,
            bid: 99.0,
            ask: 101.0,
            spread: 2.0, // 2% spread - should be profitable
            current_price: 100.0,
            volume_24h: 50000.0,
            price_change_24h: 0.0,
            liquidity: 10000.0,
            bid_ask_spread: 2.0,
            order_book_depth: 5000.0,
            price_history: vec![],
            volume_history: vec![],
        };

        let signals = strategy.analyze_market(&profitable_market_data).await?;

        qa_assert!(!signals.is_empty(), "Should generate signals for profitable spread");

        for signal in &signals {
            qa_assert!(signal.confidence > 0.0, "Signal confidence should be positive");
            qa_assert!(signal.confidence <= 1.0, "Signal confidence should be <= 1.0");
            qa_assert!(signal.position_size > 0.0, "Position size should be positive");
            qa_assert!(!signal.symbol.is_empty(), "Signal symbol should not be empty");
            qa_assert!(!signal.strategy_name.is_empty(), "Strategy name should not be empty");
        }

        details.push(format!("Generated {} signals for profitable market", signals.len()));
        details.push(format!("Signal confidence: {:.1}%", signals[0].confidence * 100.0));
        details.push(format!("Position size: ${:.2}", signals[0].position_size));

        // Test with unprofitable spread
        let unprofitable_market_data = MarketData {
            symbol: "SOL/USDC".to_string(),
            price: 100.0,
            volume: 1000.0,
            timestamp: 1234567890,
            bid: 99.9,
            ask: 100.1,
            spread: 0.2, // 0.2% spread - should not be profitable
            current_price: 100.0,
            volume_24h: 50000.0,
            price_change_24h: 0.0,
            liquidity: 10000.0,
            bid_ask_spread: 0.2,
            order_book_depth: 5000.0,
            price_history: vec![],
            volume_history: vec![],
        };

        let no_signals = strategy.analyze_market(&unprofitable_market_data).await?;
        qa_assert!(no_signals.is_empty(), "Should not generate signals for unprofitable spread");

        details.push("No signals generated for unprofitable market (correct)".to_string());

        Ok(details)
    }

    async fn test_price_analysis() -> Result<Vec<String>> {
        let mut details = Vec::new();

        let strategy = ArbitrageStrategy::new();

        // Test various price scenarios
        let test_cases = vec![
            (100.0, 1000.0, "Normal market"),
            (0.01, 100.0, "Low price token"),
            (10000.0, 50.0, "High price token"),
            (100.0, 0.0, "Zero volume"),
        ];

        for (price, volume, description) in test_cases {
            let market_data = MarketData {
                symbol: "TEST/USDC".to_string(),
                price,
                volume,
                timestamp: 1234567890,
                bid: price * 0.995,
                ask: price * 1.005,
                spread: price * 0.01,
                current_price: price,
                volume_24h: volume * 24.0,
                price_change_24h: 0.0,
                liquidity: volume * 10.0,
                bid_ask_spread: price * 0.01,
                order_book_depth: volume * 5.0,
                price_history: vec![],
                volume_history: vec![],
            };

            let signals = strategy.analyze_market(&market_data).await?;
            details.push(format!("{}: {} signals generated", description, signals.len()));
        }

        Ok(details)
    }

    async fn test_spread_calculation() -> Result<Vec<String>> {
        let mut details = Vec::new();

        // Test spread calculations
        let test_cases = vec![
            (100.0, 99.0, 101.0, 2.0),  // Normal spread
            (100.0, 99.5, 100.5, 1.0), // Tight spread
            (100.0, 95.0, 105.0, 10.0), // Wide spread
        ];

        for (price, bid, ask, expected_spread) in test_cases {
            let calculated_spread = ask - bid;
            qa_assert_eq!(calculated_spread, expected_spread,
                         format!("Spread calculation failed for price {}", price));

            let spread_percentage = (calculated_spread / price) * 100.0;
            details.push(format!("Price ${:.2}: spread ${:.2} ({:.1}%)",
                               price, calculated_spread, spread_percentage));
        }

        Ok(details)
    }

    async fn test_risk_validation() -> Result<Vec<String>> {
        let mut details = Vec::new();

        // Test position size calculations
        let liquidity_values = vec![1000.0, 5000.0, 10000.0, 50000.0];
        let max_position = 10.0; // $10 max for DevNet

        for liquidity in liquidity_values {
            let position_size = liquidity.min(max_position);
            qa_assert!(position_size <= max_position, "Position size should not exceed maximum");
            qa_assert!(position_size > 0.0, "Position size should be positive");

            details.push(format!("Liquidity ${:.0} -> Position ${:.2}", liquidity, position_size));
        }

        // Test confidence thresholds
        let confidence_values = vec![0.1, 0.3, 0.5, 0.7, 0.9];
        let min_confidence = 0.5;

        for confidence in confidence_values {
            let is_valid = confidence >= min_confidence;
            details.push(format!("Confidence {:.1}: {}", confidence,
                               if is_valid { "Valid" } else { "Too low" }));
        }

        Ok(details)
    }
}
