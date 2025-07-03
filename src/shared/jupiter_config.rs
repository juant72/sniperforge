//! Jupiter API Configuration
//! 
//! Configuration structures and defaults for Jupiter API integration.

use serde::{Deserialize, Serialize};

/// Jupiter API configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct JupiterConfig {
    pub base_url: String,
    pub api_key: Option<String>,
    pub timeout_seconds: u64,
    pub max_retries: u32,
    pub rpc_endpoint: String,
    pub network_name: String,
}

impl JupiterConfig {
    /// Create configuration for devnet
    pub fn devnet() -> Self {
        Self {
            base_url: "https://lite-api.jup.ag".to_string(),
            api_key: None,
            timeout_seconds: 30,
            max_retries: 3,
            rpc_endpoint: "https://api.devnet.solana.com".to_string(),
            network_name: "DevNet".to_string(),
        }
    }
    
    /// Create configuration for mainnet
    pub fn mainnet() -> Self {
        Self {
            base_url: "https://lite-api.jup.ag".to_string(),
            api_key: None,
            timeout_seconds: 30,
            max_retries: 3,
            rpc_endpoint: "https://api.mainnet-beta.solana.com".to_string(),
            network_name: "MainNet".to_string(),
        }
    }
    
    /// Create configuration with API key
    pub fn with_api_key(mut self, api_key: String) -> Self {
        self.api_key = Some(api_key);
        self
    }
    
    /// Set custom RPC endpoint
    pub fn with_rpc_endpoint(mut self, endpoint: String) -> Self {
        self.rpc_endpoint = endpoint;
        self
    }
    
    /// Create configuration from network config (backward compatibility)
    pub fn from_network_config(network: &crate::config::NetworkConfig) -> Self {
        match network.environment.as_str() {
            "devnet" => Self::devnet(),
            "mainnet" => Self::mainnet(),
            _ => Self::devnet(), // Default to devnet for safety
        }
    }
}

impl Default for JupiterConfig {
    fn default() -> Self {
        Self::devnet()
    }
}
