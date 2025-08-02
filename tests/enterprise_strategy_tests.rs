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
    types::{TradingOpportunity, MarketData, Token, ArbitragePair},
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
        ..Default::default()
    }
}

/// Helper para crear datos de mercado de test
fn create_test_market_data() -> MarketData {
    MarketData {
        sol_price_usd: 150.0,
        trading_volume_24h: 1_000_000.0,
        market_cap_usd: 75_000_000_000.0,
        volatility_24h: 0.15,
        timestamp: Utc::now(),
        additional_metrics: HashMap::new(),
    }
}

/// Helper para crear oportunidad de trading de test
fn create_test_opportunity() -> TradingOpportunity {
    let base_token = Token {
        symbol: "SOL".to_string(),
        mint: "So11111111111111111111111111111111111111112".to_string(),
        decimals: 9,
    };
    
    let quote_token = Token {
        symbol: "USDC".to_string(),
        mint: "EPjFWdd5AufqSSqeM2qN1xzybapC8G4wEGGkZwyTDt1v".to_string(),
        decimals: 6,
    };
    
    let arbitrage_pair = ArbitragePair {
        base_token,
        quote_token,
        pool_address: Some("TestPool123".to_string()),
        fee_rate: 0.003,
    };
    
    TradingOpportunity {
        pair: arbitrage_pair,
        profit_potential: 0.025, // 2.5% profit potential
        required_capital: 100.0,
        execution_time_estimate: 2000, // 2 seconds
        risk_score: 0.3, // Low risk
        confidence: 0.85, // High confidence
        market_conditions: "stable".to_string(),
        additional_data: HashMap::new(),
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
    
    assert_eq!(strategy.name(), "arbitrage");
    
    println!("✅ ArbitrageStrategy creation works correctly");
}

#[test]
fn test_momentum_strategy_creation() {
    let strategy = MomentumStrategy::new();
    
    assert_eq!(strategy.name(), "momentum");
    
    println!("✅ MomentumStrategy creation works correctly");
}

#[test]
fn test_mean_reversion_strategy_creation() {
    let strategy = MeanReversionStrategy::new();
    
    assert_eq!(strategy.name(), "mean_reversion");
    
    println!("✅ MeanReversionStrategy creation works correctly");
}

#[tokio::test]
async fn test_strategy_manager_initialization() -> Result<()> {
    let config = create_test_config();
    let mut manager = StrategyManager::new(config, 1000.0); // 1000 SOL total capital
    
    // Test initialization
    manager.initialize_strategies()?;
    
    println!("✅ StrategyManager initialization works correctly");
    Ok(())
}

#[tokio::test]
async fn test_strategy_manager_opportunity_analysis() -> Result<()> {
    let config = create_test_config();
    let mut manager = StrategyManager::new(config, 1000.0);
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
    let mut manager = StrategyManager::new(config, 1000.0);
    manager.initialize_strategies()?;
    
    let opportunity = create_test_opportunity();
    let market_data = create_test_market_data();
    
    // Analyze with multiple strategies
    let signals = manager.analyze_opportunity(&opportunity, &market_data).await?;
    
    // Test signal coordination
    let coordinated_signals = manager.coordinate_strategies(HashMap::new()).await?;
    
    println!("✅ Strategy coordination completed");
    println!("   Input signals: {}", signals.len());
    println!("   Coordinated signals: {}", coordinated_signals.len());
    
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
        timestamp: Utc::now(),
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
    let mut manager = StrategyManager::new(config, 1000.0);
    manager.initialize_strategies()?;
    
    // Test performance metrics access
    let performance = manager.get_global_performance();
    
    assert_eq!(performance.strategy_name, "global");
    assert_eq!(performance.total_trades, 0); // New manager should have 0 trades
    
    println!("✅ Performance tracking works correctly");
    println!("   Strategy: {}", performance.strategy_name);
    println!("   Total trades: {}", performance.total_trades);
    println!("   Win rate: {:.2}%", performance.win_rate * 100.0);
    
    Ok(())
}
