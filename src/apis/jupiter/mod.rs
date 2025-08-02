//! Jupiter API Module - Enterprise Consolidated Implementation
//!
//! This module provides a complete Jupiter API integration that consolidates
//! and improves upon existing implementations in the codebase.

pub mod config;
pub mod types;
pub mod client;
pub mod jupiter;

// Re-export main types and structs for easy access
pub use config::{JupiterApiConfig, JupiterSimpleConfig};
pub use types::{
    // Price API
    JupiterPriceResponse, TokenPriceData,
    // Quote API  
    QuoteRequest, JupiterQuoteResponse, JupiterQuote,
    PlatformFee, RoutePlan, SwapInfo,
    // DEX and common types
    DexLabel, tokens,
};
pub use client::JupiterClient;
pub use jupiter::{
    Jupiter, JupiterBuilder, JupiterConfigFile, JupiterMetrics,
    JupiterApiSettings, NetworkJupiterConfig, RateLimitingConfig,
    TradingParameters, WalletIntegrationConfig, MonitoringConfig,
    FallbackConfig, AdvancedFeatures,
};
