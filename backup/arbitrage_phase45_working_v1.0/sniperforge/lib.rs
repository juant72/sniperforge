// ================================================================================
// SNIPERFORGE LOCAL MODULES - ARBITRAGE PHASE 45 SPECIFIC
// ================================================================================
// Módulos específicos para el bot arbitrage_phase45_clean
// Estrategia: Mantener compatibilidad sin afectar el core principal
// ================================================================================

#![allow(dead_code)]
#![allow(unused_imports)]

// ===== MÓDULOS LOCALES =====
pub mod arbitrage_settings;
pub mod real_price_feeds;
pub mod fee_calculator;
pub mod triangular_arbitrage_engine;
pub mod ml_pattern_recognition;
pub mod jupiter_v6_client;
pub mod real_trade_executor;
pub mod unified_config;
pub mod advanced_performance_optimizer;

// ===== RE-EXPORTS =====
pub use arbitrage_settings::ArbitrageSettings;
pub use real_price_feeds::RealPriceFeeds;
pub use fee_calculator::FeeCalculator;
pub use triangular_arbitrage_engine::{TriangularArbitrageEngine, TriangularArbitrageConfig, TriangularOpportunity};
pub use ml_pattern_recognition::{PatternRecognitionEngine, OpportunityPattern};
pub use jupiter_v6_client::{JupiterV6Client, SwapQuote};
pub use real_trade_executor::{RealTradeExecutor, TradeRequest, TradeResult, ExecutionMode, SafetyLimits};
pub use unified_config::{UnifiedConfig, TradingConfig, RiskManagementConfig, ArbitrageConfig};
pub use advanced_performance_optimizer::{AdvancedPerformanceOptimizer, PerformanceMetrics};
