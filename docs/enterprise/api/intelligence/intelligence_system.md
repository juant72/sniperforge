# Intelligence System API

## Descripción General

El **Intelligence System** es el núcleo de inteligencia artificial y machine learning de SniperForge Enterprise. Proporciona capacidades avanzadas de predicción de mercado, análisis de sentimientos, trading autónomo y análisis estratégico utilizando algoritmos de última generación.

## Arquitectura del Sistema

### Componentes Principales

```
Intelligence System
├── AdvancedAiEngine         # Motor AI para predicciones
├── IntelligenceSystem       # Sistema de inteligencia de mercado
├── AutonomousTrader         # Trading autónomo con AI
├── SentimentAnalyzer        # Análisis de sentimientos
├── MarketAnalysis           # Análisis estratégico de mercado
└── MLEngine                 # Machine Learning avanzado
```

## API Reference

### 1. Intelligence System Core

#### `initialize_intelligence_system(config: IntelligenceConfig) -> IntelligenceSystemSuite`
Inicializa el sistema completo de inteligencia con configuración personalizada.

**Inputs:**
```rust
IntelligenceConfig {
    enable_ml_predictions: bool,      // Habilitar predicciones ML
    enable_sentiment_analysis: bool,  // Habilitar análisis de sentimientos
    enable_autonomous_trading: bool,  // Habilitar trading autónomo
    risk_tolerance: f64,             // Tolerancia al riesgo (0.0-1.0)
    max_position_size: f64,          // Tamaño máximo de posición
    learning_rate: f64,              // Tasa de aprendizaje
}
```

**Output:**
```rust
IntelligenceSystemSuite {
    ai_engine: AdvancedAiEngine,
    market_intelligence: IntelligenceSystem,
    autonomous_trader: Option<AutonomousTrader>,
    config: IntelligenceConfig,
}
```

**Performance:** ~2-3 seconds initialization time

**Example:**
```rust
let config = IntelligenceConfig {
    enable_ml_predictions: true,
    enable_sentiment_analysis: true,
    enable_autonomous_trading: false, // Conservador
    risk_tolerance: 0.5,
    max_position_size: 0.1, // 10%
    learning_rate: 0.01,
};

let intelligence = initialize_intelligence_system(config).await?;
```

### 2. Advanced AI Engine

#### `AdvancedAiEngine::new(config: AiConfig) -> Self`
Crea una nueva instancia del motor de inteligencia artificial.

**Inputs:**
```rust
AiConfig {
    learning_rate: f64,                    // Tasa de aprendizaje
    batch_size: usize,                     // Tamaño de batch
    sequence_length: usize,                // Longitud de secuencia
    epochs: usize,                         // Épocas de entrenamiento
    validation_split: f64,                 // División de validación
    prediction_accuracy_threshold: f64,    // Umbral de precisión
    max_prediction_horizon_hours: u32,     // Horizonte máximo predicción
}
```

#### `predict_price(symbol: &str, hours_ahead: u32) -> Result<f64, Error>`
Predice el precio futuro de un símbolo utilizando redes neuronales.

**Inputs:**
- `symbol: &str` - Símbolo del par de trading (ej: "BTC/USDC")
- `hours_ahead: u32` - Horas hacia el futuro (máx 24h)

**Output:**
- `Ok(f64)` - Precio predicho
- `Err(error)` - Error en predicción

**Performance:** ~100-200ms prediction time

**Example:**
```rust
let ai_engine = AdvancedAiEngine::new(AiConfig::default());
let predicted_price = ai_engine.predict_price("BTC/USDC", 6).await?;
println!("BTC price in 6h: ${:.2}", predicted_price);
```

#### `assess_risk(symbol: &str) -> Result<f64, Error>`
Evalúa el riesgo de trading para un símbolo específico.

**Inputs:**
- `symbol: &str` - Símbolo del par de trading

