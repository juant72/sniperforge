# 📚 SniperForge API Reference (Completa)

> **📅 Última actualización: July 3, 2025**
> **🎯 Estado: 100% Actualizado - Refactor Jupiter completado, 68 tests pasando**

Este documento describe TODOS los módulos importantes del backend de SniperForge, con su propósito y funciones clave (structs, traits, enums, funciones públicas), para evitar duplicidad de código.

---

## 🚀 MÓDULOS CORE - COMPLETAMENTE REFACTORIZADOS

## shared/jupiter/ (REFACTOR MODULAR COMPLETADO)

### jupiter_types.rs ✅ **NUEVO - REFACTORIZADO**
- **QuoteRequest**: Estructura para solicitudes de cotización Jupiter
  - `pub inputMint: String` - Token de entrada
  - `pub outputMint: String` - Token de salida  
  - `pub amount: u64` - Cantidad en unidades base
  - `pub slippageBps: u16` - Slippage en basis points

- **QuoteResponse**: Respuesta de cotización con helpers
  - `pub inAmount: String` - Cantidad de entrada
  - `pub outAmount: String` - Cantidad de salida
  - `pub priceImpactPct: String` - Impacto en precio
  - `pub routePlan: Vec<RoutePlan>` - Plan de ruta
  - `pub fn in_amount_f64(&self) -> f64` - Helper conversión entrada
  - `pub fn out_amount_f64(&self) -> f64` - Helper conversión salida
  - `pub fn price_impact_pct_f64(&self) -> f64` - Helper impacto precio

- **SwapRequest**: Solicitud de swap con optimizaciones
- **SwapResponse**: Respuesta de swap con datos de transacción
- **SwapResult**: Resultado simplificado para manejo

### jupiter_config.rs ✅ **NUEVO - REFACTORIZADO**
- **JupiterConfig**: Configuración centralizada
  - `pub base_url: String` - URL base de API
  - `pub quote_endpoint: String` - Endpoint de cotizaciones
  - `pub swap_endpoint: String` - Endpoint de swaps
  - `pub timeout_seconds: u64` - Timeout de requests
  - `pub max_retries: u8` - Reintentos máximos
  - `pub fn default() -> Self` - Configuración por defecto

### jupiter_client.rs ✅ **NUEVO - REFACTORIZADO**
- **JupiterClient**: Cliente HTTP para API Jupiter
  - `pub async fn new(config: JupiterConfig) -> Result<Self>`
  - `pub async fn get_quote(&self, request: QuoteRequest) -> Result<QuoteResponse>`
  - `pub async fn build_swap_transaction(&self, request: SwapRequest) -> Result<SwapResponse>`
  - Manejo robusto de errores HTTP y timeouts

### jupiter_api.rs ✅ **NUEVO - REFACTORIZADO**
- **Jupiter**: API principal de negocio con integración wallet
  - `pub async fn new(config: &JupiterConfig) -> Result<Self>`
  - `pub async fn get_quote(&self, input_mint: &str, output_mint: &str, amount: f64, slippage_bps: u16) -> Result<QuoteResponse>`
  - `pub async fn build_swap_transaction(&self, quote: &QuoteResponse, wallet_address: &str) -> Result<SwapResult>`
  - `pub async fn execute_swap_with_wallet(&self, quote: &QuoteResponse, wallet_keypair: &Keypair) -> Result<SwapExecutionResult>`
  - Eliminación completa de recursión y stack overflow

### jupiter.rs ✅ **REFACTORIZADO - LEGACY COMPATIBILITY**
- **Wrapper de compatibilidad** que re-exporta todos los módulos nuevos
- **tokens**: Constantes de tokens comunes (SOL, USDC, USDT, etc.)
- Mantiene compatibilidad con código existente

## shared/ (MÓDULOS CORE ESTABLES)

### wallet_manager.rs ✅ **ESTABLE**
- **WalletManager**: Gestión segura de wallets
  - `pub async fn new(config: Config) -> Result<Self>`
  - `pub fn get_main_wallet(&self) -> Result<ManagedWallet>`
  - `pub fn get_keypair(&self) -> Result<Keypair>`
  - `pub async fn get_balance(&self, token_mint: Option<&str>) -> Result<f64>`
  - `pub fn get_public_key(&self) -> Result<Pubkey>`

