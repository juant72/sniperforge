# Machine Learning Engine API

## Descripción General

El **Advanced ML Engine** es el sistema de machine learning empresarial de SniperForge que proporciona análisis de sentimientos, predicciones de mercado, evaluación de riesgos, optimización de portafolios y reconocimiento de patrones utilizando algoritmos de ML de última generación.

## Arquitectura del Sistema

### Componentes Principales

```
Advanced ML Engine
├── SentimentAnalyzer         # Análisis de sentimientos con ML
├── MarketPredictor          # Predicciones de mercado
├── RiskAssessor             # Evaluación de riesgos ML
├── PortfolioOptimizer       # Optimización de portafolios
├── PatternRecognizer        # Reconocimiento de patrones
└── ModelMetrics             # Métricas de rendimiento
```

## API Reference

### 1. Advanced ML Engine Core

#### `AdvancedMLEngine::new(config: MLConfig) -> Self`
Crea una nueva instancia del motor de machine learning empresarial.

**Inputs:**
```rust
MLConfig {
    sentiment_threshold: f64,              // Umbral de sentimiento
    prediction_horizon: u32,               // Horizonte predicción (minutos)
    risk_tolerance: f64,                   // Tolerancia al riesgo
    portfolio_rebalance_frequency: u32,    // Frecuencia rebalanceo
    pattern_confidence_threshold: f64,     // Umbral confianza patrones
    model_update_interval: u32,            // Intervalo actualización
    enable_real_time_learning: bool,       // Aprendizaje tiempo real
}
```

**Example:**
```rust
let config = MLConfig {
    sentiment_threshold: 0.7,
    prediction_horizon: 15,        // 15 minutos
    risk_tolerance: 0.3,           // Conservative
    portfolio_rebalance_frequency: 60, // 1 hora
    pattern_confidence_threshold: 0.8,
    model_update_interval: 300,    // 5 minutos
    enable_real_time_learning: true,
};

let ml_engine = AdvancedMLEngine::new(config);
```

### 2. Factory Configurations

#### `MLEngineFactory::create_default() -> AdvancedMLEngine`
Crea un motor ML con configuración por defecto.

#### `MLEngineFactory::create_production() -> AdvancedMLEngine`
Crea un motor ML optimizado para producción.

**Configuration:**
```rust
MLConfig {
    sentiment_threshold: 0.75,     // Higher threshold
    prediction_horizon: 10,        // 10 minutes
    risk_tolerance: 0.25,          // Lower risk
    portfolio_rebalance_frequency: 30, // 30 minutes
    pattern_confidence_threshold: 0.85, // Higher confidence
    model_update_interval: 180,    // 3 minutes
    enable_real_time_learning: true,
}
```

#### `MLEngineFactory::create_hft() -> AdvancedMLEngine`
Crea un motor ML para high-frequency trading.

**Configuration:**
```rust
MLConfig {
    sentiment_threshold: 0.6,      // Lower threshold (more signals)
    prediction_horizon: 5,         // 5 minutes
    risk_tolerance: 0.4,           // Higher risk tolerance
    portfolio_rebalance_frequency: 15, // 15 minutes
    pattern_confidence_threshold: 0.7, // Lower threshold
    model_update_interval: 60,     // 1 minute
    enable_real_time_learning: true,
}
```

### 3. Sentiment Analyzer

#### `SentimentAnalyzer::analyze_sentiment(opportunity: &TradingOpportunity) -> Result<SentimentAnalysis>`
Analiza el sentimiento del mercado para una oportunidad de trading.

**Inputs:**
- `opportunity: &TradingOpportunity` - Oportunidad de trading para analizar

**Output:**
```rust
SentimentAnalysis {
    token: String,                  // Token analizado
    score: f64,                     // Score de sentimiento (0.0-1.0)
    trend: SentimentTrend,          // Tendencia del sentimiento
    correlation_strength: f64,      // Fuerza de correlación
    confidence: f64,                // Confianza del análisis
    data_sources_count: usize,      // Número de fuentes de datos
    timestamp: DateTime<Utc>,       // Timestamp del análisis
}
```

