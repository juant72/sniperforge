# Especificación Técnica - Raydium LP Sniper

## 🔧 Especificaciones Técnicas Detalladas

### Arquitectura de Componentes

```text
┌─────────────────────────────────────────────────────────────────┐
│                    Raydium LP Sniper Bot                        │
├─────────────────────────────────────────────────────────────────┤
│                                                                 │
│  ┌──────────────┐    ┌──────────────┐    ┌──────────────┐      │
│  │   Detector   │────│  Extractor   │────│   Filter     │      │
│  │              │    │              │    │              │      │
│  │ - Pool Scan  │    │ - Token Meta │    │ - Security   │      │
│  │ - Event Sub  │    │ - Liquidity  │    │ - Honeypot   │      │
│  │ - New Pools  │    │ - Supply     │    │ - Blacklist  │      │
│  └──────────────┘    └──────────────┘    └──────────────┘      │
│         │                     │                     │          │
│         └─────────────────────┼─────────────────────┘          │
│                               │                                │
│  ┌──────────────┐    ┌──────────────┐    ┌──────────────┐      │
│  │    Scorer    │────│   Executor   │────│ Exit Monitor │      │
│  │              │    │              │    │              │      │
│  │ - ML Scoring │    │ - Swap Exec  │    │ - Price Mon  │      │
│  │ - Risk Calc  │    │ - Gas Opt    │    │ - Stop Loss  │      │
│  │ - Simulation │    │ - MEV Prot   │    │ - Take Profit│      │
│  └──────────────┘    └──────────────┘    └──────────────┘      │
│                               │                                │
│         ┌─────────────────────┼─────────────────────┐          │
│         │                     │                     │          │
│  ┌──────────────┐    ┌──────────────┐    ┌──────────────┐      │
│  │ Risk Engine  │    │    Logger    │    │   Metrics    │      │
│  │              │    │              │    │              │      │
│  │ - Position   │    │ - Structured │    │ - Prometheus │      │
│  │ - Capital    │    │ - Real-time  │    │ - Grafana    │      │
│  │ - Exposure   │    │ - Alerts     │    │ - Alerting   │      │
│  └──────────────┘    └──────────────┘    └──────────────┘      │
│                                                                 │
└─────────────────────────────────────────────────────────────────┘
```

## 📊 Flujo de Datos Detallado

### 1. Pipeline de Detección

```rust
pub struct DetectionPipeline {
    pool_scanner: PoolScanner,
    event_subscriber: EventSubscriber,
    pool_validator: PoolValidator,
}

impl DetectionPipeline {
    pub async fn detect_new_pools(&self) -> Result<Vec<PoolEvent>, Error> {
        // 1. Escanear nuevos pools via RPC polling
        let rpc_pools = self.pool_scanner.scan_new_pools().await?;
        
        // 2. Escuchar eventos de WebSocket
        let ws_events = self.event_subscriber.get_pool_events().await?;
        
        // 3. Combinar y deduplicar
        let combined = self.combine_sources(rpc_pools, ws_events);
        
        // 4. Validar estructura del pool
        let validated = self.pool_validator.validate_pools(combined).await?;
        
        Ok(validated)
    }
}
```

### 2. Extracción de Metadatos

```rust
pub struct TokenMetadata {
    pub address: Pubkey,
    pub name: String,
    pub symbol: String,
    pub decimals: u8,
    pub total_supply: u64,
    pub mint_authority: Option<Pubkey>,
    pub freeze_authority: Option<Pubkey>,
    pub is_initialized: bool,
}

pub struct PoolMetadata {
    pub pool_address: Pubkey,
    pub token_a: TokenMetadata,
    pub token_b: TokenMetadata,
    pub liquidity_sol: f64,
    pub liquidity_token: u64,
    pub pool_age: Duration,
    pub market_cap: Option<f64>,
}

impl MetadataExtractor {
    pub async fn extract_pool_metadata(&self, pool_addr: &Pubkey) -> Result<PoolMetadata, Error> {
        // Obtener datos del pool
        let pool_account = self.rpc_client.get_account(pool_addr).await?;
        let pool_data = self.parse_pool_account(&pool_account)?;
        
        // Obtener metadatos de tokens
        let token_a_meta = self.get_token_metadata(&pool_data.token_a_mint).await?;
        let token_b_meta = self.get_token_metadata(&pool_data.token_b_mint).await?;
        
        // Calcular liquidez
        let liquidity_sol = self.calculate_sol_liquidity(&pool_data).await?;
        let pool_age = self.calculate_pool_age(&pool_data)?;
        
        Ok(PoolMetadata {
            pool_address: *pool_addr,
            token_a: token_a_meta,
            token_b: token_b_meta,
            liquidity_sol,
            liquidity_token: pool_data.token_reserves,
            pool_age,
            market_cap: None, // Se calcula después
        })
    }
}
```

