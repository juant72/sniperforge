//! Advanced Intelligence and Machine Learning Engine
//! 
//! This module provides sophisticated AI-driven trading capabilities including:
//! - Neural network-based market prediction
//! - Advanced market intelligence and analysis
//! - Autonomous trading with adaptive learning
//! 
//! The system leverages cutting-edge machine learning algorithms to provide
//! intelligent trading decisions and market insights.

pub mod ml_engine;
pub mod market_analysis;
pub mod auto_trader;
pub mod sentiment; // Add sentiment module

// Re-export main components for convenience
pub use ml_engine::{AdvancedAiEngine, AiConfig, PricePredictionModel, MarketRegime, RiskAssessment, LearningMetrics};
pub use market_analysis::{
    IntelligenceSystem, SentimentAnalyzer, StrategicAnalyzer, BehavioralPredictor, 
    SentimentAnalysis, ComprehensiveAnalysis, SentimentEngine, SentimentConfig,
    AggregatedSentiment, SentimentTrend
};
pub use auto_trader::{AutonomousTrader, AutonomousConfig, StrategySelector, PositionManager, RiskManager, PerformanceMetrics};

/// Intelligence system configuration
#[derive(Debug, Clone)]
pub struct IntelligenceConfig {
    /// Enable neural network predictions
    pub enable_ml_predictions: bool,
    /// Enable market sentiment analysis
    pub enable_sentiment_analysis: bool,
    /// Enable autonomous trading
    pub enable_autonomous_trading: bool,
    /// Risk tolerance level (0.0 to 1.0)
    pub risk_tolerance: f64,
    /// Maximum position size percentage
    pub max_position_size: f64,
    /// Learning rate for adaptive algorithms
    pub learning_rate: f64,
}

impl Default for IntelligenceConfig {
    fn default() -> Self {
        Self {
            enable_ml_predictions: true,
            enable_sentiment_analysis: true,
            enable_autonomous_trading: false, // Conservative default
            risk_tolerance: 0.5,
            max_position_size: 0.1, // 10% max position
            learning_rate: 0.01,
        }
    }
}

/// Initialize the complete intelligence system
pub async fn initialize_intelligence_system(config: IntelligenceConfig) -> Result<IntelligenceSystemSuite, Box<dyn std::error::Error + Send + Sync>> {
    let ai_config = AiConfig {
        learning_rate: config.learning_rate,
        batch_size: 64,
        sequence_length: 120,
        epochs: 100,
        validation_split: 0.2,
    };

    let ai_engine = AdvancedAiEngine::new(ai_config);
    
    let intelligence_config = crate::intelligence::market_analysis::IntelligenceConfig {
        sentiment_analysis_enabled: config.enable_sentiment_analysis,
        correlation_analysis_enabled: true,
        whale_tracking_enabled: true,
        news_sentiment_enabled: true,
    };
    
    let market_intelligence = IntelligenceSystem::new(intelligence_config);
    
    let autonomous_trader = if config.enable_autonomous_trading {
        let autonomous_config = AutonomousConfig {
            max_position_size: config.max_position_size,
            risk_tolerance: config.risk_tolerance,
            stop_loss_percent: 0.05,
            take_profit_percent: 0.15,
            enable_adaptive_learning: true,
        };
        
        Some(AutonomousTrader::new(
            autonomous_config,
            std::sync::Arc::new(ai_engine.clone()),
            std::sync::Arc::new(market_intelligence.clone()),
        ))
    } else {
        None
    };

    Ok(IntelligenceSystemSuite {
        ai_engine,
        market_intelligence,
        autonomous_trader,
        config,
    })
}

/// Complete intelligence system suite
pub struct IntelligenceSystemSuite {
    pub ai_engine: AdvancedAiEngine,
    pub market_intelligence: IntelligenceSystem,
    pub autonomous_trader: Option<AutonomousTrader>,
    pub config: IntelligenceConfig,
}

impl IntelligenceSystemSuite {
    /// Get comprehensive market analysis
    pub async fn analyze_market(&mut self, symbol: &str) -> Result<MarketIntelligence, Box<dyn std::error::Error + Send + Sync>> {
        let price_prediction = self.ai_engine.predict_price(symbol, 24).await?;
        let market_analysis = self.market_intelligence.analyze_comprehensive(symbol).await?;
        let sentiment = self.market_intelligence.analyze_sentiment(symbol).await?;

        Ok(MarketIntelligence {
            symbol: symbol.to_string(),
            price_prediction,
            sentiment_score: sentiment.overall_score,
            market_regime: market_analysis.market_regime,
            risk_assessment: market_analysis.risk_level,
            trading_recommendation: market_analysis.recommendation,
        })
    }

    /// Execute autonomous trading if enabled
    pub async fn execute_autonomous_trading(&mut self, symbol: &str) -> Result<Option<TradingAction>, Box<dyn std::error::Error + Send + Sync>> {
        if let Some(trader) = &mut self.autonomous_trader {
            let market_intel = MarketIntelligence {
                symbol: symbol.to_string(),
                price_prediction: self.ai_engine.predict_price(symbol, 24).await?,
                sentiment_score: 0.0, // Simplified for now
                market_regime: "BULLISH".to_string(),
                risk_assessment: 0.5,
                trading_recommendation: "BUY".to_string(),
            };
            trader.make_trading_decision(market_intel).await.map(Some)
        } else {
            Ok(None)
        }
    }
}

/// Comprehensive market intelligence result
#[derive(Debug, Clone)]
pub struct MarketIntelligence {
    pub symbol: String,
    pub price_prediction: f64,
    pub sentiment_score: f64,
    pub market_regime: String,
    pub risk_assessment: f64,
    pub trading_recommendation: String,
}

/// Trading action from autonomous system
#[derive(Debug, Clone)]
pub struct TradingAction {
    pub action_type: String,
    pub symbol: String,
    pub quantity: f64,
    pub price: Option<f64>,
    pub confidence: f64,
    pub reasoning: String,
}
