# 📚 SniperForge API Reference (Completa)

Este documento describe TODOS los módulos importantes y secundarios del backend de SniperForge, con su propósito y funciones clave (structs, traits, enums, funciones públicas), para evitar duplicidad de código.

---

## shared/
### jupiter.rs
- **JupiterClient**
  - `pub async fn get_token_price(&self, mint: &str) -> Result<TokenPrice>`
  - `pub async fn get_quote(&self, request: QuoteRequest) -> Result<QuoteResponse>`
  - `pub async fn execute_swap_with_wallet(&self, quote: &QuoteResponse, wallet_address: &str, wallet_keypair: Option<&Keypair>) -> Result<SwapExecutionResult>`
  - ...

### wallet_manager.rs
- **WalletManager**
  - `pub async fn new(config: Config) -> Result<Self>`
  - `pub fn get_main_wallet(&self) -> Result<ManagedWallet>`
  - `pub fn get_keypair(&self) -> Result<Keypair>`
  - ...

### trade_executor.rs
- **TradeExecutor**
  - `pub async fn execute_trade(&self, request: TradeRequest, mode: TradingMode) -> Result<TradeResult>`
  - ...

### real_trading_engine.rs
- Funciones para ejecución avanzada de estrategias y manejo de errores en tiempo real

### data_feeds.rs
- **DataFeedManager**
  - `pub async fn get_price(&self, symbol: &str) -> Result<f64>`
  - ...

### risk_manager.rs
- **RiskManager**
  - `pub fn check_risk(&self, trade: &TradeRequest) -> Result<()>`
  - ...

### performance_optimizer.rs
- **PerformanceProfiler**, **PerformanceTracker**
  - Herramientas para medir y optimizar latencia y throughput

### analytics.rs
- Funciones de análisis de patrones y señales

### monitoring.rs
- Funciones de monitoreo de endpoints y sistema

### Otros (ver archivo para detalles):
- alternative_apis.rs, tatum_client.rs, tatum_rpc_client.rs, syndica_websocket.rs, helius_websocket.rs, pool_detector.rs, premium_rpc_manager.rs, transaction_monitor.rs, websocket_manager.rs, websocket_price_feed.rs

---

## trading/
### strategy_executor.rs
- **StrategyExecutor**
  - `pub async fn execute_dca_strategy(&self, config: DCAConfig) -> Result<ExecutionResult>`
  - `pub async fn execute_momentum_strategy(&self, config: MomentumConfig) -> Result<ExecutionResult>`
  - `pub async fn execute_grid_strategy(&self, config: GridConfig) -> Result<ExecutionResult>`
- **DCAConfig, MomentumConfig, GridConfig, ExecutionResult, TradeExecution, ...**

### order_manager.rs
- **OrderManager**
  - `pub async fn create_stop_loss(&self, params: StopLossParams) -> Result<String>`
  - `pub async fn create_take_profit(&self, params: TakeProfitParams) -> Result<String>`
  - `pub async fn create_trailing_stop(&self, params: TrailingStopParams) -> Result<String>`
  - `pub async fn monitor_orders(&self) -> Result<Vec<ExecutedOrder>>`
- **Order, StopLossParams, TakeProfitParams, TrailingStopParams, ExecutedOrder, ...**

### execution_optimizer.rs
- **ExecutionOptimizer**
  - `pub async fn optimize_slippage(&self, trade: &TradeParams) -> Result<f64>`
  - `pub async fn find_best_route(&self, trade: &TradeParams) -> Result<TradingRoute>`
  - `pub async fn apply_mev_protection(&self, trade: &TradeParams) -> Result<ProtectedTrade>`
- **TradeParams, TradingRoute, ...**

---

## strategies/
- **arbitrage.rs, mean_reversion.rs, momentum.rs, trend_following.rs**
  - Cada archivo define su struct principal y funciones de inicialización y ejecución de la estrategia.

## analysis/
- **patterns.rs, timeframe.rs**
  - Reconocimiento de patrones, análisis multi-timeframe, señales, etc.

## portfolio/
- **manager.rs, analytics.rs, rebalancer.rs, risk_manager.rs, price_feed.rs, real_data_integration.rs, professional_integration.rs, demo_integration.rs, ...**
  - Gestión y análisis de portafolio, feeds, riesgo, integración avanzada.

## ml/
- **advanced_analytics.rs, pattern_recognition.rs, strategy_optimizer.rs, timing_predictor.rs, risk_assessment.rs, model_manager.rs, data_preprocessor.rs**
  - Analítica avanzada, optimización, predicción, evaluación de riesgo, modelos ML.

## performance/
- **benchmarks.rs, metrics.rs, profiler.rs**
  - Benchmarks, métricas, profiling.

## platform/
- **bot_manager.rs, event_bus.rs, resource_coordinator.rs**
  - Gestión de bots, eventos, recursos.

## bots/
- **lp_sniper.rs**
  - Bot LP Sniper y lógica asociada.

## Otros archivos clave
- **cli.rs**: CLI principal y comandos
- **config.rs**: Configuración global
- **types.rs**: Tipos y enums globales
- **main.rs**: Entry point

---

Para cada archivo, revisa su documentación interna y funciones públicas antes de crear lógica nueva. Si tienes dudas sobre la función de un archivo, consulta este documento y el código fuente.

**Actualiza este archivo cuando agregues nuevos módulos o funciones importantes.**