### 3. Sistema de Filtros

```rust
pub struct FilterEngine {
    security_filters: Vec<Box<dyn SecurityFilter>>,
    economic_filters: Vec<Box<dyn EconomicFilter>>,
    risk_filters: Vec<Box<dyn RiskFilter>>,
}

#[async_trait]
pub trait SecurityFilter: Send + Sync {
    async fn filter(&self, metadata: &PoolMetadata) -> FilterResult;
}

pub struct HoneypotFilter {
    simulation_engine: SimulationEngine,
}

#[async_trait]
impl SecurityFilter for HoneypotFilter {
    async fn filter(&self, metadata: &PoolMetadata) -> FilterResult {
        // Simular compra pequeña
        let buy_result = self.simulation_engine
            .simulate_swap(&metadata.pool_address, 0.01, SwapDirection::Buy)
            .await?;
            
        // Simular venta inmediata
        let sell_result = self.simulation_engine
            .simulate_swap(&metadata.pool_address, buy_result.tokens_out, SwapDirection::Sell)
            .await?;
            
        // Verificar si se puede vender
        if sell_result.is_err() || sell_result.unwrap().sol_out < 0.005 {
            return FilterResult::Reject("Possible honeypot detected".into());
        }
        
        FilterResult::Pass
    }
}
```

### 4. Sistema de Scoring con ML

```rust
pub struct MLScorer {
    model: TensorFlowModel,
    feature_extractor: FeatureExtractor,
    historical_data: HistoricalDataProvider,
}

pub struct PoolFeatures {
    // Características técnicas
    pub liquidity_ratio: f32,
    pub token_supply_ratio: f32,
    pub pool_age_minutes: f32,
    pub sol_dominance: f32,
    
    // Características del token
    pub has_mint_authority: f32,
    pub has_freeze_authority: f32,
    pub token_decimals: f32,
    pub name_length: f32,
    
    // Características del mercado
    pub market_cap_usd: f32,
    pub volume_24h: f32,
    pub price_volatility: f32,
    
    // Características de riesgo
    pub similar_token_performance: f32,
    pub creator_reputation: f32,
}

impl MLScorer {
    pub async fn calculate_score(&self, metadata: &PoolMetadata) -> Result<f32, Error> {
        // 1. Extraer características
        let features = self.feature_extractor.extract(metadata).await?;
        
        // 2. Obtener datos históricos similares
        let historical_performance = self.historical_data
            .get_similar_pools_performance(&features, 100)
            .await?;
        
        // 3. Ejecutar modelo ML
        let ml_score = self.model.predict(&features).await?;
        
        // 4. Ajustar con datos históricos
        let adjusted_score = self.adjust_with_historical(ml_score, historical_performance);
        
        // 5. Aplicar factores de mercado actuales
        let final_score = self.apply_market_conditions(adjusted_score).await?;
        
        Ok(final_score.clamp(0.0, 1.0))
    }
}
```

### 5. Motor de Ejecución Optimizado