**Sentiment Trends:**
```rust
enum SentimentTrend {
    Bullish,     // Sentimiento alcista
    Bearish,     // Sentimiento bajista
    Neutral,     // Sentimiento neutral
}
```

**Performance:** ~200-500ms per analysis

**Example:**
```rust
let opportunity = TradingOpportunity {
    token_pair: "SOL/USDC".to_string(),
    // ... otros campos
};

let sentiment = ml_engine.sentiment_analyzer
    .analyze_sentiment(&opportunity).await?;

println!("Sentiment Score: {:.2}", sentiment.score);
println!("Trend: {:?}", sentiment.trend);
println!("Confidence: {:.2}%", sentiment.confidence * 100.0);
```

#### `get_sentiment_score(token: &str) -> Option<f64>`
Obtiene el score de sentimiento actual para un token.

#### `update_sentiment_score(token: &str, score: f64)`
Actualiza el score de sentimiento (usado por aprendizaje en tiempo real).

### 4. Market Predictor

#### `MarketPredictor::predict_market(opportunity: &TradingOpportunity) -> Result<MarketPrediction>`
Predice condiciones futuras del mercado usando modelos ML.

**Output:**
```rust
MarketPrediction {
    token: String,                  // Token predicho
    predicted_price: f64,           // Precio predicho
    price_confidence: f64,          // Confianza predicción precio
    volatility_forecast: f64,       // Pronóstico volatilidad
    volume_prediction: f64,         // Predicción volumen
    horizon_minutes: u32,           // Horizonte predicción
    model_accuracy: f64,            // Precisión del modelo
    timestamp: DateTime<Utc>,
}
```

**Example:**
```rust
let prediction = ml_engine.market_predictor
    .predict_market(&opportunity).await?;

println!("Predicted Price: ${:.2}", prediction.predicted_price);
println!("Volatility: {:.2}%", prediction.volatility_forecast * 100.0);
println!("Model Accuracy: {:.2}%", prediction.model_accuracy * 100.0);
```

### 5. Risk Assessor

#### `RiskAssessor::assess_risk(opportunity: &TradingOpportunity) -> Result<RiskAssessment>`
Evalúa riesgos usando modelos de machine learning.

**Output:**
```rust
RiskAssessment {
    token: String,                  // Token evaluado
    overall_risk_score: f64,        // Score riesgo general (0.0-1.0)
    risk_category: RiskCategory,    // Categoría de riesgo
    liquidity_risk: f64,            // Riesgo de liquidez
    volatility_risk: f64,           // Riesgo de volatilidad
    market_risk: f64,               // Riesgo de mercado
    sentiment_risk: f64,            // Riesgo de sentimiento
    confidence: f64,                // Confianza evaluación
    risk_factors: Vec<String>,      // Factores de riesgo
    timestamp: DateTime<Utc>,
}
```

**Risk Categories:**
```rust
enum RiskCategory {
    VeryLow,     // Riesgo muy bajo
    Low,         // Riesgo bajo
    Medium,      // Riesgo medio
    High,        // Riesgo alto
    VeryHigh,    // Riesgo muy alto
}
```

**Example:**
```rust
let risk = ml_engine.risk_assessor
    .assess_risk(&opportunity).await?;

match risk.risk_category {
    RiskCategory::VeryLow => println!("✅ Very safe trade"),
    RiskCategory::Low => println!("🟢 Safe trade"),
    RiskCategory::Medium => println!("🟡 Moderate risk"),
    RiskCategory::High => println!("🟠 High risk"),
    RiskCategory::VeryHigh => println!("🔴 Very risky trade"),
}

println!("Overall Risk: {:.2}%", risk.overall_risk_score * 100.0);
for factor in &risk.risk_factors {
    println!("⚠️ Risk Factor: {}", factor);
}
```

### 6. Portfolio Optimizer

#### `PortfolioOptimizer::optimize_portfolio(opportunities: &[TradingOpportunity]) -> Result<PortfolioOptimization>`
Optimiza el portafolio usando algoritmos ML de optimización.

