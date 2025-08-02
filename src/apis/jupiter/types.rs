//! Jupiter API Types - Enterprise Consolidated Version
//!
//! Comprehensive type definitions for all Jupiter API operations
//! Consolidates types from multiple sources into a unified system

use serde::{Deserialize, Serialize};
use std::collections::HashMap;

// =============================================================================
// PRICE API TYPES
// =============================================================================

/// Jupiter price response structure for V6 API
#[derive(Debug, Deserialize, Serialize)]
pub struct JupiterPriceResponse {
    pub data: HashMap<String, TokenPriceData>,
    #[serde(rename = "timeTaken")]
    pub time_taken: f64,
}

/// Token price data structure
#[derive(Debug, Deserialize, Serialize)]
pub struct TokenPriceData {
    pub id: String,
    #[serde(rename = "type")]
    pub price_type: String,
    pub price: String, // String because Jupiter returns "160.659570000"
    #[serde(rename = "mintSymbol")]
    pub mint_symbol: Option<String>,
    #[serde(rename = "vsToken")]
    pub vs_token: Option<String>,
    #[serde(rename = "vsTokenSymbol")]
    pub vs_token_symbol: Option<String>,
}

impl TokenPriceData {
    /// Convert price from String to f64
    pub fn price_as_f64(&self) -> f64 {
        self.price.parse().unwrap_or(0.0)
    }
}

// =============================================================================
// QUOTE API TYPES
// =============================================================================

/// Jupiter V6 Quote Request - ENHANCED VERSION
#[derive(Debug, Serialize, Clone, Default)]
pub struct QuoteRequest {
    #[serde(rename = "inputMint")]
    pub input_mint: String,
    #[serde(rename = "outputMint")]
    pub output_mint: String,
    pub amount: u64,
    #[serde(rename = "slippageBps")]
    pub slippage_bps: Option<u16>,
    #[serde(rename = "swapMode")]
    pub swap_mode: Option<String>,
    #[serde(rename = "dexes")]
    pub dexes: Option<Vec<String>>,
    #[serde(rename = "excludeDexes")]
    pub exclude_dexes: Option<Vec<String>>,
    #[serde(rename = "platformFeeBps")]
    pub platform_fee_bps: Option<u16>,
    #[serde(rename = "maxAccounts")]
    pub max_accounts: Option<u16>,
    /// User public key for wallet-specific operations - ENTERPRISE ENHANCEMENT
    #[serde(rename = "userPublicKey", skip_serializing_if = "Option::is_none")]
    pub user_public_key: Option<String>,
}

impl QuoteRequest {
    /// Create a basic quote request with enhanced defaults
    pub fn new(input_mint: String, output_mint: String, amount: u64) -> Self {
        Self {
            input_mint,
            output_mint,
            amount,
            slippage_bps: Some(50), // 0.5% default slippage
            swap_mode: Some("ExactIn".to_string()),
            dexes: None,
            exclude_dexes: None,
            platform_fee_bps: None,
            max_accounts: None,
            user_public_key: None, // ENTERPRISE ENHANCEMENT - Optional user key
        }
    }

    /// Set slippage in basis points
    pub fn with_slippage_bps(mut self, slippage_bps: u16) -> Self {
        self.slippage_bps = Some(slippage_bps);
        self
    }

    /// Exclude specific DEXes
    pub fn exclude_dexes(mut self, dexes: Vec<String>) -> Self {
        self.exclude_dexes = Some(dexes);
        self
    }

    /// Set user public key for wallet-specific operations - ENTERPRISE ENHANCEMENT
    pub fn with_user_public_key(mut self, user_public_key: String) -> Self {
        self.user_public_key = Some(user_public_key);
        self
    }
}

/// Jupiter V6 Quote Response - ENHANCED VERSION
#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct JupiterQuoteResponse {
    #[serde(rename = "inputMint")]
    pub input_mint: String,
    #[serde(rename = "inAmount")]
    pub in_amount: String,
    #[serde(rename = "outputMint")]
    pub output_mint: String,
    #[serde(rename = "outAmount")]
    pub out_amount: String,
    #[serde(rename = "otherAmountThreshold")]
    pub other_amount_threshold: String,
    #[serde(rename = "swapMode")]
    pub swap_mode: String,
    #[serde(rename = "slippageBps")]
    pub slippage_bps: u16,
    #[serde(rename = "platformFee")]
    pub platform_fee: Option<PlatformFee>,
    #[serde(rename = "priceImpactPct")]
    pub price_impact_pct: String,
    #[serde(rename = "routePlan")]
    pub route_plan: Vec<RoutePlan>,
    #[serde(rename = "contextSlot")]
    pub context_slot: Option<u64>,
    #[serde(rename = "timeTaken")]
    pub time_taken: Option<f64>,
}

