//! Jupiter API Integration - Legacy Compatibility Module
//! 
//! This module provides backward compatibility for existing code.
//! New code should use the modules directly:
//! - jupiter_api::Jupiter for main functionality
//! - jupiter_client::JupiterClient for HTTP operations
//! - jupiter_types for data structures
//! - jupiter_config for configuration

// Re-export everything from the new modular structure
pub use super::jupiter_types::*;
pub use super::jupiter_config::*;
pub use super::jupiter_client::*;
pub use super::jupiter_api::*;

// Backward compatibility aliases
pub use super::jupiter_api::Jupiter as JupiterWrapper;
pub use super::jupiter_client::JupiterClient as JupiterHttpClient;

/// Common Solana token mints for convenience
pub mod tokens {
    pub const SOL: &str = "So11111111111111111111111111111111111111112";
    pub const USDC: &str = "EPjFWdd5AufqSSqeM2qN1xzybapC8G4wEGGkZwyTDt1v";
    pub const USDT: &str = "Es9vMFrzaCERmJfrF4H2FYD4KCoNkY11McCe8BenwNYB";
    pub const RAY: &str = "4k3Dyjzvzp8eMZWUXbBCjEvwSkkk59S5iCNLY3QrkX6R";
    pub const SRM: &str = "SRMuApVNdxXokk5GT7XD5cUUgXMBCoAz2LHeuAoKWRt";
    pub const ORCA: &str = "orcaEKTdK7LKz57vaAYr9QeNsVEPfiu6QeMU1kektZE";
    pub const MNGO: &str = "MangoCzJ36AjZyKwVj3VnYU4GTonjfVEnJmvvWaxLac";
}