```rust
pub struct OptimizedExecutor {
    transaction_builder: TransactionBuilder,
    gas_optimizer: GasOptimizer,
    mev_protection: MEVProtection,
    slippage_calculator: SlippageCalculator,
}

pub struct ExecutionPlan {
    pub swap_amount: u64,
    pub max_slippage: f32,
    pub priority_fee: u64,
    pub use_jito: bool,
    pub jito_tip: u64,
    pub timeout: Duration,
}

impl OptimizedExecutor {
    pub async fn execute_buy(&self, pool: &PoolMetadata, plan: ExecutionPlan) -> Result<ExecutionResult, Error> {
        // 1. Optimizar gas fees
        let optimized_fees = self.gas_optimizer.optimize_fees(&plan).await?;
        
        // 2. Calcular slippage exacto
        let slippage = self.slippage_calculator
            .calculate_current_slippage(pool, plan.swap_amount)
            .await?;
        
        // 3. Construir transacción
        let transaction = self.transaction_builder
            .build_swap_transaction(pool, &plan, &optimized_fees, slippage)
            .await?;
        
        // 4. Aplicar protección MEV
        let protected_tx = if plan.use_jito {
            self.mev_protection.wrap_with_jito(transaction, plan.jito_tip).await?
        } else {
            transaction
        };
        
        // 5. Enviar y confirmar
        let signature = self.send_and_confirm(protected_tx, plan.timeout).await?;
        
        Ok(ExecutionResult {
            signature,
            actual_slippage: slippage,
            total_cost: optimized_fees.total_cost,
            execution_time: Instant::now().duration_since(start_time),
        })
    }
}
```

### 6. Monitor de Salida Inteligente

```rust
pub struct IntelligentExitMonitor {
    price_tracker: PriceTracker,
    volume_analyzer: VolumeAnalyzer,
    strategy_engine: ExitStrategyEngine,
    position_manager: PositionManager,
}

pub struct ExitStrategy {
    pub strategy_type: ExitStrategyType,
    pub take_profit_levels: Vec<TakeProfitLevel>,
    pub stop_loss_level: f32,
    pub trailing_stop: Option<TrailingStopConfig>,
    pub time_based_exit: Option<Duration>,
}

#[derive(Clone)]
pub enum ExitStrategyType {
    Conservative,  // Salida rápida con ganancias moderadas
    Aggressive,    // Esperar máximas ganancias con mayor riesgo
    Adaptive,      // Adaptarse a condiciones del mercado
}

impl IntelligentExitMonitor {
    pub async fn monitor_position(&self, position: &Position) -> Result<Option<ExitSignal>, Error> {
        // 1. Obtener precio actual
        let current_price = self.price_tracker.get_current_price(&position.token).await?;
        let price_change = (current_price - position.entry_price) / position.entry_price;
        
        // 2. Analizar volumen
        let volume_analysis = self.volume_analyzer.analyze_volume(&position.token).await?;
        
        // 3. Verificar condiciones de salida
        let strategy = &position.exit_strategy;
        
        // Stop Loss
        if price_change <= -strategy.stop_loss_level {
            return Ok(Some(ExitSignal::StopLoss {
                current_price,
                loss_pct: price_change,
            }));
        }
        
        // Take Profit
        for tp_level in &strategy.take_profit_levels {
            if price_change >= tp_level.target_pct && !tp_level.executed {
                return Ok(Some(ExitSignal::TakeProfit {
                    level: tp_level.clone(),
                    current_price,
                    profit_pct: price_change,
                }));
            }
        }
        
        // Trailing Stop
        if let Some(trailing_config) = &strategy.trailing_stop {
            if self.should_execute_trailing_stop(position, current_price, trailing_config)? {
                return Ok(Some(ExitSignal::TrailingStop {
                    current_price,
                    highest_price: position.highest_price,
                }));
            }
        }
        
        // Condiciones basadas en volumen
        if volume_analysis.is_drying_up && price_change > 0.5 {
            return Ok(Some(ExitSignal::VolumeDryUp {
                current_price,
                volume_trend: volume_analysis.trend,
            }));
        }
        
        Ok(None)
    }
}
```

## 📈 Métricas y KPIs

### Métricas de Performance

