use sniperforge::trading::strategies::{ArbitrageStrategy, TradingStrategy, StrategyManager};
use sniperforge::types::{TradingOpportunity, MarketData, OpportunityType};

#[tokio::test]
async fn test_arbitrage_consolidation_integration() {
    // Test 1: ArbitrageStrategy implements TradingStrategy correctly
    let mut arbitrage_strategy = ArbitrageStrategy::new();
    
    // Configure price feeds
    arbitrage_strategy.update_price_feed("Jupiter".to_string(), 100.50);
    arbitrage_strategy.update_price_feed("Raydium".to_string(), 102.30);
    arbitrage_strategy.update_price_feed("Orca".to_string(), 101.80);
    
    // Create test data
    let market_data = MarketData {
        current_price: 101.0,
        volume_24h: 500000.0,
        liquidity: 100000.0,
        bid_ask_spread: 0.002,
    };
    
    let opportunity = TradingOpportunity {
        opportunity_type: OpportunityType::Arbitrage,
        token_pair: "SOL/USDC".to_string(),
        profit_percentage: 2.5,
        volume_24h: 500000.0,
        liquidity: 100000.0,
        source_exchange: "Jupiter".to_string(),
        target_exchange: "Raydium".to_string(),
        ..Default::default()
    };
    
    // Test analyze method (TradingStrategy trait implementation)
    let result = arbitrage_strategy.analyze(&opportunity, &market_data);
    assert!(result.is_ok(), "Arbitrage analysis should succeed");
    
    // Test configuration access
    let config = arbitrage_strategy.get_config();
    assert!(config.capital_allocation > 0.0, "Capital allocation should be positive");
    assert!(config.min_confidence > 0.0, "Min confidence should be positive");
    
    // Test performance tracking
    let performance = arbitrage_strategy.get_performance();
    assert!(performance.total_trades >= 0, "Total trades should be non-negative");
    assert!(performance.win_rate >= 0.0 && performance.win_rate <= 1.0, "Win rate should be between 0 and 1");
    
    // Test transaction cost calculation
    let jupiter_costs = arbitrage_strategy.calculate_transaction_costs(1000.0, "Jupiter");
    let raydium_costs = arbitrage_strategy.calculate_transaction_costs(1000.0, "Raydium");
    assert!(jupiter_costs > 0.0, "Jupiter transaction costs should be positive");
    assert!(raydium_costs > 0.0, "Raydium transaction costs should be positive");
    
    println!("✅ Arbitrage consolidation integration test passed");
}

#[tokio::test]
async fn test_strategy_manager_integration() {
    // Test 2: StrategyManager can handle ArbitrageStrategy
    let mut strategy_manager = StrategyManager::new();
    
    // Create arbitrage strategy
    let mut arbitrage_strategy = ArbitrageStrategy::new();
    arbitrage_strategy.update_price_feed("Jupiter".to_string(), 100.50);
    arbitrage_strategy.update_price_feed("Raydium".to_string(), 102.30);
    
    // Add strategy to manager
    strategy_manager.add_strategy(Box::new(arbitrage_strategy));
    
    // Create test data
    let market_data = MarketData {
        current_price: 101.0,
        volume_24h: 500000.0,
        liquidity: 100000.0,
        bid_ask_spread: 0.002,
    };
    
    let opportunity = TradingOpportunity {
        opportunity_type: OpportunityType::Arbitrage,
        token_pair: "SOL/USDC".to_string(),
        profit_percentage: 2.5,
        volume_24h: 500000.0,
        liquidity: 100000.0,
        source_exchange: "Jupiter".to_string(),
        target_exchange: "Raydium".to_string(),
        ..Default::default()
    };
    
    // Test analyze_all method
    let result = strategy_manager.analyze_all(&opportunity, &market_data);
    assert!(result.is_ok(), "Strategy manager analysis should succeed");
    
    let signals = result.unwrap();
    assert!(!signals.is_empty(), "Should generate at least one signal");
    
    // Test performance summary
    let performance_summary = strategy_manager.get_performance_summary();
    assert!(!performance_summary.is_empty(), "Should have performance data for strategies");
    
    println!("✅ Strategy manager integration test passed");
}

#[tokio::test]
async fn test_ml_preservation() {
    // Test 3: Verify ML capabilities are preserved
    let mut arbitrage_strategy = ArbitrageStrategy::new();
    
    // Access the underlying ML engine
    let ml_engine = arbitrage_strategy.arbitrage_engine();
    
    // Test ML analysis (this should work if ML capabilities are preserved)
    let result = ml_engine.analyze_opportunity_with_ml(
        "SOL/USDC",
        2.5,     // profit_percentage
        500000.0, // volume_24h
        100000.0  // liquidity
    ).await;
    
    assert!(result.is_ok(), "ML analysis should succeed");
    
    let (ml_score, ml_recommendation) = result.unwrap();
    assert!(ml_score >= 0.0 && ml_score <= 1.0, "ML score should be between 0 and 1");
    assert!(!ml_recommendation.is_empty(), "ML recommendation should not be empty");
    
    println!("✅ ML preservation test passed");
    println!("   ML Score: {:.3}", ml_score);
    println!("   ML Recommendation: {}", ml_recommendation);
}

#[test]
fn test_arbitrage_consolidation_compilation() {
    // Test 4: Verify compilation and basic functionality
    use sniperforge::trading::strategies::{ArbitrageStrategy, TradingStrategy};
    
    let mut strategy = ArbitrageStrategy::new();
    
    // Test basic methods exist and work
    let config = strategy.get_config();
    assert!(config.capital_allocation > 0.0);
    
    let performance = strategy.get_performance();
    assert!(performance.total_trades >= 0);
    
    // Test price feed update
    strategy.update_price_feed("TestExchange".to_string(), 100.0);
    
    // Test transaction cost calculation
    let costs = strategy.calculate_transaction_costs(1000.0, "TestExchange");
    assert!(costs >= 0.0);
    
    println!("✅ Arbitrage consolidation compilation test passed");
}