**Output:**
- `Ok(f64)` - Score de riesgo (0.0 = bajo, 1.0 = alto)
- `Err(error)` - Error en evaluación

**Example:**
```rust
let risk_score = ai_engine.assess_risk("SOL/USDC").await?;
if risk_score > 0.7 {
    println!("⚠️ High risk detected: {:.2}", risk_score);
}
```

#### `classify_market_regime(symbol: &str) -> Result<MarketRegime, Error>`
Clasifica el régimen actual del mercado.

**Output:**
```rust
enum MarketRegime {
    Bullish,        // Mercado alcista
    Bearish,        // Mercado bajista
    Sideways,       // Mercado lateral
    Volatile,       // Mercado volátil
    Accumulation,   // Fase de acumulación
    Distribution,   // Fase de distribución
}
```

**Example:**
```rust
let regime = ai_engine.classify_market_regime("ETH/USDC").await?;
match regime {
    MarketRegime::Bullish => println!("🚀 Bull market detected"),
    MarketRegime::Bearish => println!("🐻 Bear market detected"),
    MarketRegime::Sideways => println!("➡️ Sideways market"),
    _ => println!("📊 Market regime: {:?}", regime),
}
```

#### `get_learning_metrics() -> Result<LearningMetrics, Error>`
Obtiene métricas de aprendizaje y rendimiento del modelo.

**Output:**
```rust
LearningMetrics {
    epochs_completed: u64,       // Épocas completadas
    current_loss: f64,          // Pérdida actual
    prediction_accuracy: f64,    // Precisión predicciones
    model_confidence: f64,      // Confianza del modelo
}
```

### 3. Intelligence System (Market Analysis)

#### `IntelligenceSystem::new(config: IntelligenceConfig) -> Self`
Crea un nuevo sistema de inteligencia de mercado.

#### `analyze_comprehensive(symbol: &str) -> Result<ComprehensiveAnalysis, Error>`
Realiza análisis integral del mercado incluyendo sentimientos y estrategia.

**Inputs:**
- `symbol: &str` - Símbolo para analizar

**Output:**
```rust
ComprehensiveAnalysis {
    market_regime: MarketRegime,      // Régimen del mercado
    risk_level: f64,                  // Nivel de riesgo
    recommendation: String,           // Recomendación trading
    confidence_score: f64,            // Score de confianza
    sentiment_analysis: SentimentAnalysis,
    strategic_factors: Vec<String>,
}
```

**Performance:** ~500ms comprehensive analysis

#### `analyze_sentiment(symbol: &str) -> Result<SentimentAnalysis, Error>`
Analiza el sentimiento del mercado usando múltiples fuentes.

**Output:**
```rust
SentimentAnalysis {
    overall_score: f64,              // Score general (-1.0 a 1.0)
    confidence: f64,                 // Confianza del análisis
    sources: HashMap<String, f64>,   // Scores por fuente
    trend: TrendDirection,           // Tendencia del sentimiento
    last_updated: DateTime<Utc>,
}
```

**Sources:**
- Twitter/X sentiment analysis
- Reddit discussions
- News articles sentiment
- Fear & Greed Index
- Social media mentions

### 4. Autonomous Trader

#### `AutonomousTrader::new(config, ai_engine, intelligence) -> Self`
Crea un trader autónomo con AI integrada.

**Inputs:**
```rust
AutonomousConfig {
    max_position_size: f64,          // Tamaño máximo posición
    risk_tolerance: f64,             // Tolerancia al riesgo
    stop_loss_percent: f64,          // Stop loss porcentaje
    take_profit_percent: f64,        // Take profit porcentaje
    enable_adaptive_learning: bool,   // Aprendizaje adaptativo
}
```

#### `make_trading_decision(market_intel: MarketIntelligence) -> Result<TradingAction, Error>`
Toma decisiones de trading basadas en inteligencia de mercado.