```rust
#[derive(Serialize, Deserialize)]
pub struct PerformanceMetrics {
    // Métricas de detección
    pub pools_detected_per_hour: f64,
    pub detection_latency_ms: f64,
    pub false_positive_rate: f64,
    
    // Métricas de filtrado
    pub filter_pass_rate: f64,
    pub honeypot_detection_accuracy: f64,
    pub filter_processing_time_ms: f64,
    
    // Métricas de scoring
    pub average_score: f64,
    pub score_accuracy: f64, // Comparado con resultados reales
    pub scoring_time_ms: f64,
    
    // Métricas de ejecución
    pub execution_success_rate: f64,
    pub average_slippage: f64,
    pub average_gas_cost: f64,
    pub execution_time_ms: f64,
    
    // Métricas financieras
    pub total_pnl: f64,
    pub win_rate: f64,
    pub average_win: f64,
    pub average_loss: f64,
    pub max_drawdown: f64,
    pub sharpe_ratio: f64,
    
    // Métricas de riesgo
    pub var_95: f64, // Value at Risk 95%
    pub expected_shortfall: f64,
    pub risk_adjusted_return: f64,
}

pub struct MetricsCollector {
    prometheus_registry: Registry,
    counters: HashMap<String, IntCounter>,
    histograms: HashMap<String, Histogram>,
    gauges: HashMap<String, Gauge>,
}

impl MetricsCollector {
    pub fn new() -> Self {
        let registry = Registry::new();
        
        // Registrar métricas Prometheus
        let pools_detected = IntCounter::new("pools_detected_total", "Total pools detected").unwrap();
        let execution_duration = Histogram::with_opts(
            HistogramOpts::new("execution_duration_seconds", "Execution duration in seconds")
                .buckets(vec![0.1, 0.5, 1.0, 2.0, 5.0, 10.0])
        ).unwrap();
        
        registry.register(Box::new(pools_detected.clone())).unwrap();
        registry.register(Box::new(execution_duration.clone())).unwrap();
        
        // ... más métricas
        
        Self {
            prometheus_registry: registry,
            counters: HashMap::new(),
            histograms: HashMap::new(),
            gauges: HashMap::new(),
        }
    }
}
```

## 🔒 Consideraciones de Seguridad

### Gestión de Claves Privadas

```rust
use aes_gcm::{Aes256Gcm, Key, Nonce, aead::{Aead, NewAead}};
use argon2::{Argon2, password_hash::{PasswordHasher, SaltString}};

pub struct SecureKeystore {
    cipher: Aes256Gcm,
    keystore_path: PathBuf,
}

impl SecureKeystore {
    pub fn new(password: &str, keystore_path: PathBuf) -> Result<Self, Error> {
        // Derivar clave de encriptación desde password
        let salt = SaltString::generate(&mut rand::rngs::OsRng);
        let argon2 = Argon2::default();
        let password_hash = argon2.hash_password(password.as_bytes(), &salt)?;
        
        // Crear cipher
        let key = Key::from_slice(password_hash.hash.unwrap().as_bytes());
        let cipher = Aes256Gcm::new(key);
        
        Ok(Self {
            cipher,
            keystore_path,
        })
    }
    
    pub fn store_private_key(&self, keypair: &Keypair) -> Result<(), Error> {
        let private_key_bytes = keypair.secret().to_bytes();
        
        // Generar nonce aleatorio
        let nonce = Nonce::from_slice(&rand::random::<[u8; 12]>());
        
        // Encriptar
        let encrypted = self.cipher.encrypt(nonce, private_key_bytes.as_ref())?;
        
        // Guardar con nonce
        let mut file_data = nonce.to_vec();
        file_data.extend_from_slice(&encrypted);
        
        std::fs::write(&self.keystore_path, file_data)?;
        Ok(())
    }
}
```

### Rate Limiting y Circuit Breaker

```rust
pub struct RateLimiter {
    requests_per_second: u32,
    window: Duration,
    requests: VecDeque<Instant>,
}

pub struct CircuitBreaker {
    failure_threshold: u32,
    recovery_timeout: Duration,
    state: CircuitBreakerState,
    failure_count: u32,
    last_failure: Option<Instant>,
}

#[derive(Clone, PartialEq)]
enum CircuitBreakerState {
    Closed,    // Normal operation
    Open,      // Failures detected, blocking requests
    HalfOpen,  // Testing if service recovered
}
```

Esta especificación técnica detalla la implementación completa del Raydium LP Sniper, proporcionando una base sólida para el desarrollo del bot con todas las consideraciones de performance, seguridad y escalabilidad necesarias.