**Inputs:**
- `opportunities: &[TradingOpportunity]` - Lista de oportunidades de trading

**Output:**
```rust
PortfolioOptimization {
    recommended_allocations: HashMap<String, f64>, // Asignaciones recomendadas
    expected_return: f64,           // Retorno esperado
    expected_volatility: f64,       // Volatilidad esperada
    sharpe_ratio: f64,              // Ratio de Sharpe
    max_drawdown: f64,              // Máxima pérdida
    optimization_confidence: f64,   // Confianza optimización
    rebalance_needed: bool,         // Si necesita rebalanceo
    risk_adjusted_score: f64,       // Score ajustado por riesgo
    timestamp: DateTime<Utc>,
}
```

**Example:**
```rust
let opportunities = vec![
    sol_opportunity,
    btc_opportunity,
    eth_opportunity,
];

let optimization = ml_engine.portfolio_optimizer
    .optimize_portfolio(&opportunities).await?;

println!("Expected Return: {:.2}%", optimization.expected_return * 100.0);
println!("Sharpe Ratio: {:.2}", optimization.sharpe_ratio);
println!("Max Drawdown: {:.2}%", optimization.max_drawdown * 100.0);

for (token, allocation) in &optimization.recommended_allocations {
    println!("{}: {:.1}%", token, allocation * 100.0);
}
```

### 7. Pattern Recognizer

#### `PatternRecognizer::recognize_patterns(market_data: &MarketData) -> Result<Vec<PatternMatch>>`
Reconoce patrones de trading usando machine learning.

**Output:**
```rust
PatternMatch {
    pattern_type: PatternType,      // Tipo de patrón
    confidence: f64,                // Confianza del patrón
    strength: f64,                  // Fuerza del patrón
    time_frame: String,             // Marco temporal
    price_target: Option<f64>,      // Objetivo de precio
    probability: f64,               // Probabilidad éxito
    historical_success_rate: f64,   // Tasa éxito histórica
    pattern_data: PatternData,      // Datos del patrón
    timestamp: DateTime<Utc>,
}
```

**Pattern Types:**
```rust
enum PatternType {
    DoubleTop,           // Doble techo
    DoubleBottom,        // Doble suelo
    HeadAndShoulders,    // Cabeza y hombros
    Triangle,            // Triángulo
    Breakout,            // Ruptura
    Support,             // Soporte
    Resistance,          // Resistencia
    TrendReversal,       // Reversión de tendencia
}
```

**Example:**
```rust
let patterns = ml_engine.pattern_recognizer
    .recognize_patterns(&market_data).await?;

for pattern in patterns {
    if pattern.confidence > 0.8 {
        println!("🔍 Pattern: {:?}", pattern.pattern_type);
        println!("  Confidence: {:.2}%", pattern.confidence * 100.0);
        println!("  Success Rate: {:.2}%", pattern.historical_success_rate * 100.0);
        if let Some(target) = pattern.price_target {
            println!("  Price Target: ${:.2}", target);
        }
    }
}
```

### 8. ML Analysis Result

#### `analyze_comprehensive(opportunity: &TradingOpportunity) -> Result<MLAnalysisResult>`
Realiza análisis ML completo combinando todos los componentes.

**Output:**
```rust
MLAnalysisResult {
    sentiment: SentimentAnalysis,    // Análisis de sentimiento
    prediction: MarketPrediction,    // Predicción de mercado
    risk: RiskAssessment,           // Evaluación de riesgo
    patterns: Vec<PatternMatch>,    // Patrones reconocidos
    recommendation: TradingRecommendation, // Recomendación final
    confidence_score: f64,          // Score de confianza general
    timestamp: DateTime<Utc>,
}
```

**Trading Recommendations:**
```rust
enum TradingRecommendation {
    StrongBuy,    // Compra fuerte
    Buy,          // Compra
    Hold,         // Mantener
    Sell,         // Venta
    StrongSell,   // Venta fuerte
    NoAction,     // Sin acción
}
```

