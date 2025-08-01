//! Trading engine and execution

pub mod arbitrage;
pub mod execution;  // ✅ AGREGADO: Enterprise trade execution engine
pub mod strategies; // ✅ AGREGADO: Trading strategies framework
// pub mod engine;
// pub mod executor;
pub mod risk;
pub mod portfolio;
pub mod triangular;
pub mod flash_loan;
pub mod cross_chain;
pub mod enhanced_system;
pub mod hft_engine;
pub mod route_optimizer;  // ✅ AGREGADO: Route optimization engine
// pub mod strategies;

pub use arbitrage::{ArbitrageEngine, EnhancedArbitrageOpportunity, DexData, TradeResult as ArbitrageTradeResult, PerformanceMetrics};
pub use execution::{
    TradeExecutor, TradeRequest, TradeResult as ExecutionTradeResult, ExecutionStats, 
    RealTradeExecutor, RealTradeRequest, RealTradeResult, RealTradingStats,
    RealTradingEngine, RealTradingConfig, RealSwapRequest, RealSwapResult, 
    QuoteValidation, SwapInfo
};
pub use strategies::{
    TradingStrategy, StrategyConfig, StrategySignal, StrategyPerformance, 
    StrategyManager, SignalType, RiskLevel, Timeframe, TradeResult as StrategyTradeResult,
    ArbitrageStrategy, MomentumStrategy, MeanReversionStrategy
};
pub use risk::{RiskManager};
// pub use engine::*;
// pub use executor::*;
pub use portfolio::{PortfolioManager, Position, TradeRecord, TradeSide, RiskMetrics, PortfolioSummary, PerformanceMetrics as PortfolioPerformanceMetrics};
pub use triangular::*;
pub use hft_engine::{HftEngine, HftOrder, HftMetrics, OrderSide, OrderType};
pub use flash_loan::*;