### trade_executor.rs ✅ **ACTUALIZADO**
- **TradeExecutor**: Ejecución de trades con safety checks
  - `pub async fn execute_trade(&self, request: TradeRequest, mode: TradingMode) -> Result<TradeResult>`
  - `pub async fn validate_trade_safety(&self, request: &TradeRequest) -> Result<()>`
  - Integración completa con Jupiter refactorizado

### real_trade_executor.rs ✅ **NUEVO**
- **RealTradeExecutor**: Ejecutor optimizado para trades reales
  - `pub async fn execute_real_trade(&self, params: TradeParams) -> Result<ExecutionResult>`
  - `pub async fn validate_execution_safety(&self, params: &TradeParams) -> Result<()>`

### cache_free_trading.rs ✅ **IMPLEMENTADO**
- **CacheFreeTradeEngine**: Engine de trading sin cache para máxima precisión
  - `pub async fn new(config: CacheFreeConfig) -> Result<Self>`
  - `pub async fn execute_trade_with_validation(&self, opportunity: &TradingOpportunity) -> Result<TradeResult>`
  - `pub fn get_performance_metrics(&self) -> PerformanceMetrics`

### pool_detector.rs ✅ **IMPLEMENTADO**  
- **TradingOpportunity**: Estructura de oportunidades de trading
- **DetectedPool**: Información de pools detectados
- **TokenInfo**: Información detallada de tokens
- **RiskScore**: Evaluación de riesgo de pools

## 🏦 PORTFOLIO MANAGEMENT - COMPLETAMENTE IMPLEMENTADO

### portfolio/manager.rs ✅ **COMPLETADO**
- **PortfolioManager**: Gestión completa de carteras
  - `pub async fn add_position(&self, position: Position) -> Result<()>`
  - `pub async fn remove_position(&self, position_id: Uuid) -> Result<Position>`
  - `pub async fn update_position(&self, position_id: Uuid, current_price: f64) -> Result<()>`
  - `pub async fn get_positions(&self) -> HashMap<Uuid, Position>`
  - `pub async fn calculate_metrics(&self) -> Result<PortfolioMetrics>`
  - `pub async fn rebalance_portfolio(&self) -> Result<RebalanceResult>`

### portfolio/analytics.rs ✅ **IMPLEMENTADO**
- **PortfolioAnalytics**: Análisis avanzado de cartera
  - `pub fn calculate_sharpe_ratio(&self, returns: &[f64], risk_free_rate: f64) -> f64`
  - `pub fn calculate_max_drawdown(&self, values: &[f64]) -> f64`
  - `pub fn calculate_volatility(&self, returns: &[f64]) -> f64`

### portfolio/risk_manager.rs ✅ **IMPLEMENTADO**
- **RiskManager**: Gestión de riesgo de cartera
  - `pub fn check_position_concentration(&self, positions: &[Position], total_capital: f64) -> Result<()>`
  - `pub fn calculate_var(&self, positions: &[Position], confidence: f64) -> f64`
  - `pub fn assess_correlation_risk(&self, positions: &[Position]) -> f64`

### portfolio/rebalancer.rs ✅ **IMPLEMENTADO**
- **PortfolioRebalancer**: Rebalanceo automático
  - `pub fn calculate_rebalancing_trades(&self, current: &[Position], target: &StrategyAllocation) -> Vec<RebalanceTrade>`
  - `pub fn determine_priority(&self, trade: &RebalanceTrade) -> RebalancePriority`

## 🤖 TRADING STRATEGIES - IMPLEMENTADOS

### trading/strategy_executor.rs ✅ **COMPLETADO**
- **StrategyExecutor**: Ejecutor de estrategias multi-algoritmo
  - `pub async fn execute_dca_strategy(&self, config: DCAConfig) -> Result<ExecutionResult>`
  - `pub async fn execute_momentum_strategy(&self, config: MomentumConfig) -> Result<ExecutionResult>`
  - `pub async fn execute_grid_strategy(&self, config: GridConfig) -> Result<ExecutionResult>`
  - `pub async fn execute_arbitrage_strategy(&self, config: ArbitrageConfig) -> Result<ExecutionResult>`

