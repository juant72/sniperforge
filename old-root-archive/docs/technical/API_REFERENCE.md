# 📚 SniperForge API Reference

Este documento describe los módulos principales y funciones clave del backend de SniperForge, para facilitar la reutilización de código y evitar duplicidad en el desarrollo del Trading Engine y otros componentes.

---

## Índice
- [shared/jupiter.rs](#sharedjupiterrs)
- [shared/wallet_manager.rs](#sharedwallet_managers)
- [shared/trade_executor.rs](#sharedtrade_executorrs)
- [shared/real_trading_engine.rs](#sharedreal_trading_enginers)
- [shared/data_feeds.rs](#shareddata_feedsrs)
- [shared/risk_manager.rs](#sharedrisk_managers)
- [shared/performance_optimizer.rs](#sharedperformance_optimizerrs)
- [Otros módulos útiles](#otros-módulos-útiles)

---

## shared/jupiter.rs
**Propósito:** Integración con Jupiter API para precios, cotizaciones y ejecución de swaps reales.

- `JupiterClient` (struct): Cliente principal para interactuar con Jupiter.
  - `async fn get_token_price(&self, mint: &str) -> Result<TokenPrice>`: Obtiene el precio real de un token.
  - `async fn get_quote(&self, request: QuoteRequest) -> Result<QuoteResponse>`: Obtiene una cotización real para un swap.
  - `async fn execute_swap_with_wallet(&self, quote: &QuoteResponse, wallet_address: &str, wallet_keypair: Option<&Keypair>) -> Result<SwapExecutionResult>`: Ejecuta un swap real firmado por el wallet.

## shared/wallet_manager.rs
**Propósito:** Manejo de wallets, carga de keypairs, gestión de balances y seguridad.

- `WalletManager` (struct): Administra wallets y operaciones asociadas.
  - `async fn new(config: Config) -> Result<Self>`: Inicializa el manager con la configuración.
  - `fn get_main_wallet(&self) -> Result<ManagedWallet>`: Obtiene el wallet principal para trading.
  - `fn get_keypair(&self) -> Result<Keypair>`: Devuelve el keypair para firmar transacciones.

## shared/trade_executor.rs
**Propósito:** Abstracción de ejecución de trades, útil para pruebas y para estrategias personalizadas.

- `TradeExecutor` (struct): Permite ejecutar trades usando diferentes modos.
  - `async fn execute_trade(&self, request: TradeRequest, mode: TradingMode) -> Result<TradeResult>`: Ejecuta un trade usando el modo especificado (real, simulado, etc).

## shared/real_trading_engine.rs
**Propósito:** Motor de trading real, integración avanzada con wallets y swaps.

- Funciones para ejecución avanzada de estrategias y manejo de errores en tiempo real.

## shared/data_feeds.rs
**Propósito:** Acceso a feeds de precios y datos de mercado en tiempo real.

- `DataFeedManager` (struct): Permite suscribirse a precios y eventos de mercado.
  - `async fn get_price(&self, symbol: &str) -> Result<f64>`: Precio en tiempo real.

## shared/risk_manager.rs
**Propósito:** Gestión de riesgo y límites de operación.

- `RiskManager` (struct): Evalúa riesgos antes de ejecutar trades.
  - `fn check_risk(&self, trade: &TradeRequest) -> Result<()>`

## shared/performance_optimizer.rs
**Propósito:** Optimización de performance, uso de cache y profiling.

- `PerformanceProfiler`, `PerformanceTracker`: Herramientas para medir y optimizar latencia y throughput.

## Otros módulos útiles
- `shared/analytics.rs`: Análisis de patrones y señales de mercado.
- `shared/monitoring.rs`: Monitoreo de salud de endpoints y sistema.
- `shared/alternative_apis.rs`: Integración con APIs alternativas (Tatum, Syndica, etc).

---

**Nota:** Para cada nueva función o integración, revisa primero este documento y los módulos listados para evitar duplicar lógica ya existente.

Actualiza este archivo cuando agregues nuevas funciones importantes.
