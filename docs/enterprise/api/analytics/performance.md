# 📊 Performance Analytics AI API Documentation

## Overview

The **Performance Analytics AI System** provides enterprise-grade artificial intelligence for market prediction, strategy optimization, and comprehensive performance analysis with machine learning capabilities.

---

## 📋 System Architecture

### Core Components

```rust
// Enterprise AI Engine
pub struct EnterpriseAIEngine {
    config: EnterpriseAIConfig,
    market_predictor: MarketPredictor,
    strategy_optimizer: StrategyOptimizer,
    anomaly_detector: AnomalyDetector,
    stats: AIEngineStats,
}

// Performance Analytics AI
pub struct PerformanceAnalyticsAI {
    config: PerformanceAnalyticsConfig,
    analysis_engine: AnalysisEngine,
    reporting_system: ReportingSystem,
    optimization_engine: OptimizationEngine,
    alert_manager: AlertManager,
}

// Advanced AI Engine (ML Engine)
pub struct AdvancedAiEngine {
    config: AiConfig,
    models: HashMap<String, PricePredictionModel>,
    performance_tracker: PerformanceTracker,
    learning_metrics: LearningMetrics,
}
```

---

## 🤖 Enterprise AI Engine

### Constructor

#### `new(config: Option<EnterpriseAIConfig>, settings: SimpleConfig) -> Self`

Creates a new enterprise AI engine instance with advanced ML capabilities.

**Input Parameters:**
- `config: Option<EnterpriseAIConfig>` - AI engine configuration (optional, uses defaults)
- `settings: SimpleConfig` - System configuration

**Returns:**
- `EnterpriseAIEngine` - Initialized AI engine

**Example:**
```rust
let config = EnterpriseAIConfig {
    enabled: true,
    price_prediction_model: "LSTM_Enterprise_Pro".to_string(),
    min_prediction_confidence: 0.88,
    ..EnterpriseAIConfig::default()
};

let ai_engine = EnterpriseAIEngine::new(Some(config), settings);
```

---

### Core AI Methods

#### `predict_price(token: &str, current_price: f64, horizon_minutes: u32) -> Result<Option<PricePrediction>>`

Generates AI-powered price predictions using enterprise ML models.

**Input Parameters:**
- `token: &str` - Token symbol (e.g., "SOL", "ETH")
- `current_price: f64` - Current market price in USD
- `horizon_minutes: u32` - Prediction time horizon in minutes

**Returns:**
- `Result<Option<PricePrediction>>` - Price prediction or None if disabled

**PricePrediction Structure:**
```rust
pub struct PricePrediction {
    pub token_symbol: String,
    pub current_price_usd: f64,
    pub predicted_price_usd: f64,
    pub predicted_change_percentage: f64,
    pub confidence_level: f64,              // 0.0 to 1.0
    pub prediction_horizon_minutes: u32,
    pub features_used: Vec<String>,
    pub model_version: String,
    pub generated_at: DateTime<Utc>,
}
```

**Example:**
```rust
let prediction = ai_engine.predict_price("SOL", 150.0, 30).await?;

if let Some(pred) = prediction {
    println!("SOL Price Prediction:");
    println!("  Current: ${:.2}", pred.current_price_usd);
    println!("  Predicted: ${:.2} ({:+.2}%)", 
             pred.predicted_price_usd, 
             pred.predicted_change_percentage);
    println!("  Confidence: {:.1}%", pred.confidence_level * 100.0);
}
```

---

#### `optimize_strategy(strategy_id: &str) -> Result<StrategyOptimization>`

Optimizes trading strategies using genetic algorithms and ML insights.

**Input Parameters:**
- `strategy_id: &str` - Identifier of the strategy to optimize

**Returns:**
- `Result<StrategyOptimization>` - Optimization results

**StrategyOptimization Structure:**
```rust
pub struct StrategyOptimization {
    pub strategy_id: String,
    pub optimization_type: String,
    pub expected_fitness_score: f64,
    pub expected_profit_per_trade_usd: f64,
    pub optimized_parameters: HashMap<String, f64>,
    pub optimization_iterations: u32,
    pub improvement_percentage: f64,
    pub confidence_score: f64,
    pub generated_at: DateTime<Utc>,
}
```

**Example:**
```rust
let optimization = ai_engine.optimize_strategy("arbitrage_v1").await?;

println!("Strategy Optimization Results:");
println!("  Fitness Score: {:.3}", optimization.expected_fitness_score);
println!("  Expected Profit: ${:.2}", optimization.expected_profit_per_trade_usd);
println!("  Improvement: {:.1}%", optimization.improvement_percentage);
```

---

#### `detect_market_anomalies(token: &str, current_price: f64) -> Result<Option<MarketAnomaly>>`

Detects market anomalies using statistical analysis and pattern recognition.

**Input Parameters:**
- `token: &str` - Token symbol to analyze
- `current_price: f64` - Current price to evaluate

**Returns:**
- `Result<Option<MarketAnomaly>>` - Detected anomaly or None