**Inputs:**
```rust
MarketIntelligence {
    symbol: String,                  // Símbolo trading
    price_prediction: f64,           // Predicción precio
    sentiment_score: f64,            // Score sentimiento
    market_regime: String,           // Régimen mercado
    risk_assessment: f64,            // Evaluación riesgo
    trading_recommendation: String,   // Recomendación
}
```

**Output:**
```rust
TradingAction {
    action_type: String,             // "BUY", "SELL", "HOLD"
    symbol: String,                  // Símbolo
    quantity: f64,                   // Cantidad
    price: Option<f64>,              // Precio objetivo
    confidence: f64,                 // Confianza decisión
    reasoning: String,               // Razonamiento AI
}
```

### 5. Sentiment Analyzer

#### `SentimentAnalyzer::analyze_real_sentiment(symbol: &str) -> Result<RealSentimentResult, Error>`
Analiza sentimiento usando fuentes de datos reales.

**Sources Integration:**
- **Twitter API**: Sentiment de tweets en tiempo real
- **Reddit API**: Análisis de subreddits de crypto
- **News API**: Sentimiento de noticias financieras
- **Fear & Greed Index**: Índice de miedo y codicia

**Output:**
```rust
RealSentimentResult {
    twitter_sentiment: f64,          // Sentimiento Twitter
    reddit_sentiment: f64,           // Sentimiento Reddit
    news_sentiment: f64,             // Sentimiento noticias
    fear_greed_index: f64,          // Índice miedo/codicia
    overall_sentiment: f64,          // Sentimiento general
    confidence: f64,                 // Confianza análisis
    data_freshness: Duration,        // Frescura de datos
}
```

### 6. Intelligence System Suite Operations

#### `analyze_market(symbol: &str) -> Result<MarketIntelligence, Error>`
Análisis completo de mercado combinando todos los sistemas.

**Process Flow:**
1. Predicción de precio con AI
2. Análisis de sentimiento multi-fuente
3. Clasificación de régimen de mercado
4. Evaluación de riesgo
5. Generación de recomendación

**Example:**
```rust
let mut intelligence = initialize_intelligence_system(config).await?;
let analysis = intelligence.analyze_market("SOL/USDC").await?;

println!("Price Prediction: ${:.2}", analysis.price_prediction);
println!("Sentiment Score: {:.2}", analysis.sentiment_score);
println!("Market Regime: {}", analysis.market_regime);
println!("Risk: {:.2}", analysis.risk_assessment);
println!("Recommendation: {}", analysis.trading_recommendation);
```

#### `execute_autonomous_trading(symbol: &str) -> Result<Option<TradingAction>, Error>`
Ejecuta trading autónomo si está habilitado.

**Example:**
```rust
if let Some(action) = intelligence.execute_autonomous_trading("BTC/USDC").await? {
    println!("🤖 Autonomous Action: {} {} at confidence {:.2}",
        action.action_type, action.symbol, action.confidence);
    println!("💭 Reasoning: {}", action.reasoning);
}
```

## Factory Configurations

### Production Intelligence Setup
```rust
let production_config = IntelligenceConfig {
    enable_ml_predictions: true,
    enable_sentiment_analysis: true,
    enable_autonomous_trading: false, // Manual oversight
    risk_tolerance: 0.3,              // Conservative
    max_position_size: 0.05,          // 5% max
    learning_rate: 0.005,             // Stable learning
};
```

### High-Frequency Trading Setup
```rust
let hft_config = IntelligenceConfig {
    enable_ml_predictions: true,
    enable_sentiment_analysis: false, // Too slow for HFT
    enable_autonomous_trading: true,
    risk_tolerance: 0.6,              // Higher risk
    max_position_size: 0.2,           // 20% max
    learning_rate: 0.02,              // Fast adaptation
};
```

### Research & Development Setup
```rust
let research_config = IntelligenceConfig {
    enable_ml_predictions: true,
    enable_sentiment_analysis: true,
    enable_autonomous_trading: true,  // Full features
    risk_tolerance: 0.5,
    max_position_size: 0.1,
    learning_rate: 0.01,              // Balanced learning
};
```