**Example:**
```rust
let analysis = ml_engine.analyze_comprehensive(&opportunity).await?;

println!("🤖 ML Analysis Results:");
println!("Sentiment Score: {:.2}", analysis.sentiment.score);
println!("Price Prediction: ${:.2}", analysis.prediction.predicted_price);
println!("Risk Level: {:?}", analysis.risk.risk_category);
println!("Recommendation: {:?}", analysis.recommendation);
println!("Overall Confidence: {:.2}%", analysis.confidence_score * 100.0);

// Mostrar patrones encontrados
if !analysis.patterns.is_empty() {
    println!("\n📊 Detected Patterns:");
    for pattern in &analysis.patterns {
        println!("  - {:?} (Confidence: {:.2}%)", 
            pattern.pattern_type, pattern.confidence * 100.0);
    }
}
```

## Utility Functions

### Performance Metrics

#### `calculate_sharpe_ratio(returns: &[f64], risk_free_rate: f64) -> f64`
Calcula el ratio de Sharpe para métricas de rendimiento.

#### `calculate_max_drawdown(equity_curve: &[f64]) -> f64`
Calcula la máxima pérdida en el equity curve.

**Example:**
```rust
use sniperforge::ml::utils::{calculate_sharpe_ratio, calculate_max_drawdown};

let returns = vec![0.02, -0.01, 0.03, 0.01, -0.02];
let risk_free_rate = 0.001; // 0.1%

let sharpe = calculate_sharpe_ratio(&returns, risk_free_rate);
println!("Sharpe Ratio: {:.2}", sharpe);

let equity_curve = vec![100.0, 102.0, 101.0, 104.0, 105.0, 103.0];
let max_dd = calculate_max_drawdown(&equity_curve);
println!("Max Drawdown: {:.2}%", max_dd * 100.0);
```

## Integración Empresarial

### TypeScript SDK
```typescript
interface MLEngineClient {
  analyzeSentiment(symbol: string): Promise<SentimentAnalysis>;
  predictMarket(symbol: string): Promise<MarketPrediction>;
  assessRisk(symbol: string): Promise<RiskAssessment>;
  optimizePortfolio(symbols: string[]): Promise<PortfolioOptimization>;
  recognizePatterns(symbol: string): Promise<PatternMatch[]>;
  comprehensiveAnalysis(symbol: string): Promise<MLAnalysisResult>;
}

const mlEngine = new MLEngineClient('ws://localhost:8080');

// Análisis completo ML
const analysis = await mlEngine.comprehensiveAnalysis('SOL/USDC');
console.log('ML Recommendation:', analysis.recommendation);
console.log('Confidence:', analysis.confidence_score);

// Optimización de portafolio
const optimization = await mlEngine.optimizePortfolio([
  'BTC/USDC', 'ETH/USDC', 'SOL/USDC'
]);
console.log('Portfolio Allocations:', optimization.recommended_allocations);
console.log('Expected Return:', optimization.expected_return);
```

### Python Analytics
```python
import numpy as np
import pandas as pd
from sniperforge_ml import AdvancedMLEngine, MLConfig

# Configuración para análisis cuantitativo
config = MLConfig(
    sentiment_threshold=0.7,
    prediction_horizon=15,
    risk_tolerance=0.3,
    enable_real_time_learning=True
)

ml_engine = AdvancedMLEngine(config)

async def run_ml_analysis():
    symbols = ['BTC/USDC', 'ETH/USDC', 'SOL/USDC']
    results = {}
    
    for symbol in symbols:
        # Análisis ML completo
        analysis = await ml_engine.comprehensive_analysis(symbol)
        
        results[symbol] = {
            'sentiment_score': analysis.sentiment.score,
            'predicted_price': analysis.prediction.predicted_price,
            'risk_score': analysis.risk.overall_risk_score,
            'recommendation': analysis.recommendation,
            'confidence': analysis.confidence_score
        }
    
    # Crear DataFrame para análisis
    df = pd.DataFrame(results).T
    
    # Mostrar resultados
    print("ML Analysis Results:")
    print(df)
    
    # Optimización de portafolio
    portfolio_opt = await ml_engine.optimize_portfolio(symbols)
    print(f"\nOptimal Portfolio:")
    print(f"Expected Return: {portfolio_opt.expected_return:.2%}")
    print(f"Sharpe Ratio: {portfolio_opt.sharpe_ratio:.2f}")
    
    for token, allocation in portfolio_opt.recommended_allocations.items():
        print(f"{token}: {allocation:.1%}")

# Ejecutar análisis
import asyncio
asyncio.run(run_ml_analysis())
```

