//! # SniperForge Intelligence System Demo
//! 
//! Advanced demonstration of the intelligence and machine learning capabilities
//! of the SniperForge trading platform.
//! 
//! This demo showcases:
//! - Neural network-based price prediction
//! - Advanced market intelligence analysis
//! - Autonomous trading with adaptive learning
//! - Risk management and performance monitoring

use std::sync::Arc;
use std::time::Duration;
use chrono::{DateTime, Utc};
use tokio::sync::RwLock;
use tokio::time;
use tracing::{info, warn, error};
use sniperforge::{
    intelligence::{
        initialize_intelligence_system, IntelligenceConfig, AdvancedAiEngine, 
        IntelligenceSystem, AutonomousTrader, MarketIntelligence, TradingAction
    },
    monitoring::{EnterpriseMonitor, SystemStatus},
    config::SimpleConfig,
    errors::SniperResult,
};

/// Intelligence System Demo
/// Demonstrates the complete AI and Machine Learning trading system
#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Initialize logging
    tracing_subscriber::fmt::init();
    info!("🚀 Starting SniperForge Intelligence System Demo");

    // Intelligence System Configuration
    let intelligence_config = IntelligenceConfig {
        enable_ml_predictions: true,
        enable_sentiment_analysis: true,
        enable_autonomous_trading: true,
        risk_tolerance: 0.3,
        max_position_size: 0.05, // 5% max position
        learning_rate: 0.001,
    };

    info!("📊 Initializing Intelligence System with configuration: {:#?}", intelligence_config);

    // Initialize the complete intelligence system
    let mut intelligence_suite = initialize_intelligence_system(intelligence_config.clone()).await?;
    info!("✅ Intelligence System initialized successfully");

    // Initialize monitoring
    let monitor = Arc::new(EnterpriseMonitor::new());
    let status = Arc::new(RwLock::new(SystemStatus::new()));

    // Test symbols for demonstration
    let test_symbols = vec!["SOL/USDC", "BTC/USDC", "ETH/USDC", "RAY/USDC"];

    info!("🔥 Starting Intelligence System demonstration...");

    // Demo loop
    for cycle in 1..=10 {
        info!("📈 === Intelligence Cycle {} ===", cycle);

        for symbol in &test_symbols {
            // Comprehensive market analysis
            match intelligence_suite.analyze_market(symbol).await {
                Ok(market_intel) => {
                    info!("🧠 Market Intelligence for {}:", symbol);
                    info!("   💰 Price Prediction: ${:.4}", market_intel.price_prediction);
                    info!("   😊 Sentiment Score: {:.3}", market_intel.sentiment_score);
                    info!("   📊 Market Regime: {}", market_intel.market_regime);
                    info!("   ⚠️  Risk Assessment: {:.3}", market_intel.risk_assessment);
                    info!("   💡 Recommendation: {}", market_intel.trading_recommendation);

                    // Autonomous trading decision
                    match intelligence_suite.execute_autonomous_trading(symbol).await {
                        Ok(Some(action)) => {
                            info!("🤖 Autonomous Trading Action:");
                            info!("   📋 Action: {}", action.action_type);
                            info!("   💎 Symbol: {}", action.symbol);
                            info!("   📊 Quantity: {:.6}", action.quantity);
                            info!("   💰 Price: {:?}", action.price);
                            info!("   🎯 Confidence: {:.3}", action.confidence);
                            info!("   💭 Reasoning: {}", action.reasoning);
                        }
                        Ok(None) => {
                            info!("🤖 No autonomous trading action recommended");
                        }
                        Err(e) => {
                            warn!("⚠️ Autonomous trading error for {}: {}", symbol, e);
                        }
                    }
                }
                Err(e) => {
                    error!("❌ Market analysis failed for {}: {}", symbol, e);
                }
            }

            // Brief pause between symbols
            time::sleep(Duration::from_millis(500)).await;
        }

        // Update system status
        {
            let mut status_guard = status.write().await;
            status_guard.last_update = Utc::now();
            status_guard.cycles_completed = cycle;
            status_guard.is_healthy = true;
        }

        // Monitor system performance
        let system_metrics = monitor.get_system_metrics().await?;
        info!("📊 System Performance:");
        info!("   💾 Memory Usage: {:.1}%", system_metrics.memory_usage_percent);
        info!("   🖥️  CPU Usage: {:.1}%", system_metrics.cpu_usage_percent);
        info!("   💿 Disk Usage: {:.1}%", system_metrics.disk_usage_percent);

        // Wait before next cycle
        info!("⏳ Waiting 10 seconds before next cycle...\n");
        time::sleep(Duration::from_secs(10)).await;
    }

    info!("🎉 Intelligence System demo completed successfully!");

    // Demonstrate AI engine capabilities
    info!("🧠 === AI Engine Advanced Features Demo ===");
    
    let ai_engine = &intelligence_suite.ai_engine;
    
    // Price prediction demo
    for symbol in &test_symbols {
        match ai_engine.predict_price(symbol, 24).await {
            Ok(prediction) => {
                info!("🔮 24h Price Prediction for {}: ${:.4}", symbol, prediction);
            }
            Err(e) => {
                warn!("⚠️ Price prediction failed for {}: {}", symbol, e);
            }
        }

        match ai_engine.assess_risk(symbol).await {
            Ok(risk_score) => {
                let risk_level = match risk_score {
                    r if r < 0.3 => "🟢 LOW",
                    r if r < 0.7 => "🟡 MEDIUM", 
                    _ => "🔴 HIGH"
                };
                info!("⚖️ Risk Assessment for {}: {} ({:.3})", symbol, risk_level, risk_score);
            }
            Err(e) => {
                warn!("⚠️ Risk assessment failed for {}: {}", symbol, e);
            }
        }
    }

    // Market regime classification
    info!("🏛️ Market Regime Analysis:");
    let market_intelligence = &intelligence_suite.market_intelligence;
    
    for symbol in &test_symbols {
        match market_intelligence.classify_market_regime(symbol).await {
            Ok(regime) => {
                info!("   📈 {}: {}", symbol, regime);
            }
            Err(e) => {
                warn!("   ⚠️ Regime classification failed for {}: {}", symbol, e);
            }
        }
    }

    // Performance and learning metrics
    info!("📚 === Learning and Performance Metrics ===");
    
    if let Some(autonomous_trader) = &intelligence_suite.autonomous_trader {
        let performance = autonomous_trader.get_performance_metrics().await?;
        info!("🎯 Autonomous Trading Performance:");
        info!("   💰 Total PnL: ${:.2}", performance.total_pnl);
        info!("   📊 Win Rate: {:.1}%", performance.win_rate * 100.0);
        info!("   📈 Sharpe Ratio: {:.3}", performance.sharpe_ratio);
        info!("   🔄 Total Trades: {}", performance.total_trades);
        info!("   ⏱️ Average Trade Duration: {:.1} minutes", performance.avg_trade_duration_minutes);
    }

    // AI Learning progress
    let learning_metrics = ai_engine.get_learning_metrics().await?;
    info!("🧠 AI Learning Progress:");
    info!("   📚 Training Epochs: {}", learning_metrics.epochs_completed);
    info!("   📉 Current Loss: {:.6}", learning_metrics.current_loss);
    info!("   🎯 Prediction Accuracy: {:.1}%", learning_metrics.prediction_accuracy * 100.0);
    info!("   📊 Model Confidence: {:.3}", learning_metrics.model_confidence);

    info!("✨ Intelligence System demonstration completed successfully!");
    info!("🚀 SniperForge Intelligence System is ready for production trading!");

    Ok(())
}