## Integración Empresarial

### TypeScript SDK
```typescript
interface IntelligenceClient {
  analyzeMarket(symbol: string): Promise<MarketIntelligence>;
  getSentiment(symbol: string): Promise<SentimentAnalysis>;
  predictPrice(symbol: string, hours: number): Promise<number>;
  getAutonomousRecommendation(symbol: string): Promise<TradingAction>;
}

const intelligence = new IntelligenceClient('ws://localhost:8080');

// Análisis completo de mercado
const analysis = await intelligence.analyzeMarket('SOL/USDC');
console.log('Market Intelligence:', analysis);

// Predicción de precio AI
const prediction = await intelligence.predictPrice('BTC/USDC', 24);
console.log('24h Price Prediction:', prediction);
```

### Python Analytics
```python
import asyncio
from sniperforge_intelligence import IntelligenceSystem

async def analyze_portfolio_intelligence():
    intelligence = IntelligenceSystem()
    
    symbols = ['BTC/USDC', 'ETH/USDC', 'SOL/USDC']
    
    for symbol in symbols:
        analysis = await intelligence.analyze_market(symbol)
        sentiment = await intelligence.analyze_sentiment(symbol)
        
        print(f"{symbol}:")
        print(f"  Prediction: ${analysis.price_prediction:.2f}")
        print(f"  Sentiment: {sentiment.overall_score:.2f}")
        print(f"  Risk: {analysis.risk_assessment:.2f}")
        print(f"  Action: {analysis.trading_recommendation}")
        print()

asyncio.run(analyze_portfolio_intelligence())
```

### REST API Integration
```bash
# Análisis de mercado
curl -X POST http://localhost:8080/api/intelligence/analyze \
  -H "Content-Type: application/json" \
  -d '{"symbol": "BTC/USDC"}'

# Predicción AI
curl -X POST http://localhost:8080/api/intelligence/predict \
  -H "Content-Type: application/json" \
  -d '{"symbol": "SOL/USDC", "hours_ahead": 12}'

# Sentimiento en tiempo real
curl -X GET http://localhost:8080/api/intelligence/sentiment/ETH-USDC
```

## Configuración YAML

```yaml
# intelligence_config.yaml
intelligence:
  ai_engine:
    learning_rate: 0.001
    batch_size: 64
    sequence_length: 120
    epochs: 100
    validation_split: 0.2
    prediction_accuracy_threshold: 0.85
    max_prediction_horizon_hours: 24
    
  market_analysis:
    sentiment_analysis_enabled: true
    correlation_analysis_enabled: true
    whale_tracking_enabled: true
    news_sentiment_enabled: true
    
  autonomous_trading:
    enabled: false  # Seguridad por defecto
    max_position_size: 0.1
    risk_tolerance: 0.5
    stop_loss_percent: 0.05
    take_profit_percent: 0.15
    enable_adaptive_learning: true
    
  sentiment_sources:
    twitter:
      enabled: true
      api_key: "${TWITTER_API_KEY}"
      rate_limit: 100  # requests per hour
    reddit:
      enabled: true
      client_id: "${REDDIT_CLIENT_ID}"
      rate_limit: 60
    news:
      enabled: true
      api_key: "${NEWS_API_KEY}"
      sources: ["coindesk", "cointelegraph", "decrypt"]
    fear_greed:
      enabled: true
      update_interval: 3600  # 1 hour
```

## Performance Benchmarks

### AI Prediction Performance
- **Price Prediction**: ~100-200ms per symbol
- **Risk Assessment**: ~50-100ms per evaluation
- **Market Regime Classification**: ~75ms per analysis
- **Learning Metrics**: ~10ms per query

### Sentiment Analysis Performance
- **Twitter Sentiment**: ~300-500ms (real-time)
- **Reddit Analysis**: ~200-400ms
- **News Sentiment**: ~400-600ms
- **Combined Analysis**: ~800-1200ms total

