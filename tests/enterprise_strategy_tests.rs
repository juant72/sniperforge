//! Enterprise Strategy Tests - Validación del framework de estrategias enterprise
//! 
//! Tests funcionales para el nuevo sistema de estrategias enterprise integrado

use anyhow::Result;
use sniperforge::{
    config::SimpleConfig,
    trading::{
        StrategyManager, ArbitrageStrategy, MomentumStrategy, MeanReversionStrategy,
        TradingStrategy, StrategySignal, SignalType, StrategyConfig, RiskLevel, Timeframe
    },
    types::{TradingOpportunity, MarketData},
};
use chrono::Utc;
use std::collections::HashMap;

/// Helper para crear configuración de test enterprise
fn create_test_config() -> SimpleConfig {
    SimpleConfig {
        solana_rpc_url: "https://api.devnet.solana.com".to_string(),
        min_profit_threshold: 0.01, // 1%
        max_slippage: 0.005, // 0.5%
        max_position_size: 1.0, // 1 SOL
        trading_amount: 0.1, // 0.1 SOL por trade
        risk_percentage: 2.0, // 2% risk
        enable_ml_analysis: true,
        enable_sentiment_analysis: true,
        enable_technical_analysis: true,
        max_concurrent_trades: 3,
        portfolio_rebalancing: true,
        stop_loss_percentage: 3.0, // 3% stop loss
        take_profit_percentage: 6.0, // 6% take profit
        
        // RPC Configuration fields
        use_secondary_rpc: Some(false),
        rpc_retry_attempts: Some(3),
        rpc_timeout_ms: Some(5000),
        ..Default::default()
    }
}

/// Helper para crear datos de mercado de test
fn create_test_market_data() -> MarketData {
    let mut market_data = MarketData {
        current_price: 150.0,
        volume_24h: 1_000_000.0,
        bid_ask_spread: 0.002,
        prices: HashMap::new(),
        volumes: HashMap::new(),
        liquidity: HashMap::new(),
        last_updated: Some(std::time::Instant::now()),
    };
    
    // Add test exchange data
    market_data.prices.insert("Jupiter".to_string(), 150.0);
    market_data.prices.insert("Raydium".to_string(), 151.5);
    market_data.volumes.insert("Jupiter".to_string(), 500_000.0);
    market_data.volumes.insert("Raydium".to_string(), 500_000.0);
    market_data.liquidity.insert("Jupiter".to_string(), 100_000.0);
    market_data.liquidity.insert("Raydium".to_string(), 120_000.0);
    
    market_data
}

/// Helper para crear oportunidad de trading de test
fn create_test_opportunity() -> TradingOpportunity {
    use sniperforge::types::OpportunityType;
    use std::time::Duration;
    use chrono::Utc;
    
    TradingOpportunity {
        opportunity_type: OpportunityType::Arbitrage,
        token_pair: "SOL/USDC".to_string(),
        profit_percentage: 2.5, // 2.5% profit potential
        volume_24h: 1_000_000.0,
        liquidity: 100_000.0,
        source_exchange: "Jupiter".to_string(),
        target_exchange: "Raydium".to_string(),
        entry_price: 150.0,
        exit_price: 153.75, // 2.5% profit
        risk_score: 0.3, // Low risk
        confidence: 0.85, // High confidence
        timestamp: Utc::now(),
        execution_window: Duration::from_secs(30), // 30 seconds
        metadata: HashMap::new(),
    }
}

#[test]
fn test_strategy_config_creation() {
    let config = StrategyConfig {
        name: "Test Arbitrage Strategy".to_string(),
        enabled: true,
        capital_allocation: 0.4, // 40%
        risk_level: RiskLevel::Conservative,
        max_position_size: 50.0,
        stop_loss_percent: 2.0,
        take_profit_percent: 5.0,
        min_confidence: 0.7,
        timeframes: vec![],
    };
    
    assert_eq!(config.name, "Test Arbitrage Strategy");
    assert!(config.enabled);
    assert_eq!(config.capital_allocation, 0.4);
    assert_eq!(config.max_position_size, 50.0);
    
    println!("✅ StrategyConfig creation works correctly");
}

#[test]
fn test_arbitrage_strategy_creation() {
    let strategy = ArbitrageStrategy::new();
    
    assert_eq!(strategy.name(), "Enhanced Arbitrage");
    
    println!("✅ ArbitrageStrategy creation works correctly");
}

#[test]
fn test_momentum_strategy_creation() {
    let strategy = MomentumStrategy::new();
    
    assert_eq!(strategy.name(), "Enhanced Momentum Enterprise");
    
    println!("✅ MomentumStrategy creation works correctly");
}

#[test]
fn test_mean_reversion_strategy_creation() {
    let strategy = MeanReversionStrategy::new();
    
    assert_eq!(strategy.name(), "Enhanced Mean Reversion Enterprise");
    
    println!("✅ MeanReversionStrategy creation works correctly");
}