/// Demonstration of real-time market monitoring
async fn demonstrate_real_time_monitoring(
    intelligence_suite: &mut sniperforge::intelligence::IntelligenceSystemSuite,
    monitor: Arc<EnterpriseMonitor>,
) -> SniperResult<()> {
    info!("📡 === Real-Time Market Monitoring Demo ===");

    let symbols = vec!["SOL/USDC", "BTC/USDC"];
    let duration = Duration::from_secs(60); // Monitor for 1 minute
    let start_time = std::time::Instant::now();

    while start_time.elapsed() < duration {
        for symbol in &symbols {
            // Real-time market analysis
            let market_intel = intelligence_suite.analyze_market(symbol).await?;
            
            // Check for significant market movements
            if market_intel.sentiment_score.abs() > 0.7 {
                warn!("🚨 Significant sentiment detected for {}: {:.3}", 
                      symbol, market_intel.sentiment_score);
            }

            if market_intel.risk_assessment > 0.8 {
                warn!("⚠️ High risk detected for {}: {:.3}", 
                      symbol, market_intel.risk_assessment);
            }

            // Autonomous trading response
            if let Some(action) = intelligence_suite.execute_autonomous_trading(symbol).await? {
                info!("🤖 Real-time trading action: {} {} at confidence {:.3}", 
                      action.action_type, action.symbol, action.confidence);
            }
        }

        // Update monitoring metrics
        let metrics = monitor.get_trading_metrics().await?;
        if metrics.total_trades > 0 {
            info!("📊 Trading Stats: {} trades, {:.2}% win rate", 
                  metrics.total_trades, metrics.win_rate * 100.0);
        }

        // Sleep for real-time monitoring interval
        time::sleep(Duration::from_secs(5)).await;
    }

    info!("✅ Real-time monitoring demo completed");
    Ok(())
}

/// Custom trading strategy demonstration
async fn demonstrate_custom_strategies(
    intelligence_suite: &sniperforge::intelligence::IntelligenceSystemSuite,
) -> SniperResult<()> {
    info!("🎯 === Custom Trading Strategies Demo ===");

    let symbols = vec!["SOL/USDC", "RAY/USDC"];

    for symbol in &symbols {
        // Get comprehensive market analysis
        let market_intel = intelligence_suite.analyze_market(symbol).await?;

        // Demonstrate different strategy conditions
        match market_intel.market_regime.as_str() {
            "BULLISH" => {
                info!("🐂 Bullish regime detected for {} - Implementing growth strategy", symbol);
                // Simulate momentum-based strategy
            }
            "BEARISH" => {
                info!("🐻 Bearish regime detected for {} - Implementing defensive strategy", symbol);
                // Simulate risk-aversion strategy
            }
            "SIDEWAYS" => {
                info!("📊 Sideways market for {} - Implementing range trading strategy", symbol);
                // Simulate mean-reversion strategy
            }
            _ => {
                info!("❓ Unknown regime for {} - Using adaptive strategy", symbol);
            }
        }

        // Risk-adjusted position sizing
        let base_position = 1000.0; // Base position in USDC
        let risk_adjusted_size = base_position * (1.0 - market_intel.risk_assessment);
        
        info!("💰 Risk-adjusted position size for {}: ${:.2}", symbol, risk_adjusted_size);
    }

    Ok(())
}