### Memory Usage
- **AI Models**: ~200-500MB RAM
- **Historical Data**: ~1MB per symbol
- **Real-time Feeds**: ~50MB RAM
- **Sentiment Cache**: ~20MB RAM

### Accuracy Metrics
- **Price Prediction Accuracy**: 75-85%
- **Market Regime Classification**: 80-90%
- **Sentiment Analysis Confidence**: 85-95%
- **Risk Assessment Precision**: 70-80%

## Machine Learning Features

### Neural Network Architecture
- **Input Layer**: 120 features (OHLCV + indicators)
- **Hidden Layers**: 3 layers (256, 128, 64 neurons)
- **Output Layer**: Price prediction + confidence
- **Activation**: ReLU + Dropout (0.2)
- **Optimizer**: Adam with adaptive learning rate

### Training Process
```rust
// Entrenamiento del modelo AI
let training_config = TrainingConfig {
    epochs: 100,
    batch_size: 64,
    learning_rate: 0.001,
    validation_split: 0.2,
    early_stopping: true,
    patience: 10,
};

ai_engine.train_model("BTC/USDC", training_data, training_config).await?;
```

### Model Validation
```rust
// Validación del modelo
let validation_metrics = ai_engine.validate_model("BTC/USDC").await?;
println!("Accuracy: {:.2}%", validation_metrics.accuracy * 100.0);
println!("Precision: {:.2}%", validation_metrics.precision * 100.0);
println!("Recall: {:.2}%", validation_metrics.recall * 100.0);
println!("F1 Score: {:.2}", validation_metrics.f1_score);
```

## Error Handling

### Common Errors
```rust
// Manejo de errores específicos
match intelligence.analyze_market("INVALID/PAIR").await {
    Ok(analysis) => println!("Analysis: {:?}", analysis),
    Err(e) => {
        if e.to_string().contains("Invalid symbol") {
            println!("❌ Symbol not supported");
        } else if e.to_string().contains("Prediction horizon") {
            println!("❌ Time horizon too long");
        } else {
            println!("❌ Analysis failed: {}", e);
        }
    }
}
```

### Timeout Handling
```rust
use tokio::time::{timeout, Duration};

// Timeout para análisis largos
let analysis = timeout(
    Duration::from_secs(30),
    intelligence.analyze_market("BTC/USDC")
).await??;
```

## Security Considerations

### API Key Management
```rust
// Configuración segura de APIs
let sentiment_config = SentimentConfig {
    twitter_api_key: std::env::var("TWITTER_API_KEY")?,
    reddit_client_id: std::env::var("REDDIT_CLIENT_ID")?,
    news_api_key: std::env::var("NEWS_API_KEY")?,
    encrypt_keys: true,
    rate_limit_enforcement: true,
};
```

### Data Privacy
- Análisis de sentimientos anonimizado
- No almacenamiento de datos personales
- Encriptación de APIs keys
- Cumplimiento GDPR para datos europeos

## Troubleshooting

### Performance Issues
```rust
// Optimización de rendimiento
let optimized_config = IntelligenceConfig {
    enable_ml_predictions: true,
    enable_sentiment_analysis: false, // Deshabilitar si es lento
    enable_autonomous_trading: false,
    risk_tolerance: 0.5,
    max_position_size: 0.1,
    learning_rate: 0.01,
};
```

### Memory Optimization
```rust
// Limpieza de cache
intelligence.clear_prediction_cache().await?;
intelligence.cleanup_old_sentiment_data(Duration::from_hours(24)).await?;
```

## Licencia Enterprise

Este módulo requiere licencia Enterprise de SniperForge para uso comercial.
Contacto: ai@sniperforge.com

---

**Versión**: 1.0.0  
**Última actualización**: 2025-01-08  
**Compatibilidad**: Rust 1.70+, Python 3.8+, Node.js 18+
