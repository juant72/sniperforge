//! # SniperForge Core Library
//! 
//! Professional-grade core library for Solana DeFi trading bots.
//! 
//! ## Features
//! 
//! - **Trading Engine**: High-performance trading execution
//! - **API Integrations**: Jupiter, Orca, Raydium, and more
//! - **Security**: Wallet management and transaction security
//! - **Analytics**: Performance metrics and reporting
//! - **Configuration**: Centralized configuration management

#![deny(missing_docs)]
#![deny(unsafe_code)]
#![warn(clippy::all)]
#![warn(clippy::pedantic)]
#![warn(clippy::nursery)]

pub mod config;
pub mod trading;
pub mod apis;
pub mod security;
pub mod analytics;
pub mod utils;
pub mod types;

// Re-export commonly used types
pub use config::*;
pub use trading::*;
pub use types::*;

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