/// Platform fee information
#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct PlatformFee {
    pub amount: String,
    #[serde(rename = "feeBps")]
    pub fee_bps: u16,
}

/// Route plan for the swap
#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct RoutePlan {
    #[serde(rename = "swapInfo")]
    pub swap_info: SwapInfo,
    pub percent: u8,
}

/// Swap information - ENHANCED VERSION with price impact
#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct SwapInfo {
    #[serde(rename = "ammKey")]
    pub amm_key: String,
    pub label: String,
    #[serde(rename = "inputMint")]
    pub input_mint: String,
    #[serde(rename = "outputMint")]
    pub output_mint: String,
    #[serde(rename = "inAmount")]
    pub in_amount: String,
    #[serde(rename = "outAmount")]
    pub out_amount: String,
    #[serde(rename = "feeAmount")]
    pub fee_amount: String,
    #[serde(rename = "feeMint")]
    pub fee_mint: String,
    /// Price impact percentage (0.0 to 1.0) - ENTERPRISE ENHANCEMENT
    #[serde(rename = "priceImpactPct", default)]
    pub price_impact_pct: Option<f64>,
}

// =============================================================================
// BACKWARD COMPATIBILITY TYPES
// =============================================================================

/// Legacy JupiterQuote for backward compatibility with existing triangular.rs
#[derive(Debug, Deserialize, Serialize)]
pub struct JupiterQuote {
    #[serde(rename = "inputMint")]
    pub input_mint: String,
    #[serde(rename = "inAmount")]
    pub in_amount: String,
    #[serde(rename = "outputMint")]
    pub output_mint: String,
    #[serde(rename = "outAmount")]
    pub out_amount: String,
    #[serde(rename = "priceImpactPct")]
    pub price_impact_pct: Option<String>,
    #[serde(rename = "routePlan")]
    pub route_plan: Option<Vec<RoutePlan>>,
}

impl From<JupiterQuoteResponse> for JupiterQuote {
    fn from(response: JupiterQuoteResponse) -> Self {
        Self {
            input_mint: response.input_mint,
            in_amount: response.in_amount,
            output_mint: response.output_mint,
            out_amount: response.out_amount,
            price_impact_pct: Some(response.price_impact_pct),
            route_plan: Some(response.route_plan),
        }
    }
}

// =============================================================================
// COMMON ENUMS AND CONSTANTS
// =============================================================================

/// Supported DEX labels
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum DexLabel {
    #[serde(rename = "Orca")]
    Orca,
    #[serde(rename = "Raydium")]
    Raydium,
    #[serde(rename = "Serum")]
    Serum,
    #[serde(rename = "Mercurial")]
    Mercurial,
    #[serde(rename = "Saber")]
    Saber,
    #[serde(rename = "Aldrin")]
    Aldrin,
    #[serde(rename = "Crema")]
    Crema,
    #[serde(rename = "Lifinity")]
    Lifinity,
    #[serde(rename = "Whirlpool")]
    Whirlpool,
}

/// Common Solana token mints
pub mod tokens {
    pub const SOL: &str = "So11111111111111111111111111111111111111112";
    pub const USDC: &str = "EPjFWdd5AufqSSqeM2qN1xzybapC8G4wEGGkZwyTDt1v";
    pub const USDT: &str = "Es9vMFrzaCERmJfrF4H2FYD4KCoNkY11McCe8BenwNYB";
    pub const RAY: &str = "4k3Dyjzvzp8eMZWUXbBCjEvwSkkk59S5iCNLY3QrkX6R";
    pub const SRM: &str = "SRMuApVNdxXokk5GT7XD5cUUgXMBCoAz2LHeuAoKWRt";
    pub const ORCA: &str = "orcaEKTdK7LKz57vaAYr9QeNsVEPfiu6QeMU1kektZE";
    pub const MNGO: &str = "MangoCzJ36AjZyKwVj3VnYU4GTonjfVEnJmvvWaxLac";
}
