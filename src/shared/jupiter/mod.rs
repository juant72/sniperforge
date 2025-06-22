//! Jupiter API Integration Module
//! 
//! This module provides integration with Jupiter's API for:
//! - Price quotes and route finding
//! - Swap execution on Solana
//! - Market data aggregation
//! 
//! Jupiter is Solana's leading DEX aggregator that finds the best
//! prices across multiple DEXs like Raydium, Orca, Serum, etc.

use anyhow::Result;
use serde::{Deserialize, Serialize};
use solana_sdk::pubkey::Pubkey;
use std::collections::HashMap;
use tracing::{info, warn, error, debug};
use url::Url;

pub mod client;
pub mod types;
pub mod quotes;
pub mod swaps;
pub mod ultra_fast_client;
pub mod fallback_client;

pub use client::JupiterClient;
pub use types::*;
pub use quotes::QuoteEngine;
pub use swaps::JupiterSwapService;
pub use ultra_fast_client::UltraFastJupiterClient;
pub use fallback_client::FallbackJupiterClient;

/// Jupiter API configuration
#[derive(Debug, Clone)]
pub struct JupiterConfig {
    pub api_base_url: String,
    pub rpc_url: String,
    pub timeout_seconds: u64,
    pub max_retries: u32,
    pub slippage_bps: u16, // Basis points (100 = 1%)
    pub enable_devnet: bool,
    pub enable_mainnet_paper: bool,
}

impl Default for JupiterConfig {
    fn default() -> Self {
        Self {
            api_base_url: "https://quote-api.jup.ag/v6".to_string(),
            rpc_url: "https://api.devnet.solana.com".to_string(),
            timeout_seconds: 2, // Aggressive 2s timeout for trading
            max_retries: 1, // Quick fail for speed
            slippage_bps: 50, // 0.5% default slippage
            enable_devnet: true,
            enable_mainnet_paper: false,
        }
    }
}

impl JupiterConfig {
    /// Create mainnet configuration for paper trading
    pub fn mainnet() -> Self {
        Self {
            api_base_url: "https://quote-api.jup.ag/v6".to_string(),
            rpc_url: "https://api.mainnet-beta.solana.com".to_string(),
            timeout_seconds: 5, // Longer timeout for mainnet
            max_retries: 3, // More retries for mainnet
            slippage_bps: 50, // 0.5% default slippage
            enable_devnet: false,
            enable_mainnet_paper: true, // Paper trading mode
        }
    }
}

/// Main Jupiter integration facade
pub struct Jupiter {
    config: JupiterConfig,
    client: JupiterClient,
    quote_engine: QuoteEngine,
    swap_engine: JupiterSwapService,
}

impl Jupiter {
    /// Create new Jupiter instance
    pub async fn new(config: JupiterConfig) -> Result<Self> {
        info!("🪐 Initializing Jupiter API integration");
        
        let client = JupiterClient::new(&config).await?;
        let quote_engine = QuoteEngine::new(client.clone());
        let swap_engine = JupiterSwapService::new(client.clone(), &config.rpc_url);

        info!("✅ Jupiter integration ready");
        info!("   API: {}", config.api_base_url);
        info!("   Slippage: {}%", config.slippage_bps as f64 / 100.0);
        info!("   DevNet: {}", config.enable_devnet);
        info!("   MainNet Paper: {}", config.enable_mainnet_paper);

        Ok(Self {
            config,
            client,
            quote_engine,
            swap_engine,
        })
    }

    /// Get quote engine for price queries
    pub fn quotes(&self) -> &QuoteEngine {
        &self.quote_engine
    }

    /// Get swap engine for trade execution  
    pub fn swaps(&self) -> &JupiterSwapService {
        &self.swap_engine
    }

    /// Get current configuration
    pub fn config(&self) -> &JupiterConfig {
        &self.config
    }

    /// Test Jupiter API connectivity
    pub async fn test_connectivity(&self) -> Result<bool> {
        info!("🧪 Testing Jupiter API connectivity...");
        
        match self.client.health_check().await {
            Ok(_) => {
                info!("✅ Jupiter API connection successful");
                Ok(true)
            }
            Err(e) => {
                error!("❌ Jupiter API connection failed: {}", e);
                Ok(false)
            }
        }
    }
}