**MarketAnomaly Structure:**
```rust
pub struct MarketAnomaly {
    pub token_symbol: String,
    pub anomaly_type: String,
    pub severity: f64,                    // 0.0 to 1.0
    pub detection_confidence: f64,        // 0.0 to 1.0
    pub current_price: f64,
    pub expected_price_range: (f64, f64),
    pub deviation_magnitude: f64,
    pub potential_causes: Vec<String>,
    pub detected_at: DateTime<Utc>,
}
```

**Example:**
```rust
let anomaly = ai_engine.detect_market_anomalies("SOL", 1000.0).await?;

if let Some(anom) = anomaly {
    println!("🚨 Market Anomaly Detected:");
    println!("  Type: {}", anom.anomaly_type);
    println!("  Severity: {:.1}%", anom.severity * 100.0);
    println!("  Confidence: {:.1}%", anom.detection_confidence * 100.0);
}
```

---

#### `get_market_analysis(tokens: &[String]) -> Result<HashMap<String, PricePrediction>>`

Generates comprehensive market analysis for multiple tokens.

**Input Parameters:**
- `tokens: &[String]` - List of token symbols to analyze

**Returns:**
- `Result<HashMap<String, PricePrediction>>` - Analysis results per token

**Example:**
```rust
let tokens = vec!["SOL".to_string(), "ETH".to_string(), "BTC".to_string()];
let analysis = ai_engine.get_market_analysis(&tokens).await?;

for (token, prediction) in analysis {
    println!("{}: ${:.2} → ${:.2} ({:+.2}%)",
             token,
             prediction.current_price_usd,
             prediction.predicted_price_usd,
             prediction.predicted_change_percentage);
}
```

---

### Information Methods

#### `get_statistics() -> &AIEngineStats`

Returns comprehensive AI engine statistics and performance metrics.

**AIEngineStats Structure:**
```rust
pub struct AIEngineStats {
    pub total_predictions_generated: u64,
    pub correct_predictions: u64,
    pub average_model_accuracy: f64,
    pub total_strategy_optimizations: u64,
    pub average_fitness_improvement: f64,
    pub total_anomalies_detected: u64,
    pub confirmed_anomalies: u64,
    pub anomaly_detection_rate: f64,
    pub average_prediction_time_ms: f64,
    pub average_optimization_time_seconds: f64,
    pub top_features_usage: HashMap<String, u64>,
}
```

#### `is_enabled() -> bool`

Returns whether the AI engine is currently enabled.

#### `get_config() -> &EnterpriseAIConfig`

Returns the current AI engine configuration.

---

## 📈 Performance Analytics AI

### Constructor

#### `new(config: Option<PerformanceAnalyticsConfig>, settings: SimpleConfig) -> Self`

Creates a new performance analytics AI system.

**PerformanceAnalyticsConfig Structure:**
```rust
pub struct PerformanceAnalyticsConfig {
    pub enabled: bool,
    pub analysis_window_hours: u32,              // 168 (1 week) default
    pub reporting_interval_minutes: u32,         // 30 default
    pub automatic_alerts_enabled: bool,
    pub performance_alert_threshold: f64,        // 15.0% default
    pub auto_optimization_enabled: bool,
    pub max_recommendations_per_analysis: u32,   // 15 default
    pub detailed_reporting_enabled: bool,
    pub historical_analysis_depth_days: u32,    // 180 (6 months) default
}
```

---

### Analytics Methods

#### `analyze_performance(time_window: Duration) -> Result<PerformanceReport>`

Generates comprehensive performance analysis report.

**PerformanceReport Structure:**
```rust
pub struct PerformanceReport {
    pub analysis_period: Duration,
    pub total_trades: u64,
    pub profitable_trades: u64,
    pub win_rate_percentage: f64,
    pub total_profit_loss: f64,
    pub sharpe_ratio: f64,
    pub max_drawdown: f64,
    pub risk_adjusted_returns: f64,
    pub recommendations: Vec<PerformanceRecommendation>,
    pub trend_analysis: TrendAnalysis,
    pub risk_metrics: RiskMetrics,
}
```

#### `generate_optimization_recommendations() -> Result<Vec<OptimizationRecommendation>>`

Generates AI-powered optimization recommendations.

#### `track_real_time_performance() -> Result<LivePerformanceMetrics>`

Provides real-time performance tracking and metrics.

---

## 🧠 Advanced AI Engine (ML Engine)

### Constructor

#### `new(config: AiConfig) -> Self`

Creates advanced ML engine for autonomous trading decisions.

**AiConfig Structure:**
```rust
pub struct AiConfig {
    pub learning_rate: f64,                      // 0.001 default
    pub batch_size: usize,                       // 64 default
    pub sequence_length: usize,                  // 120 default
    pub epochs: usize,                           // 100 default
    pub validation_split: f64,                   // 0.2 default
    pub prediction_accuracy_threshold: f64,      // 0.85 default
    pub max_prediction_horizon_hours: u32,       // 24 default
}
```

---

