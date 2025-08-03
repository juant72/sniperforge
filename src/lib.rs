//! # `SniperForge` Core Library
//! 
//! Professional-grade core library for Solana `DeFi` trading bots.
//! 
//! ## Features
//! 
//! - **Trading Engine**: High-performance trading execution
//! - **API Integrations**: Jupiter, Orca, Raydium, and more
//! - **Security**: Wallet management and transaction security
//! - **Analytics**: Performance metrics and reporting
//! - **Configuration**: Centralized configuration management

// Temporarily disabled for migration - will be re-enabled after documentation
// #![deny(missing_docs)]
#![deny(unsafe_code)]
#![warn(clippy::all)]
#![warn(clippy::pedantic)]
#![warn(clippy::nursery)]
// Allow some verbose warnings during development
#![allow(clippy::missing_errors_doc)]
#![allow(clippy::missing_panics_doc)]
#![allow(clippy::module_name_repetitions)]
#![allow(clippy::too_many_lines)]
#![allow(clippy::cast_precision_loss)]
#![allow(clippy::cast_possible_truncation)]
#![allow(clippy::cast_sign_loss)]
#![allow(clippy::too_many_arguments)]
#![allow(clippy::redundant_else)]
#![allow(clippy::too_long_first_doc_paragraph)]
#![allow(clippy::uninlined_format_args)]
#![allow(clippy::must_use_candidate)]
#![allow(clippy::missing_const_for_fn)]

pub mod config;
pub mod trading;
pub mod apis;
pub mod security;
pub mod analytics;
pub mod utils;
pub mod types;
pub mod errors;
pub mod monitoring;
pub mod intelligence;
pub mod ml; // ✅ NUEVO: ML module export

// Re-export commonly used types
pub use config::{SimpleConfig, EnterpriseConfig};
pub use trading::{
    ArbitrageEngine, PortfolioManager, Position, TradeSide, RiskMetrics, PortfolioSummary, HftEngine, 
    TradeExecutor, TradeRequest, ExecutionStats, 
    RealTradeExecutor, RealTradeRequest, RealTradeResult,
    RealTradingEngine, RealTradingConfig, RealSwapRequest, RealSwapResult
};
pub use monitoring::{EnterpriseMonitor, SystemStatus, TradingMetrics, SystemMetrics};
pub use intelligence::{AdvancedAiEngine, IntelligenceSystem, AutonomousTrader, IntelligenceConfig, MarketIntelligence, TradingAction};
pub use types::*;
pub use errors::{SniperForgeError, SniperResult, ErrorExt};

/// Current version of the SniperForge Core library
pub const VERSION: &str = env!("CARGO_PKG_VERSION");

/// Initialize the SniperForge Core library with logging
pub fn init() -> anyhow::Result<()> {
    tracing_subscriber::fmt()
        .with_env_filter(tracing_subscriber::EnvFilter::from_default_env())
        .with_target(false)
        .with_thread_ids(true)
        .with_file(true)
        .with_line_number(true)
        .init();
    
    tracing::info!("SniperForge Core v{} initialized", VERSION);
    Ok(())
}