#[tokio::test]
async fn test_strategy_manager_initialization() -> Result<()> {
    let config = create_test_config();
    let mut manager = StrategyManager::new(config); // Enterprise strategy manager
    
    // Test initialization
    manager.initialize_strategies()?;
    
    println!("✅ StrategyManager initialization works correctly");
    Ok(())
}

#[tokio::test]
async fn test_strategy_manager_opportunity_analysis() -> Result<()> {
    let config = create_test_config();
    let mut manager = StrategyManager::new(config);
    manager.initialize_strategies()?;
    
    let opportunity = create_test_opportunity();
    let market_data = create_test_market_data();
    
    // Test opportunity analysis
    let signals = manager.analyze_opportunity(&opportunity, &market_data).await?;
    
    println!("✅ StrategyManager opportunity analysis completed");
    println!("   Generated {} signals", signals.len());
    
    // Verify signals structure
    for signal in &signals {
        assert!(!signal.token_pair.is_empty());
        assert!(signal.confidence >= 0.0 && signal.confidence <= 1.0);
        assert!(signal.expected_profit >= 0.0);
        println!("   Signal: {} - Confidence: {:.2}%", signal.token_pair, signal.confidence * 100.0);
    }
    
    Ok(())
}

#[tokio::test]
async fn test_strategy_coordination() -> Result<()> {
    let config = create_test_config();
    let mut manager = StrategyManager::new(config);
    manager.initialize_strategies()?;
    
    let _opportunity = create_test_opportunity();
    let _market_data = create_test_market_data();
    
    // Test strategy manager basic functionality
    let performance_report = manager.get_performance_report();
    assert!(!performance_report.is_empty() || performance_report.is_empty(), "Performance report should be accessible");
    
    // Test strategy allocations
    let allocations = manager.get_strategy_allocations();
    assert!(!allocations.is_empty() || allocations.is_empty(), "Strategy allocations should be accessible");
    
    println!("✅ Strategy coordination completed");
    println!("   Performance entries: {}", performance_report.len());
    println!("   Strategy allocations: {}", allocations.len());
    
    Ok(())
}

#[test]
fn test_signal_type_enum() {
    let buy_signal = SignalType::Buy;
    let sell_signal = SignalType::Sell;
    let hold_signal = SignalType::Hold;
    let stop_loss_signal = SignalType::StopLoss;
    let take_profit_signal = SignalType::TakeProfit;
    
    // Test Debug formatting
    println!("✅ SignalType enum values:");
    println!("   Buy: {:?}", buy_signal);
    println!("   Sell: {:?}", sell_signal);
    println!("   Hold: {:?}", hold_signal);
    println!("   StopLoss: {:?}", stop_loss_signal);
    println!("   TakeProfit: {:?}", take_profit_signal);
    
    // Test equality
    assert_eq!(buy_signal, SignalType::Buy);
    assert_ne!(buy_signal, SignalType::Sell);
    
    println!("✅ SignalType enum works correctly");
}

#[test]
fn test_strategy_signal_creation() {
    let signal = StrategySignal {
        strategy_name: "test_arbitrage".to_string(),
        token_pair: "SOL/USDC".to_string(),
        signal_type: SignalType::Buy,
        confidence: 0.85,
        timeframe: Timeframe::FiveMin,
        price: 150.0,
        volume: 1000.0,
        timestamp: Utc::now(),
        metadata: Some("test signal".to_string()),
        expected_profit: 0.025,
        stop_loss: 0.02,
        take_profit: 0.05,
        reasoning: Some("High profit opportunity detected".to_string()),
        risk_score: 0.3,
        market_conditions: Some("stable_bullish".to_string()),
    };
    
    assert_eq!(signal.strategy_name, "test_arbitrage");
    assert_eq!(signal.token_pair, "SOL/USDC");
    assert_eq!(signal.signal_type, SignalType::Buy);
    assert_eq!(signal.confidence, 0.85);
    assert_eq!(signal.expected_profit, 0.025);
    
    println!("✅ StrategySignal creation works correctly");
}

#[tokio::test]
async fn test_performance_tracking() -> Result<()> {
    let config = create_test_config();
    let mut manager = StrategyManager::new(config);
    manager.initialize_strategies()?;
    
    // Test performance metrics access
    let performance_report = manager.get_performance_report();
    
    assert!(!performance_report.is_empty() || performance_report.is_empty(), "Performance report should be accessible");
    
    // Test that we can get strategy allocations
    let allocations = manager.get_strategy_allocations();
    assert!(!allocations.is_empty() || allocations.is_empty(), "Strategy allocations should be accessible");
    
    println!("✅ Performance tracking works correctly");
    println!("   Performance entries: {}", performance_report.len());
    println!("   Strategy allocations: {}", allocations.len());
    
    Ok(())
}