### ML Methods

#### `predict_price(symbol: &str, hours_ahead: u32) -> Result<f64>`

Uses LSTM models for price prediction.

#### `assess_risk(symbol: &str) -> Result<f64>`

Provides ML-based risk assessment.

#### `classify_market_regime(symbol: &str) -> Result<MarketRegime>`

Classifies current market conditions.

**MarketRegime Enum:**
```rust
pub enum MarketRegime {
    Bullish,
    Bearish,
    Sideways,
    Volatile,
    Accumulation,
    Distribution,
}
```

#### `get_learning_metrics() -> Result<LearningMetrics>`

Returns ML model learning progress and accuracy metrics.

---

## ⚙️ Configuration

### Enterprise AI Configuration

```rust
pub struct EnterpriseAIConfig {
    pub enabled: bool,                              // true
    pub price_prediction_model: String,             // "LSTM_v2"
    pub historical_analysis_window_minutes: u32,    // 120 (2 hours)
    pub min_prediction_confidence: f64,             // 0.75 (75%)
    pub max_analysis_features: u32,                 // 50
    pub strategy_optimization_enabled: bool,        // true
    pub optimization_search_depth: u32,             // 5
    pub anomaly_detection_enabled: bool,            // true
    pub anomaly_threshold: f64,                     // 2.5 std deviations
    pub ai_logging_level: String,                   // "info"
}
```

---

## 🎯 Performance Guarantees

### AI Engine Performance
- **Prediction Generation:** < 500ms per prediction
- **Strategy Optimization:** < 2 seconds per optimization
- **Anomaly Detection:** < 100ms per analysis
- **Model Accuracy:** > 75% minimum confidence

### Analytics Performance
- **Report Generation:** < 5 seconds for comprehensive reports
- **Real-time Metrics:** < 100ms update frequency
- **Historical Analysis:** < 10 seconds for 6-month depth
- **Recommendation Engine:** < 1 second per recommendation

### Reliability Targets
- **Model Uptime:** > 99.9%
- **Prediction Accuracy:** > 75% verified predictions
- **Alert Precision:** > 90% true positive rate
- **Data Processing:** > 99.8% accuracy

---

## 🔌 Integration Examples

### Complete AI Trading Setup
```rust
use sniperforge::analytics::{EnterpriseAIEngine, PerformanceAnalyticsAI};
use sniperforge::intelligence::AdvancedAiEngine;

#[tokio::main]
async fn main() -> Result<()> {
    // Initialize AI systems
    let mut ai_engine = EnterpriseAIEngine::new(None, config.clone());
    let mut analytics = PerformanceAnalyticsAI::new(None, config.clone());
    let ml_engine = AdvancedAiEngine::new(AiConfig::default());
    
    // Trading loop with AI assistance
    loop {
        // Get market predictions
        let sol_prediction = ai_engine.predict_price("SOL", 150.0, 30).await?;
        
        // Check for anomalies
        let anomaly = ai_engine.detect_market_anomalies("SOL", 150.0).await?;
        
        // Optimize strategies
        let optimization = ai_engine.optimize_strategy("main_strategy").await?;
        
        // Generate performance report
        let report = analytics.analyze_performance(Duration::from_hours(24)).await?;
        
        // Make trading decisions based on AI insights
        if let Some(pred) = sol_prediction {
            if pred.confidence_level > 0.8 && pred.predicted_change_percentage > 5.0 {
                println!("🚀 High-confidence bullish prediction detected!");
            }
        }
        
        tokio::time::sleep(Duration::from_secs(60)).await;
    }
}
```

### Real-time Performance Monitoring
```rust
// Monitor AI performance
tokio::spawn(async move {
    loop {
        let stats = ai_engine.get_statistics();
        
        println!("🤖 AI Engine Status:");
        println!("  Predictions: {} (Accuracy: {:.1}%)",
                 stats.total_predictions_generated,
                 stats.average_model_accuracy * 100.0);
        println!("  Anomalies: {} (Detection Rate: {:.1}%)",
                 stats.total_anomalies_detected,
                 stats.anomaly_detection_rate * 100.0);
        
        tokio::time::sleep(Duration::from_secs(30)).await;
    }
});
```

---

## 🛡️ Error Handling

### Common Error Types
- `ModelLoadError` - ML model loading failures
- `PredictionError` - Prediction generation failures
- `OptimizationError` - Strategy optimization failures
- `DataInsufficientError` - Insufficient data for analysis

### Error Recovery
- Automatic model fallback mechanisms
- Graceful degradation during failures
- Comprehensive error logging
- Alert system for critical failures

---

## 📊 Monitoring & Alerts

### Key Metrics
- Model prediction accuracy trends
- Processing latency distributions
- Memory and CPU usage patterns
- Error rates and recovery times

### Alert Conditions
- Prediction accuracy below threshold (< 75%)
- Processing latency spikes (> 1 second)
- Model training failures
- Anomaly detection false positive rates

---

*Last Updated: August 8, 2025*
*API Version: 3.0*