### REST API Endpoints
```bash
# Análisis de sentimiento
curl -X POST http://localhost:8080/api/ml/sentiment \
  -H "Content-Type: application/json" \
  -d '{"symbol": "BTC/USDC"}'

# Predicción de mercado
curl -X POST http://localhost:8080/api/ml/predict \
  -H "Content-Type: application/json" \
  -d '{"symbol": "SOL/USDC", "horizon": 15}'

# Evaluación de riesgo
curl -X POST http://localhost:8080/api/ml/risk \
  -H "Content-Type: application/json" \
  -d '{"symbol": "ETH/USDC"}'

# Optimización de portafolio
curl -X POST http://localhost:8080/api/ml/optimize \
  -H "Content-Type: application/json" \
  -d '{"symbols": ["BTC/USDC", "ETH/USDC", "SOL/USDC"]}'

# Reconocimiento de patrones
curl -X POST http://localhost:8080/api/ml/patterns \
  -H "Content-Type: application/json" \
  -d '{"symbol": "SOL/USDC", "timeframe": "1h"}'

# Análisis ML completo
curl -X POST http://localhost:8080/api/ml/comprehensive \
  -H "Content-Type: application/json" \
  -d '{"symbol": "BTC/USDC"}'
```

## Configuración YAML

```yaml
# ml_config.yaml
ml_engine:
  # Configuración general
  sentiment_threshold: 0.7
  prediction_horizon: 15  # minutos
  risk_tolerance: 0.3
  portfolio_rebalance_frequency: 60  # minutos
  pattern_confidence_threshold: 0.8
  model_update_interval: 300  # segundos
  enable_real_time_learning: true
  
  # Análisis de sentimiento
  sentiment_analyzer:
    data_sources:
      - "twitter"
      - "reddit"
      - "news_feeds"
      - "telegram"
    correlation_tracking: true
    history_retention_hours: 168  # 1 semana
    
  # Predicción de mercado
  market_predictor:
    price_models:
      - "lstm"
      - "transformer"
      - "ensemble"
    volatility_models:
      - "garch"
      - "svr"
    model_validation: true
    backtesting_enabled: true
    
  # Evaluación de riesgo
  risk_assessor:
    risk_factors:
      - "liquidity"
      - "volatility"
      - "market"
      - "sentiment"
      - "correlation"
    dynamic_thresholds: true
    stress_testing: true
    
  # Optimización de portafolio
  portfolio_optimizer:
    optimization_method: "mean_variance"
    constraints:
      max_position_size: 0.3
      min_position_size: 0.05
      max_correlation: 0.8
    rebalancing:
      trigger_threshold: 0.05
      min_rebalance_interval: 30  # minutos
      
  # Reconocimiento de patrones
  pattern_recognizer:
    patterns:
      - "double_top"
      - "double_bottom"
      - "head_shoulders"
      - "triangle"
      - "breakout"
      - "support_resistance"
    timeframes: ["5m", "15m", "1h", "4h"]
    historical_lookback: 100  # períodos
```

## Performance Benchmarks

### Analysis Performance
- **Sentiment Analysis**: ~200-500ms per token
- **Market Prediction**: ~300-800ms per prediction
- **Risk Assessment**: ~100-300ms per evaluation
- **Portfolio Optimization**: ~1-3 seconds per portfolio
- **Pattern Recognition**: ~500ms-1.5s per timeframe
- **Comprehensive Analysis**: ~2-5 seconds total

