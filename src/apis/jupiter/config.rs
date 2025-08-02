//! Jupiter API Configuration - Enterprise Consolidated Version
//!
//! Combines and improves all Jupiter configurations from different sources
//! into a single, comprehensive configuration system.

use serde::{Deserialize, Serialize};

/// Comprehensive Jupiter API configuration for enterprise trading
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct JupiterApiConfig {
    /// Enable Jupiter integration
    pub enabled: bool,
    /// API base URL
    pub base_url: String,
    /// Optional API key for premium endpoints
    pub api_key: Option<String>,
    /// Request timeout in seconds
    pub timeout_seconds: u64,
    /// Maximum retry attempts
    pub max_retries: u32,
    /// Rate limit (requests per second)
    pub rate_limit_rps: u32,
    /// Solana RPC endpoint for Jupiter operations
    pub rpc_endpoint: String,
    /// Network name (mainnet, devnet, testnet)
    pub network_name: String,
}

impl JupiterApiConfig {
    /// Create configuration for devnet
    pub fn devnet() -> Self {
        Self {
            enabled: true,
            base_url: "https://quote-api.jup.ag".to_string(),
            api_key: None,
            timeout_seconds: 30,
            max_retries: 3,
            rate_limit_rps: 10,
            rpc_endpoint: "https://api.devnet.solana.com".to_string(),
            network_name: "devnet".to_string(),
        }
    }

    /// Create configuration for mainnet
    pub fn mainnet() -> Self {
        Self {
            enabled: true,
            base_url: "https://quote-api.jup.ag".to_string(),
            api_key: None,
            timeout_seconds: 30,
            max_retries: 3,
            rate_limit_rps: 10,
            rpc_endpoint: "https://api.mainnet-beta.solana.com".to_string(),
            network_name: "mainnet".to_string(),
        }
    }

    /// Create configuration with API key for premium features
    pub fn with_api_key(mut self, api_key: String) -> Self {
        self.api_key = Some(api_key);
        self
    }

    /// Set custom RPC endpoint
    pub fn with_rpc_endpoint(mut self, endpoint: String) -> Self {
        self.rpc_endpoint = endpoint;
        self
    }

    /// Set rate limit
    pub fn with_rate_limit(mut self, rps: u32) -> Self {
        self.rate_limit_rps = rps;
        self
    }

    /// Enable/disable Jupiter integration
    pub fn set_enabled(mut self, enabled: bool) -> Self {
        self.enabled = enabled;
        self
    }
}

impl Default for JupiterApiConfig {
    fn default() -> Self {
        Self::devnet()
    }
}

/// Legacy compatibility for existing simple config
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct JupiterSimpleConfig {
    pub api_url: String,
}

impl From<JupiterApiConfig> for JupiterSimpleConfig {
    fn from(config: JupiterApiConfig) -> Self {
        Self {
            api_url: config.base_url,
        }
    }
}