### trading/order_manager.rs ✅ **COMPLETADO**
- **OrderManager**: Gestión avanzada de órdenes
  - `pub async fn create_stop_loss(&self, params: StopLossParams) -> Result<String>`
  - `pub async fn create_take_profit(&self, params: TakeProfitParams) -> Result<String>`
  - `pub async fn create_trailing_stop(&self, params: TrailingStopParams) -> Result<String>`
  - `pub async fn monitor_orders(&self) -> Result<Vec<ExecutedOrder>>`

### trading/execution_optimizer.rs ✅ **COMPLETADO**
- **ExecutionOptimizer**: Optimización de ejecución
  - `pub async fn optimize_slippage(&self, trade: &TradeParams) -> Result<f64>`
  - `pub async fn find_best_route(&self, trade: &TradeParams) -> Result<TradingRoute>`
  - `pub async fn apply_mev_protection(&self, trade: &TradeParams) -> Result<ProtectedTrade>`

## 🧠 MACHINE LEARNING - IMPLEMENTADO

### ml/model_manager.rs ✅ **IMPLEMENTADO**
- **MLModelManager**: Gestión de modelos de ML
  - `pub fn register_model(&mut self, model_id: String, model: Box<dyn MLModel>)`
  - `pub async fn load_model(&self, model_path: &str) -> Result<()>`
  - `pub fn get_model(&self, model_id: &str) -> Option<&dyn MLModel>`

### ml/pattern_recognition.rs ✅ **IMPLEMENTADO**
- **PatternRecognizer**: Reconocimiento de patrones técnicos
  - `pub fn detect_patterns(&self, market_data: &MarketData) -> Vec<Pattern>`
  - `pub fn generate_predictions(&self, patterns: &[Pattern]) -> Vec<Prediction>`

### ml/timing_predictor.rs ✅ **IMPLEMENTADO**
- **TimingPredictor**: Predicción de timing de entrada/salida
  - `pub fn predict_optimal_entry(&self, market_data: &MarketData) -> TimingPrediction`
  - `pub fn update_market_data(&mut self, data: MarketData)`

## 📊 PERFORMANCE & MONITORING

### shared/performance_profiler.rs ✅ **IMPLEMENTADO**
- **PerformanceProfiler**: Profiling completo del sistema
  - `pub fn start_operation(&self, operation_name: &str) -> OperationTimer`
  - `pub fn end_operation(&self, timer: OperationTimer)`
  - `pub fn generate_report(&self) -> PerformanceReport`

### shared/real_time_blockchain.rs ✅ **IMPLEMENTADO**
- **RealTimeBlockchainEngine**: Engine de datos blockchain en tiempo real
  - `pub async fn get_cached_price(&self, token_mint: &str) -> Option<f64>`
  - `pub async fn update_price_cache(&self, token_mint: &str, price: f64)`

## 🔗 INTEGRATIONS & EXTERNAL APIs

### shared/tatum_rpc_client.rs ✅ **IMPLEMENTADO**
- **TatumRpcClient**: Cliente robusto para Tatum RPC
  - `pub async fn test_connection(&self) -> Result<bool>`
  - `pub async fn get_balance(&self, address: &str) -> Result<f64>`
  - Integración completa con devnet y mainnet

---

## 📝 TYPES & CONFIGURATIONS

### Tipos Core ✅ **DEFINIDOS**
- **Position**: Posición en cartera con métricas de riesgo
- **PortfolioMetrics**: Métricas completas de cartera
- **TradeRequest**: Solicitud de trade con validaciones
- **ExecutionResult**: Resultado detallado de ejecución
- **TradingOpportunity**: Oportunidad de trading detectada

### Configuraciones ✅ **IMPLEMENTADAS**
- **Config**: Configuración principal del sistema
- **DCAConfig**: Configuración de Dollar Cost Averaging
- **PortfolioConfiguration**: Configuración de gestión de cartera
- **JupiterConfig**: Configuración de integración Jupiter

---

## 🚀 CLI COMMANDS REFERENCE

Ver `docs/user-guides/CLI_COMMANDS.md` para referencia completa de comandos CLI implementados.

---

> **🏆 ESTADO ACTUAL**: API completamente funcional, 68 tests pasando, arquitectura modular lista para producción, integración real con Jupiter y Solana blockchain.