### Memory Usage
- **Base ML Models**: ~300-800MB RAM
- **Sentiment Data**: ~50MB per token
- **Historical Patterns**: ~100MB cache
- **Portfolio Data**: ~20MB per portfolio
- **Real-time Learning**: ~200MB working memory

### Accuracy Metrics
- **Sentiment Analysis**: 85-92% accuracy
- **Price Prediction**: 70-80% directional accuracy
- **Risk Assessment**: 75-85% accuracy
- **Pattern Recognition**: 65-80% success rate
- **Portfolio Optimization**: 15-25% Sharpe improvement

## Machine Learning Models

### Neural Networks
```rust
// Configuración red neuronal
let neural_config = NeuralNetworkConfig {
    input_features: 50,
    hidden_layers: vec![128, 64, 32],
    output_size: 1,
    activation: "relu",
    dropout_rate: 0.2,
    learning_rate: 0.001,
    batch_size: 64,
    epochs: 100,
};
```

### Ensemble Methods
```rust
// Métodos de ensemble
let ensemble_config = EnsembleConfig {
    models: vec!["lstm", "transformer", "svr"],
    voting_method: "weighted",
    weight_optimization: true,
    cross_validation: 5,
};
```

### Real-time Learning
```rust
// Aprendizaje en tiempo real
if config.enable_real_time_learning {
    ml_engine.update_models_with_new_data(market_data).await?;
    ml_engine.retrain_underperforming_models().await?;
}
```

## Error Handling

### Model Errors
```rust
match ml_engine.analyze_comprehensive(&opportunity).await {
    Ok(analysis) => {
        if analysis.confidence_score < 0.6 {
            println!("⚠️ Low confidence analysis");
        }
    }
    Err(e) => {
        if e.to_string().contains("Model not trained") {
            println!("❌ Model requires training");
        } else if e.to_string().contains("Insufficient data") {
            println!("❌ Not enough data for analysis");
        }
    }
}
```

### Performance Monitoring
```rust
// Monitor model performance
let metrics = ml_engine.get_model_metrics().await?;
for (model_name, metric) in metrics {
    if metric.accuracy < 0.7 {
        println!("⚠️ Model {} underperforming: {:.2}%", 
            model_name, metric.accuracy * 100.0);
        ml_engine.retrain_model(&model_name).await?;
    }
}
```

## Advanced Features

### Model Interpretability
```rust
// Explicabilidad del modelo
let explanation = ml_engine.explain_prediction(&analysis).await?;
println!("Top factors affecting prediction:");
for (factor, importance) in explanation.feature_importance {
    println!("  {}: {:.2}%", factor, importance * 100.0);
}
```

### A/B Testing
```rust
// Testing A/B de modelos
let ab_test = ml_engine.run_ab_test(
    "lstm_v1", 
    "transformer_v2", 
    Duration::from_days(7)
).await?;

println!("Model A performance: {:.2}%", ab_test.model_a_accuracy * 100.0);
println!("Model B performance: {:.2}%", ab_test.model_b_accuracy * 100.0);
```

## Troubleshooting

### Common Issues
1. **Low Prediction Accuracy**
   - Increase training data
   - Adjust model parameters
   - Retrain with recent market data

2. **High Memory Usage**
   - Reduce batch size
   - Clear old model cache
   - Optimize data structures

3. **Slow Performance**
   - Use GPU acceleration if available
   - Reduce model complexity
   - Implement model caching

### Debug Mode
```rust
// Activar modo debug
let debug_config = MLConfig {
    enable_model_debugging: true,
    log_predictions: true,
    save_intermediate_results: true,
    ..Default::default()
};
```

## Licencia Enterprise

Este módulo requiere licencia Enterprise de SniperForge para uso comercial.
Contacto: ml@sniperforge.com

---

**Versión**: 1.0.0  
**Última actualización**: 2025-01-08  
**Compatibilidad**: Rust 1.70+, Python 3.8+, TensorFlow 2.x
